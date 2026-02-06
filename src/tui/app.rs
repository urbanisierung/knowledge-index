use crate::config::Config;
use crate::core::Searcher;
use crate::db::{Database, Repository, SearchResult};

/// Application mode/view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Search,
    Repos,
    Help,
}

/// Status message level
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum StatusLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// Main application state
#[allow(dead_code)]
pub struct App {
    pub db: Database,
    pub config: Config,
    pub searcher: Searcher,

    // UI State
    pub mode: AppMode,
    pub should_quit: bool,
    pub first_run: bool,

    // Search state
    pub search_input: String,
    pub search_results: Vec<SearchResult>,
    pub search_selected: usize,
    pub search_loading: bool,

    // Repository state
    pub repos: Vec<Repository>,
    pub repos_selected: usize,

    // Status
    pub status_message: Option<(String, StatusLevel)>,
}

impl App {
    pub fn new(db: Database, config: Config) -> Self {
        let searcher = Searcher::new(db.clone());
        let repos = db.list_repositories().unwrap_or_default();
        let first_run = repos.is_empty();

        Self {
            db,
            config,
            searcher,
            mode: AppMode::Search,
            should_quit: false,
            first_run,
            search_input: String::new(),
            search_results: Vec::new(),
            search_selected: 0,
            search_loading: false,
            repos,
            repos_selected: 0,
            status_message: None,
        }
    }

    /// Perform search
    pub fn search(&mut self) {
        if self.search_input.is_empty() {
            self.search_results.clear();
            return;
        }

        self.search_loading = true;
        
        match self.searcher.search(&self.search_input, None, None, 50, 0) {
            Ok(results) => {
                self.search_results = results;
                self.search_selected = 0;
                self.search_loading = false;
            }
            Err(e) => {
                self.set_status(format!("Search error: {e}"), StatusLevel::Error);
                self.search_loading = false;
            }
        }
    }

    /// Refresh repository list
    pub fn refresh_repos(&mut self) {
        self.repos = self.db.list_repositories().unwrap_or_default();
    }

    /// Set status message
    pub fn set_status(&mut self, message: String, level: StatusLevel) {
        self.status_message = Some((message, level));
    }

    /// Clear status message
    #[allow(dead_code)]
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Select next item in current list
    pub fn select_next(&mut self) {
        match self.mode {
            AppMode::Search => {
                if !self.search_results.is_empty() {
                    self.search_selected = (self.search_selected + 1) % self.search_results.len();
                }
            }
            AppMode::Repos => {
                if !self.repos.is_empty() {
                    self.repos_selected = (self.repos_selected + 1) % self.repos.len();
                }
            }
            AppMode::Help => {}
        }
    }

    /// Select previous item in current list
    pub fn select_prev(&mut self) {
        match self.mode {
            AppMode::Search => {
                if !self.search_results.is_empty() {
                    self.search_selected = if self.search_selected == 0 {
                        self.search_results.len() - 1
                    } else {
                        self.search_selected - 1
                    };
                }
            }
            AppMode::Repos => {
                if !self.repos.is_empty() {
                    self.repos_selected = if self.repos_selected == 0 {
                        self.repos.len() - 1
                    } else {
                        self.repos_selected - 1
                    };
                }
            }
            AppMode::Help => {}
        }
    }

    /// Open selected file in editor
    pub fn open_selected(&mut self) {
        if self.mode != AppMode::Search || self.search_results.is_empty() {
            return;
        }

        let result = &self.search_results[self.search_selected];
        let path = &result.absolute_path;

        let _editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
        
        // We need to restore terminal, run editor, then reinitialize
        // For simplicity, just show a message for now
        self.set_status(format!("Open: {}", path.display()), StatusLevel::Info);
    }

    /// Delete selected repository
    pub fn delete_selected_repo(&mut self) {
        if self.mode != AppMode::Repos || self.repos.is_empty() {
            return;
        }

        let repo = &self.repos[self.repos_selected];
        
        if let Err(e) = self.db.delete_repository(repo.id) {
            self.set_status(format!("Delete error: {e}"), StatusLevel::Error);
            return;
        }

        self.set_status(format!("Removed: {}", repo.name), StatusLevel::Success);
        self.refresh_repos();
        
        if self.repos_selected >= self.repos.len() && !self.repos.is_empty() {
            self.repos_selected = self.repos.len() - 1;
        }
    }
}
