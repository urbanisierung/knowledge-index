mod cli;
mod config;
mod core;
mod db;
mod error;
mod tui;

use atty::is;
use clap::Parser;
use cli::args::{Args, Commands};
use cli::commands;
use error::Result;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(ref cmd) => run_command(cmd.clone(), &args),
        None => {
            // No subcommand: if TTY, launch TUI; otherwise show help
            if is(atty::Stream::Stdout) && is(atty::Stream::Stdin) {
                tui::run()
            } else {
                Args::parse_from(["knowledge-index", "--help"]);
                Ok(())
            }
        }
    }
}

fn run_command(cmd: Commands, args: &Args) -> Result<()> {
    match cmd {
        Commands::Index { path, name } => commands::index::run(&path, name, args),
        Commands::Search {
            query,
            repo,
            file_type,
            limit,
        } => commands::search::run(query, repo, file_type, limit, args),
        Commands::List {} => commands::list::run(args),
        Commands::Update { path, all } => commands::update::run(path, all, args),
        Commands::Remove { path, force } => commands::remove::run(&path, force, args),
        Commands::Config { key, value, reset } => commands::config::run(key, value, reset, args),
        Commands::Mcp {} => {
            eprintln!("MCP server not yet implemented");
            Ok(())
        }
    }
}
