use chrono::Utc;
use owo_colors::OwoColorize;

use crate::cli::args::Args;
use crate::db::{Database, RepoStatus, SourceType};
use crate::error::Result;

use super::use_colors;

#[allow(clippy::too_many_lines)]
pub fn run(args: &Args) -> Result<()> {
    let colors = use_colors(args.no_color);
    let db = Database::open()?;

    let repos = db.list_repositories()?;

    if repos.is_empty() {
        if args.json {
            println!("{}", serde_json::json!({"repositories": []}));
        } else if !args.quiet {
            println!("No repositories indexed yet.");
            println!();
            println!("Get started by indexing a project:");
            if colors {
                println!("  {}", "kdex index /path/to/project".cyan());
                println!("  {}", "kdex add --remote owner/repo".cyan());
            } else {
                println!("  kdex index /path/to/project");
                println!("  kdex add --remote owner/repo");
            }
        }
        return Ok(());
    }

    if args.json {
        let json_repos: Vec<_> = repos
            .iter()
            .map(|r| {
                serde_json::json!({
                    "name": r.name,
                    "path": r.path.to_string_lossy(),
                    "file_count": r.file_count,
                    "total_size_bytes": r.total_size_bytes,
                    "status": r.status.as_str(),
                    "source_type": r.source_type.as_str(),
                    "remote_url": r.remote_url,
                    "remote_branch": r.remote_branch,
                    "last_indexed_at": r.last_indexed_at.map(|dt| dt.to_rfc3339()),
                    "last_synced_at": r.last_synced_at.map(|dt| dt.to_rfc3339()),
                    "created_at": r.created_at.to_rfc3339(),
                })
            })
            .collect();

        println!("{}", serde_json::json!({"repositories": json_repos}));
    } else if !args.quiet {
        let now = Utc::now();

        for repo in &repos {
            // Status indicator
            let status_icon = match repo.status {
                RepoStatus::Ready => {
                    if colors {
                        "●".green().to_string()
                    } else {
                        "●".to_string()
                    }
                }
                RepoStatus::Pending => {
                    if colors {
                        "○".yellow().to_string()
                    } else {
                        "○".to_string()
                    }
                }
                RepoStatus::Indexing | RepoStatus::Syncing => {
                    if colors {
                        "◐".cyan().to_string()
                    } else {
                        "◐".to_string()
                    }
                }
                RepoStatus::Cloning => {
                    if colors {
                        "↓".cyan().to_string()
                    } else {
                        "↓".to_string()
                    }
                }
                RepoStatus::Error => {
                    if colors {
                        "!".red().to_string()
                    } else {
                        "!".to_string()
                    }
                }
            };

            // Source type indicator
            let source_indicator = match repo.source_type {
                SourceType::Remote => {
                    if colors {
                        "☁".dimmed().to_string()
                    } else {
                        "☁".to_string()
                    }
                }
                SourceType::Local => " ".to_string(),
            };

            // Format time ago
            let time_ago = repo.last_indexed_at.map_or_else(
                || "never".to_string(),
                |dt| format_time_ago(now.signed_duration_since(dt)),
            );

            // Format size
            #[allow(clippy::cast_sign_loss)]
            let size_str = format_bytes(repo.total_size_bytes as u64);

            if colors {
                println!(
                    "{}{} {:<20} │ {:>6} files │ {:>8} │ {}",
                    status_icon,
                    source_indicator,
                    repo.name.blue(),
                    repo.file_count,
                    size_str,
                    time_ago.dimmed()
                );
            } else {
                println!(
                    "{}{} {:<20} │ {:>6} files │ {:>8} │ {}",
                    status_icon, source_indicator, repo.name, repo.file_count, size_str, time_ago
                );
            }
        }

        println!();
        let remote_count = repos
            .iter()
            .filter(|r| r.source_type == SourceType::Remote)
            .count();
        let local_count = repos.len() - remote_count;
        println!(
            "{} local, {} remote │ Status: {} ready  {} pending  {} syncing  {} error",
            local_count,
            remote_count,
            if colors {
                "●".green().to_string()
            } else {
                "●".to_string()
            },
            if colors {
                "○".yellow().to_string()
            } else {
                "○".to_string()
            },
            if colors {
                "◐".cyan().to_string()
            } else {
                "◐".to_string()
            },
            if colors {
                "!".red().to_string()
            } else {
                "!".to_string()
            },
        );
    }

    Ok(())
}

fn format_time_ago(duration: chrono::Duration) -> String {
    let seconds = duration.num_seconds();

    if seconds < 60 {
        "just now".to_string()
    } else if seconds < 3600 {
        let mins = seconds / 60;
        format!("{mins} min{} ago", if mins == 1 { "" } else { "s" })
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        format!("{hours} hour{} ago", if hours == 1 { "" } else { "s" })
    } else {
        let days = seconds / 86400;
        format!("{days} day{} ago", if days == 1 { "" } else { "s" })
    }
}

#[allow(clippy::cast_precision_loss)]
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}
