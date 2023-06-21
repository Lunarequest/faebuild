//this defines build config as a struct along with a set of helper functions to deal with sources namely updating, downloading and verifying them
use super::utils::download_with_pb;
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use serde::Deserialize;
use sha256::digest;
use std::{collections::HashMap, fs::File, path::PathBuf, process::exit};
use tar::Archive;
use url::Url;
use xz2::read::XzDecoder;
use zip::ZipArchive;
use zstd::stream::Decoder;

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
    pub async fn fetch(self, src: PathBuf, workdir: PathBuf) -> Result<PathBuf, String> {
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
                        let bytes = download_with_pb(url, src_out).await?;
                        let sha = digest(bytes.as_ref());
                        if sha256sum != sha {
                            eprintln!("expected sha256sum: {sha256sum} got {sha}");
                            exit(1);
                        }

                        if src_out.ends_with(".gz") {
                            let tar_gz = File::open(src_out).unwrap();
                            let tar = GzDecoder::new(tar_gz);
                            let mut archive = Archive::new(tar);
                            archive.unpack(workdir).unwrap();
                        } else if src_out.ends_with(".xz") {
                            let tar_xz = File::open(src_out).unwrap();
                            let tar = XzDecoder::new(tar_xz);
                            let mut archive = Archive::new(tar);
                            archive.unpack(workdir).unwrap();
                        } else if src_out.ends_with(".bz2") {
                            let tar_bz = File::open(src_out).unwrap();
                            let tar = BzDecoder::new(tar_bz);
                            let mut archive = Archive::new(tar);
                            archive.unpack(workdir).unwrap();
                        } else if src_out.ends_with(".zstd") {
                            let tar_zstd = File::open(src_out).unwrap();
                            let tar = Decoder::new(tar_zstd).unwrap();
                            let mut archive = Archive::new(tar);
                            archive.unpack(workdir).unwrap();
                        } else if src_out.ends_with("zip") {
                            let zip = File::open(src_out).unwrap();
                            let mut zip_archive = ZipArchive::new(zip).unwrap();
                            zip_archive.extract(workdir).unwrap();
                        }
                        Ok(workdir)
                    }
                },
            },
            SourceType::Git => {
                if self.url.is_none() {
                    eprintln!("Url is required for git source");
                    exit(78);
                }

                if self.commit.is_none() {
                    eprintln!("Commit is required for git sources");
                    exit(78);
                }

                if let Some(url) = self.url {
                    let url_path = url.path_segments().unwrap();
                    let basename = url_path.last().unwrap().replace(".git", "");
                    let out = src.join(basename);
                }

                Ok(out)
            }
            SourceType::File => {}
            SourceType::Patch => {}
        }
    }
}
