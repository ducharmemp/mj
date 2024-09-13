use std::error::Error;

use browser::{MjBrowser, NavigateTo};
use clap::Parser;
use cli::MjCliArgs;
use ractor::{cast, Actor};

#[macro_use]
extern crate html5ever;

mod browser;
mod cli;
mod dom;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = MjCliArgs::parse();
    let (actor, handle) = Actor::spawn(None, MjBrowser, ()).await?;

    cast!(actor, NavigateTo(args.url))?;

    handle.await?;
    Ok(())
}
