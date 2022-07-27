use super::recipe::Recpie;
use serde_yaml::from_reader;
use std::{fs::File, path::PathBuf, process::exit, str::FromStr};

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

    let recipe_file = match File::open(&path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open {}, rust std error {e}", path.display());
            exit(2);
        }
    };
    let recpie = match from_reader::<File, Recpie>(recipe_file) {
        Ok(recipe) => recipe,
        Err(e) => {
            eprintln!("A error occured reading the recipe {e}");
            exit(1);
        }
    };
    for source in recpie.build.sources {
        match source.r#type.as_str() {
            "git" => {}
            _ => {
                eprintln!("{} not valid source", source.r#type);
                exit(1);
            }
        }
    }
}
