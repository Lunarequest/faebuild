use super::recipe::Recpie;
use super::src;
use serde_yaml::from_reader;
use std::{
    env,
    fs::{create_dir_all, File},
    path::PathBuf,
    process::exit,
    str::FromStr,
};

pub fn build(path: PathBuf) {
    let mut path = path;
    if path.is_dir() {
        env::set_current_dir(&path).unwrap();
    }
    let pkgfile = PathBuf::from_str("FAEPKG").expect("build time bust");
    if !pkgfile.exists() {
        eprintln!("The path {}, does not exist", &path.display());
        exit(1)
    }

    let recipe_file = match File::open("FAEPKG") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open {}, rust std error {e}", path.display());
            exit(2);
        }
    };
    let srcdir = PathBuf::from_str("src").unwrap();
    if !srcdir.exists() {
        create_dir_all(&srcdir).unwrap();
    }
    env::set_current_dir(srcdir).unwrap();
    let recpie = match from_reader::<File, Recpie>(recipe_file) {
        Ok(recipe) => recipe,
        Err(e) => {
            eprintln!("A error occured reading the recipe {e}");
            exit(1);
        }
    };
    for source in recpie.build.sources {
        match source.r#type.as_str() {
            "git" => {
                src::git::git_solver(source);
            }
            _ => {
                eprintln!("{} not valid source", source.r#type);
                exit(1);
            }
        }
    }
}
