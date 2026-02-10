use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

use crate::cli::args::{Args, ConfigAction};
use crate::config::Config;
use crate::core::remote::{clone_repository, get_clone_path, parse_github_url};
use crate::db::{Database, SourceType};
use crate::error::{AppError, Result};

use super::{print_success, print_warning, use_colors};

/// Portable configuration format for import/export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableConfig {
    pub version: u32,
    #[serde(default)]
    pub repositories: Vec<PortableRepo>,
    #[serde(default)]
    pub settings: PortableSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableRepo {
    #[serde(rename = "type")]
    pub repo_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortableSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_file_size_mb: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_semantic_search: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_search_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_patterns: Option<Vec<String>>,
}

#[allow(clippy::too_many_lines)]
pub fn run(
    action: Option<ConfigAction>,
    key: Option<String>,
    value: Option<String>,
    reset: bool,
    args: &Args,
) -> Result<()> {
    // Handle new subcommands
    if let Some(action) = action {
        return match action {
            ConfigAction::Show => run_show(args),
            ConfigAction::Export {
                output,
                remotes_only,
                include_local,
                format,
            } => run_export(
                output.as_deref(),
                remotes_only,
                include_local,
                &format,
                args,
            ),
            ConfigAction::Import {
                file,
                merge,
                skip_clone,
            } => run_import(&file, merge, skip_clone, args),
        };
    }

    // Legacy behavior for backwards compatibility
    let colors = use_colors(args.no_color);
    let config_path = Config::config_file_path()?;

    if reset {
        let config = Config::default();
        config.save()?;
        if !args.quiet {
            println!("Configuration reset to defaults.");
        }
        return Ok(());
    }

    if let Some(key) = key {
        if let Some(value) = value {
            // Set value
            let mut config = Config::load()?;
            match key.as_str() {
                "max_file_size_mb" => {
                    config.max_file_size_mb = value
                        .parse()
                        .map_err(|_| AppError::Other("Invalid number".into()))?;
                }
                "color_enabled" => {
                    config.color_enabled = value
                        .parse()
                        .map_err(|_| AppError::Other("Invalid boolean".into()))?;
                }
                "watcher_debounce_ms" => {
                    config.watcher_debounce_ms = value
                        .parse()
                        .map_err(|_| AppError::Other("Invalid number".into()))?;
                }
                "batch_size" => {
                    config.batch_size = value
                        .parse()
                        .map_err(|_| AppError::Other("Invalid number".into()))?;
                }
                "enable_semantic_search" => {
                    config.enable_semantic_search = value
                        .parse()
                        .map_err(|_| AppError::Other("Invalid boolean".into()))?;
                }
                "strip_markdown_syntax" => {
                    config.strip_markdown_syntax = value
                        .parse()
                        .map_err(|_| AppError::Other("Invalid boolean".into()))?;
                }
                "index_code_blocks" => {
                    config.index_code_blocks = value
                        .parse()
                        .map_err(|_| AppError::Other("Invalid boolean".into()))?;
                }
                "embedding_model" => {
                    config.embedding_model.clone_from(&value);
                }
                "default_search_mode" => {
                    if !["lexical", "semantic", "hybrid"].contains(&value.as_str()) {
                        return Err(AppError::Other(
                            "Invalid search mode. Must be: lexical, semantic, or hybrid".into(),
                        ));
                    }
                    config.default_search_mode.clone_from(&value);
                }
                _ => {
                    return Err(AppError::Other(format!("Unknown config key: {key}")));
                }
            }
            config.save()?;
            if !args.quiet {
                println!("Set {key} = {value}");
            }
        } else {
            // Show single value
            let config = Config::load()?;
            let value = match key.as_str() {
                "max_file_size_mb" => config.max_file_size_mb.to_string(),
                "color_enabled" => config.color_enabled.to_string(),
                "watcher_debounce_ms" => config.watcher_debounce_ms.to_string(),
                "batch_size" => config.batch_size.to_string(),
                "enable_semantic_search" => config.enable_semantic_search.to_string(),
                "strip_markdown_syntax" => config.strip_markdown_syntax.to_string(),
                "index_code_blocks" => config.index_code_blocks.to_string(),
                "embedding_model" => config.embedding_model,
                "default_search_mode" => config.default_search_mode,
                _ => {
                    return Err(AppError::Other(format!("Unknown config key: {key}")));
                }
            };
            println!("{value}");
        }
        return Ok(());
    }

    // Show current config (same as `kdex config show`)
    run_show_internal(&config_path, args, colors)
}

