use owo_colors::OwoColorize;

use crate::cli::args::Args;
use crate::config::Config;
use crate::error::Result;

use super::use_colors;

#[allow(clippy::too_many_lines)]
pub fn run(key: Option<String>, value: Option<String>, reset: bool, args: &Args) -> Result<()> {
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
                    config.max_file_size_mb = value.parse().map_err(|_| {
                        crate::error::AppError::Other("Invalid number".into())
                    })?;
                }
                "color_enabled" => {
                    config.color_enabled = value.parse().map_err(|_| {
                        crate::error::AppError::Other("Invalid boolean".into())
                    })?;
                }
                "watcher_debounce_ms" => {
                    config.watcher_debounce_ms = value.parse().map_err(|_| {
                        crate::error::AppError::Other("Invalid number".into())
                    })?;
                }
                "batch_size" => {
                    config.batch_size = value.parse().map_err(|_| {
                        crate::error::AppError::Other("Invalid number".into())
                    })?;
                }
                _ => {
                    return Err(crate::error::AppError::Other(format!(
                        "Unknown config key: {key}"
                    )));
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
                _ => {
                    return Err(crate::error::AppError::Other(format!(
                        "Unknown config key: {key}"
                    )));
                }
            };
            println!("{value}");
        }
        return Ok(());
    }

    // Show current config
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
                "knowledge-index config max_file_size_mb 20".cyan()
            );
        } else {
            println!("Tip: Set value: knowledge-index config max_file_size_mb 20");
        }
    }

    Ok(())
}
