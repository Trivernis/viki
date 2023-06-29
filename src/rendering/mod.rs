use std::{path::PathBuf, sync::Arc};

use miette::{IntoDiagnostic, Result};
use tera::Tera;
use tokio::{fs, sync::Mutex};

use crate::{
    common::{SaveFile, SaveFileParams},
    context::Context,
    data::FolderData,
};

use crate::pipeline::*;

use self::style::{load_stylesheets, Stylesheets};

mod load_dir_content;
mod render_page;
mod style;

use load_dir_content::*;
use render_page::*;

// renders content using the given template folder
pub struct ContentRenderer {
    template_glob: String,
    ctx: Arc<Context>,
    styles: Arc<Mutex<Stylesheets>>,
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

        let out_dir = self.ctx.dirs.output_dir.to_owned();
        let styles = Arc::clone(&self.styles);
        let ctx = Arc::clone(&self.ctx);

        LoadDirContent
            .construct(move |(files, default_template)| {
                let step = RenderPage {
                    tera: tera.clone(),
                    styles: styles.clone(),
                    ctx: ctx.clone(),
                    default_template,
                }
                .map(map_path_to_output(out_dir.clone()))
                .chain(SaveFile)
                .parallel();

                (files, step)
            })
            .parallel()
            .process(dirs)
            .await?;

        Ok(())
    }
}

fn map_path_to_output(out_dir: PathBuf) -> impl Fn((PathBuf, String)) -> SaveFileParams {
    move |(path, contents)| {
        let path = out_dir.join(path).with_extension("html");

        SaveFileParams {
            path,
            contents: contents.into_bytes(),
        }
    }
}
