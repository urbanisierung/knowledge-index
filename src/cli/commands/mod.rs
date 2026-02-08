mod config_cmd;
mod index_cmd;
mod list_cmd;
mod rebuild_embeddings_cmd;
mod remove_cmd;
mod search_cmd;
mod update_cmd;

pub mod index {
    pub use super::index_cmd::run;
}
pub mod search {
    pub use super::search_cmd::run;
}
pub mod list {
    pub use super::list_cmd::run;
}
pub mod update {
    pub use super::update_cmd::run;
}
pub mod remove {
    pub use super::remove_cmd::run;
}
pub mod config {
    pub use super::config_cmd::run;
}
pub mod rebuild_embeddings {
    pub use super::rebuild_embeddings_cmd::run;
}

use owo_colors::OwoColorize;
use std::io::{self, IsTerminal, Write};

/// Check if colors should be used
pub fn use_colors(no_color: bool) -> bool {
    if no_color {
        return false;
    }
    // Respect NO_COLOR environment variable
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    // Check if stdout is a terminal
    std::io::stdout().is_terminal()
}

/// Print success message
pub fn print_success(msg: &str, use_colors: bool) {
    if use_colors {
        println!("{} {}", "✓".green(), msg);
    } else {
        println!("✓ {msg}");
    }
}

/// Print error message
#[allow(dead_code)]
pub fn print_error(msg: &str, use_colors: bool) {
    if use_colors {
        eprintln!("{} {}", "✗".red(), msg);
    } else {
        eprintln!("✗ {msg}");
    }
}

/// Print warning message
pub fn print_warning(msg: &str, use_colors: bool) {
    if use_colors {
        eprintln!("{} {}", "!".yellow(), msg);
    } else {
        eprintln!("! {msg}");
    }
}

/// Prompt for confirmation
pub fn confirm(prompt: &str) -> bool {
    print!("{prompt} [y/N] ");
    io::stdout().flush().ok();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
