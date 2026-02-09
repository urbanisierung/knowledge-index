use crossterm::event::{KeyCode, KeyModifiers};

use super::app::{App, AppMode};

/// Handle keyboard input
pub fn handle_key_event(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    // Handle confirmation dialog first if active
    if app.confirm_dialog.is_some() {
        handle_confirm_keys(app, code);
        return;
    }

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
        AppMode::Welcome => handle_welcome_keys(app, code),
        AppMode::Help => handle_help_keys(app, code),
        AppMode::Search => handle_search_keys(app, code, modifiers),
        AppMode::Repos => handle_repos_keys(app, code),
    }
}

fn handle_welcome_keys(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter | KeyCode::Char(' ') => {
            app.dismiss_welcome();
        }
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('?') => {
            app.dismiss_welcome();
            app.mode = AppMode::Help;
        }
        _ => {}
    }
}

fn handle_confirm_keys(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('y' | 'Y') | KeyCode::Enter => {
            app.confirm_action();
        }
        KeyCode::Char('n' | 'N') | KeyCode::Esc => {
            app.cancel_confirm();
        }
        _ => {}
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
    // Handle preview mode separately
    if app.show_preview {
        match code {
            KeyCode::Char('p') if modifiers.contains(KeyModifiers::CONTROL) => {
                app.toggle_preview();
            }
            KeyCode::Esc => {
                app.toggle_preview();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.preview_scroll_down(20);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.preview_scroll_up();
            }
            KeyCode::Char('q') if modifiers.contains(KeyModifiers::CONTROL) => {
                app.should_quit = true;
            }
            KeyCode::Tab => {
                app.show_preview = false;
                app.mode = AppMode::Repos;
                app.refresh_repos();
            }
            _ => {}
        }
        return;
    }

    match code {
        // Ctrl+Q to quit (works even when typing)
        KeyCode::Char('q') if modifiers.contains(KeyModifiers::CONTROL) => {
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
        KeyCode::Down => {
            app.select_next();
            app.update_preview_if_visible();
        }
        // Ctrl+J to move down (works even when typing)
        KeyCode::Char('j') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.select_next();
            app.update_preview_if_visible();
        }
        KeyCode::Up => {
            app.select_prev();
            app.update_preview_if_visible();
        }
        // Ctrl+K to move up (works even when typing)
        KeyCode::Char('k') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.select_prev();
            app.update_preview_if_visible();
        }
        // Ctrl+P to toggle preview (works even when typing)
        KeyCode::Char('p') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.toggle_preview();
        }
        KeyCode::Enter if !app.search_input.is_empty() => {
            // Already searching on each keystroke, nothing to do
        }
        KeyCode::Char('o') if modifiers.contains(KeyModifiers::CONTROL) => {
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
            // Show confirmation dialog instead of immediate delete
            app.request_delete_repo();
        }
        KeyCode::Char('r') => {
            app.refresh_repos();
            app.set_status("Refreshed".to_string(), super::app::StatusLevel::Info);
        }
        _ => {}
    }
}
