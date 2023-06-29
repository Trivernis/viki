use std::path::PathBuf;

use async_trait::async_trait;
use miette::{IntoDiagnostic, Result};
use tokio::fs;

use crate::pipeline::ProcessingStep;

pub struct SaveFile;

pub struct SaveFileParams {
    pub path: PathBuf,
    pub contents: Vec<u8>,
}

#[async_trait]
impl ProcessingStep for SaveFile {
    type Input = SaveFileParams;
    type Output = ();

    #[tracing::instrument(name = "save file", level = "trace", skip_all)]
    async fn process(
        &self,
        SaveFileParams { path, contents }: Self::Input,
    ) -> Result<Self::Output> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await.into_diagnostic()?;
            }
        }

        fs::write(path, contents).await.into_diagnostic()?;

        Ok(())
    }
}
