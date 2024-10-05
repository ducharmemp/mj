use std::error::Error;

use browser::MjBrowser;
use tracing_subscriber::EnvFilter;
use winit::event_loop::EventLoop;

mod browser;
mod cli;
mod protocol;
mod webview;

// Simple struct to hold the state of the renderer

pub fn main() -> Result<(), Box<dyn Error>> {
    // Setup a bunch of state:
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_env("MJ_LOG"))
        .init();

    let event_loop = EventLoop::new()?;
    let mut browser = MjBrowser::new()?;
    event_loop.run_app(&mut browser)?;
    Ok(())
}
