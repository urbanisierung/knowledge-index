//! Context builder command for AI prompts.

use crate::cli::args::Args;
use crate::config::Config;
use crate::core::{Embedder, SearchMode, Searcher};
use crate::db::Database;
use crate::error::Result;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::fs;

use super::use_colors;

#[derive(Serialize)]
struct ContextFile {
    path: String,
    repo: String,
    content: String,
    tokens_approx: usize,
}

#[derive(Serialize)]
struct ContextOutput {
    query: String,
    files_included: usize,
    total_tokens_approx: usize,
    context: String,
    files: Vec<ContextFile>,
}

/// Approximate token count (roughly 4 chars per token)
fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

/// Build context from search results for AI prompts
#[allow(clippy::too_many_lines)]
pub fn run(query: &str, limit: usize, max_tokens: usize, format: &str, args: &Args) -> Result<()> {
    let db = Database::open()?;
    let config = Config::load()?;
    let colors = use_colors(args.no_color);

    // Create searcher with embedder if available
    let searcher = if config.enable_semantic_search {
        match Embedder::new(&config.embedding_model) {
            Ok(embedder) => Searcher::with_embedder(db, embedder),
            Err(_) => Searcher::new(db),
        }
    } else {
        Searcher::new(db)
    };

    // Search for relevant files
    let results =
        searcher.search_with_mode(query, SearchMode::Lexical, None, None, limit * 2, 0)?;

    if results.is_empty() {
        if args.json {
            let output = ContextOutput {
                query: query.to_string(),
                files_included: 0,
                total_tokens_approx: 0,
                context: String::new(),
                files: vec![],
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else if !args.quiet {
            println!("No results found for: {query}");
        }
        return Ok(());
    }

    // Build context respecting token limits
    let mut context_parts: Vec<String> = Vec::new();
    let mut files: Vec<ContextFile> = Vec::new();
    let mut total_tokens = 0;
    let mut files_included = 0;

    for result in results {
        if files_included >= limit {
            break;
        }

        // Try to read the full file content
        let Ok(content) = fs::read_to_string(&result.absolute_path) else {
            continue;
        };

        let file_tokens = estimate_tokens(&content);

        // Check if adding this file would exceed the limit
        if total_tokens + file_tokens > max_tokens && files_included > 0 {
            // Try to include a truncated version
            let remaining_tokens = max_tokens.saturating_sub(total_tokens);
            if remaining_tokens > 100 {
                let truncated_len = remaining_tokens * 4;
                let truncated: String = content.chars().take(truncated_len).collect();
                let truncated_content = format!("{truncated}\n\n[... truncated ...]");

                let header = format!("## {}/{}\n\n", result.repo_name, result.file_path.display());
                context_parts.push(format!("{header}{truncated_content}"));

                files.push(ContextFile {
                    path: result.file_path.display().to_string(),
                    repo: result.repo_name.clone(),
                    content: truncated_content,
                    tokens_approx: remaining_tokens,
                });

                total_tokens += remaining_tokens;
                files_included += 1;
            }
            break;
        }

        // Add full file content
        let header = format!("## {}/{}\n\n", result.repo_name, result.file_path.display());
        context_parts.push(format!("{header}{content}"));

        files.push(ContextFile {
            path: result.file_path.display().to_string(),
            repo: result.repo_name,
            content,
            tokens_approx: file_tokens,
        });

        total_tokens += file_tokens;
        files_included += 1;
    }

    let context = context_parts.join("\n---\n\n");

    // Output based on format
    match format {
        "json" => {
            let output = ContextOutput {
                query: query.to_string(),
                files_included,
                total_tokens_approx: total_tokens,
                context,
                files,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        "text" => {
            println!("{context}");
        }
        _ => {
            if args.json {
                let output = ContextOutput {
                    query: query.to_string(),
                    files_included,
                    total_tokens_approx: total_tokens,
                    context: context.clone(),
                    files,
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                // Print header with stats
                if !args.quiet {
                    if colors {
                        println!("{} {}", "Context for:".bold(), query.cyan());
                        println!(
                            "{} files, ~{} tokens",
                            files_included.to_string().green(),
                            total_tokens.to_string().green()
                        );
                        println!("{}", "─".repeat(50).dimmed());
                        println!();
                    } else {
                        println!("Context for: {query}");
                        println!("{files_included} files, ~{total_tokens} tokens");
                        println!("{}", "─".repeat(50));
                        println!();
                    }
                }

                println!("{context}");

                if !args.quiet {
                    println!();
                    if colors {
                        println!("{}", "Tip: Pipe to clipboard or AI tool".dimmed());
                    }
                }
            }
        }
    }

    Ok(())
}
