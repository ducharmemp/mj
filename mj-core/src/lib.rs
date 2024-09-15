use std::error::Error;

use browser::MjBrowser;
use clap::Parser;
use cli::MjCliArgs;
use protocol::handler::MjProtocolHandler;
use ractor::Actor;
use tracing_subscriber::EnvFilter;
use url::Url;

mod browser;
mod cli;
mod dom;
mod protocol;
mod webview;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_env("MJ_LOG"))
        .init();
    let args = MjCliArgs::parse();
    Actor::spawn(
        Some("mj:protocol_handler".to_string()),
        MjProtocolHandler,
        (),
    )
    .await?;
    let (_browser_actor, handle) = Actor::spawn(
        None,
        MjBrowser,
        Some(Url::parse(&args.url).expect("Could not parse url")),
    )
    .await?;

    handle.await?;
    Ok(())
}
