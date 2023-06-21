use clap::{Parser, Subcommand};
use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Parser)]
#[command(author="Luna D Dragon <luna@nullrequest.com>", 
          version=VERSION, 
          about="faebuild a tool to build faepkgs", 
          long_about = None)]
pub struct Cli {
    #[arg(short='v', long="verbose")]
    pub verbose: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug,Subcommand)]
pub enum Commands {
    #[command(alias="b")]
    Build {
        path: Option<PathBuf>
    }
}