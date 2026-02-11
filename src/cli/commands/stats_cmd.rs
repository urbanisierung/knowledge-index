//! Knowledge statistics command.

use crate::cli::args::Args;
use crate::db::Database;
use crate::error::Result;
use owo_colors::OwoColorize;
use serde::Serialize;

use super::use_colors;

#[derive(Serialize)]
struct StatsOutput {
    total_files: usize,
    total_repos: usize,
    file_types: Vec<FileTypeCount>,
    total_tags: usize,
    total_links: usize,
    files_with_embeddings: usize,
    database_size_bytes: u64,
    database_size_human: String,
}

#[derive(Serialize)]
struct FileTypeCount {
    file_type: String,
    count: i64,
}

/// Format bytes as human-readable size
#[allow(clippy::cast_precision_loss)]
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} bytes")
    }
}

/// Display knowledge statistics
pub fn run(args: &Args) -> Result<()> {
    let db = Database::open()?;
    let colors = use_colors(args.no_color);

    let stats = db.get_stats()?;

    if args.json {
        let output = StatsOutput {
            total_files: stats.total_files,
            total_repos: stats.total_repos,
            file_types: stats
                .file_counts
                .iter()
                .map(|(ft, count)| FileTypeCount {
                    file_type: ft.clone(),
                    count: *count,
                })
                .collect(),
            total_tags: stats.total_tags,
            total_links: stats.total_links,
            files_with_embeddings: stats.files_with_embeddings,
            database_size_bytes: stats.database_size_bytes,
            database_size_human: format_bytes(stats.database_size_bytes),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    if colors {
        println!("{}", "Knowledge Index Statistics".bold());
        println!("{}", "═".repeat(40).dimmed());
        println!();

        // Repositories and files
        println!("{}", "📁 Content".bold());
        println!("   Repositories: {}", stats.total_repos.to_string().cyan());
        println!("   Total files:  {}", stats.total_files.to_string().cyan());
        println!();

        // File type breakdown
        println!("{}", "📄 File Types".bold());
        for (file_type, count) in &stats.file_counts {
            let icon = match file_type.as_str() {
                "markdown" => "📝",
                "code" => "💻",
                "config" => "⚙️",
                _ => "📄",
            };
            println!("   {} {}: {}", icon, file_type, count.to_string().green());
        }
        println!();

        // Knowledge graph
        println!("{}", "🔗 Knowledge Graph".bold());
        println!("   Tags:  {}", stats.total_tags.to_string().cyan());
        println!("   Links: {}", stats.total_links.to_string().cyan());
        println!();

        // Embeddings
        println!("{}", "🧠 Semantic Search".bold());
        println!(
            "   Files with embeddings: {}",
            stats.files_with_embeddings.to_string().cyan()
        );
        #[allow(clippy::cast_precision_loss)]
        let coverage = if stats.total_files > 0 {
            (stats.files_with_embeddings as f64 / stats.total_files as f64) * 100.0
        } else {
            0.0
        };
        println!("   Coverage: {coverage:.1}%");
        println!();

        // Storage
        println!("{}", "💾 Storage".bold());
        println!(
            "   Database: {}",
            format_bytes(stats.database_size_bytes).cyan()
        );
    } else {
        println!("Knowledge Index Statistics");
        println!("{}", "═".repeat(40));
        println!();

        println!("Content");
        println!("  Repositories: {}", stats.total_repos);
        println!("  Total files:  {}", stats.total_files);
        println!();

        println!("File Types");
        for (file_type, count) in &stats.file_counts {
            println!("  {file_type}: {count}");
        }
        println!();

        println!("Knowledge Graph");
        println!("  Tags:  {}", stats.total_tags);
        println!("  Links: {}", stats.total_links);
        println!();

        println!("Semantic Search");
        println!("  Files with embeddings: {}", stats.files_with_embeddings);
        println!();

        println!("Storage");
        println!("  Database: {}", format_bytes(stats.database_size_bytes));
    }

    Ok(())
}
