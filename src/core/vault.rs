//! Vault type detection for knowledge management tools.

#![allow(dead_code)]

use std::path::Path;

/// Types of knowledge vaults supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VaultType {
    /// Obsidian vault (has .obsidian folder)
    Obsidian,
    /// Logseq graph (has logseq folder)
    Logseq,
    /// Dendron workspace (has dendron.yml)
    Dendron,
    /// Generic repository or notes folder
    #[default]
    Generic,
}

impl VaultType {
    /// Detect vault type from a directory path
    pub fn detect(path: &Path) -> Self {
        // Check for Obsidian (.obsidian folder)
        if path.join(".obsidian").is_dir() {
            return Self::Obsidian;
        }

        // Check for Logseq (logseq folder with config)
        if path.join("logseq").is_dir() {
            return Self::Logseq;
        }

        // Check for Dendron (dendron.yml or dendron.code-workspace)
        if path.join("dendron.yml").is_file() || path.join("dendron.code-workspace").is_file() {
            return Self::Dendron;
        }

        Self::Generic
    }

    /// Get the vault type as a string for display
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Obsidian => "obsidian",
            Self::Logseq => "logseq",
            Self::Dendron => "dendron",
            Self::Generic => "generic",
        }
    }

    /// Parse vault type from string
    #[allow(dead_code)]
    #[must_use]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "obsidian" => Self::Obsidian,
            "logseq" => Self::Logseq,
            "dendron" => Self::Dendron,
            _ => Self::Generic,
        }
    }

    /// Get display name with icon
    #[allow(dead_code)]
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Obsidian => "📓 Obsidian",
            Self::Logseq => "📔 Logseq",
            Self::Dendron => "🌳 Dendron",
            Self::Generic => "📁 Generic",
        }
    }

    /// Get recommended file patterns for this vault type
    #[allow(dead_code, clippy::match_same_arms)]
    #[must_use]
    pub fn recommended_patterns(self) -> &'static [&'static str] {
        match self {
            Self::Obsidian => &["**/*.md"],
            Self::Logseq => &["pages/**/*.md", "journals/**/*.md"],
            Self::Dendron => &["**/*.md"],
            Self::Generic => &["**/*.md", "**/*.txt"],
        }
    }

    /// Get paths to exclude for this vault type
    #[allow(dead_code, clippy::match_same_arms)]
    #[must_use]
    pub fn excluded_paths(self) -> &'static [&'static str] {
        match self {
            Self::Obsidian => &[".obsidian", ".trash"],
            Self::Logseq => &["logseq/.recycle", ".git"],
            Self::Dendron => &[".git"],
            Self::Generic => &[".git", "node_modules", "target"],
        }
    }
}

impl std::fmt::Display for VaultType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_type_from_str() {
        assert_eq!(VaultType::from_str("obsidian"), VaultType::Obsidian);
        assert_eq!(VaultType::from_str("LOGSEQ"), VaultType::Logseq);
        assert_eq!(VaultType::from_str("dendron"), VaultType::Dendron);
        assert_eq!(VaultType::from_str("unknown"), VaultType::Generic);
    }

    #[test]
    fn test_vault_type_as_str() {
        assert_eq!(VaultType::Obsidian.as_str(), "obsidian");
        assert_eq!(VaultType::Generic.as_str(), "generic");
    }
}
