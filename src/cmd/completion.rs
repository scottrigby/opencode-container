use anyhow::Result;
use clap_complete::{generate, Shell};
use std::io;

pub fn run(bash: bool, zsh: bool) -> Result<()> {
    let mut app = crate::cli::app();

    if bash {
        generate(Shell::Bash, &mut app, "opencode-container", &mut io::stdout());
    } else if zsh {
        generate(Shell::Zsh, &mut app, "opencode-container", &mut io::stdout());
    } else {
        println!("Usage: opencode-container completion --bash|--zsh");
        println!();
        println!("Examples:");
        println!("  opencode-container completion --bash > /etc/bash_completion.d/opencode-container");
        println!("  opencode-container completion --zsh > \"${{fpath[1]}}/_opencode-container\"");
    }

    Ok(())
}
