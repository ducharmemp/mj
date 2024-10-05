use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct MjCliArgs {
    #[arg(short, long)]
    pub url: String,
}
