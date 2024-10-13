use std::error::Error;

use browser::MjBrowser;
use env_logger::Env;
use winit::event_loop::EventLoop;

mod browser;
mod cli;
mod protocol;
mod webview;

// Simple struct to hold the state of the renderer

pub fn main() -> Result<(), Box<dyn Error>> {
    let env = Env::new().filter("MJ_LOG").write_style("MJ_LOG_STYLE");
    env_logger::init_from_env(env);

    let event_loop = EventLoop::new()?;
    let mut browser = MjBrowser::new()?;
    event_loop.run_app(&mut browser)?;
    Ok(())
}
