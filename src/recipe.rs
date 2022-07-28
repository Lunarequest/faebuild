use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Recpie {
    pub name: String,
    pub pkgver: String,
    pub rel: String,
    pub desc: String,
    pub arch: Vec<String>,
    pub url: String,
    pub license: Vec<String>,
    pub makedepends: Option<Vec<String>>,
    pub depends: Vec<String>,
    pub build: Build,
}

#[derive(Debug, Deserialize)]
pub struct Build {
    pub buildsystem: String,
    pub build_commands: Option<Vec<String>>,
    pub config_opts: Option<String>,
    pub sources: Vec<Sources>,
}

#[derive(Debug, Deserialize)]
pub struct Sources {
    pub r#type: String,
    pub url: Option<String>,
    pub path: Option<String>,
    pub commit: Option<String>,
    pub recursive: Option<bool>,
    pub tag: Option<String>,
}
