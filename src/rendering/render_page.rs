use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use miette::{IntoDiagnostic, Result};
use tera::{Context as TeraContext, Tera};
use tokio::sync::Mutex;

use crate::{context::Context, data::load_page, pipeline::ProcessingStep};

use super::style::Stylesheets;

pub struct RenderPage {
    pub tera: Tera,
    pub styles: Arc<Mutex<Stylesheets>>,
    pub ctx: Arc<Context>,
    pub default_template: String,
}

#[async_trait]
impl ProcessingStep for RenderPage {
    type Input = PathBuf;
    type Output = (PathBuf, String);

    #[tracing::instrument(name = "render page", level = "trace", skip_all)]
    async fn process(&self, page_path: Self::Input) -> Result<Self::Output> {
        let page = load_page(&page_path).await?;
        let mut context = TeraContext::new();
        let mut template_name = None;

        match page {
            crate::data::Page::Data(data) => {
                template_name = data.metadata.template;
                context.insert("data", &data.data);
            }
            crate::data::Page::Content(content) => context.insert("content", &content),
        }
        let template_name = template_name.as_ref().unwrap_or(&self.default_template);
        {
            let mut styles = self.styles.lock().await;
            let style_embed = styles
                .get_style_embed(template_name, &self.ctx.dirs.output_dir)
                .await?;
            context.insert("style", &style_embed);
        };

        tracing::debug!("context = {context:?}");

        let html = self
            .tera
            .render(&format!("{template_name}.html"), &context)
            .into_diagnostic()?;
        let rel_path = page_path
            .strip_prefix(&self.ctx.dirs.content_dir)
            .into_diagnostic()?;

        Ok((rel_path.to_owned(), html))
    }
}
