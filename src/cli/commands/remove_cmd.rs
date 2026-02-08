use owo_colors::OwoColorize;
use std::path::Path;

use crate::cli::args::Args;
use crate::db::Database;
use crate::error::{AppError, Result};

use super::{confirm, print_success, use_colors};

pub fn run(path: &Path, force: bool, args: &Args) -> Result<()> {
    let colors = use_colors(args.no_color);
    let db = Database::open()?;

    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    // Check if repository exists
    let repo = db
        .get_repository_by_path(&canonical)?
        .ok_or_else(|| AppError::RepoNotFound(canonical.clone()))?;

    // Confirm deletion
    if !force && !args.json {
        let prompt = format!(
            "Remove \"{}\" from index? ({} files will be removed from the index)",
            repo.name, repo.file_count
        );

        if !confirm(&prompt) {
            if !args.quiet {
                println!("Cancelled.");
            }
            return Ok(());
        }
    }

    // Delete repository
    db.delete_repository(repo.id)?;

    if args.json {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "name": repo.name,
                "path": canonical.to_string_lossy(),
                "files_removed": repo.file_count,
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
        println!("Note: The actual files were not affected.");
    }

    Ok(())
}
