use build::*;
use clap::Parser;
use cli::Commands;
use std::{path::PathBuf, process::exit, str::FromStr};
mod build;
mod cli;

fn main() {
    let cli_flags = cli::Cli::parse();
    match cli_flags.args {
        Commands::Build { path } => {
            let path = match path {
                Some(a) => a,
                None => PathBuf::from_str(".").expect("failed to convert '.' to path"),
            };
            if !path.exists() {
                eprintln!("The path {}, does not exist", path.display());
                exit(1)
            }
            build(path)
        }
    }
}
