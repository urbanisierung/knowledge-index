use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::path::Path;

use crate::cli::args::Args;
use crate::config::Config;
use crate::core::Indexer;
use crate::db::Database;
use crate::error::Result;

use super::{print_success, print_warning, use_colors};

#[allow(clippy::too_many_lines)]
pub fn run(path: &Path, name: Option<String>, args: &Args) -> Result<()> {
    let colors = use_colors(args.no_color);
    let config = Config::load()?;
    let db = Database::open()?;

    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    if !args.quiet && !args.json {
        if colors {
            println!(
                "Indexing {}...",
                canonical.display().to_string().cyan()
            );
        } else {
            println!("Indexing {}...", canonical.display());
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

    let indexer = Indexer::new(db, config);

    // Create progress bar
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
            
            // Truncate filename for display
            let display_file = if progress.current_file.len() > 40 {
                format!("...{}", &progress.current_file[progress.current_file.len() - 37..])
            } else {
                progress.current_file.clone()
            };
            pb.set_message(display_file);
        }
    })?;

    if let Some(pb) = progress_bar {
        pb.finish_and_clear();
    }

    // Output results
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
                "files_skipped": result.files_skipped,
                "total_bytes": result.total_bytes,
                "elapsed_secs": result.elapsed_secs,
            })
        );
    } else if !args.quiet {
        let total_files = result.files_added + result.files_updated + result.files_unchanged;
        
        if colors {
            print_success(
                &format!(
                    "Indexed {} files in {:.1}s",
                    total_files.to_string().green(),
                    result.elapsed_secs
                ),
                true,
            );
        } else {
            print_success(&format!("Indexed {total_files} files in {:.1}s", result.elapsed_secs), false);
        }

        // Show details
        if result.files_added > 0 {
            println!("  Added: {}", result.files_added);
        }
        if result.files_updated > 0 {
            println!("  Updated: {}", result.files_updated);
        }
        if result.files_deleted > 0 {
            println!("  Deleted: {}", result.files_deleted);
        }
        if result.files_skipped > 0 {
            println!("  Skipped: {} (binary/too large)", result.files_skipped);
        }

        // Next steps hint for first-time users
        println!();
        println!("What's next:");
        if colors {
            println!("  {} Search: {}", "•".dimmed(), "knowledge-index search \"your query\"".cyan());
            println!("  {} Browse: {}", "•".dimmed(), "knowledge-index".cyan());
        } else {
            println!("  • Search: knowledge-index search \"your query\"");
            println!("  • Browse: knowledge-index");
        }
    }

    Ok(())
}
