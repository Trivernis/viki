use std::{path::Path, sync::Arc};

use args::BuildArgs;
use clap::Parser;
use config::{read_config, Config};
use context::{Context, Dirs};
use data::DirLoader;
use miette::Result;
use rendering::ContentRenderer;
use tracing::metadata::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;

use crate::args::Args;

mod args;
mod config;
mod context;
pub mod data;
mod pipeline;
mod processors;
mod rendering;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();
    init_tracing();

    match &args.command {
        args::Command::Build(build_args) => {
            let cfg = read_config(&args.directory).await?;
            build(&args, &build_args, cfg).await
        }
    }
}

async fn build(args: &Args, _build_args: &BuildArgs, cfg: Config) -> Result<()> {
    let base_path = &args.directory;
    let ctx = Arc::new(build_context(&base_path, &cfg));

    let dirs = DirLoader::new(ctx.dirs.content_dir.to_owned())
        .read_content()
        .await?;

    ContentRenderer::new(ctx).await?.render_all(dirs).await?;

    Ok(())
}

fn build_context(base_path: &Path, config: &Config) -> Context {
    let folders = config.folders.clone();
    let content_dir = base_path.join(folders.content.unwrap_or("content".into()));
    let template_dir = base_path.join(folders.templates.unwrap_or("templates".into()));
    let output_dir = base_path.join(folders.output.unwrap_or("dist".into()));
    let stylesheet_dir = base_path.join(folders.stylesheets.unwrap_or("styles".into()));

    Context {
        dirs: Dirs {
            content_dir,
            template_dir,
            stylesheet_dir,
            output_dir,
        },
    }
}

fn init_tracing() {
    tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_max_level(LevelFilter::TRACE)
        .with_writer(std::io::stderr)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .compact()
        .init();
}
