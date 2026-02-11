//! Shell completions generation command.

use crate::cli::args::{Args, Shell};
use clap::CommandFactory;
use clap_complete::{generate, Shell as ClapShell};
use std::io;

/// Generate shell completions
pub fn run(shell: Shell) {
    let mut cmd = Args::command();
    let clap_shell = match shell {
        Shell::Bash => ClapShell::Bash,
        Shell::Zsh => ClapShell::Zsh,
        Shell::Fish => ClapShell::Fish,
        Shell::PowerShell => ClapShell::PowerShell,
        Shell::Elvish => ClapShell::Elvish,
    };

    generate(clap_shell, &mut cmd, "kdex", &mut io::stdout());
}
