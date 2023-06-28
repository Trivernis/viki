use std::path::{Path, PathBuf};

use async_walkdir::WalkDir;
use futures::StreamExt;
use globset::{Glob, GlobSetBuilder};
use miette::{Context, IntoDiagnostic, Result};
use tokio::fs;

use super::IndexData;

/// loads directory data
pub struct DirLoader {
    base_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct FolderData {
    pub path: PathBuf,
    pub index: IndexData,
    pub pages: Vec<PathBuf>,
}

impl DirLoader {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Asynchronously reads all the entries at the given content location
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn read_content(&self) -> Result<Vec<FolderData>> {
        let mut entries = WalkDir::new(&self.base_path);
        let mut paths = Vec::new();
        paths.push(self.base_path.to_owned());

        while let Some(res) = entries.next().await {
            match res {
                Ok(entry) => {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        paths.push(entry_path)
                    }
                }
                Err(e) => return Err(e).into_diagnostic(),
            }
        }

        let results = futures::future::join_all(paths.into_iter().map(Self::read_dir)).await;
        let mut folder_data = Vec::new();

        for res in results {
            match res {
                Ok(Some(data)) => folder_data.push(data),
                Err(e) => return Err(e),
                _ => {}
            }
        }

        Ok(folder_data)
    }

    #[tracing::instrument(level = "trace")]
    async fn read_dir(path: PathBuf) -> Result<Option<FolderData>> {
        let index_path = path.join("_index.md");

        if !index_path.exists() {
            return Ok(None);
        }
        let index_data = read_index_data(&index_path).await?;
        let pages = find_pages(&path, &index_data).await?;

        Ok(Some(FolderData {
            path,
            index: index_data,
            pages,
        }))
    }
}

#[tracing::instrument(level = "trace")]
async fn read_index_data(path: &Path) -> Result<IndexData> {
    let index_str = fs::read_to_string(path)
        .await
        .into_diagnostic()
        .context("reading index file")?;
    toml::from_str(&index_str).into_diagnostic()
}

#[tracing::instrument(level = "trace")]
async fn find_pages(dir: &Path, index_data: &IndexData) -> Result<Vec<PathBuf>> {
    let include_set = build_glob_set(&index_data.include_files)
        .build()
        .into_diagnostic()?;
    let excluded_set = build_glob_set(&index_data.excluded_files)
        .build()
        .into_diagnostic()?;

    let mut read_dir = fs::read_dir(dir).await.into_diagnostic()?;
    let mut pages = Vec::new();

    while let Some(entry) = read_dir.next_entry().await.into_diagnostic()? {
        let entry_path = entry.path();

        if entry_path.is_file()
            && include_set.is_match(&entry_path)
            && !excluded_set.is_match(&entry_path)
        {
            pages.push(entry_path);
        }
    }

    Ok(pages)
}

#[tracing::instrument(level = "trace")]
fn build_glob_set(globs: &Vec<Glob>) -> GlobSetBuilder {
    let mut builder = GlobSetBuilder::new();
    globs.iter().fold(&mut builder, |b, g| b.add(g.clone()));

    builder
}
