//! File system watcher for automatic re-indexing.

use notify::{
    event::{CreateKind, ModifyKind, RemoveKind},
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::config::Config as AppConfig;
use crate::error::Result;

/// Type of change detected in a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
}

/// A pending change to be processed.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PendingChange {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub detected_at: Instant,
}

/// Batched changes for a repository.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct RepoBatch {
    pub repo_path: PathBuf,
    pub changes: Vec<PendingChange>,
}

/// File system watcher for automatic re-indexing.
#[allow(dead_code)]
pub struct IndexWatcher {
    watcher: RecommendedWatcher,
    watched_paths: Arc<Mutex<Vec<PathBuf>>>,
    pending_changes: Arc<Mutex<HashMap<PathBuf, PendingChange>>>,
    event_receiver: Receiver<notify::Result<Event>>,
    debounce_duration: Duration,
    config: Arc<AppConfig>,
}

#[allow(dead_code)]
impl IndexWatcher {
    /// Create a new watcher instance.
    pub fn new(config: Arc<AppConfig>) -> Result<Self> {
        let (tx, rx): (Sender<notify::Result<Event>>, Receiver<notify::Result<Event>>) =
            mpsc::channel();

        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )?;

        Ok(Self {
            watcher,
            watched_paths: Arc::new(Mutex::new(Vec::new())),
            pending_changes: Arc::new(Mutex::new(HashMap::new())),
            event_receiver: rx,
            debounce_duration: Duration::from_millis(500),
            config,
        })
    }

    /// Watch a repository path for changes.
    pub fn watch(&mut self, path: PathBuf) -> Result<()> {
        self.watcher.watch(&path, RecursiveMode::Recursive)?;
        if let Ok(mut paths) = self.watched_paths.lock() {
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
        Ok(())
    }

    /// Stop watching a repository path.
    pub fn unwatch(&mut self, path: &PathBuf) -> Result<()> {
        self.watcher.unwatch(path)?;
        if let Ok(mut paths) = self.watched_paths.lock() {
            paths.retain(|p| p != path);
        }
        Ok(())
    }

    /// Process incoming events and return debounced changes ready for processing.
    pub fn poll_changes(&self) -> Vec<RepoBatch> {
        // Collect new events
        while let Ok(event_result) = self.event_receiver.try_recv() {
            if let Ok(event) = event_result {
                self.process_event(event);
            }
        }

        // Extract changes that have been debounced long enough
        let now = Instant::now();
        let mut ready_changes: Vec<PendingChange> = Vec::new();

        if let Ok(mut pending) = self.pending_changes.lock() {
            let ready_paths: Vec<PathBuf> = pending
                .iter()
                .filter(|(_, change)| now.duration_since(change.detected_at) >= self.debounce_duration)
                .map(|(path, _)| path.clone())
                .collect();

            for path in ready_paths {
                if let Some(change) = pending.remove(&path) {
                    ready_changes.push(change);
                }
            }
        }

        // Group by repository
        self.group_by_repo(ready_changes)
    }

    /// Check if there are pending changes waiting for debounce.
    pub fn has_pending_changes(&self) -> bool {
        self.pending_changes
            .lock()
            .map(|p| !p.is_empty())
            .unwrap_or(false)
    }

    /// Get count of pending changes.
    pub fn pending_count(&self) -> usize {
        self.pending_changes
            .lock()
            .map(|p| p.len())
            .unwrap_or(0)
    }

    /// Process a single notify event.
    fn process_event(&self, event: Event) {
        let change_type = match event.kind {
            EventKind::Create(CreateKind::File) => Some(ChangeType::Created),
            EventKind::Modify(ModifyKind::Data(_)) => Some(ChangeType::Modified),
            EventKind::Remove(RemoveKind::File) => Some(ChangeType::Deleted),
            _ => None,
        };

        let Some(change_type) = change_type else {
            return;
        };

        for path in event.paths {
            // Skip if path matches ignore patterns
            if self.should_ignore(&path) {
                continue;
            }

            // Skip binary files for Created/Modified
            if change_type != ChangeType::Deleted && Self::is_binary_extension(&path) {
                continue;
            }

            if let Ok(mut pending) = self.pending_changes.lock() {
                pending.insert(
                    path.clone(),
                    PendingChange {
                        path,
                        change_type,
                        detected_at: Instant::now(),
                    },
                );
            }
        }
    }

    /// Check if a path should be ignored.
    fn should_ignore(&self, path: &std::path::Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check config ignore patterns
        for pattern in &self.config.ignore_patterns {
            if path_str.contains(pattern) {
                return true;
            }
        }

        // Check common ignore patterns
        let common_ignores = [
            ".git/",
            ".svn/",
            "node_modules/",
            "target/",
            "__pycache__/",
            ".obsidian/",
            ".vscode/",
            ".idea/",
        ];

        for ignore in common_ignores {
            if path_str.contains(ignore) {
                return true;
            }
        }

        false
    }

    /// Check if a file has a binary extension.
    fn is_binary_extension(path: &std::path::Path) -> bool {
        let binary_extensions = [
            "exe", "dll", "so", "dylib", "bin", "o", "a", "lib", "png", "jpg", "jpeg", "gif",
            "bmp", "ico", "svg", "webp", "mp3", "mp4", "wav", "avi", "mov", "mkv", "webm", "pdf",
            "doc", "docx", "xls", "xlsx", "ppt", "pptx", "zip", "tar", "gz", "rar", "7z", "woff",
            "woff2", "ttf", "otf", "eot",
        ];

        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| binary_extensions.contains(&ext.to_lowercase().as_str()))
    }

    /// Group changes by their parent repository.
    fn group_by_repo(&self, changes: Vec<PendingChange>) -> Vec<RepoBatch> {
        let watched = self.watched_paths.lock().ok();
        let watched_paths = watched.as_ref().map_or(&[][..], |v| v.as_slice());

        let mut batches: HashMap<PathBuf, RepoBatch> = HashMap::new();

        for change in changes {
            // Find which watched repo this change belongs to
            let repo_path = watched_paths
                .iter()
                .find(|repo| change.path.starts_with(repo))
                .cloned();

            if let Some(repo_path) = repo_path {
                batches
                    .entry(repo_path.clone())
                    .or_insert_with(|| RepoBatch {
                        repo_path,
                        changes: Vec::new(),
                    })
                    .changes
                    .push(change);
            }
        }

        batches.into_values().collect()
    }

    /// Get list of currently watched paths.
    pub fn watched_paths(&self) -> Vec<PathBuf> {
        self.watched_paths
            .lock()
            .map(|p| p.clone())
            .unwrap_or_default()
    }
}
