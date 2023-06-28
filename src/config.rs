use std::path::{Path, PathBuf};

use miette::{IntoDiagnostic, Result};
use serde::Deserialize;
use tokio::fs;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub folders: Folders,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Folders {
    pub content: Option<PathBuf>,
    pub templates: Option<PathBuf>,
    pub stylesheets: Option<PathBuf>,
    pub output: Option<PathBuf>,
}

pub async fn read_config(dir: &Path) -> Result<Config> {
    let cfg_string = fs::read_to_string(dir.join("viki.toml"))
        .await
        .into_diagnostic()?;
    toml::from_str(&cfg_string).into_diagnostic()
}
