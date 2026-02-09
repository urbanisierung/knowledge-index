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
#[allow(clippy::struct_excessive_bools)]
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
    /// Strip markdown syntax from indexed content for cleaner FTS
    pub strip_markdown_syntax: bool,
    /// Index code blocks with their language tags
    pub index_code_blocks: bool,
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
            strip_markdown_syntax: false,
            index_code_blocks: true,
        }
    }
}

impl Config {
    /// Get the configuration directory path for the current OS
    pub fn config_dir() -> Result<PathBuf> {
        // Allow override via environment variable (useful for testing)
        if let Ok(dir) = std::env::var("KNOWLEDGE_INDEX_CONFIG_DIR") {
            return Ok(PathBuf::from(dir));
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.max_file_size_mb, 10);
        assert!(config.ignore_patterns.contains(&".git".to_string()));
        assert!(config.color_enabled);
        assert_eq!(config.batch_size, 100);
        assert!(!config.enable_semantic_search);
        assert_eq!(config.default_search_mode, "lexical");
    }

    #[test]
    fn test_max_file_size_bytes() {
        let config = Config::default();
        assert_eq!(config.max_file_size_bytes(), 10 * 1024 * 1024);

        let config = Config {
            max_file_size_mb: 5,
            ..Default::default()
        };
        assert_eq!(config.max_file_size_bytes(), 5 * 1024 * 1024);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("max_file_size_mb"));
        assert!(toml_str.contains("ignore_patterns"));

        // Round-trip
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.max_file_size_mb, config.max_file_size_mb);
        assert_eq!(parsed.batch_size, config.batch_size);
    }

    #[test]
    fn test_config_partial_parsing() {
        // Config should use defaults for missing fields
        let partial_toml = r"
            max_file_size_mb = 20
        ";
        let config: Config = toml::from_str(partial_toml).unwrap();
        assert_eq!(config.max_file_size_mb, 20);
        assert_eq!(config.batch_size, 100); // default
        assert!(config.color_enabled); // default
    }
}
