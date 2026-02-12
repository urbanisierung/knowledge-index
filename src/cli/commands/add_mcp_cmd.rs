use crate::cli::args::McpTool;
use crate::error::{AppError, Result};
use std::fs;
use std::path::PathBuf;

/// Get the config path for each tool
fn get_config_path(tool: McpTool) -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Other("Could not determine home directory".to_string()))?;

    let path = match tool {
        McpTool::Copilot => {
            #[cfg(target_os = "windows")]
            {
                dirs::data_dir()
                    .unwrap_or_else(|| home.clone())
                    .join("GitHub Copilot")
                    .join("mcp.json")
            }
            #[cfg(not(target_os = "windows"))]
            {
                home.join(".config/github-copilot/mcp.json")
            }
        }
        McpTool::Gemini => home.join(".gemini/settings.json"),
        McpTool::Claude => {
            #[cfg(target_os = "macos")]
            {
                home.join("Library/Application Support/Claude/claude_desktop_config.json")
            }
            #[cfg(target_os = "windows")]
            {
                dirs::data_dir()
                    .unwrap_or_else(|| home.clone())
                    .join("Claude")
                    .join("claude_desktop_config.json")
            }
            #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
            {
                home.join(".config/claude/claude_desktop_config.json")
            }
        }
    };

    Ok(path)
}

/// Get the kdex binary path
fn get_kdex_path() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| "kdex".to_string())
}

/// Generate MCP config for kdex
fn generate_mcp_config(tool: McpTool) -> serde_json::Value {
    let kdex_path = get_kdex_path();

    match tool {
        McpTool::Copilot | McpTool::Claude => {
            serde_json::json!({
                "command": kdex_path,
                "args": ["mcp"]
            })
        }
        McpTool::Gemini => {
            serde_json::json!({
                "command": kdex_path,
                "args": ["mcp"],
                "timeout": 30000
            })
        }
    }
}

pub fn run(tool: McpTool, json_output: bool) -> Result<()> {
    let config_path = get_config_path(tool)?;
    let tool_name = match tool {
        McpTool::Copilot => "GitHub Copilot CLI",
        McpTool::Gemini => "Gemini CLI",
        McpTool::Claude => "Claude Desktop",
    };

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::Other(format!(
                "Failed to create config directory {}: {}",
                parent.display(),
                e
            ))
        })?;
    }

    // Read existing config or create new one
    let mut config: serde_json::Value = if config_path.exists() {
        let content = fs::read_to_string(&config_path).map_err(|e| {
            AppError::Other(format!(
                "Failed to read config {}: {}",
                config_path.display(),
                e
            ))
        })?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Ensure mcpServers object exists
    if config.get("mcpServers").is_none() {
        config["mcpServers"] = serde_json::json!({});
    }

    // Check if kdex is already configured
    let already_configured = config
        .get("mcpServers")
        .and_then(|s| s.get("kdex"))
        .is_some();

    // Add kdex MCP config
    config["mcpServers"]["kdex"] = generate_mcp_config(tool);

    // Write config
    let formatted = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, &formatted).map_err(|e| {
        AppError::Other(format!(
            "Failed to write config {}: {}",
            config_path.display(),
            e
        ))
    })?;

    if json_output {
        let result = serde_json::json!({
            "success": true,
            "tool": format!("{tool:?}").to_lowercase(),
            "config_path": config_path.to_string_lossy(),
            "action": if already_configured { "updated" } else { "added" }
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if already_configured {
            println!("✅ Updated kdex MCP configuration for {tool_name}");
        } else {
            println!("✅ Added kdex MCP configuration for {tool_name}");
        }
        println!("   Config: {}", config_path.display());
        println!();
        println!("   {tool_name} can now use kdex to search your knowledge.");
        println!();
        println!("   Available MCP tools:");
        println!("   • search       - Search indexed content");
        println!("   • list_repos   - List indexed repositories");
        println!("   • get_file     - Get file contents");
        println!("   • get_context  - Get code context around a line");
    }

    Ok(())
}
