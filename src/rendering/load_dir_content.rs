use std::path::PathBuf;

use async_trait::async_trait;
use miette::Result;

use crate::{data::FolderData, pipeline::ProcessingStep};

pub struct LoadDirContent;

#[async_trait]
impl ProcessingStep for LoadDirContent {
    type Input = FolderData;
    type Output = Vec<(PathBuf, String)>;

    #[tracing::instrument(name = "load dir", level = "trace", skip_all)]
    async fn process(&self, input: Self::Input) -> Result<Self::Output> {
        let dir_name = input
            .path
            .components()
            .last()
            .unwrap()
            .as_os_str()
            .to_string_lossy();
        let default_template = input
            .index
            .default_template
            .to_owned()
            .unwrap_or(dir_name.into());

        Ok(input
            .pages
            .into_iter()
            .map(|p| (p, default_template.clone()))
            .collect())
    }
}
