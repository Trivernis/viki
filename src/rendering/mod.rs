use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use miette::{IntoDiagnostic, Result};
use tera::{Context as TeraContext, Tera};
use tokio::{fs, sync::Mutex};

use crate::{
    context::Context,
    data::{load_page, FolderData},
    pipeline::{ProcessingStep, ProcessingStepChain, ProcessingStepParallel},
};

use self::style::{load_stylesheets, Stylesheets};

mod style;

// renders content using the given template folder
pub struct ContentRenderer {
    template_glob: String,
    ctx: Arc<Context>,
    styles: Arc<Mutex<Stylesheets>>,
}

pub struct LoadDir;

#[async_trait]
impl ProcessingStep for LoadDir {
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

struct RenderPage {
    tera: Tera,
    styles: Arc<Mutex<Stylesheets>>,
    ctx: Arc<Context>,
}

#[async_trait]
impl ProcessingStep for RenderPage {
    type Input = (PathBuf, String);
    type Output = (PathBuf, String);

    #[tracing::instrument(name = "render page", level = "trace", skip_all)]
    async fn process(&self, (page_path, default_template): Self::Input) -> Result<Self::Output> {
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
                .get_style_embed(&style_name, &self.ctx.dirs.output_dir)
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

pub struct SaveOutput {
    out_dir: PathBuf,
    extension: &'static str,
}

#[async_trait]
impl ProcessingStep for SaveOutput {
    type Input = (PathBuf, String);
    type Output = ();

    #[tracing::instrument(name = "save output", level = "trace", skip_all)]
    async fn process(&self, (rel_path, content): Self::Input) -> Result<Self::Output> {
        let mut out_path = self.out_dir.join(rel_path);
        out_path.set_extension(self.extension);
        let parent = out_path.parent().unwrap();

        if !parent.exists() {
            fs::create_dir_all(parent).await.into_diagnostic()?;
        }
        fs::write(out_path, content).await.into_diagnostic()?;

        Ok(())
    }
}

impl ContentRenderer {
    pub async fn new(ctx: Arc<Context>) -> Result<Self> {
        let template_glob = format!("{}/**/*", ctx.dirs.template_dir.to_string_lossy());
        let styles = load_stylesheets(&ctx.dirs.stylesheet_dir).await?;

        Ok(Self {
            template_glob,
            ctx,
            styles: Arc::new(Mutex::new(styles)),
        })
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn render_all(&self, dirs: Vec<FolderData>) -> Result<()> {
        if self.ctx.dirs.output_dir.exists() {
            fs::remove_dir_all(&self.ctx.dirs.output_dir)
                .await
                .into_diagnostic()?;
        }
        let mut tera = Tera::new(&self.template_glob).into_diagnostic()?;
        super::processors::register_all(&mut tera);

        LoadDir
            .chain(
                RenderPage {
                    tera,
                    styles: self.styles.clone(),
                    ctx: self.ctx.clone(),
                }
                .chain(SaveOutput {
                    out_dir: self.ctx.dirs.output_dir.to_owned(),
                    extension: "html",
                })
                .parallel(),
            )
            .parallel()
            .process(dirs)
            .await?;

        Ok(())
    }
}