fn run_show(args: &Args) -> Result<()> {
    let colors = use_colors(args.no_color);
    let config_path = Config::config_file_path()?;
    run_show_internal(&config_path, args, colors)
}

fn run_show_internal(config_path: &Path, args: &Args, colors: bool) -> Result<()> {
    let config = Config::load()?;

    if args.json {
        println!(
            "{}",
            serde_json::json!({
                "config_path": config_path.to_string_lossy(),
                "config": {
                    "max_file_size_mb": config.max_file_size_mb,
                    "ignore_patterns": config.ignore_patterns,
                    "color_enabled": config.color_enabled,
                    "watcher_debounce_ms": config.watcher_debounce_ms,
                    "batch_size": config.batch_size,
                    "enable_semantic_search": config.enable_semantic_search,
                    "embedding_model": config.embedding_model,
                    "default_search_mode": config.default_search_mode,
                    "strip_markdown_syntax": config.strip_markdown_syntax,
                    "index_code_blocks": config.index_code_blocks,
                }
            })
        );
    } else {
        if colors {
            println!("{}", "Configuration".blue().bold());
            println!("{}", "─".repeat(40).dimmed());
        } else {
            println!("Configuration");
            println!("{}", "─".repeat(40));
        }

        println!("Config file: {}", config_path.display());
        println!();
        println!("max_file_size_mb: {}", config.max_file_size_mb);
        println!("color_enabled: {}", config.color_enabled);
        println!("watcher_debounce_ms: {}", config.watcher_debounce_ms);
        println!("batch_size: {}", config.batch_size);
        println!("enable_semantic_search: {}", config.enable_semantic_search);
        println!("embedding_model: {}", config.embedding_model);
        println!("default_search_mode: {}", config.default_search_mode);
        println!("strip_markdown_syntax: {}", config.strip_markdown_syntax);
        println!("index_code_blocks: {}", config.index_code_blocks);
        println!();
        println!("ignore_patterns:");
        for pattern in &config.ignore_patterns {
            println!("  - {pattern}");
        }

        println!();
        if colors {
            println!(
                "{} Set value: {}",
                "Tip:".dimmed(),
                "kdex config max_file_size_mb 20".cyan()
            );
        } else {
            println!("Tip: Set value: kdex config max_file_size_mb 20");
        }
    }

    Ok(())
}

