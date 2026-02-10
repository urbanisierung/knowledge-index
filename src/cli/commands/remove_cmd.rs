use owo_colors::OwoColorize;
use std::path::Path;

use crate::cli::args::Args;
use crate::core::remote::{delete_clone, is_remote_clone};
use crate::db::{Database, SourceType};
use crate::error::{AppError, Result};

use super::{confirm, print_success, print_warning, use_colors};

pub fn run(path: &Path, force: bool, args: &Args) -> Result<()> {
    let colors = use_colors(args.no_color);
    let db = Database::open()?;

    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    // Check if repository exists
    let repo = db
        .get_repository_by_path(&canonical)?
        .ok_or_else(|| AppError::RepoNotFound(canonical.clone()))?;

    let is_remote = repo.source_type == SourceType::Remote;
    let will_delete_clone = is_remote && is_remote_clone(&repo.path).unwrap_or(false);

    // Confirm deletion
    if !force && !args.json {
        let prompt = if will_delete_clone {
            format!(
                "Remove \"{}\" from index AND delete cloned files at {}? ({} files)",
                repo.name,
                repo.path.display(),
                repo.file_count
            )
        } else {
            format!(
                "Remove \"{}\" from index? ({} files will be removed from the index)",
                repo.name, repo.file_count
            )
        };

        if !confirm(&prompt) {
            if !args.quiet {
                println!("Cancelled.");
            }
            return Ok(());
        }
    }

    // Delete repository from database
    db.delete_repository(repo.id)?;

    // If remote, also delete the cloned directory
    let clone_deleted = if will_delete_clone {
        match delete_clone(&repo.path) {
            Ok(()) => true,
            Err(e) => {
                if !args.quiet && !args.json {
                    print_warning(&format!("Could not delete clone directory: {e}"), colors);
                }
                false
            }
        }
    } else {
        false
    };

    if args.json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "name": repo.name,
                "path": canonical.to_string_lossy(),
                "files_removed": repo.file_count,
                "clone_deleted": clone_deleted,
                "source_type": if is_remote { "remote" } else { "local" },
            })
        );
    } else if !args.quiet {
        if colors {
            print_success(
                &format!(
                    "Removed \"{}\" ({} files)",
                    repo.name.cyan(),
                    repo.file_count
                ),
                true,
            );
        } else {
            print_success(
                &format!("Removed \"{}\" ({} files)", repo.name, repo.file_count),
                false,
            );
        }

        if clone_deleted {
            println!("Cloned directory deleted.");
        } else if !is_remote {
            println!("Note: The actual files were not affected.");
        }
    }

    Ok(())
}
