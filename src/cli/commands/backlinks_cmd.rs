//! Backlinks discovery command.

use crate::cli::args::Args;
use crate::config::Config;
use crate::db::Database;
use crate::error::Result;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::path::Path;

use super::use_colors;

#[derive(Serialize)]
struct BacklinkInfo {
    file: String,
    repo: String,
    link_text: String,
    line: Option<usize>,
}

#[derive(Serialize)]
struct BacklinksOutput {
    target: String,
    count: usize,
    backlinks: Vec<BacklinkInfo>,
}

/// Find all files linking to a specific file
pub fn run(file: &Path, args: &Args) -> Result<()> {
    let db = Database::open()?;
    let _config = Config::load()?;
    let colors = use_colors(args.no_color);

    // Normalize the file path - we'll search for files that contain links to this
    let target_name = file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();

    if target_name.is_empty() {
        return Err(crate::error::AppError::Other(format!(
            "Invalid file path: {}",
            file.display()
        )));
    }

    // Get all backlinks to this file
    let backlinks = db.get_backlinks(target_name)?;

    if args.json {
        let output = BacklinksOutput {
            target: target_name.to_string(),
            count: backlinks.len(),
            backlinks: backlinks
                .into_iter()
                .map(|(file_path, repo_name, link_text, line)| BacklinkInfo {
                    file: file_path,
                    repo: repo_name,
                    link_text,
                    line,
                })
                .collect(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    if backlinks.is_empty() {
        if !args.quiet {
            println!("No backlinks found for: {target_name}");
            println!();
            println!("Backlinks are discovered from [[wiki-link]] syntax in markdown files.");
        }
        return Ok(());
    }

    if !args.quiet {
        if colors {
            println!("{} {}", "Backlinks to".bold(), target_name.cyan().bold());
            println!("{}", "─".repeat(50).dimmed());
        } else {
            println!("Backlinks to {target_name}");
            println!("{}", "─".repeat(50));
        }
    }

    for (file_path, repo_name, link_text, line) in &backlinks {
        let line: &Option<usize> = line;
        if colors {
            let line_info = line.map_or(String::new(), |l| format!(":{l}"));
            println!(
                "  {} {}{}",
                repo_name.dimmed(),
                file_path.cyan(),
                line_info.dimmed()
            );
            if link_text != target_name {
                println!("    {} {}", "→".dimmed(), link_text.dimmed());
            }
        } else {
            let line_info = line.map_or(String::new(), |l| format!(":{l}"));
            println!("  {repo_name}: {file_path}{line_info}");
            if link_text != target_name {
                println!("    → {link_text}");
            }
        }
    }

    if !args.quiet {
        println!();
        if colors {
            println!("{} files link to this", backlinks.len().to_string().green());
        } else {
            println!("{} files link to this", backlinks.len());
        }
    }

    Ok(())
}
