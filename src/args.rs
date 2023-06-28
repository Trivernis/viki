use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Clone, Debug, Parser)]
#[clap(infer_subcommands = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,

    #[clap(long, short, default_value = ".")]
    pub directory: PathBuf,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Builds the project
    Build(BuildArgs),
}

#[derive(Clone, Debug, Parser)]
pub struct BuildArgs {}
