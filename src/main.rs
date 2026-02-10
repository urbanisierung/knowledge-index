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
    let args = Args::parse();

    if let Err(e) = run_with_args(&args) {
        if args.debug {
            eprintln!("Error: {e:?}");
        } else {
            eprintln!("Error: {e}");
            eprintln!("Run with --debug for more details.");
        }
        std::process::exit(1);
    }
}

fn run_with_args(args: &Args) -> Result<()> {
    // Enable backtraces in debug mode
    if args.debug {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    match &args.command {
        Some(cmd) => run_command(cmd.clone(), args),
        None => {
            // No subcommand: if TTY, launch TUI; otherwise show help
            if is(atty::Stream::Stdout) && is(atty::Stream::Stdin) {
                tui::run()
            } else {
                Args::parse_from(["kdex", "--help"]);
                Ok(())
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
fn run_command(cmd: Commands, args: &Args) -> Result<()> {
    match cmd {
        Commands::Index { path, name } => commands::index::run(&path, name, args),
        Commands::Add {
            path,
            remote,
            branch,
            shallow,
            name,
        } => commands::add::run(
            path.as_deref(),
            remote.as_deref(),
            branch.as_deref(),
            shallow,
            name,
            args,
        ),
        Commands::Search {
            query,
            repo,
            file_type,
            limit,
            group_by_repo,
            semantic,
            hybrid,
            lexical,
        } => commands::search::run(
            query,
            repo,
            file_type,
            limit,
            group_by_repo,
            semantic,
            hybrid,
            lexical,
            args,
        ),
        Commands::List {} => commands::list::run(args),
        Commands::Update { path, all } => commands::update::run(path, all, args),
        Commands::Sync { repo, no_index } => commands::sync::run(repo.as_deref(), no_index, args),
        Commands::Remove { path, force } => commands::remove::run(&path, force, args),
        Commands::Config {
            action,
            key,
            value,
            reset,
        } => commands::config::run(action, key, value, reset, args),
        Commands::Mcp {} => run_mcp_server(),
        Commands::Watch { all, path } => run_watcher(all, path, args),
        Commands::RebuildEmbeddings { repo } => commands::rebuild_embeddings::run(repo, args),
    }
}

fn run_watcher(all: bool, path: Option<std::path::PathBuf>, args: &Args) -> Result<()> {
    use crate::core::{check_inotify_limit, estimate_directory_count, IndexWatcher};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    let db = db::Database::open()?;
    let config = Arc::new(config::Config::load()?);

    let repos = if all {
        db.list_repositories()?
    } else if let Some(p) = path {
        let abs_path = std::fs::canonicalize(&p)?;
        db.list_repositories()?
            .into_iter()
            .filter(|r| r.path == abs_path)
            .collect()
    } else {
        let cwd = std::env::current_dir()?;
        db.list_repositories()?
            .into_iter()
            .filter(|r| r.path == cwd)
            .collect()
    };

    if repos.is_empty() {
        if !args.quiet {
            eprintln!("No repositories to watch. Index a directory first.");
        }
        return Ok(());
    }

    // Check platform limits (Linux inotify)
    if !args.quiet {
        let total_dirs: usize = repos
            .iter()
            .filter_map(|r| estimate_directory_count(&r.path).ok())
            .sum();

        let limits = check_inotify_limit(total_dirs);
        if let Some(warning) = limits.warning {
            eprintln!("{warning}");
            eprintln!();
        }
    }

    if !args.quiet {
        println!(
            "Watching {} repositor{} for changes...",
            repos.len(),
            if repos.len() == 1 { "y" } else { "ies" }
        );
        for repo in &repos {
            println!("  • {}", repo.path.display());
        }
        println!("Press Ctrl+C to stop.");
    }

    let mut watcher = IndexWatcher::new(config)?;

    // Add all repository paths to watch
    for repo in &repos {
        watcher.watch(repo.path.clone())?;
    }

    // Main watch loop
    loop {
        let batches = watcher.poll_changes();

        for batch in batches {
            if !args.quiet {
                println!("Changes detected in {}:", batch.repo_path.display());
            }

            for change in &batch.changes {
                if !args.quiet {
                    let action = match change.change_type {
                        crate::core::ChangeType::Created => "created",
                        crate::core::ChangeType::Modified => "modified",
                        crate::core::ChangeType::Deleted => "deleted",
                    };
                    println!("  {} {}", action, change.path.display());
                }
            }

            // Re-index the changed repository
            if let Some(repo) = repos.iter().find(|r| r.path == batch.repo_path) {
                let indexer_config = config::Config::load()?;
                let indexer_db = db::Database::open()?;
                let indexer = crate::core::Indexer::new(indexer_db, indexer_config);

                match indexer.index(&repo.path, Some(repo.name.clone()), |_| {}) {
                    Ok(result) => {
                        if !args.quiet {
                            println!(
                                "  ✓ Re-indexed: {} added, {} updated, {} deleted",
                                result.files_added, result.files_updated, result.files_deleted
                            );
                        }
                    }
                    Err(e) => {
                        if !args.quiet {
                            eprintln!("  ✗ Failed to re-index: {e}");
                        }
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

fn run_mcp_server() -> Result<()> {
    let config = config::Config::load()?;
    let db = db::Database::open()?;

    tokio::runtime::Runtime::new()
        .map_err(|e| error::AppError::Other(format!("Failed to create runtime: {e}")))?
        .block_on(mcp::run_mcp_server(db, config))
}
