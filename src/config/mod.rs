use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::{AppError, Result};

pub const APP_NAME: &str = "knowledge-index";
#[allow(dead_code)]
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const DATABASE_FILE_NAME: &str = "index.db";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Maximum file size in MB to index (files larger are skipped)
    pub max_file_size_mb: u32,
    /// Additional glob patterns to ignore
    pub ignore_patterns: Vec<String>,
    /// Enable colored output
    pub color_enabled: bool,
    /// Debounce duration for file watcher in milliseconds
    pub watcher_debounce_ms: u64,
    /// Number of files per database transaction batch
    pub batch_size: usize,
    /// Enable semantic search with embeddings
    pub enable_semantic_search: bool,
    /// Embedding model name (from fastembed)
    pub embedding_model: String,
    /// Default search mode: "lexical", "semantic", or "hybrid"
    pub default_search_mode: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_file_size_mb: 10,
            ignore_patterns: vec![
                String::from(".obsidian"),
                String::from(".git"),
                String::from("node_modules"),
                String::from("target"),
                String::from("__pycache__"),
                String::from(".venv"),
                String::from("venv"),
            ],
            color_enabled: true,
            watcher_debounce_ms: 500,
            batch_size: 100,
            enable_semantic_search: false,
            embedding_model: String::from("all-MiniLM-L6-v2"),
            default_search_mode: String::from("lexical"),
        }
    }
}

impl Config {
    /// Get the configuration directory path for the current OS
    pub fn config_dir() -> Result<PathBuf> {
        dirs::config_dir()
            .map(|p| p.join(APP_NAME))
            .ok_or_else(|| AppError::Config("Could not determine config directory".into()))
    }

    /// Get the path to the config file
    pub fn config_file_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join(CONFIG_FILE_NAME))
    }

    /// Get the path to the database file
    pub fn database_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join(DATABASE_FILE_NAME))
    }

    /// Load configuration from file, creating defaults if needed
    pub fn load() -> Result<Self> {
        let config_dir = Self::config_dir()?;
        let config_path = Self::config_file_path()?;

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        // Load or create config file
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            toml::from_str(&content)
                .map_err(|e| AppError::Config(format!("Failed to parse config: {e}")))
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| AppError::Config(format!("Failed to serialize config: {e}")))?;
        fs::write(config_path, content)?;
        Ok(())
    }

    /// Maximum file size in bytes
    #[must_use]
    pub fn max_file_size_bytes(&self) -> u64 {
        u64::from(self.max_file_size_mb) * 1024 * 1024
    }
}
