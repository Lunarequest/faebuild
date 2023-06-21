mod buildconfig;
mod cli;
mod utils;
use std::{path::PathBuf, process::exit};

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
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
