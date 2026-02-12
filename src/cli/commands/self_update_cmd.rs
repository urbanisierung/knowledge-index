use crate::error::{AppError, Result};
use std::fs;
#[cfg(not(target_os = "windows"))]
use std::process::Command;

#[cfg(not(target_os = "windows"))]
const INSTALL_SCRIPT_URL: &str = "https://urbanisierung.github.io/kdex/install.sh";

/// Check if kdex was installed via the install script
fn was_installed_via_script() -> bool {
    let config_dir = dirs::config_dir()
        .map(|d| d.join("kdex"))
        .or_else(|| dirs::home_dir().map(|h| h.join(".config/kdex")));

    if let Some(config_dir) = config_dir {
        let marker_path = config_dir.join(".install-method");
        if let Ok(content) = fs::read_to_string(&marker_path) {
            return content.trim() == "script";
        }
    }

    false
}

pub fn run(json_output: bool) -> Result<()> {
    if !was_installed_via_script() {
        if json_output {
            let result = serde_json::json!({
                "success": false,
                "error": "not_script_install",
                "message": "kdex was not installed via the install script"
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("❌ Self-update is only available for script-based installations.");
            println!();
            println!("   Update using your original installation method:");
            println!();
            println!(
                "   • Install script:  curl -sSf https://urbanisierung.github.io/kdex/install.sh | sh"
            );
            println!("   • Cargo:           cargo install kdex");
            println!("   • From source:     git pull && cargo install --path .");
            println!();
        }
        return Ok(());
    }

    if json_output {
        println!(
            "{}",
            serde_json::json!({
                "status": "updating",
                "message": "Running install script..."
            })
        );
    } else {
        println!("🔄 Updating kdex...");
        println!();
    }

    // Run the install script
    #[cfg(target_os = "windows")]
    {
        Err(AppError::Other(
            "Self-update via script is not supported on Windows. Use: cargo install kdex"
                .to_string(),
        ))
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix: curl the script and pipe to sh
        let status = Command::new("sh")
            .arg("-c")
            .arg(format!("curl -sSf {INSTALL_SCRIPT_URL} | sh"))
            .status()
            .map_err(|e| AppError::Other(format!("Failed to run install script: {e}")))?;

        if status.success() {
            if json_output {
                let result = serde_json::json!({
                    "success": true,
                    "message": "kdex updated successfully"
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            // Success message is printed by the install script itself
        } else if json_output {
            let result = serde_json::json!({
                "success": false,
                "error": "update_failed",
                "exit_code": status.code()
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("❌ Update failed. Exit code: {:?}", status.code());
            println!();
            println!("   Try updating manually:");
            println!("   curl -sSf {INSTALL_SCRIPT_URL} | sh");
        }

        Ok(())
    }
}
