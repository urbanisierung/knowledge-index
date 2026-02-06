use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::config::Config;
use crate::error::{AppError, Result};

mod schema;


/// Repository status in the index
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoStatus {
    Pending,
    Indexing,
    Ready,
    Error,
}

impl RepoStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Indexing => "indexing",
            Self::Ready => "ready",
            Self::Error => "error",
        }
    }

    #[must_use]
    pub fn from_str(s: &str) -> Self {
        match s {
            "indexing" => Self::Indexing,
            "ready" => Self::Ready,
            "error" => Self::Error,
            _ => Self::Pending,
        }
    }
}

/// File type classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    Code(String),     // Language name
    Markdown,
    PlainText,
    OrgMode,
    ReStructuredText,
    Config,
    Unknown,
}

impl FileType {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Code(lang) => lang,
            Self::Markdown => "markdown",
            Self::PlainText => "plaintext",
            Self::OrgMode => "orgmode",
            Self::ReStructuredText => "rst",
            Self::Config => "config",
            Self::Unknown => "unknown",
        }
    }

    #[must_use]
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            // Code
            "rs" => Self::Code("rust".into()),
            "py" | "pyw" => Self::Code("python".into()),
            "js" | "jsx" | "mjs" => Self::Code("javascript".into()),
            "ts" | "tsx" => Self::Code("typescript".into()),
            "go" => Self::Code("go".into()),
            "java" => Self::Code("java".into()),
            "c" | "h" => Self::Code("c".into()),
            "cpp" | "cc" | "cxx" | "hpp" => Self::Code("cpp".into()),
            "cs" => Self::Code("csharp".into()),
            "rb" => Self::Code("ruby".into()),
            "php" => Self::Code("php".into()),
            "swift" => Self::Code("swift".into()),
            "kt" | "kts" => Self::Code("kotlin".into()),
            "scala" => Self::Code("scala".into()),
            "r" => Self::Code("r".into()),
            "lua" => Self::Code("lua".into()),
            "sh" | "bash" | "zsh" => Self::Code("shell".into()),
            "sql" => Self::Code("sql".into()),
            "html" | "htm" => Self::Code("html".into()),
            "css" | "scss" | "sass" | "less" => Self::Code("css".into()),
            "vue" => Self::Code("vue".into()),
            "svelte" => Self::Code("svelte".into()),
            "zig" => Self::Code("zig".into()),
            "ex" | "exs" => Self::Code("elixir".into()),
            "erl" | "hrl" => Self::Code("erlang".into()),
            "hs" => Self::Code("haskell".into()),
            "clj" | "cljs" => Self::Code("clojure".into()),
            "ml" | "mli" => Self::Code("ocaml".into()),
            "fs" | "fsx" => Self::Code("fsharp".into()),
            "nim" => Self::Code("nim".into()),
            "v" => Self::Code("v".into()),
            "d" => Self::Code("d".into()),
            // Markdown/Documentation
            "md" | "markdown" | "mdown" | "mkd" => Self::Markdown,
            "txt" => Self::PlainText,
            "org" => Self::OrgMode,
            "rst" => Self::ReStructuredText,
            // Config
            "json" | "jsonc" | "yaml" | "yml" | "toml" | "xml" | "ini" | "cfg" | "env" => {
                Self::Config
            }
            _ => Self::Unknown,
        }
    }
}

/// Repository record
#[derive(Debug, Clone)]
pub struct Repository {
    pub id: i64,
    pub path: PathBuf,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_indexed_at: Option<DateTime<Utc>>,
    pub file_count: i64,
    pub total_size_bytes: i64,
    pub status: RepoStatus,
}

/// File record
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileRecord {
    pub id: i64,
    pub repo_id: i64,
    pub relative_path: PathBuf,
    pub content_hash: String,
    pub file_size_bytes: i64,
    pub last_modified_at: DateTime<Utc>,
    pub file_type: String,
}

/// Search result
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SearchResult {
    pub repo_name: String,
    pub repo_path: PathBuf,
    pub file_path: PathBuf,
    pub absolute_path: PathBuf,
    pub snippet: String,
    pub file_type: String,
    pub score: f64,
}

