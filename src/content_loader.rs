use std::{collections::HashMap, path::PathBuf};

use async_walkdir::{Filtering, WalkDir};
use futures::{future, StreamExt};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TemplateName(String);

impl AsRef<str> for TemplateName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Deserialize)]
pub struct Page {
    template: TemplateName,
    #[serde(flatten)]
    data: HashMap<String, toml::Value>,
}

pub struct ContentLoader {
    path: PathBuf,
}

impl ContentLoader {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    async fn load_pages(&self) -> Vec<Page> {
        todo!()
    }

    async fn find_files(&self) -> Vec<PathBuf> {
        WalkDir::new(&self.path)
            .filter(|e| async move {
                e.path()
                    .extension()
                    .map(|e| {
                        if e == "toml" {
                            Filtering::Continue
                        } else {
                            Filtering::Ignore
                        }
                    })
                    .unwrap_or(Filtering::Ignore)
            })
            .map(|e| e.expect("failed to read dir").path())
            .collect::<Vec<_>>()
            .await
    }
}

fn parse_page(path: PathBuf) {}
