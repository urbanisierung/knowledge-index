use crate::config::Config;
use crate::core::Searcher;
use crate::db::{Database, Repository, SearchResult};

/// Application mode/view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Welcome,
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

/// Confirmation dialog state
#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub on_confirm: ConfirmAction,
}

/// Actions that can be confirmed
#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteRepo(i64, String),
}

/// Main application state
#[allow(dead_code)]
#[allow(clippy::struct_excessive_bools)]
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

    // Preview state
    pub show_preview: bool,
    pub preview_content: Option<String>,
    pub preview_scroll: usize,

    // Repository state
    pub repos: Vec<Repository>,
    pub repos_selected: usize,

    // Confirmation dialog
    pub confirm_dialog: Option<ConfirmDialog>,

    // Status
    pub status_message: Option<(String, StatusLevel)>,

    // Loading indicator
    pub loading: bool,
    pub loading_message: Option<String>,
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
            mode: if first_run {
                AppMode::Welcome
            } else {
                AppMode::Search
            },
            should_quit: false,
            first_run,
            search_input: String::new(),
            search_results: Vec::new(),
            search_selected: 0,
            search_loading: false,
            show_preview: false,
            preview_content: None,
            preview_scroll: 0,
            repos,
            repos_selected: 0,
            confirm_dialog: None,
            status_message: None,
            loading: false,
            loading_message: None,
        }
    }

    /// Dismiss welcome screen and go to search mode
    pub fn dismiss_welcome(&mut self) {
        self.mode = AppMode::Search;
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
            AppMode::Welcome | AppMode::Help => {}
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
            AppMode::Welcome | AppMode::Help => {}
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

    /// Delete selected repository (direct, no confirmation)
    #[allow(dead_code)]
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

    /// Toggle preview pane for selected search result
    pub fn toggle_preview(&mut self) {
        if self.mode != AppMode::Search || self.search_results.is_empty() {
            return;
        }

        if self.show_preview {
            self.show_preview = false;
            self.preview_content = None;
            self.preview_scroll = 0;
        } else {
            self.load_preview();
        }
    }

    /// Load preview content for selected result
    fn load_preview(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        let result = &self.search_results[self.search_selected];
        let path = &result.absolute_path;

        match std::fs::read_to_string(path) {
            Ok(content) => {
                self.preview_content = Some(content);
                self.show_preview = true;
                self.preview_scroll = 0;
            }
            Err(e) => {
                self.set_status(format!("Cannot read file: {e}"), StatusLevel::Error);
            }
        }
    }

    /// Update preview when selection changes
    pub fn update_preview_if_visible(&mut self) {
        if self.show_preview {
            self.load_preview();
        }
    }

    /// Scroll preview up
    pub fn preview_scroll_up(&mut self) {
        if self.preview_scroll > 0 {
            self.preview_scroll = self.preview_scroll.saturating_sub(1);
        }
    }

    /// Scroll preview down
    pub fn preview_scroll_down(&mut self, max_lines: usize) {
        if let Some(ref content) = self.preview_content {
            let total_lines = content.lines().count();
            if self.preview_scroll + max_lines < total_lines {
                self.preview_scroll += 1;
            }
        }
    }

    /// Request deletion confirmation
    pub fn request_delete_repo(&mut self) {
        if self.mode != AppMode::Repos || self.repos.is_empty() {
            return;
        }

        let repo = &self.repos[self.repos_selected];
        self.confirm_dialog = Some(ConfirmDialog {
            title: "Delete Repository".to_string(),
            message: format!(
                "Remove \"{}\" from index?\n\nThis will delete the index data.\nThe actual files won't be affected.",
                repo.name
            ),
            on_confirm: ConfirmAction::DeleteRepo(repo.id, repo.name.clone()),
        });
    }

    /// Confirm the pending action
    pub fn confirm_action(&mut self) {
        if let Some(dialog) = self.confirm_dialog.take() {
            match dialog.on_confirm {
                ConfirmAction::DeleteRepo(id, name) => {
                    if let Err(e) = self.db.delete_repository(id) {
                        self.set_status(format!("Delete error: {e}"), StatusLevel::Error);
                        return;
                    }
                    self.set_status(format!("Removed: {name}"), StatusLevel::Success);
                    self.refresh_repos();

                    if self.repos_selected >= self.repos.len() && !self.repos.is_empty() {
                        self.repos_selected = self.repos.len() - 1;
                    }
                }
            }
        }
    }

    /// Cancel the pending confirmation
    pub fn cancel_confirm(&mut self) {
        self.confirm_dialog = None;
    }

    /// Set loading state
    #[allow(dead_code)]
    pub fn set_loading(&mut self, loading: bool, message: Option<&str>) {
        self.loading = loading;
        self.loading_message = message.map(String::from);
    }
}
