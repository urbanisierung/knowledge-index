//! Sync command - sync remote repositories with their origins

use owo_colors::OwoColorize;

use crate::cli::args::Args;
use crate::config::Config;
use crate::core::remote::sync_repository;
use crate::core::Indexer;
use crate::db::{Database, RepoStatus, SourceType};
use crate::error::Result;

use super::{print_success, print_warning, use_colors};

/// Run the sync command
#[allow(clippy::too_many_lines)]
pub fn run(repo_filter: Option<&str>, no_index: bool, args: &Args) -> Result<()> {
    let colors = use_colors(args.no_color);
    let config = Config::load()?;
    let db = Database::open()?;

    // Get remote repositories
    let remote_repos = db.get_remote_repositories()?;

    if remote_repos.is_empty() {
        if !args.quiet && !args.json {
            print_warning(
                "No remote repositories to sync. Add one with: kdex add --remote owner/repo",
                colors,
            );
        }
        return Ok(());
    }

    // Filter by repo name if specified
    let repos_to_sync: Vec<_> = if let Some(filter) = repo_filter {
        remote_repos
            .into_iter()
            .filter(|r| r.name.contains(filter) || r.path.to_string_lossy().contains(filter))
            .collect()
    } else {
        remote_repos
    };

    if repos_to_sync.is_empty() {
        if !args.quiet && !args.json {
            print_warning(
                &format!(
                    "No remote repositories matching '{}'",
                    repo_filter.unwrap_or("")
                ),
                colors,
            );
        }
        return Ok(());
    }

    if !args.quiet && !args.json {
        println!(
            "Syncing {} remote repositor{}...",
            repos_to_sync.len(),
            if repos_to_sync.len() == 1 { "y" } else { "ies" }
        );
    }

    let mut synced = 0;
    let mut updated = 0;
    let mut failed = 0;

    for repo in &repos_to_sync {
        if !args.quiet && !args.json {
            if colors {
                print!("  {} ", repo.name.cyan());
            } else {
                print!("  {} ", repo.name);
            }
        }

        // Update status to syncing
        db.update_repository_status(repo.id, RepoStatus::Syncing)?;

        // Sync the repository
        let branch = repo.remote_branch.as_deref();
        match sync_repository(&repo.path, branch) {
            Ok(had_changes) => {
                synced += 1;

                if had_changes {
                    updated += 1;
                    if !args.quiet && !args.json {
                        if colors {
                            println!("{}", "updated".green());
                        } else {
                            println!("updated");
                        }
                    }

                    // Re-index if not skipped
                    if !no_index {
                        if !args.quiet && !args.json {
                            print!("    Re-indexing... ");
                        }

                        let indexer = Indexer::new(db.clone(), config.clone());
                        match indexer.index(&repo.path, Some(repo.name.clone()), |_| {}) {
                            Ok(result) => {
                                if !args.quiet && !args.json {
                                    let total = result.files_added + result.files_updated;
                                    if colors {
                                        println!("{}", format!("{total} files").green());
                                    } else {
                                        println!("{total} files");
                                    }
                                }
                            }
                            Err(e) => {
                                if !args.quiet && !args.json {
                                    if colors {
                                        println!("{}: {}", "error".red(), e);
                                    } else {
                                        println!("error: {e}");
                                    }
                                }
                            }
                        }
                    }
                } else if !args.quiet && !args.json {
                    if colors {
                        println!("{}", "up to date".dimmed());
                    } else {
                        println!("up to date");
                    }
                }

                // Update sync time
                db.update_repository_synced(repo.id)?;
            }
            Err(e) => {
                failed += 1;
                db.update_repository_status(repo.id, RepoStatus::Error)?;

                if !args.quiet && !args.json {
                    if colors {
                        println!("{}: {}", "failed".red(), e);
                    } else {
                        println!("failed: {e}");
                    }
                }
            }
        }
    }

    // Summary
    if args.json {
        println!(
            "{}",
            serde_json::json!({
                "success": failed == 0,
                "synced": synced,
                "updated": updated,
                "failed": failed,
            })
        );
    } else if !args.quiet {
        println!();
        if failed == 0 {
            print_success(
                &format!(
                    "Synced {} repositor{} ({} updated)",
                    synced,
                    if synced == 1 { "y" } else { "ies" },
                    updated
                ),
                colors,
            );
        } else {
            print_warning(&format!("Synced {synced}, failed {failed}",), colors);
        }
    }

    Ok(())
}

/// Background sync for stale remote repositories (called during search)
#[allow(dead_code)]
pub fn background_sync(db: &Database, config: &Config, stale_minutes: i64) -> Result<()> {
    use chrono::Utc;
    use std::thread;

    let remote_repos = db.get_remote_repositories()?;
    let now = Utc::now();

    let stale_repos: Vec<_> = remote_repos
        .into_iter()
        .filter(|r| {
            if let Some(last_sync) = r.last_synced_at {
                let elapsed = now.signed_duration_since(last_sync);
                elapsed.num_minutes() > stale_minutes
            } else {
                true // Never synced
            }
        })
        .collect();

    if stale_repos.is_empty() {
        return Ok(());
    }

    // Clone what we need for the background thread
    let db = db.clone();
    let config = config.clone();

    // Spawn background sync thread
    thread::spawn(move || {
        for repo in stale_repos {
            if repo.source_type != SourceType::Remote {
                continue;
            }

            let branch = repo.remote_branch.as_deref();
            if let Ok(true) = sync_repository(&repo.path, branch) {
                // Re-index on changes
                let indexer = Indexer::new(db.clone(), config.clone());
                let _ = indexer.index(&repo.path, Some(repo.name.clone()), |_| {});
                let _ = db.update_repository_synced(repo.id);
            }
        }
    });

    Ok(())
}
