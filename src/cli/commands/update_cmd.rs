use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::path::PathBuf;

use crate::cli::args::Args;
use crate::config::Config;
use crate::core::Indexer;
use crate::db::Database;
use crate::error::{AppError, Result};

use super::{print_success, print_warning, use_colors};

#[allow(clippy::too_many_lines)]
pub fn run(
    path: Option<PathBuf>,
    all: bool,
    args: &Args,
) -> Result<()> {
    let colors = use_colors(args.no_color);
    let config = Config::load()?;
    let db = Database::open()?;

    if all {
        // Update all repositories
        let repos = db.list_repositories()?;
        
        if repos.is_empty() {
            if !args.quiet && !args.json {
                print_warning("No repositories indexed. Use 'index' command first.", colors);
            }
            return Ok(());
        }

        let mut results = Vec::new();

        for repo in &repos {
            if !args.quiet && !args.json {
                if colors {
                    println!("Updating {}...", repo.name.cyan());
                } else {
                    println!("Updating {}...", repo.name);
                }
            }

            let indexer = Indexer::new(db.clone(), config.clone());
            
            match indexer.index(&repo.path, None, |_| {}) {
                Ok(result) => {
                    results.push(serde_json::json!({
                        "name": repo.name,
                        "path": repo.path.to_string_lossy(),
                        "success": true,
                        "files_added": result.files_added,
                        "files_updated": result.files_updated,
                        "files_deleted": result.files_deleted,
                    }));

                    if !args.quiet && !args.json {
                        print_success(
                            &format!(
                                "{}: +{} ~{} -{}",
                                repo.name,
                                result.files_added,
                                result.files_updated,
                                result.files_deleted
                            ),
                            colors,
                        );
                    }
                }
                Err(e) => {
                    results.push(serde_json::json!({
                        "name": repo.name,
                        "path": repo.path.to_string_lossy(),
                        "success": false,
                        "error": e.to_string(),
                    }));

                    if !args.quiet && !args.json {
                        print_warning(&format!("{}: {}", repo.name, e), colors);
                    }
                }
            }
        }

        if args.json {
            println!("{}", serde_json::json!({"results": results}));
        }
    } else {
        // Update single repository
        let path = path.ok_or_else(|| {
            AppError::Other("Specify a path or use --all to update all repositories".into())
        })?;

        let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());

        // Check if repository exists
        if db.get_repository_by_path(&canonical)?.is_none() {
            return Err(AppError::RepoNotFound(canonical));
        }

        if !args.quiet && !args.json {
            if colors {
                println!("Updating {}...", canonical.display().to_string().cyan());
            } else {
                println!("Updating {}...", canonical.display());
            }
        }

        let indexer = Indexer::new(db, config);

        let progress_bar = if !args.quiet && !args.json {
            let pb = ProgressBar::new(0);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                    .unwrap()
                    .progress_chars("█▓░"),
            );
            Some(pb)
        } else {
            None
        };

        let result = indexer.index(&canonical, None, |progress| {
            if let Some(pb) = &progress_bar {
                pb.set_length(progress.total_files as u64);
                pb.set_position(progress.processed_files as u64);
            }
        })?;

        if let Some(pb) = progress_bar {
            pb.finish_and_clear();
        }

        if args.json {
            println!(
                "{}",
                serde_json::json!({
                    "success": true,
                    "path": canonical.to_string_lossy(),
                    "files_added": result.files_added,
                    "files_updated": result.files_updated,
                    "files_deleted": result.files_deleted,
                    "files_unchanged": result.files_unchanged,
                })
            );
        } else if !args.quiet {
            print_success(
                &format!(
                    "Updated in {:.1}s: +{} added, ~{} updated, -{} deleted, {} unchanged",
                    result.elapsed_secs,
                    result.files_added,
                    result.files_updated,
                    result.files_deleted,
                    result.files_unchanged
                ),
                colors,
            );
        }
    }

    Ok(())
}
