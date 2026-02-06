use crate::db::{Database, SearchResult};
use crate::error::Result;

/// Search engine wrapper
pub struct Searcher {
    db: Database,
}

impl Searcher {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Search indexed content
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
