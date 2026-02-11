//! Health check command - find orphans, broken links, and stale repos.

use crate::cli::args::Args;
use crate::db::Database;
use crate::error::Result;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::collections::HashSet;

use super::use_colors;

#[derive(Serialize)]
struct HealthReport {
    orphan_files: Vec<OrphanFile>,
    broken_links: Vec<BrokenLink>,
    summary: HealthSummary,
}

#[derive(Serialize)]
struct OrphanFile {
    path: String,
    repo: String,
}

#[derive(Serialize)]
struct BrokenLink {
    source_path: String,
    source_repo: String,
    target: String,
}

#[derive(Serialize)]
struct HealthSummary {
    total_orphans: usize,
    total_broken_links: usize,
    health_score: u8,
}

/// Run health diagnostics on the knowledge index
#[allow(clippy::too_many_lines)]
pub fn run(repo: Option<&str>, args: &Args) -> Result<()> {
    let db = Database::open()?;
    let colors = use_colors(args.no_color);

    // Get all links and files
    let links = db.get_all_links(repo)?;
    let all_files = db.get_all_file_paths()?;

    // Build set of known file stems (for matching [[links]])
    let mut known_files: HashSet<String> = HashSet::new();
    let mut known_stems: HashSet<String> = HashSet::new();

    for (path, repo_name) in &all_files {
        if repo.is_none() || repo == Some(repo_name.as_str()) {
            known_files.insert(path.clone());
            // Add file stem (without extension) for matching
            if let Some(stem) = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
            {
                known_stems.insert(stem.to_lowercase());
            }
        }
    }

    // Find broken links (target doesn't exist)
    let mut broken_links: Vec<BrokenLink> = Vec::new();
    for link in &links {
        if repo.is_some() && repo != Some(link.source_repo.as_str()) {
            continue;
        }

        let target_lower = link.target_name.to_lowercase();
        let suffix = format!("/{target_lower}.md");
        let target_exists = known_files.contains(&link.target_name)
            || known_stems.contains(&target_lower)
            || known_files.iter().any(|f| {
                f.to_lowercase().contains(&target_lower) || f.to_lowercase().ends_with(&suffix)
            });

        if !target_exists {
            broken_links.push(BrokenLink {
                source_path: link.source_path.clone(),
                source_repo: link.source_repo.clone(),
                target: link.target_name.clone(),
            });
        }
    }

    // Find orphan files (markdown files with no incoming links)
    let orphan_files = db.get_orphan_files(repo)?;
    let orphans: Vec<OrphanFile> = orphan_files
        .into_iter()
        .map(|(path, repo_name)| OrphanFile {
            path,
            repo: repo_name,
        })
        .collect();

    // Calculate health score (0-100)
    let total_md_files = all_files
        .iter()
        .filter(|(p, r)| {
            std::path::Path::new(p)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
                && (repo.is_none() || repo == Some(r.as_str()))
        })
        .count();

    #[allow(clippy::cast_possible_truncation)]
    let health_score = if total_md_files == 0 {
        100
    } else {
        let orphan_penalty = (orphans.len() * 100 / total_md_files.max(1)).min(50);
        let broken_penalty = (broken_links.len() * 5).min(50);
        100_u8.saturating_sub((orphan_penalty + broken_penalty) as u8)
    };

    let orphan_count = orphans.len();
    let broken_count = broken_links.len();

    if args.json {
        let report = HealthReport {
            orphan_files: orphans,
            broken_links,
            summary: HealthSummary {
                total_orphans: orphan_count,
                total_broken_links: broken_count,
                health_score,
            },
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    // Display results
    if colors {
        println!("{}", "Knowledge Index Health Report".bold());
        println!("{}", "═".repeat(40).dimmed());
        println!();

        // Health score
        let score_color = if health_score >= 80 {
            health_score.to_string().green().to_string()
        } else if health_score >= 50 {
            health_score.to_string().yellow().to_string()
        } else {
            health_score.to_string().red().to_string()
        };
        println!("Health Score: {score_color}/100");
        println!();

        // Broken links
        if broken_links.is_empty() {
            println!("{} No broken links found", "✓".green());
        } else {
            println!(
                "{} {} broken links:",
                "✗".red(),
                broken_links.len().to_string().red()
            );
            for bl in broken_links.iter().take(10) {
                println!(
                    "  {} → {} (target: {})",
                    bl.source_repo.dimmed(),
                    bl.source_path,
                    bl.target.yellow()
                );
            }
            if broken_links.len() > 10 {
                println!("  ... and {} more", broken_links.len() - 10);
            }
        }
        println!();

        // Orphan files
        if orphans.is_empty() {
            println!("{} No orphan files found", "✓".green());
        } else {
            println!(
                "{} {} orphan files (no incoming links):",
                "!".yellow(),
                orphans.len().to_string().yellow()
            );
            for orphan in orphans.iter().take(10) {
                println!("  {} {}", orphan.repo.dimmed(), orphan.path);
            }
            if orphans.len() > 10 {
                println!("  ... and {} more", orphans.len() - 10);
            }
        }
    } else {
        println!("Knowledge Index Health Report");
        println!("{}", "═".repeat(40));
        println!();
        println!("Health Score: {health_score}/100");
        println!();

        if broken_links.is_empty() {
            println!("✓ No broken links found");
        } else {
            println!("✗ {} broken links:", broken_links.len());
            for bl in broken_links.iter().take(10) {
                println!(
                    "  {} → {} (target: {})",
                    bl.source_repo, bl.source_path, bl.target
                );
            }
            if broken_links.len() > 10 {
                println!("  ... and {} more", broken_links.len() - 10);
            }
        }
        println!();

        if orphans.is_empty() {
            println!("✓ No orphan files found");
        } else {
            println!("! {} orphan files:", orphans.len());
            for orphan in orphans.iter().take(10) {
                println!("  {} {}", orphan.repo, orphan.path);
            }
            if orphans.len() > 10 {
                println!("  ... and {} more", orphans.len() - 10);
            }
        }
    }

    Ok(())
}
