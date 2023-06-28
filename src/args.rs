use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Clone, Debug, Parser)]
#[clap(infer_subcommands = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Builds the project
    Build(BuildArgs),
}

#[derive(Clone, Debug, Parser)]
pub struct BuildArgs {
    #[clap(default_value = ".")]
    pub directory: PathBuf,
}
