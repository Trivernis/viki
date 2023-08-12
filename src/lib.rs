use std::path::PathBuf;

use config::ConfigLoader;

use miette::Result;

mod config;
mod content_loader;

#[derive(Debug)]
pub struct Paths {
    pub config: PathBuf,
}

pub struct Viki {
    config_loader: ConfigLoader,
}

impl Viki {
    #[tracing::instrument(level = "trace")]
    pub async fn load(paths: Paths) -> Result<Self> {
        let config_loader = ConfigLoader::load(paths.config).await?;

        Ok(Self { config_loader })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn reload(&mut self) -> Result<()> {
        self.config_loader.reload().await?;

        Ok(())
    }
}
