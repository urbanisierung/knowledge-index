use owo_colors::OwoColorize;

use crate::cli::args::Args;
use crate::core::Searcher;
use crate::db::Database;
use crate::error::Result;

use super::use_colors;

#[allow(clippy::needless_pass_by_value)]
pub fn run(
    query: String,
    repo: Option<String>,
    file_type: Option<String>,
    limit: usize,
    args: &Args,
) -> Result<()> {
    let colors = use_colors(args.no_color);
    let db = Database::open()?;
    let searcher = Searcher::new(db);

    let results = searcher.search(&query, repo.as_deref(), file_type.as_deref(), limit, 0)?;

    if results.is_empty() {
        if args.json {
            println!(
                "{}",
                serde_json::json!({
                    "results": [],
                    "total": 0,
                    "query": query
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
            println!("  • Use prefix matching: \"func*\"");
        }
        return Ok(());
    }

    if args.json {
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
            })
        );
    } else if !args.quiet {
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

    Ok(())
}
