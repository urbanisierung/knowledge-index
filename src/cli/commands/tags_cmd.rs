//! Tags listing command.

use crate::cli::args::Args;
use crate::config::Config;
use crate::db::Database;
use crate::error::Result;
use owo_colors::OwoColorize;
use serde::Serialize;

use super::use_colors;

#[derive(Serialize)]
struct TagInfo {
    tag: String,
    count: usize,
}

#[derive(Serialize)]
struct TagsOutput {
    total_tags: usize,
    tags: Vec<TagInfo>,
}

/// List all tags from indexed files
pub fn run(args: &Args) -> Result<()> {
    let db = Database::open()?;
    let _config = Config::load()?;
    let colors = use_colors(args.no_color);

    // Get all tags with counts
    let tags = db.get_all_tags()?;

    if args.json {
        let output = TagsOutput {
            total_tags: tags.len(),
            tags: tags
                .into_iter()
                .map(|(tag, count)| TagInfo { tag, count })
                .collect(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    if tags.is_empty() {
        if !args.quiet {
            println!("No tags found in indexed files.");
            println!();
            println!("Tags are extracted from YAML frontmatter in markdown files:");
            println!("  ---");
            println!("  tags: [rust, cli, tutorial]");
            println!("  ---");
        }
        return Ok(());
    }

    if !args.quiet {
        if colors {
            println!("{}", "Tags".bold());
            println!("{}", "─".repeat(40).dimmed());
        } else {
            println!("Tags");
            println!("{}", "─".repeat(40));
        }
    }

    // Sort by count descending
    let mut tags: Vec<_> = tags.into_iter().collect();
    tags.sort_by(|a, b| b.1.cmp(&a.1));

    for (tag, count) in &tags {
        if colors {
            println!(
                "  {} {} {}",
                "#".dimmed(),
                tag.cyan(),
                format!("({count})").dimmed()
            );
        } else {
            println!("  #{tag} ({count})");
        }
    }

    if !args.quiet {
        println!();
        if colors {
            println!("{} unique tags", tags.len().to_string().green());
        } else {
            println!("{} unique tags", tags.len());
        }
        println!();
        println!("Filter by tag: kdex search \"query\" --tag <tagname>");
    }

    Ok(())
}
