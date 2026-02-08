mod app;
mod event;
mod ui;
mod views;

pub use app::App;

use crossterm::{
    event::{
        self as crossterm_event, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};
use std::panic;

use crate::config::Config;
use crate::db::Database;
use crate::error::Result;

/// Run the TUI application
pub fn run() -> Result<()> {
    // Setup panic hook to restore terminal
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app
    let config = Config::load()?;
    let db = Database::open()?;
    let mut app = App::new(db, config);

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    restore_terminal()?;

    result
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        if crossterm_event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = crossterm_event::read()? {
                if key.kind == KeyEventKind::Press {
                    event::handle_key_event(app, key.code, key.modifiers);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
