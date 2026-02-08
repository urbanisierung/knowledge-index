//! Rebuild embeddings command handler

use owo_colors::OwoColorize;
use std::io::{self, Write};

use crate::cli::args::Args;
use crate::config::Config;
use crate::core::Embedder;
use crate::db::Database;
use crate::error::Result;

use super::use_colors;

/// Rebuild embeddings for all or specific repositories
#[allow(clippy::too_many_lines)]
pub fn run(repo_filter: Option<String>, args: &Args) -> Result<()> {
    let colors = use_colors(args.no_color);
    let config = Config::load()?;

    // Check if semantic search is enabled
    if !config.enable_semantic_search {
        if colors {
            eprintln!("{} Semantic search is not enabled.", "Error:".red());
            eprintln!(
                "  Enable it in config: {}",
                "enable_semantic_search = true".cyan()
            );
        } else {
            eprintln!("Error: Semantic search is not enabled.");
            eprintln!("  Enable it in config: enable_semantic_search = true");
        }
        return Ok(());
    }

    // Load embedder
    if !args.quiet {
        if colors {
            print!("{} Loading embedding model... ", "→".blue());
        } else {
            print!("Loading embedding model... ");
        }
        io::stdout().flush().ok();
    }

    let embedder = match Embedder::new(&config.embedding_model) {
        Ok(e) => {
            if !args.quiet {
                if colors {
                    println!("{}", "done".green());
                } else {
                    println!("done");
                }
            }
            e
        }
        Err(e) => {
            if !args.quiet {
                if colors {
                    println!("{}", "failed".red());
                } else {
                    println!("failed");
                }
            }
            eprintln!("Failed to load embedding model: {e}");
            return Ok(());
        }
    };

    let db = Database::open()?;

    // Get repositories to process
    let repos = db.list_repositories()?;
    let repos_to_process: Vec<_> = if let Some(ref filter) = repo_filter {
        repos
            .into_iter()
            .filter(|r| r.name.contains(filter))
            .collect()
    } else {
        repos
    };

    if repos_to_process.is_empty() {
        if !args.quiet {
            if let Some(filter) = repo_filter {
                if colors {
                    println!(
                        "{} No repositories matching \"{}\"",
                        "!".yellow(),
                        filter.cyan()
                    );
                } else {
                    println!("No repositories matching \"{filter}\"");
                }
            } else if colors {
                println!("{} No repositories indexed", "!".yellow());
            } else {
                println!("No repositories indexed");
            }
        }
        return Ok(());
    }

    let mut total_files = 0;
    let mut total_embeddings = 0;

    for repo in &repos_to_process {
        // Get files for this repository
        let files = db.get_repository_files(repo.id)?;
        let file_count = files.len();

        if !args.quiet {
            if colors {
                println!(
                    "{} Processing {} ({} files)...",
                    "→".blue(),
                    repo.name.cyan(),
                    file_count
                );
            } else {
                println!("Processing {} ({} files)...", repo.name, file_count);
            }
        }

        let mut processed = 0;

        for file in &files {
            processed += 1;

            // Show progress every 10 files or at the end
            if !args.quiet && (processed % 10 == 0 || processed == file_count) {
                if colors {
                    print!("\r  {} [{}/{}] ", "⟳".blue(), processed, file_count);
                } else {
                    print!("\r  [{processed}/{file_count}] ");
                }
                io::stdout().flush().ok();
            }

            // Read file content
            let full_path = repo.path.join(&file.relative_path);
            let Ok(content) = std::fs::read_to_string(&full_path) else {
                continue; // Skip files we can't read
            };

            // Generate embeddings
            if let Ok(chunk_embeddings) = embedder.embed_content(&content) {
                if chunk_embeddings.is_empty() {
                    continue;
                }

                let embeddings: Vec<(usize, usize, usize, &str, &[f32])> = chunk_embeddings
                    .iter()
                    .enumerate()
                    .map(|(idx, ce)| {
                        (
                            idx,
                            ce.chunk.start_offset,
                            ce.chunk.end_offset,
                            ce.chunk.text.as_str(),
                            ce.embedding.as_slice(),
                        )
                    })
                    .collect();

                if db.store_embeddings(file.id, &embeddings).is_ok() {
                    total_files += 1;
                    total_embeddings += embeddings.len();
                }
            }
        }

        // Clear the progress line
        if !args.quiet && file_count > 0 {
            print!("\r                                                  \r");
            io::stdout().flush().ok();
        }
    }

    if !args.quiet {
        if colors {
            println!(
                "{} Rebuilt embeddings for {} file{} ({} chunks) in {} repositor{}",
                "✓".green(),
                total_files.to_string().green(),
                if total_files == 1 { "" } else { "s" },
                total_embeddings.to_string().green(),
                repos_to_process.len().to_string().green(),
                if repos_to_process.len() == 1 {
                    "y"
                } else {
                    "ies"
                }
            );
        } else {
            println!();
            println!(
                "Rebuilt embeddings for {} file(s) ({} chunks) in {} repositor{}",
                total_files,
                total_embeddings,
                repos_to_process.len(),
                if repos_to_process.len() == 1 {
                    "y"
                } else {
                    "ies"
                }
            );
        }
    }

    Ok(())
}
