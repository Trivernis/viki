use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use miette::{IntoDiagnostic, Result};
use serde::Deserialize;
use std::sync::{Arc, RwLock};
use tokio::fs;

lazy_static! {
    static ref CONFIG: Arc<RwLock<Option<Config>>> = Arc::new(RwLock::new(None));
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub folders: Folders,
}

impl Config {
    pub fn get() -> Self {
        CONFIG
            .read()
            .unwrap()
            .clone()
            .expect("Config hasn't been read yet")
    }
}

/// Accessor struct to load the config into the global config variable
/// and later reload it when required
pub struct ConfigLoader {
    dir: PathBuf,
}

impl ConfigLoader {
    pub async fn load(dir: PathBuf) -> Result<Self> {
        Self::load_config(&dir).await?;

        Ok(Self { dir })
    }

    pub async fn reload(&self) -> Result<()> {
        Self::load_config(&self.dir).await
    }

    async fn load_config(dir: &Path) -> Result<()> {
        let config = read_config(dir).await?;
        *CONFIG.write().unwrap() = Some(config);

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Folders {
    pub content: Option<PathBuf>,
    pub templates: Option<PathBuf>,
    pub stylesheets: Option<PathBuf>,
    pub output: Option<PathBuf>,
}

async fn read_config(dir: &Path) -> Result<Config> {
    let cfg_string = fs::read_to_string(dir.join("viki.toml"))
        .await
        .into_diagnostic()?;
    toml::from_str(&cfg_string).into_diagnostic()
}
