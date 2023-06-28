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
    pub content_root: PathBuf,
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

        let folder_data =
            futures::future::try_join_all(paths.into_iter().map(|p| self.read_dir(p)))
                .await?
                .into_iter()
                .filter_map(|f| f)
                .collect();

        Ok(folder_data)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    async fn read_dir(&self, path: PathBuf) -> Result<Option<FolderData>> {
        let index_path = path.join("_index.toml");

        if !index_path.exists() {
            return Ok(None);
        }
        let index_data = read_index_data(&index_path).await?;
        let pages = find_pages(&path, &index_data).await?;

        Ok(Some(FolderData {
            path,
            index: index_data,
            pages,
            content_root: self.base_path.to_owned(),
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
            && !entry_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("_")
            && include_set.is_match(&entry_path)
            && !excluded_set.is_match(&entry_path)
        {
            pages.push(entry_path);
        }
    }

    Ok(pages)
}

#[tracing::instrument(level = "trace")]
fn build_glob_set(globs: &Vec<String>) -> GlobSetBuilder {
    let mut builder = GlobSetBuilder::new();
    globs
        .iter()
        .filter_map(|pattern| Glob::new(pattern).ok())
        .fold(&mut builder, |b, g| b.add(g));

    builder
}
