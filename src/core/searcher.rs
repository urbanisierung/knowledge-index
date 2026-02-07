use crate::core::Embedder;
use crate::db::{Database, SearchResult, VectorSearchResult};
use crate::error::Result;

/// Search mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchMode {
    /// Full-text search (FTS5)
    #[default]
    Lexical,
    /// Vector/embedding search
    Semantic,
    /// Combined search with RRF fusion
    Hybrid,
}

impl SearchMode {
    #[must_use]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "semantic" | "vector" => Self::Semantic,
            "hybrid" | "combined" => Self::Hybrid,
            _ => Self::Lexical,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lexical => "lexical",
            Self::Semantic => "semantic",
            Self::Hybrid => "hybrid",
        }
    }
}

/// Unified search result
#[derive(Debug, Clone)]
pub struct UnifiedSearchResult {
    pub repo_name: String,
    #[allow(dead_code)]
    pub repo_path: std::path::PathBuf,
    pub file_path: std::path::PathBuf,
    pub absolute_path: std::path::PathBuf,
    pub snippet: String,
    pub file_type: String,
    pub score: f64,
    pub search_mode: SearchMode,
}

impl From<SearchResult> for UnifiedSearchResult {
    fn from(r: SearchResult) -> Self {
        Self {
            repo_name: r.repo_name,
            repo_path: r.repo_path,
            file_path: r.file_path,
            absolute_path: r.absolute_path,
            snippet: r.snippet,
            file_type: r.file_type,
            score: r.score,
            search_mode: SearchMode::Lexical,
        }
    }
}

impl From<VectorSearchResult> for UnifiedSearchResult {
    fn from(r: VectorSearchResult) -> Self {
        Self {
            repo_name: r.repo_name,
            repo_path: r.repo_path,
            file_path: r.file_path,
            absolute_path: r.absolute_path,
            snippet: r.chunk_text,
            file_type: r.file_type,
            score: f64::from(r.similarity),
            search_mode: SearchMode::Semantic,
        }
    }
}

/// Search engine wrapper
pub struct Searcher {
    db: Database,
    embedder: Option<Embedder>,
}

impl Searcher {
    pub fn new(db: Database) -> Self {
        Self { db, embedder: None }
    }

    /// Create searcher with embedding support
    pub fn with_embedder(db: Database, embedder: Embedder) -> Self {
        Self {
            db,
            embedder: Some(embedder),
        }
    }

    /// Search indexed content with specified mode
    pub fn search_with_mode(
        &self,
        query: &str,
        mode: SearchMode,
        repo: Option<&str>,
        file_type: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<UnifiedSearchResult>> {
        match mode {
            SearchMode::Lexical => self.lexical_search(query, repo, file_type, limit, offset),
            SearchMode::Semantic => self.semantic_search(query, repo, file_type, limit),
            SearchMode::Hybrid => self.hybrid_search(query, repo, file_type, limit),
        }
    }

    /// Lexical (FTS5) search
    fn lexical_search(
        &self,
        query: &str,
        repo: Option<&str>,
        file_type: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<UnifiedSearchResult>> {
        let escaped_query = Self::escape_fts_query(query);
        let results = self.db.search(&escaped_query, repo, file_type, limit, offset)?;
        Ok(results.into_iter().map(UnifiedSearchResult::from).collect())
    }

    /// Semantic (vector) search
    fn semantic_search(
        &self,
        query: &str,
        repo: Option<&str>,
        file_type: Option<&str>,
        limit: usize,
    ) -> Result<Vec<UnifiedSearchResult>> {
        let embedder = self.embedder.as_ref().ok_or_else(|| {
            crate::error::AppError::Config("Semantic search requires embeddings. Enable with: enable_semantic_search = true".into())
        })?;

        let query_embedding = embedder.embed_query(query)?;
        let results = self.db.vector_search(&query_embedding, repo, file_type, limit)?;
        Ok(results.into_iter().map(UnifiedSearchResult::from).collect())
    }

    /// Hybrid search with Reciprocal Rank Fusion
    fn hybrid_search(
        &self,
        query: &str,
        repo: Option<&str>,
        file_type: Option<&str>,
        limit: usize,
    ) -> Result<Vec<UnifiedSearchResult>> {
        // RRF fusion with k=60 (standard constant)
        const RRF_K: f64 = 60.0;

        // Get results from both search methods
        let lexical_results = self.lexical_search(query, repo, file_type, limit * 2, 0)?;
        let semantic_results = self.semantic_search(query, repo, file_type, limit * 2)?;

        // Calculate RRF scores
        let mut scores: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        let mut result_map: std::collections::HashMap<String, UnifiedSearchResult> = std::collections::HashMap::new();

        for (rank, result) in lexical_results.into_iter().enumerate() {
            let key = result.absolute_path.to_string_lossy().to_string();
            #[allow(clippy::cast_precision_loss)]
            let rrf_score = 1.0 / (RRF_K + (rank as f64) + 1.0);
            *scores.entry(key.clone()).or_insert(0.0) += rrf_score;
            result_map.entry(key).or_insert(result);
        }

        for (rank, result) in semantic_results.into_iter().enumerate() {
            let key = result.absolute_path.to_string_lossy().to_string();
            #[allow(clippy::cast_precision_loss)]
            let rrf_score = 1.0 / (RRF_K + (rank as f64) + 1.0);
            *scores.entry(key.clone()).or_insert(0.0) += rrf_score;
            result_map.entry(key).or_insert(result);
        }

        // Sort by combined RRF score
        let mut sorted_keys: Vec<_> = scores.into_iter().collect();
        sorted_keys.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top results with updated scores
        let mut results: Vec<UnifiedSearchResult> = Vec::new();
        for (key, score) in sorted_keys.into_iter().take(limit) {
            if let Some(mut result) = result_map.remove(&key) {
                result.score = score;
                result.search_mode = SearchMode::Hybrid;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Legacy search method (lexical only)
    pub fn search(
        &self,
        query: &str,
        repo: Option<&str>,
        file_type: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<SearchResult>> {
        // Escape special FTS5 characters in query
        let escaped_query = Self::escape_fts_query(query);
        self.db.search(&escaped_query, repo, file_type, limit, offset)
    }

    /// Count total results
    #[allow(dead_code)]
    pub fn count(
        &self,
        query: &str,
        repo: Option<&str>,
        file_type: Option<&str>,
    ) -> Result<i64> {
        let escaped_query = Self::escape_fts_query(query);
        self.db.search_count(&escaped_query, repo, file_type)
    }

    /// Check if semantic search is available
    #[must_use]
    pub fn has_semantic_search(&self) -> bool {
        self.embedder.is_some()
    }

    /// Escape special FTS5 characters
    fn escape_fts_query(query: &str) -> String {
        // Handle quoted phrases
        if query.starts_with('"') && query.ends_with('"') {
            return query.to_string();
        }

        // Escape special characters except * (wildcard)
        let mut result = String::with_capacity(query.len());
        for c in query.chars() {
            match c {
                '"' | '\'' | '(' | ')' | ':' | '^' | '-' => {
                    result.push(' ');
                }
                _ => result.push(c),
            }
        }

        result
    }
}
