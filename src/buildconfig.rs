//this defines build config as a struct along with a set of helper functions to deal with sources namely updating, downloading and verifying them
use super::utils::{calculate_sha56sum, download_and_extract_with_sha, git};
use git2::{build::CheckoutBuilder, Oid, Repository};
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf, process::exit, str};
use url::Url;

#[derive(Debug, Deserialize)]
pub struct BuildConfig {
    pub name: PkgName,
    pub version: String,
    pub rel: u32,
    pub arch: Vec<String>,
    pub url: Url,        //ensure this is a url
    pub license: String, //TODO: VERIFY SPDX
    pub depends: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub builttype: BuildType,
    pub configopts: Vec<String>,
    pub builddepends: Option<String>,
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
#[serde(untagged)]
pub enum Arch {
    #[serde(rename = "any")]
    Any,
    MutliPackage(Vec<String>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
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
#[serde(untagged)]
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
    pub async fn fetch(self, src: &PathBuf, workdir: &PathBuf) -> Result<PathBuf, String> {
        match self.r#type {
            SourceType::Archive => match self.url {
                None => {
                    eprintln!("Source type was set to archive but no url was provided");
                    exit(78);
                }
                Some(url) => match self.sha256sum {
                    None => {
                        eprintln!("Source type was set to archive but no sha256sum was provided");
                        exit(78);
                    }
                    Some(sha256sum) => {
                        let selfpath = self.path;
                        let out: PathBuf;

                        if let Some(selfpath) = selfpath {
                            out = selfpath;
                        } else {
                            let segments = url.path_segments().map(|c| c.collect::<Vec<_>>());
                            if let Some(segments) = segments {
                                out = PathBuf::from(segments.last().copied().unwrap());
                            } else {
                                out = PathBuf::from("out.tar.gz");
                            }
                        };
                        let src_out = src.join(out);
                        if src_out.exists() {
                            let cached_sum = calculate_sha56sum(&src_out)
                                .await
                                .map_err(|e| e.to_string())?;

                            if cached_sum == sha256sum {
                                return Ok(workdir.to_owned());
                            } else {
                                download_and_extract_with_sha(url, sha256sum, &src_out, &workdir)
                                    .await
                            }
                        } else {
                            download_and_extract_with_sha(url, sha256sum, &src_out, &workdir).await
                        }
                    }
                },
            },
            SourceType::Git => {
                if self.commit.is_none() {
                    eprintln!("Commit is required for git sources");
                    exit(78);
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
                        repo = Repository::open(&out).map_err(|e| e.to_string())?;
                        git::fetch(&repo)?;
                    } else {
                        if recursive {
                            repo = match Repository::clone_recurse(&url.to_string(), &out) {
                                Ok(repo) => repo,
                                Err(e) => {
                                    eprintln!("Failed to clone repo: {url}\n{e}");
                                    exit(1);
                                }
                            }
                        } else {
                            repo = match Repository::clone(&url.to_string(), &out) {
                                Ok(repo) => repo,
                                Err(e) => {
                                    eprintln!("Failed to clone repo: {url}\n{e}");
                                    exit(1);
                                }
                            }
                        }
                    }
                    if let Some(commit_str) = self.commit {
                        if let Some(tag) = self.tag {
                            let oid = repo.refname_to_id(&tag).map_err(|e| e.to_string())?;
                            let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;
                            if oid.to_string() != commit_str {
                                eprintln!(
                                    "expected tag: {tag} to resolve to {commit_str} was {}",
                                    commit.id()
                                );
                                exit(1);
                            }
                            let mut checkout_options = CheckoutBuilder::new();
                            repo.checkout_tree(commit.as_object(), Some(&mut checkout_options))
                                .map_err(|e| e.to_string())?;
                            repo.set_head_detached(oid).map_err(|e| e.to_string())?;
                        } else {
                            let oid = Oid::from_str(&commit_str).unwrap();
                            let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;
                            let mut checkout_options = CheckoutBuilder::new();
                            repo.checkout_tree(commit.as_object(), Some(&mut checkout_options))
                                .map_err(|e| e.to_string())?;
                            repo.set_head_detached(oid).map_err(|e| e.to_string())?;
                        }
                    } else {
                        unreachable!();
                    }

                    Ok(out)
                } else {
                    eprintln!("Url is required for git source");
                    exit(78);
                }
            }

            SourceType::File => {
                unimplemented!()
            }
            SourceType::Patch => {
                unimplemented!()
            }
        }
    }
}