fn run_export(
    output: Option<&Path>,
    remotes_only: bool,
    include_local: bool,
    format: &str,
    args: &Args,
) -> Result<()> {
    let colors = use_colors(args.no_color);
    let config = Config::load()?;
    let db = Database::open()?;
    let repos = db.list_repositories()?;

    // Build portable config
    let mut portable = PortableConfig {
        version: 1,
        repositories: Vec::new(),
        settings: PortableSettings {
            max_file_size_mb: Some(config.max_file_size_mb),
            enable_semantic_search: Some(config.enable_semantic_search),
            default_search_mode: Some(config.default_search_mode.clone()),
            ignore_patterns: Some(config.ignore_patterns.clone()),
        },
    };

    for repo in repos {
        match repo.source_type {
            SourceType::Remote => {
                portable.repositories.push(PortableRepo {
                    repo_type: "remote".to_string(),
                    path: None,
                    url: repo.remote_url.clone(),
                    branch: repo.remote_branch.clone(),
                    name: Some(repo.name.clone()),
                });
            }
            SourceType::Local => {
                if include_local && !remotes_only {
                    portable.repositories.push(PortableRepo {
                        repo_type: "local".to_string(),
                        path: Some(repo.path.to_string_lossy().to_string()),
                        url: None,
                        branch: None,
                        name: Some(repo.name.clone()),
                    });
                }
            }
        }
    }

    // Serialize
    let output_str = match format {
        "json" => serde_json::to_string_pretty(&portable)
            .map_err(|e| AppError::Other(format!("JSON serialization failed: {e}")))?,
        _ => serde_yaml::to_string(&portable)
            .map_err(|e| AppError::Other(format!("YAML serialization failed: {e}")))?,
    };

    // Write output
    if let Some(path) = output {
        fs::write(path, &output_str)?;
        if !args.quiet {
            print_success(&format!("Exported to {}", path.display()), colors);
        }
    } else {
        println!("{output_str}");
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn run_import(file: &Path, merge: bool, skip_clone: bool, args: &Args) -> Result<()> {
    let colors = use_colors(args.no_color);

    // Read input
    let content = if file.to_string_lossy() == "-" {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        fs::read_to_string(file)?
    };

    // Parse (try YAML first, then JSON)
    let portable: PortableConfig = serde_yaml::from_str(&content)
        .or_else(|_| serde_json::from_str(&content))
        .map_err(|e| AppError::Other(format!("Failed to parse config: {e}")))?;

    if portable.version != 1 {
        return Err(AppError::Other(format!(
            "Unsupported config version: {}",
            portable.version
        )));
    }

    let db = Database::open()?;
    let mut config = Config::load()?;

    // Apply settings
    if !merge {
        if let Some(v) = portable.settings.max_file_size_mb {
            config.max_file_size_mb = v;
        }
        if let Some(v) = portable.settings.enable_semantic_search {
            config.enable_semantic_search = v;
        }
        if let Some(v) = &portable.settings.default_search_mode {
            config.default_search_mode.clone_from(v);
        }
        if let Some(v) = &portable.settings.ignore_patterns {
            config.ignore_patterns.clone_from(v);
        }
        config.save()?;
    }

    if !args.quiet && !args.json {
        println!(
            "Importing {} repositor{}...",
            portable.repositories.len(),
            if portable.repositories.len() == 1 {
                "y"
            } else {
                "ies"
            }
        );
    }

    let mut added = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for repo in &portable.repositories {
        match repo.repo_type.as_str() {
            "remote" => {
                if let Some(url) = &repo.url {
                    // Parse and check if already exists
                    if let Ok((_, owner, repo_name)) = parse_github_url(url) {
                        let clone_path = get_clone_path(&owner, &repo_name)?;

                        if clone_path.exists() {
                            if !args.quiet && !args.json {
                                print_warning(
                                    &format!("Skipping {owner}/{repo_name} (already exists)"),
                                    colors,
                                );
                            }
                            skipped += 1;
                            continue;
                        }

                        if skip_clone {
                            if !args.quiet && !args.json {
                                println!("  Would clone: {owner}/{repo_name}");
                            }
                            continue;
                        }

                        if !args.quiet && !args.json {
                            print!("  Cloning {owner}/{repo_name}... ");
                            io::stdout().flush()?;
                        }

                        let name = repo.name.clone().unwrap_or(format!("{owner}/{repo_name}"));
                        db.add_remote_repository(&clone_path, &name, url, repo.branch.as_deref())?;

                        match clone_repository(
                            url,
                            &clone_path,
                            repo.branch.as_deref(),
                            false,
                            None,
                        ) {
                            Ok(()) => {
                                db.update_repository_synced(
                                    db.get_repository_by_path(&clone_path)?.map_or(0, |r| r.id),
                                )?;
                                if !args.quiet && !args.json {
                                    if colors {
                                        println!("{}", "done".green());
                                    } else {
                                        println!("done");
                                    }
                                }
                                added += 1;
                            }
                            Err(e) => {
                                if !args.quiet && !args.json {
                                    if colors {
                                        println!("{}: {}", "failed".red(), e);
                                    } else {
                                        println!("failed: {e}");
                                    }
                                }
                                failed += 1;
                            }
                        }
                    }
                }
            }
            "local" => {
                if let Some(path) = &repo.path {
                    let path = Path::new(path);
                    if !path.exists() {
                        if !args.quiet && !args.json {
                            print_warning(
                                &format!("Local path not found: {}", path.display()),
                                colors,
                            );
                        }
                        skipped += 1;
                        continue;
                    }

                    if db.get_repository_by_path(path)?.is_some() {
                        skipped += 1;
                        continue;
                    }

                    db.add_repository(path, repo.name.clone())?;
                    added += 1;
                }
            }
            _ => {
                if !args.quiet && !args.json {
                    print_warning(&format!("Unknown repo type: {}", repo.repo_type), colors);
                }
            }
        }
    }

    if args.json {
        println!(
            "{}",
            serde_json::json!({
                "success": failed == 0,
                "added": added,
                "skipped": skipped,
                "failed": failed,
            })
        );
    } else if !args.quiet {
        println!();
        print_success(
            &format!("Imported {added} repositories ({skipped} skipped, {failed} failed)"),
            colors,
        );
    }

    Ok(())
}
