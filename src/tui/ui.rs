use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::app::{App, AppMode, StatusLevel};
use super::views;

const MIN_WIDTH: u16 = 60;
const MIN_HEIGHT: u16 = 15;

/// Main render function
pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Check minimum size
    if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
        render_size_warning(frame, size);
        return;
    }

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Status bar
        ])
        .split(size);

    // Render header
    render_header(frame, app, chunks[0]);

    // Render content based on mode
    match app.mode {
        AppMode::Welcome => views::welcome::render(frame, app, chunks[1]),
        AppMode::Search => views::search::render(frame, app, chunks[1]),
        AppMode::Repos => views::repos::render(frame, app, chunks[1]),
        AppMode::Help => {
            views::search::render(frame, app, chunks[1]);
            views::help::render(frame, chunks[1]);
        }
    }

    // Render status bar
    render_status_bar(frame, app, chunks[2]);

    // Render loading overlay if active
    if app.loading {
        render_loading(frame, app, size);
    }

    // Render confirmation dialog if active
    if let Some(ref dialog) = app.confirm_dialog {
        render_confirm_dialog(frame, dialog, size);
    }
}

fn render_size_warning(frame: &mut Frame, size: Rect) {
    let warning = Paragraph::new(vec![
        Line::from("Terminal too small!"),
        Line::from(""),
        Line::from(format!("Current: {}x{}", size.width, size.height)),
        Line::from(format!("Required: {MIN_WIDTH}x{MIN_HEIGHT}")),
        Line::from(""),
        Line::from("Please resize your terminal."),
    ])
    .style(Style::default().fg(Color::Red))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(warning, size);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let tabs: Vec<Span> = vec![
        Span::styled(
            " Search ",
            if app.mode == AppMode::Search {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
        Span::raw(" "),
        Span::styled(
            " Repos ",
            if app.mode == AppMode::Repos {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
    ];

    let header = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "knowledge-index",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )]),
        Line::from(tabs),
    ])
    .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(header, area);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (message, style) = if let Some((ref msg, level)) = app.status_message {
        let color = match level {
            StatusLevel::Info => Color::Blue,
            StatusLevel::Success => Color::Green,
            StatusLevel::Warning => Color::Yellow,
            StatusLevel::Error => Color::Red,
        };
        (msg.clone(), Style::default().fg(color))
    } else {
        let hints = match app.mode {
            AppMode::Welcome => "Enter continue │ ? help │ q quit",
            AppMode::Search => {
                if app.show_preview {
                    "j/k scroll preview │ p close preview │ Tab repos │ q quit"
                } else {
                    "Type to search │ ↑↓ navigate │ p preview │ Enter open │ Tab repos │ ? help │ q quit"
                }
            }
            AppMode::Repos => "↑↓ navigate │ d delete │ r refresh │ Tab search │ ? help │ q quit",
            AppMode::Help => "Press ? or Esc to close",
        };
        (hints.to_string(), Style::default().fg(Color::DarkGray))
    };

    let status = Paragraph::new(message).style(style);
    frame.render_widget(status, area);
}

fn render_loading(frame: &mut Frame, app: &App, size: Rect) {
    let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let spinner = spinner_chars[0]; // In real impl, animate based on time

    let message = app.loading_message.as_deref().unwrap_or("Loading...");
    let text = format!("{spinner} {message}");

    #[allow(clippy::cast_possible_truncation)]
    let width = (text.len() + 4) as u16;
    let height = 3;
    let area = centered_rect(width.min(size.width - 4), height, size);

    frame.render_widget(Clear, area);
    let loading = Paragraph::new(text).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)),
    );
    frame.render_widget(loading, area);
}

fn render_confirm_dialog(frame: &mut Frame, dialog: &super::app::ConfirmDialog, size: Rect) {
    let width = 45u16.min(size.width - 4);
    let height = 9u16.min(size.height - 4);
    let area = centered_rect(width, height, size);

    frame.render_widget(Clear, area);

    let content = vec![
        Line::from(""),
        Line::from(dialog.message.as_str()),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [Y]es  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                "  [N]o  ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let confirm = Paragraph::new(content)
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(format!(" {} ", dialog.title))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

    frame.render_widget(confirm, area);
}

/// Helper to create a centered rect
fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + (r.width.saturating_sub(width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(r.width), height.min(r.height))
}
