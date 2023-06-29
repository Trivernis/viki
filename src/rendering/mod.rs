use std::{path::PathBuf, sync::Arc};

use futures::future;
use miette::{IntoDiagnostic, Result};
use tera::{Context as TeraContext, Tera};
use tokio::{fs, sync::Mutex};

use crate::{
    context::Context,
    data::{load_page, FolderData},
};

use self::style::{load_stylesheets, Stylesheets};

mod style;

// renders content using the given template folder
pub struct ContentRenderer {
    template_glob: String,
    ctx: Arc<Context>,
    styles: Arc<Mutex<Stylesheets>>,
}

impl ContentRenderer {
    pub async fn new(ctx: Arc<Context>) -> Result<Self> {
        let template_glob = format!("{}/**/*", ctx.template_dir.to_string_lossy());
        let styles = load_stylesheets(&ctx.stylesheet_dir).await?;

        Ok(Self {
            template_glob,
            ctx,
            styles: Arc::new(Mutex::new(styles)),
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn render_all(&self, dirs: Vec<FolderData>) -> Result<()> {
        if self.ctx.output_dir.exists() {
            fs::remove_dir_all(&self.ctx.output_dir)
                .await
                .into_diagnostic()?;
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
        let mut context = TeraContext::new();
        let mut template_name = default_template;
        let mut style_name = template_name.to_owned();

        match page {
            crate::data::Page::Data(data) => {
                if let Some(tmpl) = data.metadata.template {
                    template_name = tmpl.to_owned();
                    style_name = tmpl;
                }
                context.insert("data", &data.data);
            }
            crate::data::Page::Content(content) => context.insert("content", &content),
        }
        {
            let mut styles = self.styles.lock().await;
            let style_embed = styles
                .get_style_embed(&style_name, &self.ctx.output_dir)
                .await?;
            context.insert("style", &style_embed);
        };

        tracing::debug!("context = {context:?}");

        let html = tera
            .render(&format!("{template_name}.html"), &context)
            .into_diagnostic()?;
        let rel_path = page_path
            .strip_prefix(&self.ctx.content_dir)
            .into_diagnostic()?;
        let mut out_path = self.ctx.output_dir.join(rel_path);
        out_path.set_extension("html");
        let parent = out_path.parent().unwrap();

        if !parent.exists() {
            fs::create_dir_all(parent).await.into_diagnostic()?;
        }
        fs::write(out_path, html).await.into_diagnostic()?;

        Ok(())
    }
}
