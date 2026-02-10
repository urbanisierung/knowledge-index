use rusqlite::Connection;

use crate::error::Result;

pub const SCHEMA_VERSION: i32 = 3;

/// Initialize database schema
pub fn initialize(conn: &Connection) -> Result<()> {
    // Check and update schema version
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY)",
        [],
    )?;

    let current_version: Option<i32> = conn
        .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
            row.get(0)
        })
        .ok();

    match current_version {
        None => {
            // Fresh database, create all tables
            create_schema(conn)?;
            conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                [SCHEMA_VERSION],
            )?;
        }
        Some(v) if v < SCHEMA_VERSION => {
            // Run migrations
            migrate(conn, v)?;
            conn.execute("UPDATE schema_version SET version = ?1", [SCHEMA_VERSION])?;
        }
        _ => {
            // Schema is up to date
        }
    }

    Ok(())
}

fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r"
        -- Indexed repositories
        CREATE TABLE IF NOT EXISTS repositories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            last_indexed_at TEXT,
            file_count INTEGER DEFAULT 0,
            total_size_bytes INTEGER DEFAULT 0,
            status TEXT DEFAULT 'pending',
            source_type TEXT DEFAULT 'local',
            remote_url TEXT,
            remote_branch TEXT,
            last_synced_at TEXT
        );

        -- Individual files
        CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            repo_id INTEGER NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
            relative_path TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            file_size_bytes INTEGER NOT NULL,
            last_modified_at TEXT NOT NULL,
            file_type TEXT,
            UNIQUE(repo_id, relative_path)
        );

        -- Full-text search content
        CREATE VIRTUAL TABLE IF NOT EXISTS contents USING fts5(
            file_id UNINDEXED,
            content,
            tokenize='porter unicode61'
        );

        -- Markdown metadata (optional)
        CREATE TABLE IF NOT EXISTS markdown_meta (
            file_id INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
            title TEXT,
            tags TEXT,
            links TEXT,
            headings TEXT
        );

        -- Vector embeddings for semantic search
        CREATE TABLE IF NOT EXISTS embeddings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
            chunk_index INTEGER NOT NULL,
            start_offset INTEGER NOT NULL,
            end_offset INTEGER NOT NULL,
            chunk_text TEXT NOT NULL,
            embedding BLOB NOT NULL,
            UNIQUE(file_id, chunk_index)
        );

        -- Indexes
        CREATE INDEX IF NOT EXISTS idx_files_repo ON files(repo_id);
        CREATE INDEX IF NOT EXISTS idx_files_hash ON files(content_hash);
        CREATE INDEX IF NOT EXISTS idx_files_type ON files(file_type);
        CREATE INDEX IF NOT EXISTS idx_embeddings_file ON embeddings(file_id);
        CREATE INDEX IF NOT EXISTS idx_repos_source_type ON repositories(source_type);
        ",
    )?;

    Ok(())
}

fn migrate(conn: &Connection, from_version: i32) -> Result<()> {
    if from_version < 2 {
        // Add embeddings table for version 2
        conn.execute_batch(
            r"
            -- Vector embeddings for semantic search
            CREATE TABLE IF NOT EXISTS embeddings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
                chunk_index INTEGER NOT NULL,
                start_offset INTEGER NOT NULL,
                end_offset INTEGER NOT NULL,
                chunk_text TEXT NOT NULL,
                embedding BLOB NOT NULL,
                UNIQUE(file_id, chunk_index)
            );

            CREATE INDEX IF NOT EXISTS idx_embeddings_file ON embeddings(file_id);
            ",
        )?;
    }

    if from_version < 3 {
        // Add remote repository support for version 3
        conn.execute_batch(
            r"
            -- Add columns for remote repository support
            ALTER TABLE repositories ADD COLUMN source_type TEXT DEFAULT 'local';
            ALTER TABLE repositories ADD COLUMN remote_url TEXT;
            ALTER TABLE repositories ADD COLUMN remote_branch TEXT;
            ALTER TABLE repositories ADD COLUMN last_synced_at TEXT;

            CREATE INDEX IF NOT EXISTS idx_repos_source_type ON repositories(source_type);
            ",
        )?;
    }

    Ok(())
}
