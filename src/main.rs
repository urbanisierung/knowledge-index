mod cli;
mod config;
mod core;
mod db;
mod error;
mod mcp;
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
            semantic,
            hybrid,
            lexical,
        } => commands::search::run(query, repo, file_type, limit, semantic, hybrid, lexical, args),
        Commands::List {} => commands::list::run(args),
        Commands::Update { path, all } => commands::update::run(path, all, args),
        Commands::Remove { path, force } => commands::remove::run(&path, force, args),
        Commands::Config { key, value, reset } => commands::config::run(key, value, reset, args),
        Commands::Mcp {} => run_mcp_server(),
    }
}

fn run_mcp_server() -> Result<()> {
    let config = config::Config::load()?;
    let db = db::Database::open()?;

    tokio::runtime::Runtime::new()
        .map_err(|e| error::AppError::Other(format!("Failed to create runtime: {e}")))?
        .block_on(mcp::run_mcp_server(db, config))
}
