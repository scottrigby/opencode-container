use anyhow::Result;
use clap::Parser;

mod cli;
mod cmd;
mod util;

fn main() -> Result<()> {
    let args = cli::Cli::parse();
    cli::run(args)
}
