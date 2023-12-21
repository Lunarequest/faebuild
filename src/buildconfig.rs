//this defines build config as a struct along with a set of helper functions to deal with sources namely updating, downloading and verifying them
use super::utils::{
    calculate_sha56sum, download_with_pb, extract_with_sha, get_filename_from_url, git,
};
use anyhow::{anyhow, Result};
use git2::{build::CheckoutBuilder, Oid, Repository};
use serde::Deserialize;
use std::{collections::HashMap, fs::copy, path::PathBuf, str};
use url::Url;

#[derive(Debug, Deserialize)]
pub struct BuildConfig {
    pub name: PkgName,
    pub version: String,
    pub rel: u32,
    pub arch: Vec<String>,
    pub url: Url,        //ensure this is a url
    pub license: String, //TODO: VERIFY SPDX
    pub depends: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub subdir: Option<PathBuf>,
    pub buildtype: BuildType,
    pub configopts: Option<Vec<String>>,
    pub builddepends: Option<Vec<String>>,
    pub buildsteps: Vec<String>,
    pub sources: Vec<Sources>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PkgName {
    Name(String),
    MutliPackage(Vec<String>),
}

#[derive(Debug, Deserialize)]
pub enum Arch {
    #[serde(rename = "any")]
    Any,
    MutliPackage(Vec<String>),
}

#[derive(Debug, Deserialize)]
pub enum BuildType {
    #[serde(rename = "simple")]
    Simple,
    #[serde(rename = "cmake-ninja")]
    CmakeNinja,
    #[serde(rename = "cmake")]
    Cmake,
    #[serde(rename = "meson")]
    Meson,
    #[serde(rename = "autotools")]
    AutoTools,
}

#[derive(Debug, Deserialize)]
pub struct Sources {
    pub r#type: SourceType,
    pub path: Option<PathBuf>,
    pub url: Option<Url>,
    pub sha256sum: Option<String>,
    pub commit: Option<String>,
    pub tag: Option<String>,
    pub recursive: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub enum SourceType {
    #[serde(rename = "git")]
    Git,
    #[serde(rename = "archive")]
    Archive,
    #[serde(rename = "file")]
    File,
    #[serde(rename = "patch")]
    Patch,
}

impl Sources {
    pub async fn fetch(self, src: &PathBuf, workdir: &PathBuf) -> Result<PathBuf> {
        match self.r#type {
            SourceType::Archive => match self.url {
                None => {
                    return Err(anyhow!(
                        "Source type was set to archive but no url was provided"
                    ));
                }
                Some(url) => match self.sha256sum {
                    None => {
                        return Err(anyhow!(
                            "Source type was set to archive but no sha256sum was provided"
                        ));
                    }
                    Some(sha256sum) => {
                        let selfpath = self.path;

                        let out: PathBuf = if let Some(selfpath) = selfpath {
                            selfpath
                        } else {
                            match get_filename_from_url(&url) {
                                Some(path) => PathBuf::from(path),
                                None => PathBuf::from("out.tar.gz"),
                            }
                        };

                        let src_out = src.join(out);
                        if src_out.exists() {
                            let cached_sum = calculate_sha56sum(&src_out).await?;

                            if cached_sum == sha256sum {
                                extract_with_sha(sha256sum, &src_out, &workdir).await
                            } else {
                                download_with_pb(url, &src_out).await?;
                                extract_with_sha(sha256sum, &src_out, &workdir).await
                            }
                        } else {
                            download_with_pb(url, &src_out).await?;
                            extract_with_sha(sha256sum, &src_out, &workdir).await
                        }
                    }
                },
            },
            SourceType::Git => {
                if self.commit.is_none() {
                    return Err(anyhow!("Commit is required for git sources"));
                }

                if let Some(url) = self.url {
                    let url_path = url.path_segments().unwrap();
                    let basename = url_path.last().unwrap().replace(".git", "");
                    let out = src.join(basename);
                    let mut recursive = true;
                    let repo: Repository;
                    if let Some(rec) = self.recursive {
                        recursive = rec
                    }
                    if out.exists() {
                        repo = Repository::open(&out)?;
                        git::fetch(&repo)?;
                    } else {
                        if recursive {
                            repo = match Repository::clone_recurse(&url.to_string(), &out) {
                                Ok(repo) => repo,
                                Err(e) => {
                                    return Err(anyhow!("Failed to clone repo: {url}\n{e}"));
                                }
                            }
                        } else {
                            repo = match Repository::clone(&url.to_string(), &out) {
                                Ok(repo) => repo,
                                Err(e) => {
                                    return Err(anyhow!("Failed to clone repo: {url}\n{e}"));
                                }
                            }
                        }
                    }
                    if let Some(commit_str) = self.commit {
                        if let Some(tag) = self.tag {
                            let oid = repo.refname_to_id(&tag)?;
                            let commit = repo.find_commit(oid)?;
                            if oid.to_string() != commit_str {
                                return Err(anyhow!(
                                    "expected tag: {tag} to resolve to {commit_str} was {}",
                                    commit.id()
                                ));
                            }
                            let mut checkout_options = CheckoutBuilder::new();
                            repo.checkout_tree(commit.as_object(), Some(&mut checkout_options))?;
                            repo.set_head_detached(oid)?;
                        } else {
                            let oid = Oid::from_str(&commit_str).unwrap();
                            let commit = repo.find_commit(oid)?;
                            let mut checkout_options = CheckoutBuilder::new();
                            repo.checkout_tree(commit.as_object(), Some(&mut checkout_options))?;
                            repo.set_head_detached(oid)?;
                        }
                    } else {
                        unreachable!();
                    }

                    Ok(out)
                } else {
                    return Err(anyhow!("Url is required for git source"));
                }
            }

            SourceType::File => {
                if self.url.is_some() {
                    if self.sha256sum.is_none() {
                        return Err(anyhow!("Source type was set to File and there was a url but no sha256sum was provided"));
                    }
                    if let Some(url) = self.url {
                        let out = match get_filename_from_url(&url) {
                            Some(path) => PathBuf::from(path),
                            None => PathBuf::from("out.tar.gz"),
                        };

                        let outfile = src.join(&out);

                        download_with_pb(url, &outfile).await?;

                        let shasumactual = calculate_sha56sum(&outfile).await?;

                        if let Some(sha256sum) = self.sha256sum {
                            if sha256sum != shasumactual {
                                return Err(anyhow!(
                                    "expected sha for {} was {} expected {}",
                                    out.display(),
                                    shasumactual,
                                    sha256sum
                                ));
                            }
                        } else {
                            unreachable!()
                        }

                        Ok(out)
                    } else {
                        unreachable!()
                    }
                } else {
                    if let Some(path) = self.path {
                        let srcpath = src.join(&path);
                        copy(path, &srcpath)?;
                        Ok(srcpath)
                    } else {
                        Err(anyhow!("either url or path is required"))
                    }
                }
            }
            SourceType::Patch => {
                if self.url.is_some() {
                    if self.sha256sum.is_none() {
                        return Err(anyhow!("Source type was set to File and there was a url but no sha256sum was provided"));
                    }
                    if let Some(url) = self.url {
                        let out = match get_filename_from_url(&url) {
                            Some(path) => PathBuf::from(path),
                            None => PathBuf::from("out.tar.gz"),
                        };

                        let outfile = src.join(&out);

                        download_with_pb(url, &outfile).await?;

                        let shasumactual = calculate_sha56sum(&outfile).await?;

                        if let Some(sha256sum) = self.sha256sum {
                            if sha256sum != shasumactual {
                                return Err(anyhow!(
                                    "expected sha for {} was {} expected {}",
                                    out.display(),
                                    shasumactual,
                                    sha256sum
                                ));
                            }
                        } else {
                            unreachable!()
                        }

                        Ok(out)
                    } else {
                        unreachable!()
                    }
                } else {
                    if let Some(path) = self.path {
                        let srcpath = src.join(&path);
                        copy(path, &srcpath)?;
                        Ok(srcpath)
                    } else {
                        Err(anyhow!("either url or path is required"))
                    }
                }
            }
        }
    }
}
