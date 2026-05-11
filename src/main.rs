use anyhow::Result;
use clap::Parser;

mod cli;
mod cmd;
mod util;

fn main() -> Result<()> {
    let args = preprocess_args();
    let cli = cli::Cli::parse_from(args);
    cli::run(cli)
}

/// Preprocess raw args to support the legacy `--` passthrough syntax.
///
/// When no subcommand is present and `--` appears in the arg list,
/// inserts `run` before `--` so clap routes the trailing args to the
/// Run subcommand's `opencode_args`.
fn preprocess_args() -> Vec<String> {
    let mut args: Vec<String> = std::env::args().collect();
    let known_subcommands = ["run", "projects", "completion", "help"];

    // Check if a known subcommand already appears before any `--`
    let subcommand_present = args
        .iter()
        .skip(1)
        .take_while(|a| a.as_str() != "--")
        .any(|a| known_subcommands.contains(&a.as_str()));

    if !subcommand_present {
        if let Some(pos) = args.iter().position(|a| a == "--") {
            args.insert(pos, "run".to_string());
        }
    }

    args
}
