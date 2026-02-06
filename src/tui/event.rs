use crossterm::event::{KeyCode, KeyModifiers};

use super::app::{App, AppMode};

/// Handle keyboard input
pub fn handle_key_event(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    // Global keys
    match code {
        KeyCode::Char('c' | 'd') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
            return;
        }
        KeyCode::Char('?') => {
            app.mode = if app.mode == AppMode::Help {
                AppMode::Search
            } else {
                AppMode::Help
            };
            return;
        }
        _ => {}
    }

    // Mode-specific keys
    match app.mode {
        AppMode::Help => handle_help_keys(app, code),
        AppMode::Search => handle_search_keys(app, code, modifiers),
        AppMode::Repos => handle_repos_keys(app, code),
    }
}

fn handle_help_keys(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = AppMode::Search;
        }
        _ => {}
    }
}

fn handle_search_keys(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match code {
        KeyCode::Char('q') if app.search_input.is_empty() => {
            app.should_quit = true;
        }
        KeyCode::Esc => {
            if !app.search_input.is_empty() {
                app.search_input.clear();
                app.search_results.clear();
            }
        }
        KeyCode::Tab => {
            app.mode = AppMode::Repos;
            app.refresh_repos();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_prev();
        }
        KeyCode::Enter | KeyCode::Char('o') => {
            app.open_selected();
        }
        KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.search_input.clear();
            app.search_results.clear();
        }
        KeyCode::Backspace => {
            app.search_input.pop();
            app.search();
        }
        KeyCode::Char(c) => {
            app.search_input.push(c);
            app.search();
        }
        _ => {}
    }
}

fn handle_repos_keys(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Tab => {
            app.mode = AppMode::Search;
        }
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_prev();
        }
        KeyCode::Char('d') => {
            app.delete_selected_repo();
        }
        KeyCode::Char('r') => {
            app.refresh_repos();
            app.set_status("Refreshed".to_string(), super::app::StatusLevel::Info);
        }
        _ => {}
    }
}
