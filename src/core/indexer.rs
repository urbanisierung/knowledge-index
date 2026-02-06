use chrono::{DateTime, Utc};
use ignore::WalkBuilder;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

use crate::config::Config;
use crate::db::{Database, FileRecord, FileType, RepoStatus, Repository};
use crate::error::{AppError, Result};

/// Progress information for indexing
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IndexProgress {
    pub total_files: usize,
    pub processed_files: usize,
    pub skipped_files: usize,
    pub current_file: String,
    pub bytes_processed: u64,
    pub elapsed_secs: f64,
}

/// Result of indexing operation
#[derive(Debug, Clone)]
pub struct IndexResult {
    pub files_added: usize,
    pub files_updated: usize,
    pub files_deleted: usize,
    pub files_unchanged: usize,
    pub files_skipped: usize,
    pub total_bytes: u64,
    pub elapsed_secs: f64,
}

/// File indexer
pub struct Indexer {
    db: Database,
    config: Config,
}

// Binary file extensions to skip
const BINARY_EXTENSIONS: &[&str] = &[
    "exe", "dll", "so", "dylib", "bin", "obj", "o", "a", "lib",
    "png", "jpg", "jpeg", "gif", "bmp", "ico", "webp", "svg", "tiff",
    "mp3", "mp4", "avi", "mov", "mkv", "wav", "flac", "ogg", "webm",
    "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "iso",
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
    "ttf", "otf", "woff", "woff2", "eot",
    "pyc", "pyo", "class", "jar", "war",
    "db", "sqlite", "sqlite3",
    "lock", "sum",
];

impl Indexer {
    pub fn new(db: Database, config: Config) -> Self {
        Self { db, config }
    }

