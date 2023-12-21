mod buildconfig;
mod cli;
mod utils;
use anyhow::{anyhow, Result};
use buildconfig::BuildConfig;
use clap::Parser;
use cli::{Cli, Commands};
use serde_yaml::from_reader;
use std::{
    fs::{create_dir, remove_dir_all, File},
    path::PathBuf,
};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Build { path } => {
            let builddir = path.unwrap_or(PathBuf::from(".")).canonicalize()?;
            if builddir.exists() {
                let buildconfig = builddir.join("faebuild.yaml");
                if !buildconfig.exists() {
                    return Err(anyhow!("failed to find faebuild.yaml, does it exist?"));
                }

                let workdir = builddir.join("build");
                let srcdir = builddir.join("src");
                if workdir.exists() {
                    remove_dir_all(&workdir).unwrap();
                }
                create_dir(&workdir).expect("failed to create dir build");
                if !srcdir.exists() {
                    create_dir(&srcdir).expect("failed to create directory src");
                }

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
                if !patches.is_empty() {
                    utils::patch(patches, &workdir)?;
                }
            } else {
                if args.verbose {
                    eprintln!("DEBUG RESOLVED DIR: {}", builddir.display());
                }
                return Err(anyhow!("failed to find directory, does it exist?"));
            }
        }
    }
    Ok(())
}
