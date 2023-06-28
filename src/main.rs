use args::BuildArgs;
use clap::Parser;
use config::{read_config, Config};
use data::DirLoader;
use miette::Result;
use rendering::ContentRenderer;
use tracing::metadata::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;

use crate::args::Args;

mod args;
mod config;
pub mod data;
mod rendering;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();
    init_tracing();

    match args.command {
        args::Command::Build(build_args) => {
            let cfg = read_config(&build_args.directory).await?;
            build(cfg, build_args).await
        }
    }
}

async fn build(cfg: Config, args: BuildArgs) -> Result<()> {
    let folders = cfg.folders;
    let base_path = args.directory;
    let content_dir = base_path.join(folders.content.unwrap_or("content".into()));
    let template_dir = base_path.join(folders.templates.unwrap_or("templates".into()));
    let out_dir = base_path.join(folders.output.unwrap_or("public".into()));

    let dirs = DirLoader::new(content_dir).read_content().await?;
    let template_glob = format!("{}/**/*", template_dir.to_string_lossy());
    ContentRenderer::new(template_glob, out_dir)
        .render_all(dirs)
        .await?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_max_level(LevelFilter::TRACE)
        .with_writer(std::io::stderr)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .compact()
        .init();
}
