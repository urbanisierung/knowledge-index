use std::collections::BTreeMap;

use owo_colors::OwoColorize;

use crate::cli::args::Args;
use crate::config::Config;
use crate::core::{Embedder, SearchMode, Searcher};
use crate::db::Database;
use crate::error::Result;

use super::use_colors;

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::fn_params_excessive_bools)]
pub fn run(
    query: String,
    repo: Option<String>,
    file_type: Option<String>,
    limit: usize,
    group_by_repo: bool,
    semantic: bool,
    hybrid: bool,
    lexical: bool,
    args: &Args,
) -> Result<()> {
    let colors = use_colors(args.no_color);
    let db = Database::open()?;
    let config = Config::load()?;

    // Determine search mode
    let mode = if semantic {
        SearchMode::Semantic
    } else if hybrid {
        SearchMode::Hybrid
    } else if lexical {
        SearchMode::Lexical
    } else {
        SearchMode::from_str(&config.default_search_mode)
    };

    // Create searcher with embedder if needed for semantic/hybrid
    let searcher = if (mode == SearchMode::Semantic || mode == SearchMode::Hybrid)
        && config.enable_semantic_search
    {
        match Embedder::new(&config.embedding_model) {
            Ok(embedder) => Searcher::with_embedder(db, embedder),
            Err(e) => {
                if !args.quiet {
                    eprintln!(
                        "{} Could not load embeddings: {}. Falling back to lexical search.",
                        "Warning:".yellow(),
                        e
                    );
                }
                Searcher::new(db)
            }
        }
    } else {
        Searcher::new(db)
    };

    // Check if semantic search was requested but not available
    let effective_mode = if (mode == SearchMode::Semantic || mode == SearchMode::Hybrid)
        && !searcher.has_semantic_search()
    {
        if !args.quiet && !args.json {
            if colors {
                eprintln!(
                    "{} Semantic search not enabled. Using lexical search.",
                    "Note:".blue()
                );
                eprintln!("  Enable with: {}", "enable_semantic_search = true".cyan());
            } else {
                eprintln!("Note: Semantic search not enabled. Using lexical search.");
            }
        }
        SearchMode::Lexical
    } else {
        mode
    };

    let results = searcher.search_with_mode(
        &query,
        effective_mode,
        repo.as_deref(),
        file_type.as_deref(),
        limit,
        0,
    )?;

    if results.is_empty() {
        if args.json {
            println!(
                "{}",
                serde_json::json!({
                    "results": [],
                    "total": 0,
                    "query": query,
                    "mode": effective_mode.as_str()
                })
            );
        } else if !args.quiet {
            if colors {
                println!("{} No results for \"{}\"", "!".yellow(), query.cyan());
            } else {
                println!("No results for \"{query}\"");
            }
            println!();
            println!("Suggestions:");
            println!("  • Check spelling");
            println!("  • Try broader search terms");
            if effective_mode == SearchMode::Lexical {
                println!("  • Use prefix matching: \"func*\"");
                println!("  • Try --semantic for conceptual matching");
            }
        }
        return Ok(());
    }

    if args.json {
        if group_by_repo {
            // Group results by repository for JSON output
            let mut grouped: BTreeMap<String, Vec<serde_json::Value>> = BTreeMap::new();
            for r in &results {
                let entry = grouped.entry(r.repo_name.clone()).or_default();
                entry.push(serde_json::json!({
                    "file": r.file_path.to_string_lossy(),
                    "absolute_path": r.absolute_path.to_string_lossy(),
                    "snippet": r.snippet,
                    "file_type": r.file_type,
                    "score": r.score,
                    "search_mode": r.search_mode.as_str(),
                }));
            }

            println!(
                "{}",
                serde_json::json!({
                    "grouped_results": grouped,
                    "total": results.len(),
                    "repo_count": grouped.len(),
                    "query": query,
                    "limit": limit,
                    "mode": effective_mode.as_str(),
                })
            );
        } else {
            let json_results: Vec<_> = results
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "repo": r.repo_name,
                        "file": r.file_path.to_string_lossy(),
                        "absolute_path": r.absolute_path.to_string_lossy(),
                        "snippet": r.snippet,
                        "file_type": r.file_type,
                        "score": r.score,
                        "search_mode": r.search_mode.as_str(),
                    })
                })
                .collect();

            println!(
                "{}",
                serde_json::json!({
                    "results": json_results,
                    "total": results.len(),
                    "query": query,
                    "limit": limit,
                    "mode": effective_mode.as_str(),
                })
            );
        }
    } else if !args.quiet {
        // Show search mode if not lexical
        if effective_mode != SearchMode::Lexical && colors {
            println!(
                "{} {} search",
                "Mode:".dimmed(),
                effective_mode.as_str().blue()
            );
            println!();
        }

        if group_by_repo {
            // Group results by repository for display
            let mut grouped: BTreeMap<String, Vec<_>> = BTreeMap::new();
            for result in &results {
                grouped
                    .entry(result.repo_name.clone())
                    .or_default()
                    .push(result);
            }

            for (repo_name, repo_results) in &grouped {
                // Print repository header
                if colors {
                    println!(
                        "{} {} ({} result{})",
                        "▶".blue(),
                        repo_name.blue().bold(),
                        repo_results.len(),
                        if repo_results.len() == 1 { "" } else { "s" }
                    );
                } else {
                    println!(
                        "▶ {} ({} result{})",
                        repo_name,
                        repo_results.len(),
                        if repo_results.len() == 1 { "" } else { "s" }
                    );
                }

                for result in repo_results {
                    // Format: indented path
                    if colors {
                        println!("  {}", result.file_path.display().to_string().cyan());
                    } else {
                        println!("  {}", result.file_path.display());
                    }

                    // Show snippet with highlighting
                    let snippet = result.snippet.trim();
                    if !snippet.is_empty() {
                        let formatted = if colors {
                            snippet
                                .replace(">>>", "\x1b[1;33m")
                                .replace("<<<", "\x1b[0m")
                        } else {
                            snippet.replace(">>>", "[").replace("<<<", "]")
                        };

                        for line in formatted.lines() {
                            if colors {
                                println!("    {}", line.dimmed());
                            } else {
                                println!("    {line}");
                            }
                        }
                    }
                }
                println!();
            }

            // Show count info
            if colors {
                println!(
                    "{} {} result{} in {} repositor{}",
                    "─".dimmed(),
                    results.len().to_string().green(),
                    if results.len() == 1 { "" } else { "s" },
                    grouped.len().to_string().green(),
                    if grouped.len() == 1 { "y" } else { "ies" }
                );
            } else {
                println!(
                    "─ {} result{} in {} repositor{}",
                    results.len(),
                    if results.len() == 1 { "" } else { "s" },
                    grouped.len(),
                    if grouped.len() == 1 { "y" } else { "ies" }
                );
            }
        } else {
            for result in &results {
                // Format: repo:path
                if colors {
                    println!(
                        "{}{}{}",
                        result.repo_name.blue(),
                        ":".dimmed(),
                        result.file_path.display().to_string().cyan()
                    );
                } else {
                    println!("{}:{}", result.repo_name, result.file_path.display());
                }

                // Show snippet with highlighting
                let snippet = result.snippet.trim();
                if !snippet.is_empty() {
                    // Replace >>> and <<< markers with colors or brackets
                    let formatted = if colors {
                        snippet
                            .replace(">>>", "\x1b[1;33m")
                            .replace("<<<", "\x1b[0m")
                    } else {
                        snippet.replace(">>>", "[").replace("<<<", "]")
                    };

                    for line in formatted.lines() {
                        if colors {
                            println!("  {}", line.dimmed());
                        } else {
                            println!("  {line}");
                        }
                    }
                }
                println!();
            }

            // Show count info
            if colors {
                println!(
                    "{} Showing {} result{}",
                    "─".dimmed(),
                    results.len().to_string().green(),
                    if results.len() == 1 { "" } else { "s" }
                );
            } else {
                println!(
                    "─ Showing {} result{}",
                    results.len(),
                    if results.len() == 1 { "" } else { "s" }
                );
            }
        }
    }

    Ok(())
}
