//! Add command - add local or remote repositories

use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::path::Path;

use crate::cli::args::Args;
use crate::config::Config;
use crate::core::remote::{clone_repository, get_clone_path, parse_github_url};
use crate::core::Indexer;
use crate::db::Database;
use crate::error::Result;

use super::{print_success, print_warning, use_colors};

/// Run the add command
#[allow(clippy::too_many_lines)]
pub fn run(
    path: Option<&Path>,
    remote: Option<&str>,
    branch: Option<&str>,
    shallow: bool,
    name: Option<String>,
    args: &Args,
) -> Result<()> {
    let colors = use_colors(args.no_color);
    let config = Config::load()?;
    let db = Database::open()?;

    // Determine if this is a local or remote add
    if let Some(remote_url) = remote {
        add_remote(
            &db, &config, remote_url, branch, shallow, name, args, colors,
        )
    } else {
        // Default to current directory if no path specified
        let path = path.unwrap_or_else(|| Path::new("."));
        add_local(&db, &config, path, name, args, colors)
    }
}

/// Add a local repository
fn add_local(
    db: &Database,
    config: &Config,
    path: &Path,
    name: Option<String>,
    args: &Args,
    colors: bool,
) -> Result<()> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    if !args.quiet && !args.json {
        if colors {
            println!(
                "Adding local repository: {}",
                canonical.display().to_string().cyan()
            );
        } else {
            println!("Adding local repository: {}", canonical.display());
        }
    }

    // Check if already indexed
    if let Some(existing) = db.get_repository_by_path(&canonical)? {
        if !args.quiet && !args.json {
            print_warning(
                &format!(
                    "Repository already indexed ({} files). Updating...",
                    existing.file_count
                ),
                colors,
            );
        }
    }

    // Index the repository
    let indexer = Indexer::new(db.clone(), config.clone());

    let progress_bar = if !args.quiet && !args.json {
        let pb = ProgressBar::new(0);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
                .unwrap()
                .progress_chars("█▓░"),
        );
        Some(pb)
    } else {
        None
    };

    let result = indexer.index(&canonical, name, |progress| {
        if let Some(pb) = &progress_bar {
            pb.set_length(progress.total_files as u64);
            pb.set_position(progress.processed_files as u64);
            let display_file = if progress.current_file.len() > 40 {
                format!(
                    "...{}",
                    &progress.current_file[progress.current_file.len() - 37..]
                )
            } else {
                progress.current_file.clone()
            };
            pb.set_message(display_file);
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
                "type": "local",
                "path": canonical.to_string_lossy(),
                "files_added": result.files_added,
                "files_updated": result.files_updated,
            })
        );
    } else if !args.quiet {
        let total_files = result.files_added + result.files_updated + result.files_unchanged;
        print_success(
            &format!("Added {} files in {:.1}s", total_files, result.elapsed_secs),
            colors,
        );
    }

    Ok(())
}

/// Add a remote GitHub repository
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
fn add_remote(
    db: &Database,
    config: &Config,
    remote_url: &str,
    branch: Option<&str>,
    shallow: bool,
    name: Option<String>,
    args: &Args,
    colors: bool,
) -> Result<()> {
    // Parse the GitHub URL
    let (url, owner, repo) = parse_github_url(remote_url)?;
    let repo_name = name.unwrap_or_else(|| format!("{owner}/{repo}"));
    let clone_path = get_clone_path(&owner, &repo)?;

    if !args.quiet && !args.json {
        if colors {
            println!(
                "Adding remote repository: {}",
                format!("{owner}/{repo}").cyan()
            );
            println!("  URL: {}", url.dimmed());
            println!(
                "  Clone path: {}",
                clone_path.display().to_string().dimmed()
            );
        } else {
            println!("Adding remote repository: {owner}/{repo}");
            println!("  URL: {url}");
            println!("  Clone path: {}", clone_path.display());
        }
    }

    // Check if already exists
    if clone_path.exists() {
        if let Some(existing) = db.get_repository_by_path(&clone_path)? {
            if !args.quiet && !args.json {
                print_warning(
                    &format!(
                        "Repository already cloned ({} files). Use 'kdex sync' to update.",
                        existing.file_count
                    ),
                    colors,
                );
            }
            return Ok(());
        }
        // Path exists but not in DB - clean up and re-clone
        std::fs::remove_dir_all(&clone_path)?;
    }

    // Add to database first (with cloning status)
    db.add_remote_repository(&clone_path, &repo_name, &url, branch)?;

    // Clone the repository
    let progress_bar = if !args.quiet && !args.json {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Cloning repository...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    let clone_result = clone_repository(&url, &clone_path, branch, shallow, None);

    if let Some(pb) = &progress_bar {
        pb.finish_and_clear();
    }

    if let Err(e) = clone_result {
        // Clean up database entry on failure
        if let Some(repo) = db.get_repository_by_path(&clone_path)? {
            db.delete_repository(repo.id)?;
        }
        return Err(e);
    }

    if !args.quiet && !args.json {
        print_success("Cloned successfully", colors);
        println!();
        println!("Indexing repository...");
    }

    // Index the cloned repository
    let indexer = Indexer::new(db.clone(), config.clone());

    let progress_bar = if !args.quiet && !args.json {
        let pb = ProgressBar::new(0);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
                .unwrap()
                .progress_chars("█▓░"),
        );
        Some(pb)
    } else {
        None
    };

    let result = indexer.index(&clone_path, Some(repo_name.clone()), |progress| {
        if let Some(pb) = &progress_bar {
            pb.set_length(progress.total_files as u64);
            pb.set_position(progress.processed_files as u64);
            let display_file = if progress.current_file.len() > 40 {
                format!(
                    "...{}",
                    &progress.current_file[progress.current_file.len() - 37..]
                )
            } else {
                progress.current_file.clone()
            };
            pb.set_message(display_file);
        }
    })?;

    if let Some(pb) = progress_bar {
        pb.finish_and_clear();
    }

    // Update sync time
    if let Some(repo) = db.get_repository_by_path(&clone_path)? {
        db.update_repository_synced(repo.id)?;
    }

    if args.json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "type": "remote",
                "url": url,
                "path": clone_path.to_string_lossy(),
                "owner": owner,
                "repo": repo,
                "files_added": result.files_added,
            })
        );
    } else if !args.quiet {
        let total_files = result.files_added + result.files_updated + result.files_unchanged;
        print_success(
            &format!(
                "Added remote repository: {} ({} files in {:.1}s)",
                repo_name, total_files, result.elapsed_secs
            ),
            colors,
        );
    }

    Ok(())
}
