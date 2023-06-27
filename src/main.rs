mod buildconfig;
mod cli;
mod utils;
use buildconfig::BuildConfig;
use clap::Parser;
use cli::{Cli, Commands};
use serde_yaml::from_reader;
use std::{
    fs::{create_dir, File},
    path::PathBuf,
    process::exit,
};

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Build { path } => {
            let builddir = match path.unwrap_or(PathBuf::from(".")).canonicalize() {
                Ok(config) => config,
                Err(e) => {
                    if args.verbose {
                        eprintln!("DEBUG TRACE BACK: {e}");
                    }
                    eprintln!("failed to resolve directory, does it exist?");
                    exit(72);
                }
            };
            if builddir.exists() {
                let buildconfig = builddir.join("faebuild.yaml");
                if buildconfig.exists() {
                } else {
                    eprintln!("failed to find faebuild.yaml, does it exist?");
                    exit(72);
                }

                let workdir = builddir.join("build");
                let srcdir = builddir.join("src");
                create_dir(&workdir).expect("failed to create dir build");
                create_dir(&srcdir).expect("failed to create directory src");

                let file = File::open(buildconfig).unwrap();

                let config: BuildConfig = from_reader(file).unwrap();
                let mut patches: Vec<PathBuf> = vec![];

                for source in config.sources {
                    match source.r#type {
                        buildconfig::SourceType::Patch => {
                            let path = source.fetch(&srcdir, &workdir).await.unwrap();
                            patches.append(&mut vec![path]);
                        }
                        _ => {
                            source.fetch(&srcdir, &workdir).await.unwrap();
                        }
                    }
                }
            } else {
                if args.verbose {
                    eprintln!("DEBUG RESOLVED DIR: {}", builddir.display());
                }
                eprintln!("failed to find directory, does it exist?");
                exit(72);
            }
        }
    }
}
