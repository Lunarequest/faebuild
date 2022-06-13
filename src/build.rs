use std::{path::PathBuf, process::exit, str::FromStr};

pub fn build(path: PathBuf) {
    let mut path = path;
    if path.is_dir() {
        let pkgfile = PathBuf::from_str("FAEPKG").expect("build time bust");
        path = path.join(pkgfile);
        if !path.exists() {
            eprintln!("The path {}, does not exist", path.display());
            exit(1)
        }
    }
}
