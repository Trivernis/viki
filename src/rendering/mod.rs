use std::path::PathBuf;

use futures::future;
use miette::{IntoDiagnostic, Result};
use tera::{Context, Tera};
use tokio::fs;

use crate::data::{load_page, FolderData};

// renders content using the given template folder
pub struct ContentRenderer {
    template_glob: String,
    out_dir: PathBuf,
    content_dir: PathBuf,
}

impl ContentRenderer {
    pub fn new(template_glob: String, content_dir: PathBuf, out_dir: PathBuf) -> Self {
        Self {
            template_glob,
            content_dir,
            out_dir,
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn render_all(&self, dirs: Vec<FolderData>) -> Result<()> {
        if self.out_dir.exists() {
            fs::remove_dir_all(&self.out_dir).await.into_diagnostic()?;
        }
        let mut tera = Tera::new(&self.template_glob).into_diagnostic()?;
        super::processors::register_all(&mut tera);
        future::try_join_all(dirs.into_iter().map(|data| self.render_folder(&tera, data))).await?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn render_folder(&self, tera: &Tera, data: FolderData) -> Result<()> {
        let dir_name = data
            .path
            .components()
            .last()
            .unwrap()
            .as_os_str()
            .to_string_lossy();
        let default_template = data
            .index
            .default_template
            .to_owned()
            .unwrap_or(dir_name.into());

        future::try_join_all(
            data.pages
                .into_iter()
                .map(|page| self.render_page(tera, default_template.clone(), page)),
        )
        .await?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn render_page(
        &self,
        tera: &Tera,
        default_template: String,
        page_path: PathBuf,
    ) -> Result<()> {
        tracing::debug!("Rendering {page_path:?}");

        let page = load_page(&page_path).await?;
        let mut context = Context::new();
        let mut template_name = default_template;

        match page {
            crate::data::Page::Data(data) => {
                if let Some(tmpl) = data.metadata.template {
                    template_name = tmpl;
                }
                context.insert("data", &data.data);
            }
            crate::data::Page::Content(content) => context.insert("content", &content),
        }

        tracing::debug!("context = {context:?}");

        let html = tera.render(&template_name, &context).into_diagnostic()?;
        let rel_path = page_path
            .strip_prefix(&self.content_dir)
            .into_diagnostic()?;
        let mut out_path = self.out_dir.join(rel_path);
        out_path.set_extension("html");
        let parent = out_path.parent().unwrap();

        if !parent.exists() {
            fs::create_dir_all(parent).await.into_diagnostic()?;
        }
        fs::write(out_path, html).await.into_diagnostic()?;

        Ok(())
    }
}
