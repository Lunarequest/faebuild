use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author="Luna D. Dragon", version="0.1.0", about="", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub args: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Build { path: Option<PathBuf> },
}
