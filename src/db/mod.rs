use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::config::Config;
use crate::core::VaultType;
use crate::error::{AppError, Result};

mod schema;

/// Repository status in the index
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoStatus {
    Pending,
    Indexing,
    Ready,
    Error,
    Cloning,
    Syncing,
}

impl RepoStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Indexing => "indexing",
            Self::Ready => "ready",
            Self::Error => "error",
            Self::Cloning => "cloning",
            Self::Syncing => "syncing",
        }
    }

    #[must_use]
    pub fn from_str(s: &str) -> Self {
        match s {
            "indexing" => Self::Indexing,
            "ready" => Self::Ready,
            "error" => Self::Error,
            "cloning" => Self::Cloning,
            "syncing" => Self::Syncing,
            _ => Self::Pending,
        }
    }
}

/// Repository source type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Local,
    Remote,
}

impl SourceType {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Remote => "remote",
        }
    }

    #[must_use]
    pub fn from_str(s: &str) -> Self {
        match s {
            "remote" => Self::Remote,
            _ => Self::Local,
        }
    }
}

/// File type classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    Code(String), // Language name
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
    pub source_type: SourceType,
    pub remote_url: Option<String>,
    pub remote_branch: Option<String>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub vault_type: VaultType,
}

impl Repository {
    /// Check if this is a remote repository
    #[must_use]
    #[allow(dead_code)]
    pub fn is_remote(&self) -> bool {
        self.source_type == SourceType::Remote
    }
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
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        schema::initialize(&conn)?;
        Ok(())
    }

    /// Add a new repository
    pub fn add_repository(&self, path: &Path, name: Option<String>) -> Result<Repository> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        let canonical = path.canonicalize()?;
        let name = name.unwrap_or_else(|| {
            canonical.file_name().map_or_else(
                || "unknown".to_string(),
                |n| n.to_string_lossy().to_string(),
            )
        });
        let now = Utc::now();
        let vault_type = VaultType::detect(&canonical);

        conn.execute(
            "INSERT INTO repositories (path, name, created_at, status, vault_type) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                canonical.to_string_lossy(),
                name,
                now.to_rfc3339(),
                RepoStatus::Pending.as_str(),
                vault_type.as_str(),
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
            source_type: SourceType::Local,
            remote_url: None,
            remote_branch: None,
            last_synced_at: None,
            vault_type,
        })
    }

    /// Add a remote repository
    pub fn add_remote_repository(
        &self,
        path: &Path,
        name: &str,
        remote_url: &str,
        branch: Option<&str>,
    ) -> Result<Repository> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        let now = Utc::now();
        // For remote repos, we detect vault type after clone completes
        // For now, use Generic and update later
        let vault_type = VaultType::Generic;

        conn.execute(
            "INSERT INTO repositories (path, name, created_at, status, source_type, remote_url, remote_branch, vault_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                path.to_string_lossy(),
                name,
                now.to_rfc3339(),
                RepoStatus::Cloning.as_str(),
                SourceType::Remote.as_str(),
                remote_url,
                branch,
                vault_type.as_str(),
            ],
        )?;

        let id = conn.last_insert_rowid();

        Ok(Repository {
            id,
            path: path.to_path_buf(),
            name: name.to_string(),
            created_at: now,
            last_indexed_at: None,
            file_count: 0,
            total_size_bytes: 0,
            status: RepoStatus::Cloning,
            source_type: SourceType::Remote,
            remote_url: Some(remote_url.to_string()),
            remote_branch: branch.map(String::from),
            last_synced_at: None,
            vault_type,
        })
    }

    /// Get repository by path
    pub fn get_repository_by_path(&self, path: &Path) -> Result<Option<Repository>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        let mut stmt = conn.prepare(
            "SELECT id, path, name, created_at, last_indexed_at, file_count, total_size_bytes, status,
                    source_type, remote_url, remote_branch, last_synced_at, vault_type
             FROM repositories WHERE path = ?1"
        )?;

        let result = stmt.query_row(params![canonical.to_string_lossy()], |row| {
            Ok(Repository {
                id: row.get(0)?,
                path: PathBuf::from(row.get::<_, String>(1)?),
                name: row.get(2)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc)),
                last_indexed_at: row
                    .get::<_, Option<String>>(4)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                file_count: row.get(5)?,
                total_size_bytes: row.get(6)?,
                status: RepoStatus::from_str(&row.get::<_, String>(7)?),
                source_type: SourceType::from_str(
                    &row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                ),
                remote_url: row.get(9)?,
                remote_branch: row.get(10)?,
                last_synced_at: row
                    .get::<_, Option<String>>(11)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                vault_type: VaultType::from_str(
                    &row.get::<_, Option<String>>(12)?.unwrap_or_default(),
                ),
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
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, path, name, created_at, last_indexed_at, file_count, total_size_bytes, status,
                    source_type, remote_url, remote_branch, last_synced_at, vault_type
             FROM repositories ORDER BY name"
        )?;

        let repos = stmt
            .query_map([], |row| {
                Ok(Repository {
                    id: row.get(0)?,
                    path: PathBuf::from(row.get::<_, String>(1)?),
                    name: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc)),
                    last_indexed_at: row
                        .get::<_, Option<String>>(4)?
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                    file_count: row.get(5)?,
                    total_size_bytes: row.get(6)?,
                    status: RepoStatus::from_str(&row.get::<_, String>(7)?),
                    source_type: SourceType::from_str(
                        &row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                    ),
                    remote_url: row.get(9)?,
                    remote_branch: row.get(10)?,
                    last_synced_at: row
                        .get::<_, Option<String>>(11)?
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                    vault_type: VaultType::from_str(
                        &row.get::<_, Option<String>>(12)?.unwrap_or_default(),
                    ),
                })
            })?
            .filter_map(std::result::Result::ok)
            .collect();

        Ok(repos)
    }

    /// Get remote repositories that need syncing
    pub fn get_remote_repositories(&self) -> Result<Vec<Repository>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, path, name, created_at, last_indexed_at, file_count, total_size_bytes, status,
                    source_type, remote_url, remote_branch, last_synced_at, vault_type
             FROM repositories WHERE source_type = 'remote' ORDER BY name"
        )?;

        let repos = stmt
            .query_map([], |row| {
                Ok(Repository {
                    id: row.get(0)?,
                    path: PathBuf::from(row.get::<_, String>(1)?),
                    name: row.get(2)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc)),
                    last_indexed_at: row
                        .get::<_, Option<String>>(4)?
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                    file_count: row.get(5)?,
                    total_size_bytes: row.get(6)?,
                    status: RepoStatus::from_str(&row.get::<_, String>(7)?),
                    source_type: SourceType::Remote,
                    remote_url: row.get(9)?,
                    remote_branch: row.get(10)?,
                    last_synced_at: row
                        .get::<_, Option<String>>(11)?
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                    vault_type: VaultType::from_str(
                        &row.get::<_, Option<String>>(12)?.unwrap_or_default(),
                    ),
                })
            })?
            .filter_map(std::result::Result::ok)
            .collect();

        Ok(repos)
    }

    /// Update last synced time for a repository
    pub fn update_repository_synced(&self, repo_id: i64) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        let now = Utc::now();

        conn.execute(
            "UPDATE repositories SET last_synced_at = ?1, status = ?2 WHERE id = ?3",
            params![now.to_rfc3339(), RepoStatus::Ready.as_str(), repo_id],
        )?;
        Ok(())
    }

    /// Get repository by ID
    #[allow(dead_code)]
    pub fn get_repository_by_id(&self, repo_id: i64) -> Result<Option<Repository>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, path, name, created_at, last_indexed_at, file_count, total_size_bytes, status,
                    source_type, remote_url, remote_branch, last_synced_at, vault_type
             FROM repositories WHERE id = ?1"
        )?;

        let result = stmt.query_row(params![repo_id], |row| {
            Ok(Repository {
                id: row.get(0)?,
                path: PathBuf::from(row.get::<_, String>(1)?),
                name: row.get(2)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc)),
                last_indexed_at: row
                    .get::<_, Option<String>>(4)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                file_count: row.get(5)?,
                total_size_bytes: row.get(6)?,
                status: RepoStatus::from_str(&row.get::<_, String>(7)?),
                source_type: SourceType::from_str(
                    &row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                ),
                remote_url: row.get(9)?,
                remote_branch: row.get(10)?,
                last_synced_at: row
                    .get::<_, Option<String>>(11)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                vault_type: VaultType::from_str(
                    &row.get::<_, Option<String>>(12)?.unwrap_or_default(),
                ),
            })
        });

        match result {
            Ok(repo) => Ok(Some(repo)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Update repository status
    pub fn update_repository_status(&self, repo_id: i64, status: RepoStatus) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
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
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
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

    /// Update vault type for a repository (typically after clone completes)
    #[allow(dead_code)]
    pub fn update_repository_vault_type(&self, repo_id: i64, vault_type: VaultType) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        conn.execute(
            "UPDATE repositories SET vault_type = ?1 WHERE id = ?2",
            params![vault_type.as_str(), repo_id],
        )?;
        Ok(())
    }

    /// Delete a repository and all its files
    pub fn delete_repository(&self, repo_id: i64) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

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
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        conn.execute("BEGIN TRANSACTION", [])?;
        Ok(())
    }

    /// Commit the current transaction
    pub fn commit_batch(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        conn.execute("COMMIT", [])?;
        Ok(())
    }

    /// Rollback the current transaction
    #[allow(dead_code)]
    pub fn rollback_batch(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
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
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

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
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        let mut stmt = conn.prepare(
            "SELECT id, repo_id, relative_path, content_hash, file_size_bytes, last_modified_at, file_type
             FROM files WHERE repo_id = ?1"
        )?;

        let files = stmt
            .query_map(params![repo_id], |row| {
                Ok(FileRecord {
                    id: row.get(0)?,
                    repo_id: row.get(1)?,
                    relative_path: PathBuf::from(row.get::<_, String>(2)?),
                    content_hash: row.get(3)?,
                    file_size_bytes: row.get(4)?,
                    last_modified_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .map_or_else(|_| Utc::now(), |dt| dt.with_timezone(&Utc)),
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

        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

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
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        // Build query with optional filters
        let mut sql = String::from(
            "SELECT r.name, r.path, f.relative_path, f.file_type,
                    snippet(contents, 1, '>>>', '<<<', '...', 64) as snippet,
                    bm25(contents) as score
             FROM contents c
             JOIN files f ON c.file_id = f.id
             JOIN repositories r ON f.repo_id = r.id
             WHERE contents MATCH ?1",
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

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(std::convert::AsRef::as_ref).collect();

        let results = stmt
            .query_map(params_refs.as_slice(), |row| {
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
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        let mut sql = String::from(
            "SELECT COUNT(*) FROM contents c
             JOIN files f ON c.file_id = f.id
             JOIN repositories r ON f.repo_id = r.id
             WHERE contents MATCH ?1",
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

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(std::convert::AsRef::as_ref).collect();

        let count: i64 = conn.query_row(&sql, params_refs.as_slice(), |row| row.get(0))?;
        Ok(count)
    }

    // =========================================================================
    // Markdown Metadata
    // =========================================================================

    /// Store markdown metadata for a file
    pub fn store_markdown_meta(
        &self,
        file_id: i64,
        title: Option<&str>,
        tags_json: &str,
        links_json: &str,
        headings_json: &str,
    ) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        conn.execute(
            "INSERT OR REPLACE INTO markdown_meta (file_id, title, tags, links, headings)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![file_id, title, tags_json, links_json, headings_json],
        )?;

        Ok(())
    }

    /// Delete markdown metadata for specific files
    #[allow(dead_code)]
    pub fn delete_markdown_meta(&self, file_ids: &[i64]) -> Result<()> {
        if file_ids.is_empty() {
            return Ok(());
        }

        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        let placeholders: Vec<String> = file_ids.iter().map(|_| "?".to_string()).collect();
        let placeholders_str = placeholders.join(",");

        conn.execute(
            &format!("DELETE FROM markdown_meta WHERE file_id IN ({placeholders_str})"),
            rusqlite::params_from_iter(file_ids),
        )?;

        Ok(())
    }

    // =========================================================================
    // Embeddings
    // =========================================================================

    /// Store embeddings for a file
    pub fn store_embeddings(
        &self,
        file_id: i64,
        embeddings: &[(usize, usize, usize, &str, &[f32])], // (chunk_index, start, end, text, embedding)
    ) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        // Delete existing embeddings for this file
        conn.execute(
            "DELETE FROM embeddings WHERE file_id = ?1",
            params![file_id],
        )?;

        let mut stmt = conn.prepare(
            "INSERT INTO embeddings (file_id, chunk_index, start_offset, end_offset, chunk_text, embedding)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )?;

        for (chunk_index, start_offset, end_offset, chunk_text, embedding) in embeddings {
            // Serialize embedding as bytes (f32 little-endian)
            let embedding_bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();

            #[allow(clippy::cast_possible_wrap)]
            stmt.execute(params![
                file_id,
                *chunk_index as i64,
                *start_offset as i64,
                *end_offset as i64,
                *chunk_text,
                embedding_bytes,
            ])?;
        }

        Ok(())
    }

    /// Delete embeddings for specific files
    #[allow(dead_code)]
    pub fn delete_embeddings(&self, file_ids: &[i64]) -> Result<()> {
        if file_ids.is_empty() {
            return Ok(());
        }

        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        let placeholders: Vec<String> = file_ids.iter().map(|_| "?".to_string()).collect();
        let placeholders_str = placeholders.join(",");

        conn.execute(
            &format!("DELETE FROM embeddings WHERE file_id IN ({placeholders_str})"),
            rusqlite::params_from_iter(file_ids),
        )?;

        Ok(())
    }

    /// Search by vector similarity
    pub fn vector_search(
        &self,
        query_embedding: &[f32],
        repo_filter: Option<&str>,
        file_type_filter: Option<&str>,
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        // Build query with optional filters
        let mut sql = String::from(
            "SELECT r.name, r.path, f.relative_path, f.file_type,
                    e.chunk_text, e.embedding, e.start_offset, e.end_offset
             FROM embeddings e
             JOIN files f ON e.file_id = f.id
             JOIN repositories r ON f.repo_id = r.id
             WHERE 1=1",
        );

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(repo) = repo_filter {
            sql.push_str(" AND r.name LIKE ?");
            params_vec.push(Box::new(format!("%{repo}%")));
        }

        if let Some(file_type) = file_type_filter {
            sql.push_str(" AND f.file_type = ?");
            params_vec.push(Box::new(file_type.to_string()));
        }

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(std::convert::AsRef::as_ref).collect();

        let mut stmt = conn.prepare(&sql)?;

        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            let repo_name: String = row.get(0)?;
            let repo_path: String = row.get(1)?;
            let relative_path: String = row.get(2)?;
            let file_type: String = row.get(3)?;
            let chunk_text: String = row.get(4)?;
            let embedding_bytes: Vec<u8> = row.get(5)?;
            let start_offset: i64 = row.get(6)?;
            let end_offset: i64 = row.get(7)?;

            Ok((
                repo_name,
                repo_path,
                relative_path,
                file_type,
                chunk_text,
                embedding_bytes,
                start_offset,
                end_offset,
            ))
        })?;

        // Calculate similarities and collect results
        let mut results: Vec<VectorSearchResult> = Vec::new();

        for row_result in rows {
            let (
                repo_name,
                repo_path,
                relative_path,
                file_type,
                chunk_text,
                embedding_bytes,
                start_offset,
                end_offset,
            ) = row_result?;

            // Deserialize embedding from bytes
            let doc_embedding: Vec<f32> = embedding_bytes
                .chunks(4)
                .filter_map(|chunk| {
                    if chunk.len() == 4 {
                        Some(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    } else {
                        None
                    }
                })
                .collect();

            // Calculate cosine similarity
            let similarity = Self::cosine_sim(query_embedding, &doc_embedding);

            let repo_path = PathBuf::from(&repo_path);
            let file_path = PathBuf::from(&relative_path);
            let absolute_path = repo_path.join(&file_path);

            results.push(VectorSearchResult {
                repo_name,
                repo_path,
                file_path,
                absolute_path,
                chunk_text,
                file_type,
                similarity,
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                start_offset: start_offset as usize,
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                end_offset: end_offset as usize,
            });
        }

        // Sort by similarity (descending) and take top N
        results.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        Ok(results)
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }

    /// Check if embeddings are enabled (table exists and has data)
    #[allow(dead_code)]
    pub fn has_embeddings(&self) -> Result<bool> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM embeddings", [], |row| row.get(0))?;
        Ok(count > 0)
    }

    /// Get all unique tags with counts
    pub fn get_all_tags(&self) -> Result<Vec<(String, usize)>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        let mut stmt = conn
            .prepare("SELECT tag, COUNT(*) as count FROM tags GROUP BY tag ORDER BY count DESC")?;

        let tags = stmt
            .query_map([], |row| {
                let tag: String = row.get(0)?;
                let count: i64 = row.get(1)?;
                Ok((tag, usize::try_from(count).unwrap_or(0)))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(tags)
    }

    /// Get backlinks to a file (files that link to the given target)
    #[allow(clippy::type_complexity)]
    pub fn get_backlinks(
        &self,
        target_name: &str,
    ) -> Result<Vec<(String, String, String, Option<usize>)>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        let mut stmt = conn.prepare(
            r"
            SELECT f.relative_path, r.name, l.link_text, l.line_number
            FROM links l
            JOIN files f ON l.source_file_id = f.id
            JOIN repositories r ON f.repo_id = r.id
            WHERE l.target_name = ?1 OR l.target_name LIKE ?2
            ORDER BY r.name, f.relative_path
            ",
        )?;

        // Search for exact match or partial match (file without extension)
        let pattern = format!("%{target_name}%");

        let backlinks = stmt
            .query_map(rusqlite::params![target_name, pattern], |row| {
                let file_path: String = row.get(0)?;
                let repo_name: String = row.get(1)?;
                let link_text: String = row.get(2)?;
                let line_number: Option<i64> = row.get(3)?;
                Ok((
                    file_path,
                    repo_name,
                    link_text,
                    line_number.and_then(|n| usize::try_from(n).ok()),
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(backlinks)
    }

    /// Add tags for a file (replaces existing tags)
    pub fn add_tags(&self, file_id: i64, tags: &[String]) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        // First delete existing tags for this file
        conn.execute("DELETE FROM tags WHERE file_id = ?1", [file_id])?;

        // Insert new tags
        for tag in tags {
            conn.execute(
                "INSERT INTO tags (file_id, tag) VALUES (?1, ?2)",
                rusqlite::params![file_id, tag],
            )?;
        }

        Ok(())
    }

    /// Add links for a file (replaces existing links).
    /// Each link is a tuple of (target name, optional line number).
    pub fn add_links(&self, file_id: i64, links: &[(String, Option<usize>)]) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        // First delete existing links for this file
        conn.execute("DELETE FROM links WHERE source_file_id = ?1", [file_id])?;

        // Insert new links
        for (target_name, line_number) in links {
            conn.execute(
                "INSERT INTO links (source_file_id, target_name, link_text, line_number) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    file_id,
                    target_name,
                    target_name, // link_text is same as target for now
                    line_number.map(|n| i64::try_from(n).unwrap_or(0))
                ],
            )?;
        }

        Ok(())
    }

    /// Get knowledge statistics
    pub fn get_stats(&self) -> Result<KnowledgeStats> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        let total_files: i64 =
            conn.query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))?;
        let total_repos: i64 =
            conn.query_row("SELECT COUNT(*) FROM repositories", [], |row| row.get(0))?;

        // Count by file type
        let mut stmt = conn.prepare("SELECT file_type, COUNT(*) FROM files GROUP BY file_type")?;
        let file_counts: Vec<(String, i64)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(std::result::Result::ok)
            .collect();

        let total_tags: i64 = conn
            .query_row("SELECT COUNT(DISTINCT tag) FROM tags", [], |row| row.get(0))
            .unwrap_or(0);
        let total_links: i64 = conn
            .query_row("SELECT COUNT(*) FROM links", [], |row| row.get(0))
            .unwrap_or(0);
        let total_embeddings: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT file_id) FROM embeddings",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Database size
        let db_path = Config::database_path()?;
        let db_size = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);

        Ok(KnowledgeStats {
            total_files: usize::try_from(total_files).unwrap_or(0),
            total_repos: usize::try_from(total_repos).unwrap_or(0),
            file_counts,
            total_tags: usize::try_from(total_tags).unwrap_or(0),
            total_links: usize::try_from(total_links).unwrap_or(0),
            files_with_embeddings: usize::try_from(total_embeddings).unwrap_or(0),
            database_size_bytes: db_size,
        })
    }

    /// Get all links for graph visualization.
    /// Returns vector of `GraphLink` structs.
    pub fn get_all_links(&self, repo_filter: Option<&str>) -> Result<Vec<GraphLink>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        let query = if repo_filter.is_some() {
            r"
            SELECT f.relative_path, r.name, l.target_name
            FROM links l
            JOIN files f ON l.source_file_id = f.id
            JOIN repositories r ON f.repo_id = r.id
            WHERE r.name = ?1
            ORDER BY f.relative_path
            "
        } else {
            r"
            SELECT f.relative_path, r.name, l.target_name
            FROM links l
            JOIN files f ON l.source_file_id = f.id
            JOIN repositories r ON f.repo_id = r.id
            ORDER BY r.name, f.relative_path
            "
        };

        let mut stmt = conn.prepare(query)?;

        let links = if let Some(repo) = repo_filter {
            stmt.query_map([repo], |row| {
                Ok(GraphLink {
                    source_path: row.get(0)?,
                    source_repo: row.get(1)?,
                    target_name: row.get(2)?,
                })
            })?
            .filter_map(std::result::Result::ok)
            .collect()
        } else {
            stmt.query_map([], |row| {
                Ok(GraphLink {
                    source_path: row.get(0)?,
                    source_repo: row.get(1)?,
                    target_name: row.get(2)?,
                })
            })?
            .filter_map(std::result::Result::ok)
            .collect()
        };

        Ok(links)
    }

    /// Get all indexed file paths for health checks
    pub fn get_all_file_paths(&self) -> Result<Vec<(String, String)>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        let mut stmt = conn.prepare(
            r"
            SELECT f.relative_path, r.name
            FROM files f
            JOIN repositories r ON f.repo_id = r.id
            ORDER BY r.name, f.relative_path
            ",
        )?;

        let paths = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(std::result::Result::ok)
            .collect();

        Ok(paths)
    }

    /// Get files with no incoming links (orphans)
    pub fn get_orphan_files(&self, repo_filter: Option<&str>) -> Result<Vec<(String, String)>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AppError::Other(e.to_string()))?;

        let query = if repo_filter.is_some() {
            r"
            SELECT f.relative_path, r.name
            FROM files f
            JOIN repositories r ON f.repo_id = r.id
            WHERE r.name = ?1
              AND f.file_type = 'markdown'
              AND NOT EXISTS (
                SELECT 1 FROM links l
                WHERE l.target_name = f.relative_path
                   OR f.relative_path LIKE '%' || l.target_name || '%'
              )
            ORDER BY f.relative_path
            "
        } else {
            r"
            SELECT f.relative_path, r.name
            FROM files f
            JOIN repositories r ON f.repo_id = r.id
            WHERE f.file_type = 'markdown'
              AND NOT EXISTS (
                SELECT 1 FROM links l
                WHERE l.target_name = f.relative_path
                   OR f.relative_path LIKE '%' || l.target_name || '%'
              )
            ORDER BY r.name, f.relative_path
            "
        };

        let mut stmt = conn.prepare(query)?;

        let orphans = if let Some(repo) = repo_filter {
            stmt.query_map([repo], |row| Ok((row.get(0)?, row.get(1)?)))?
                .filter_map(std::result::Result::ok)
                .collect()
        } else {
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .filter_map(std::result::Result::ok)
                .collect()
        };

        Ok(orphans)
    }
}

/// Link for graph visualization
#[derive(Debug, Clone)]
pub struct GraphLink {
    pub source_path: String,
    pub source_repo: String,
    pub target_name: String,
}

/// Knowledge statistics
#[derive(Debug, Clone)]
pub struct KnowledgeStats {
    pub total_files: usize,
    pub total_repos: usize,
    pub file_counts: Vec<(String, i64)>,
    pub total_tags: usize,
    pub total_links: usize,
    pub files_with_embeddings: usize,
    pub database_size_bytes: u64,
}

/// Vector search result
#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub repo_name: String,
    pub repo_path: PathBuf,
    pub file_path: PathBuf,
    pub absolute_path: PathBuf,
    pub chunk_text: String,
    pub file_type: String,
    pub similarity: f32,
    #[allow(dead_code)]
    pub start_offset: usize,
    #[allow(dead_code)]
    pub end_offset: usize,
}
