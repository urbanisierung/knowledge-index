//! Search history management for the TUI.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::error::Result;

const HISTORY_FILE_NAME: &str = "search_history.json";
const DEFAULT_MAX_HISTORY: usize = 50;

/// Search history storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistory {
    /// Recent search queries (newest first)
    pub queries: VecDeque<String>,
    /// Maximum number of queries to store
    #[serde(default = "default_max_history")]
    pub max_size: usize,
}

fn default_max_history() -> usize {
    DEFAULT_MAX_HISTORY
}

impl Default for SearchHistory {
    fn default() -> Self {
        Self {
            queries: VecDeque::new(),
            max_size: DEFAULT_MAX_HISTORY,
        }
    }
}

impl SearchHistory {
    /// Load search history from disk
    pub fn load() -> Result<Self> {
        let path = Self::history_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)?;
        let history: Self = serde_json::from_str(&content).unwrap_or_default();
        Ok(history)
    }

    /// Save search history to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::history_path()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Add a query to history
    pub fn add(&mut self, query: &str) {
        let query = query.trim().to_string();
        if query.is_empty() {
            return;
        }

        // Remove if already exists (to move to front)
        self.queries.retain(|q| q != &query);

        // Add to front
        self.queries.push_front(query);

        // Trim to max size
        while self.queries.len() > self.max_size {
            self.queries.pop_back();
        }
    }

    /// Get recent queries
    #[allow(dead_code)]
    pub fn recent(&self) -> impl Iterator<Item = &String> {
        self.queries.iter()
    }

    /// Get query at index (0 = most recent)
    pub fn get(&self, index: usize) -> Option<&String> {
        self.queries.get(index)
    }

    /// Number of queries in history
    pub fn len(&self) -> usize {
        self.queries.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.queries.is_empty()
    }

    /// Clear all history
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.queries.clear();
    }

    /// Get history file path
    fn history_path() -> Result<PathBuf> {
        let config_dir = Config::config_dir()?;
        Ok(config_dir.join(HISTORY_FILE_NAME))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_query() {
        let mut history = SearchHistory::default();
        history.add("first query");
        history.add("second query");

        assert_eq!(history.len(), 2);
        assert_eq!(history.get(0), Some(&"second query".to_string()));
        assert_eq!(history.get(1), Some(&"first query".to_string()));
    }

    #[test]
    fn test_dedup() {
        let mut history = SearchHistory::default();
        history.add("query");
        history.add("other");
        history.add("query"); // Should move to front

        assert_eq!(history.len(), 2);
        assert_eq!(history.get(0), Some(&"query".to_string()));
    }

    #[test]
    fn test_max_size() {
        let mut history = SearchHistory {
            max_size: 3,
            ..Default::default()
        };

        history.add("1");
        history.add("2");
        history.add("3");
        history.add("4");

        assert_eq!(history.len(), 3);
        assert_eq!(history.get(0), Some(&"4".to_string()));
    }
}
