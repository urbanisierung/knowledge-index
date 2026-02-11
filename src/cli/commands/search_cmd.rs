use std::collections::BTreeMap;

use owo_colors::OwoColorize;
use regex::Regex;

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
    _tag: Option<String>, // TODO: Implement tag filtering
    limit: usize,
    group_by_repo: bool,
    semantic: bool,
    hybrid: bool,
    lexical: bool,
    fuzzy: bool,
    regex: bool,
    args: &Args,
) -> Result<()> {
    let colors = use_colors(args.no_color);
    let db = Database::open()?;
    let config = Config::load()?;

    // Handle regex search mode
    if regex {
        return run_regex_search(
            &query,
            repo.as_deref(),
            file_type.as_deref(),
            limit,
            group_by_repo,
            args,
        );
    }

    // Handle fuzzy search mode
    if fuzzy {
        return run_fuzzy_search(
            &query,
            repo.as_deref(),
            file_type.as_deref(),
            limit,
            group_by_repo,
            args,
        );
    }

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

/// Run fuzzy search with typo tolerance
#[allow(clippy::too_many_arguments)]
fn run_fuzzy_search(
    query: &str,
    repo: Option<&str>,
    file_type: Option<&str>,
    limit: usize,
    group_by_repo: bool,
    args: &Args,
) -> Result<()> {
    use strsim::jaro_winkler;

    let colors = use_colors(args.no_color);
    let db = Database::open()?;

    // First get a broader set of results with prefix matching via FTS
    let wildcard_query = format!(
        "{}*",
        query.split_whitespace().collect::<Vec<_>>().join("* ")
    );
    let mut results = db.search(&wildcard_query, repo, file_type, limit * 5, 0)?;

    // Also do an exact match search
    if let Ok(exact_results) = db.search(query, repo, file_type, limit * 5, 0) {
        for r in exact_results {
            if !results
                .iter()
                .any(|existing| existing.file_path == r.file_path)
            {
                results.push(r);
            }
        }
    }

    // Score by fuzzy similarity
    let query_lower = query.to_lowercase();
    #[allow(clippy::cast_precision_loss)]
    let mut scored: Vec<_> = results
        .into_iter()
        .map(|r| {
            let snippet_lower = r.snippet.to_lowercase();
            let path_lower = r.file_path.display().to_string().to_lowercase();

            let snippet_score = query_lower
                .split_whitespace()
                .map(|word| {
                    snippet_lower
                        .split_whitespace()
                        .map(|s| jaro_winkler(word, s))
                        .fold(0.0_f64, f64::max)
                })
                .sum::<f64>()
                / query_lower.split_whitespace().count().max(1) as f64;

            let path_score = jaro_winkler(&query_lower, &path_lower);
            let score = snippet_score.max(path_score);
            (r, score)
        })
        .filter(|(_, score)| *score > 0.6)
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);

    let results: Vec<_> = scored.into_iter().map(|(r, _)| r).collect();

    if results.is_empty() {
        if args.json {
            println!(
                "{}",
                serde_json::json!({ "results": [], "total": 0, "query": query, "mode": "fuzzy" })
            );
        } else if !args.quiet {
            println!("No fuzzy matches for \"{query}\"");
        }
        return Ok(());
    }

    display_search_results(&results, query, "fuzzy", group_by_repo, colors, args);
    Ok(())
}

/// Run regex search
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
fn run_regex_search(
    pattern: &str,
    repo: Option<&str>,
    file_type: Option<&str>,
    limit: usize,
    group_by_repo: bool,
    args: &Args,
) -> Result<()> {
    let colors = use_colors(args.no_color);
    let db = Database::open()?;

    let regex = match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => {
            if args.json {
                println!(
                    "{}",
                    serde_json::json!({ "error": format!("Invalid regex: {e}") })
                );
            } else {
                eprintln!("Invalid regex pattern: {e}");
                eprintln!();
                eprintln!("Examples of valid patterns:");
                eprintln!("  fn\\s+\\w+         Match function definitions");
                eprintln!("  TODO|FIXME        Match TODO or FIXME");
            }
            return Ok(());
        }
    };

    let repos = db.list_repositories()?;
    let mut results = Vec::new();

    for repo_info in &repos {
        if let Some(filter) = &repo {
            if !repo_info.name.contains(filter) {
                continue;
            }
        }

        let files = db.get_repository_files(repo_info.id)?;

        for file in &files {
            if let Some(ft) = &file_type {
                if !file.file_type.contains(ft) {
                    continue;
                }
            }

            let full_path = repo_info.path.join(&file.relative_path);
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                if let Some(m) = regex.find(&content) {
                    let start = content[..m.start()].rfind('\n').map_or(0, |p| p + 1);
                    let end = content[m.end()..]
                        .find('\n')
                        .map_or(content.len(), |p| m.end() + p);
                    let snippet = &content[start..end];

                    results.push(crate::db::SearchResult {
                        repo_name: repo_info.name.clone(),
                        repo_path: repo_info.path.clone(),
                        file_path: std::path::PathBuf::from(&file.relative_path),
                        absolute_path: full_path,
                        snippet: format!(">>>{snippet}<<<<"),
                        file_type: file.file_type.clone(),
                        score: 1.0,
                    });

                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        if results.len() >= limit {
            break;
        }
    }

    if results.is_empty() {
        if args.json {
            println!(
                "{}",
                serde_json::json!({ "results": [], "total": 0, "pattern": pattern, "mode": "regex" })
            );
        } else if !args.quiet {
            println!("No matches for regex /{pattern}/");
        }
        return Ok(());
    }

    display_search_results(&results, pattern, "regex", group_by_repo, colors, args);
    Ok(())
}

/// Display search results (shared between search modes)
#[allow(clippy::too_many_lines)]
fn display_search_results(
    results: &[crate::db::SearchResult],
    query: &str,
    mode: &str,
    group_by_repo: bool,
    colors: bool,
    args: &Args,
) {
    if args.json {
        let json_results: Vec<_> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "repo": r.repo_name,
                    "path": r.file_path,
                    "snippet": r.snippet.replace(">>>", "").replace("<<<", ""),
                    "file_type": r.file_type
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::json!({
                "results": json_results,
                "total": results.len(),
                "query": query,
                "mode": mode
            })
        );
        return;
    }

    if group_by_repo {
        let mut grouped: BTreeMap<String, Vec<_>> = BTreeMap::new();
        for result in results {
            grouped
                .entry(result.repo_name.clone())
                .or_default()
                .push(result);
        }

        for (repo_name, repo_results) in &grouped {
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
                if colors {
                    println!("  {}", result.file_path.display().to_string().cyan());
                } else {
                    println!("  {}", result.file_path.display());
                }
            }
            println!();
        }
    } else {
        for result in results {
            if colors {
                println!(
                    "{}:{}",
                    result.repo_name.blue(),
                    result.file_path.display().to_string().cyan()
                );
            } else {
                println!("{}:{}", result.repo_name, result.file_path.display());
            }

            let snippet = result.snippet.trim();
            if !snippet.is_empty() {
                let formatted = if colors {
                    snippet
                        .replace(">>>", "\x1b[1;33m")
                        .replace("<<<", "\x1b[0m")
                } else {
                    snippet.replace(">>>", "[").replace("<<<", "]")
                };
                for line in formatted.lines().take(3) {
                    if colors {
                        println!("  {}", line.dimmed());
                    } else {
                        println!("  {line}");
                    }
                }
            }
        }
    }

    if !args.quiet {
        println!();
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
