//! Welcome screen view for first-time users.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;

/// Render the welcome screen for first-time users.
pub fn render(frame: &mut Frame, _app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(area);

    let content_area = chunks[1];

    let title = vec![
        Line::from(vec![
            Span::styled(
                "Welcome to ",
                Style::default().fg(Color::White),
            ),
            Span::styled(
                "knowledge-index",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Index and search your code repositories and knowledge bases",
            Style::default().fg(Color::Gray),
        )),
    ];

    let getting_started = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Getting Started",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  1. ", Style::default().fg(Color::Cyan)),
            Span::raw("Index a directory from the command line:"),
        ]),
        Line::from(vec![
            Span::styled("     $ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "knowledge-index index ~/projects/myapp",
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  2. ", Style::default().fg(Color::Cyan)),
            Span::raw("Index an Obsidian vault or notes folder:"),
        ]),
        Line::from(vec![
            Span::styled("     $ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "knowledge-index index ~/Documents/notes",
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  3. ", Style::default().fg(Color::Cyan)),
            Span::raw("Then return here to search across all indexed content"),
        ]),
    ];

    let hints = vec![
        Line::from(""),
        Line::from(Span::styled(
            "─────────────────────────────────────────",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled("Enter", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" to continue  •  ", Style::default().fg(Color::Gray)),
            Span::styled("?", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" for help  •  ", Style::default().fg(Color::Gray)),
            Span::styled("q", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" to quit", Style::default().fg(Color::Gray)),
        ]),
    ];

    let mut all_lines = title;
    all_lines.extend(getting_started);
    all_lines.extend(hints);

    let welcome = Paragraph::new(all_lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" 🚀 First Run ")
                .title_alignment(Alignment::Center),
        );

    frame.render_widget(welcome, content_area);
}
