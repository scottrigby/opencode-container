use anyhow::Result;
use clap::{Command, CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "opencode-container")]
#[command(about = "Run opencode in a Podman container with per-project isolation")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(trailing_var_arg = true)]
pub struct Cli {
    /// Force rebuild the container image before running
    #[arg(short, long)]
    pub build: bool,

    /// Merge .features from a JSON file into the generated devcontainer.json
    #[arg(short, long, value_name = "PATH")]
    pub feature_file: Vec<PathBuf>,

    /// Pass an environment file to the container
    #[arg(long, value_name = "PATH")]
    pub env_file: Vec<PathBuf>,

    /// Set an environment variable in the container
    #[arg(short, long, value_name = "VAR=value")]
    pub env: Vec<String>,

    /// Pass an environment variable from the host into the container
    #[arg(long, value_name = "VAR")]
    pub local_env: Vec<String>,

    /// Run in web UI mode instead of TUI
    #[arg(short, long)]
    pub web: bool,

    /// Port to listen on (web mode only)
    #[arg(short, long, default_value = "4096")]
    pub port: u16,

    /// Do not auto-open the browser (web mode only)
    #[arg(long)]
    pub no_open: bool,

    /// Mount the current working directory instead of auto-detecting the git repository root
    #[arg(long)]
    pub no_git_root: bool,

    /// Do not auto-initialise an empty git repo in non-git directories
    #[arg(long)]
    pub no_git_init: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Arguments passed through to opencode
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub opencode_args: Vec<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all projects with isolated session data
    Projects,

    /// Generate shell completion scripts
    Completion {
        /// Output bash completion script
        #[arg(long)]
        bash: bool,

        /// Output zsh completion script
        #[arg(long)]
        zsh: bool,
    },
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Commands::Projects) => crate::cmd::projects::run(),
        Some(Commands::Completion { bash, zsh }) => crate::cmd::completion::run(bash, zsh),
        None => crate::cmd::run::run(cli),
    }
}

pub fn app() -> Command {
    Cli::command()
}
