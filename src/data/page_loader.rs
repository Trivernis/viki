use std::path::Path;

use miette::{Context, IntoDiagnostic, Result};
use tokio::fs;

use super::Page;

pub struct PageLoader;

/// loads a page and parses the data depending on the extension
#[tracing::instrument(level = "trace")]
pub async fn load_page(path: &Path) -> Result<Page> {
    let string_content = load_string_content(path).await?;

    if let Some(extension) = path.extension() {
        let extension_lower = extension.to_string_lossy().to_lowercase();
        match extension_lower.as_str() {
            "toml" => Ok(Page::Data(
                toml::from_str(&string_content).into_diagnostic()?,
            )),
            _ => Ok(Page::Content(string_content)),
        }
    } else {
        Ok(Page::Content(string_content))
    }
}

#[tracing::instrument(level = "trace")]
async fn load_string_content(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .await
        .into_diagnostic()
        .context("reading page content")
}