    /// Index a directory
    pub fn index<F>(&self, path: &Path, name: Option<String>, progress_callback: F) -> Result<IndexResult>
    where
        F: Fn(&IndexProgress) + Send + Sync,
    {
        let start = Instant::now();
        
        // Validate path
        if !path.exists() {
            return Err(AppError::PathNotFound(path.to_path_buf()));
        }
        if !path.is_dir() {
            return Err(AppError::NotADirectory(path.to_path_buf()));
        }

        let canonical = path.canonicalize()?;

        // Check if already indexed
        let existing = self.db.get_repository_by_path(&canonical)?;
        let repo = if let Some(repo) = existing {
            // Update existing
            return self.update_repository(&repo, progress_callback);
        } else {
            self.db.add_repository(&canonical, name)?
        };

        // Set status to indexing
        self.db.update_repository_status(repo.id, RepoStatus::Indexing)?;

        // Collect files
        let files = self.collect_files(&canonical);
        let total_files = files.len();

        // Progress tracking
        let processed = AtomicUsize::new(0);
        let skipped = AtomicUsize::new(0);
        let bytes_processed = AtomicU64::new(0);

        // Process files
        self.db.begin_batch()?;
        
        let mut batch_count = 0;
        for file_path in &files {
            let relative = file_path.strip_prefix(&canonical).unwrap_or(file_path);
            
            // Update progress
            let current_processed = processed.fetch_add(1, Ordering::Relaxed) + 1;
            progress_callback(&IndexProgress {
                total_files,
                processed_files: current_processed,
                skipped_files: skipped.load(Ordering::Relaxed),
                current_file: relative.to_string_lossy().to_string(),
                bytes_processed: bytes_processed.load(Ordering::Relaxed),
                elapsed_secs: start.elapsed().as_secs_f64(),
            });

            // Process file
            match self.process_file(&canonical, file_path, repo.id) {
                Ok(size) => {
                    bytes_processed.fetch_add(size, Ordering::Relaxed);
                    batch_count += 1;
                    
                    if batch_count >= self.config.batch_size {
                        self.db.commit_batch()?;
                        self.db.begin_batch()?;
                        batch_count = 0;
                    }
                }
                Err(_) => {
                    skipped.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        self.db.commit_batch()?;

        // Update repository stats
        #[allow(clippy::cast_possible_wrap)]
        let file_count = (total_files - skipped.load(Ordering::Relaxed)) as i64;
        #[allow(clippy::cast_possible_wrap)]
        let total_bytes = bytes_processed.load(Ordering::Relaxed) as i64;
        self.db.update_repository_indexed(repo.id, file_count, total_bytes)?;

        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        Ok(IndexResult {
            files_added: file_count as usize,
            files_updated: 0,
            files_deleted: 0,
            files_unchanged: 0,
            files_skipped: skipped.load(Ordering::Relaxed),
            total_bytes: bytes_processed.load(Ordering::Relaxed),
            elapsed_secs: start.elapsed().as_secs_f64(),
        })
    }

    /// Update an existing repository (incremental indexing)
    fn update_repository<F>(&self, repo: &Repository, progress_callback: F) -> Result<IndexResult>
    where
        F: Fn(&IndexProgress) + Send + Sync,
    {
        let start = Instant::now();
        
        self.db.update_repository_status(repo.id, RepoStatus::Indexing)?;

        // Get existing files
        let existing_files = self.db.get_repository_files(repo.id)?;
        let existing_map: std::collections::HashMap<PathBuf, FileRecord> = existing_files
            .into_iter()
            .map(|f| (f.relative_path.clone(), f))
            .collect();
        let existing_paths: HashSet<PathBuf> = existing_map.keys().cloned().collect();

        // Collect current files
        let current_files = self.collect_files(&repo.path);
        let current_paths: HashSet<PathBuf> = current_files
            .iter()
            .filter_map(|p| p.strip_prefix(&repo.path).ok())
            .map(PathBuf::from)
            .collect();

        // Determine changes
        let deleted: Vec<_> = existing_paths.difference(&current_paths).cloned().collect();
        let new_files: Vec<_> = current_paths.difference(&existing_paths).cloned().collect();
        
        let mut modified = Vec::new();
        let mut unchanged = Vec::new();
        
        for path in current_paths.intersection(&existing_paths) {
            let full_path = repo.path.join(path);
            if let Ok(metadata) = fs::metadata(&full_path) {
                let existing = &existing_map[path];
                let mtime = metadata.modified().map_or_else(|_| Utc::now(), DateTime::<Utc>::from);
                
                #[allow(clippy::cast_possible_wrap)]
                let file_size = metadata.len() as i64;
                if mtime > existing.last_modified_at || file_size != existing.file_size_bytes {
                    modified.push(path.clone());
                } else {
                    unchanged.push(path.clone());
                }
            }
        }

        let total_to_process = new_files.len() + modified.len();
        let processed = AtomicUsize::new(0);
        let skipped = AtomicUsize::new(0);
        let bytes_processed = AtomicU64::new(0);

        // Delete removed files
        let deleted_ids: Vec<i64> = deleted
            .iter()
            .filter_map(|p| existing_map.get(p).map(|f| f.id))
            .collect();
        self.db.delete_files(&deleted_ids)?;

        // Process new and modified files
        self.db.begin_batch()?;
        let mut batch_count = 0;

        for relative_path in new_files.iter().chain(modified.iter()) {
            let full_path = repo.path.join(relative_path);
            
            let current_processed = processed.fetch_add(1, Ordering::Relaxed) + 1;
            progress_callback(&IndexProgress {
                total_files: total_to_process,
                processed_files: current_processed,
                skipped_files: skipped.load(Ordering::Relaxed),
                current_file: relative_path.to_string_lossy().to_string(),
                bytes_processed: bytes_processed.load(Ordering::Relaxed),
                elapsed_secs: start.elapsed().as_secs_f64(),
            });

            // Delete existing if modified
            if let Some(existing) = existing_map.get(relative_path) {
                self.db.delete_files(&[existing.id])?;
            }

            match self.process_file(&repo.path, &full_path, repo.id) {
                Ok(size) => {
                    bytes_processed.fetch_add(size, Ordering::Relaxed);
                    batch_count += 1;
                    
                    if batch_count >= self.config.batch_size {
                        self.db.commit_batch()?;
                        self.db.begin_batch()?;
                        batch_count = 0;
                    }
                }
                Err(_) => {
                    skipped.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        self.db.commit_batch()?;

        // Update repository stats
        #[allow(clippy::cast_possible_wrap)]
        let file_count = (current_files.len() - skipped.load(Ordering::Relaxed)) as i64;
        #[allow(clippy::cast_possible_wrap)]
        let total_bytes = bytes_processed.load(Ordering::Relaxed) as i64;
        self.db.update_repository_indexed(repo.id, file_count, total_bytes)?;

        Ok(IndexResult {
            files_added: new_files.len() - skipped.load(Ordering::Relaxed),
            files_updated: modified.len(),
            files_deleted: deleted.len(),
            files_unchanged: unchanged.len(),
            files_skipped: skipped.load(Ordering::Relaxed),
            total_bytes: bytes_processed.load(Ordering::Relaxed),
            elapsed_secs: start.elapsed().as_secs_f64(),
        })
    }

    /// Collect all indexable files in a directory
    fn collect_files(&self, root: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();

        let mut builder = WalkBuilder::new(root);
        builder
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true);

        // Add custom ignore patterns
        for pattern in &self.config.ignore_patterns {
            builder.add_ignore(root.join(pattern));
        }

        for entry in builder.build().flatten() {
            let path = entry.path();
            
            if path.is_file() && self.should_index(path) {
                files.push(path.to_path_buf());
            }
        }

        files
    }

    /// Check if a file should be indexed
    fn should_index(&self, path: &Path) -> bool {
        // Check extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            if BINARY_EXTENSIONS.contains(&ext_lower.as_str()) {
                return false;
            }
        }

        // Check size
        if let Ok(metadata) = fs::metadata(path) {
            if metadata.len() > self.config.max_file_size_bytes() {
                return false;
            }
        }

        // Check if in ignored directory
        let path_str = path.to_string_lossy();
        for pattern in &self.config.ignore_patterns {
            if path_str.contains(pattern) {
                return false;
            }
        }

        true
    }

    /// Process a single file
    fn process_file(&self, root: &Path, path: &Path, repo_id: i64) -> Result<u64> {
        let relative = path.strip_prefix(root).unwrap_or(path);
        
        // Read file
        let mut file = File::open(path)?;
        let metadata = file.metadata()?;
        let size = metadata.len();
        
        // Check size limit
        if size > self.config.max_file_size_bytes() {
            return Err(AppError::Other("File too large".into()));
        }

        #[allow(clippy::cast_possible_truncation)]
        let mut content = Vec::with_capacity(size as usize);
        file.read_to_end(&mut content)?;

        // Check for binary content (null bytes in first 8KB)
        let check_len = std::cmp::min(8192, content.len());
        if content[..check_len].contains(&0) {
            return Err(AppError::Other("Binary file".into()));
        }

        // Convert to string
        let content_str = String::from_utf8_lossy(&content);

        // Compute hash
        let hash = blake3::hash(content_str.as_bytes());
        let hash_str = hash.to_hex().to_string();

        // Detect file type
        let file_type = path
            .extension()
            .and_then(|e| e.to_str())
            .map_or(FileType::Unknown, FileType::from_extension);

        // Get modification time
        let mtime = metadata.modified().map_or_else(|_| Utc::now(), DateTime::<Utc>::from);

        // Insert into database
        #[allow(clippy::cast_possible_wrap)]
        self.db.insert_file(
            repo_id,
            relative,
            &hash_str,
            size as i64,
            mtime,
            file_type.as_str(),
            &content_str,
        )?;

        Ok(size)
    }
}
