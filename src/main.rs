use clap::Parser;
use miette::Result;
use tracing::metadata::LevelFilter;
use tracing_subscriber::fmt::format::FmtSpan;

use crate::args::Args;

mod args;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = Args::parse();
    init_tracing();

    match &args.command {
        args::Command::Build(_build_args) => {}
    }

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