/// Database connection wrapper
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Open or create the database
    pub fn open() -> Result<Self> {
        let db_path = Config::database_path()?;
        
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        db.initialize()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing)
    #[allow(dead_code)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.initialize()?;
        Ok(db)
    }

    /// Initialize database schema
    fn initialize(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;
        schema::initialize(&conn)?;
        Ok(())
    }

    /// Add a new repository
    pub fn add_repository(&self, path: &Path, name: Option<String>) -> Result<Repository> {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;
        
        let canonical = path.canonicalize()?;
        let name = name.unwrap_or_else(|| {
            canonical
                .file_name().map_or_else(|| "unknown".to_string(), |n| n.to_string_lossy().to_string())
        });
        let now = Utc::now();

        conn.execute(
            "INSERT INTO repositories (path, name, created_at, status) VALUES (?1, ?2, ?3, ?4)",
            params![
                canonical.to_string_lossy(),
                name,
                now.to_rfc3339(),
                RepoStatus::Pending.as_str(),
            ],
        )?;

        let id = conn.last_insert_rowid();
        
        Ok(Repository {
            id,
            path: canonical,
            name,
            created_at: now,
            last_indexed_at: None,
            file_count: 0,
            total_size_bytes: 0,
            status: RepoStatus::Pending,
        })
    }

    /// Get repository by path
    pub fn get_repository_by_path(&self, path: &Path) -> Result<Option<Repository>> {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        
        let mut stmt = conn.prepare(
            "SELECT id, path, name, created_at, last_indexed_at, file_count, total_size_bytes, status 
             FROM repositories WHERE path = ?1"
        )?;

        let result = stmt.query_row(params![canonical.to_string_lossy()], |row| {
            Ok(Repository {
                id: row.get(0)?,
                path: PathBuf::from(row.get::<_, String>(1)?),
                name: row.get(2)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?).map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc)),
                last_indexed_at: row.get::<_, Option<String>>(4)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                file_count: row.get(5)?,
                total_size_bytes: row.get(6)?,
                status: RepoStatus::from_str(&row.get::<_, String>(7)?),
            })
        });

        match result {
            Ok(repo) => Ok(Some(repo)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all repositories
    pub fn list_repositories(&self) -> Result<Vec<Repository>> {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;
        
        let mut stmt = conn.prepare(
            "SELECT id, path, name, created_at, last_indexed_at, file_count, total_size_bytes, status 
             FROM repositories ORDER BY name"
        )?;

        let repos = stmt.query_map([], |row| {
            Ok(Repository {
                id: row.get(0)?,
                path: PathBuf::from(row.get::<_, String>(1)?),
                name: row.get(2)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?).map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc)),
                last_indexed_at: row.get::<_, Option<String>>(4)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                file_count: row.get(5)?,
                total_size_bytes: row.get(6)?,
                status: RepoStatus::from_str(&row.get::<_, String>(7)?),
            })
        })?
        .filter_map(std::result::Result::ok)
        .collect();

        Ok(repos)
    }

    /// Update repository status
    pub fn update_repository_status(&self, repo_id: i64, status: RepoStatus) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;
        conn.execute(
            "UPDATE repositories SET status = ?1 WHERE id = ?2",
            params![status.as_str(), repo_id],
        )?;
        Ok(())
    }

    /// Update repository after indexing
    pub fn update_repository_indexed(
        &self,
        repo_id: i64,
        file_count: i64,
        total_size_bytes: i64,
    ) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;
        let now = Utc::now();
        
        conn.execute(
            "UPDATE repositories SET last_indexed_at = ?1, file_count = ?2, total_size_bytes = ?3, status = ?4 
             WHERE id = ?5",
            params![
                now.to_rfc3339(),
                file_count,
                total_size_bytes,
                RepoStatus::Ready.as_str(),
                repo_id,
            ],
        )?;
        Ok(())
    }

    /// Delete a repository and all its files
    pub fn delete_repository(&self, repo_id: i64) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;
        
        // Delete FTS content first
        conn.execute(
            "DELETE FROM contents WHERE file_id IN (SELECT id FROM files WHERE repo_id = ?1)",
            params![repo_id],
        )?;
        
        // Delete files
        conn.execute("DELETE FROM files WHERE repo_id = ?1", params![repo_id])?;
        
        // Delete repository
        conn.execute("DELETE FROM repositories WHERE id = ?1", params![repo_id])?;
        
        Ok(())
    }

    /// Delete repository by path
    /// Delete repository by path
    #[allow(dead_code)]
    pub fn delete_repository_by_path(&self, path: &Path) -> Result<()> {
        if let Some(repo) = self.get_repository_by_path(path)? {
            self.delete_repository(repo.id)?;
        }
        Ok(())
    }

    /// Begin a transaction for batch operations
    pub fn begin_batch(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;
        conn.execute("BEGIN TRANSACTION", [])?;
        Ok(())
    }

    /// Commit the current transaction
    pub fn commit_batch(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;
        conn.execute("COMMIT", [])?;
        Ok(())
    }

    /// Rollback the current transaction
    #[allow(dead_code)]
    pub fn rollback_batch(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;
        conn.execute("ROLLBACK", [])?;
        Ok(())
    }

    /// Insert a file record
    #[allow(clippy::too_many_arguments)]
    pub fn insert_file(
        &self,
        repo_id: i64,
        relative_path: &Path,
        content_hash: &str,
        file_size_bytes: i64,
        last_modified: DateTime<Utc>,
        file_type: &str,
        content: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;

        conn.execute(
            "INSERT OR REPLACE INTO files (repo_id, relative_path, content_hash, file_size_bytes, last_modified_at, file_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                repo_id,
                relative_path.to_string_lossy(),
                content_hash,
                file_size_bytes,
                last_modified.to_rfc3339(),
                file_type,
            ],
        )?;

        let file_id = conn.last_insert_rowid();

        // Insert into FTS table
        conn.execute(
            "INSERT INTO contents (file_id, content) VALUES (?1, ?2)",
            params![file_id, content],
        )?;

        Ok(file_id)
    }

    /// Get existing files for a repository (for incremental updates)
    pub fn get_repository_files(&self, repo_id: i64) -> Result<Vec<FileRecord>> {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;
        
        let mut stmt = conn.prepare(
            "SELECT id, repo_id, relative_path, content_hash, file_size_bytes, last_modified_at, file_type 
             FROM files WHERE repo_id = ?1"
        )?;

        let files = stmt.query_map(params![repo_id], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                repo_id: row.get(1)?,
                relative_path: PathBuf::from(row.get::<_, String>(2)?),
                content_hash: row.get(3)?,
                file_size_bytes: row.get(4)?,
                last_modified_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?).map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc)),
                file_type: row.get(6)?,
            })
        })?
        .filter_map(std::result::Result::ok)
        .collect();

        Ok(files)
    }

    /// Delete files by IDs
    pub fn delete_files(&self, file_ids: &[i64]) -> Result<()> {
        if file_ids.is_empty() {
            return Ok(());
        }
        
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;
        
        let placeholders: Vec<String> = file_ids.iter().map(|_| "?".to_string()).collect();
        let placeholders_str = placeholders.join(",");
        
        // Delete from FTS
        conn.execute(
            &format!("DELETE FROM contents WHERE file_id IN ({placeholders_str})"),
            rusqlite::params_from_iter(file_ids),
        )?;
        
        // Delete from files
        conn.execute(
            &format!("DELETE FROM files WHERE id IN ({placeholders_str})"),
            rusqlite::params_from_iter(file_ids),
        )?;
        
        Ok(())
    }

    /// Search content using FTS5
    pub fn search(
        &self,
        query: &str,
        repo_filter: Option<&str>,
        file_type_filter: Option<&str>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<SearchResult>> {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;

        // Build query with optional filters
        let mut sql = String::from(
            "SELECT r.name, r.path, f.relative_path, f.file_type,
                    snippet(contents, 1, '>>>', '<<<', '...', 64) as snippet,
                    bm25(contents) as score
             FROM contents c
             JOIN files f ON c.file_id = f.id
             JOIN repositories r ON f.repo_id = r.id
             WHERE contents MATCH ?1"
        );

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(query.to_string())];

        if let Some(repo) = repo_filter {
            sql.push_str(" AND r.name LIKE ?");
            params_vec.push(Box::new(format!("%{repo}%")));
        }

        if let Some(file_type) = file_type_filter {
            sql.push_str(" AND f.file_type = ?");
            params_vec.push(Box::new(file_type.to_string()));
        }

        sql.push_str(" ORDER BY score LIMIT ? OFFSET ?");
        #[allow(clippy::cast_possible_wrap)]
        params_vec.push(Box::new(limit as i64));
        #[allow(clippy::cast_possible_wrap)]
        params_vec.push(Box::new(offset as i64));

        let mut stmt = conn.prepare(&sql)?;
        
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(std::convert::AsRef::as_ref).collect();
        
        let results = stmt.query_map(params_refs.as_slice(), |row| {
            let repo_path = PathBuf::from(row.get::<_, String>(1)?);
            let relative_path = PathBuf::from(row.get::<_, String>(2)?);
            let absolute_path = repo_path.join(&relative_path);
            
            Ok(SearchResult {
                repo_name: row.get(0)?,
                repo_path,
                file_path: relative_path,
                absolute_path,
                snippet: row.get(4)?,
                file_type: row.get(3)?,
                score: row.get(5)?,
            })
        })?
        .filter_map(std::result::Result::ok)
        .collect();

        Ok(results)
    }

    /// Count total search results
    #[allow(dead_code)]
    pub fn search_count(
        &self,
        query: &str,
        repo_filter: Option<&str>,
        file_type_filter: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| AppError::Other(e.to_string()))?;

        let mut sql = String::from(
            "SELECT COUNT(*) FROM contents c
             JOIN files f ON c.file_id = f.id
             JOIN repositories r ON f.repo_id = r.id
             WHERE contents MATCH ?1"
        );

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(query.to_string())];

        if let Some(repo) = repo_filter {
            sql.push_str(" AND r.name LIKE ?");
            params_vec.push(Box::new(format!("%{repo}%")));
        }

        if let Some(file_type) = file_type_filter {
            sql.push_str(" AND f.file_type = ?");
            params_vec.push(Box::new(file_type.to_string()));
        }

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(std::convert::AsRef::as_ref).collect();
        
        let count: i64 = conn.query_row(&sql, params_refs.as_slice(), |row| row.get(0))?;
        Ok(count)
    }
}
