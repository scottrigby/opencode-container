use anyhow::Result;
use clap_complete::{generate, Shell};
use std::io::{self, Write};

pub fn run(bash: bool, zsh: bool) -> Result<()> {
    let mut app = crate::cli::app();

    if bash {
        let mut buf = Vec::new();
        generate(Shell::Bash, &mut app, "opencode-container", &mut buf);
        let mut script = String::from_utf8(buf)?;

        // clap_complete generates mismatched case labels for subcommands:
        // the for-loop sets cmd="opencode__container__subcmd__..." but
        // the case statement checks "opencode__subcmd__container__subcmd__...".
        // This regex fixes the case labels so they match.
        script = script.replace(
            "opencode__subcmd__container__subcmd__",
            "opencode__container__subcmd__",
        );

        io::stdout().write_all(script.as_bytes())?;
    } else if zsh {
        generate(
            Shell::Zsh,
            &mut app,
            "opencode-container",
            &mut io::stdout(),
        );
    } else {
        println!("Usage: opencode-container completion --bash|--zsh");
        println!();
        println!("Examples:");
        println!(
            "  opencode-container completion --bash > /etc/bash_completion.d/opencode-container"
        );
        println!("  opencode-container completion --zsh > \"${{fpath[1]}}/_opencode-container\"");
    }

    Ok(())
}
