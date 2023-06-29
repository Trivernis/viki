use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use async_walkdir::WalkDir;
use futures::StreamExt;
use miette::{IntoDiagnostic, Result};
use rsass::output::Format;
use tokio::fs;

const DEFAULT_SHEET_NAME: &str = "style";
const EMBED_THRESHOLD: usize = 512;

pub struct Stylesheets {
    page_styles: HashMap<String, PathBuf>,
    processed_styles: HashMap<String, String>,
}

#[tracing::instrument(level = "trace")]
pub async fn load_stylesheets(base_dir: &PathBuf) -> Result<Stylesheets> {
    let mut entries = WalkDir::new(base_dir);
    let mut page_styles = HashMap::new();
    let empty_path = PathBuf::new();

    while let Some(res) = entries.next().await {
        match res {
            Ok(entry) => {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    let rel_path = entry_path.strip_prefix(base_dir).into_diagnostic()?;

                    if let Some(file_name) = entry_path.file_stem() {
                        let file_name = rel_path.parent().unwrap_or(&empty_path).join(file_name);
                        let file_name = file_name.to_string_lossy().into_owned();
                        page_styles.insert(file_name, entry_path.to_owned());
                    }
                }
            }
            Err(e) => return Err(e).into_diagnostic(),
        }
    }
    tracing::debug!("Styles {page_styles:?}");

    Ok(Stylesheets {
        page_styles,
        processed_styles: HashMap::new(),
    })
}

impl Stylesheets {
    #[tracing::instrument(level = "trace", skip(self, out_dir))]
    pub async fn get_style_embed(&mut self, name: &str, out_dir: &Path) -> Result<String> {
        let mut styles: Vec<String> = Vec::with_capacity(2);

        if let Some(default_style) = self
            .get_processed_style(DEFAULT_SHEET_NAME, out_dir)
            .await?
        {
            styles.push(default_style);
        }
        if let Some(style) = self.get_processed_style(name, out_dir).await? {
            styles.push(style);
        }

        Ok(styles.join(""))
    }

    #[tracing::instrument(level = "trace", skip(self, out_dir))]
    async fn get_processed_style(&mut self, name: &str, out_dir: &Path) -> Result<Option<String>> {
        if let Some(processed) = self.processed_styles.get(name) {
            Ok(Some(processed.to_owned()))
        } else if let Some(source) = self.page_styles.get(name) {
            let format = Format {
                style: rsass::output::Style::Compressed,
                ..Default::default()
            };
            let style_contents = rsass::compile_scss_path(source, format).into_diagnostic()?;
            let style_html = if style_contents.len() < EMBED_THRESHOLD {
                let utf_contents = String::from_utf8(style_contents).into_diagnostic()?;

                format!(r#"<style type="text/css">{utf_contents}</style>"#)
            } else {
                let output_path = out_dir.join(name).with_extension("css");
                let parent = output_path.parent().unwrap();
                if !parent.exists() {
                    fs::create_dir_all(parent).await.into_diagnostic()?;
                }
                fs::write(output_path, style_contents)
                    .await
                    .into_diagnostic()?;

                format!(r#"<link rel="stylesheet" href="/{name}.css">"#)
            };
            self.processed_styles
                .insert(name.to_owned(), style_html.to_owned());

            Ok(Some(style_html))
        } else {
            Ok(None)
        }
    }
}
