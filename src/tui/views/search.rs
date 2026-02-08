use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(0),    // Results (and preview)
        ])
        .split(area);

    // Search input
    let input_style = Style::default().fg(Color::Yellow);
    let input = Paragraph::new(format!("› {}", app.search_input))
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search ")
                .border_style(Style::default().fg(Color::Blue)),
        );
    frame.render_widget(input, chunks[0]);

    // Results or empty state
    if app.search_input.is_empty() {
        render_empty_state(frame, app, chunks[1]);
    } else if app.search_results.is_empty() {
        render_no_results(frame, &app.search_input, chunks[1]);
    } else if app.show_preview {
        render_results_with_preview(frame, app, chunks[1]);
    } else {
        render_results(frame, app, chunks[1]);
    }
}

fn render_empty_state(frame: &mut Frame, app: &App, area: Rect) {
    let content = if app.first_run {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "Welcome to knowledge-index!",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from("Get started:"),
            Line::from("  1. Press Tab to view repositories"),
            Line::from("  2. Or run: knowledge-index index /path/to/project"),
            Line::from(""),
            Line::from(Span::styled(
                "Press ? for help",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "Start typing to search...",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Examples:",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  async fn          - search for text",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  \"exact phrase\"    - match exactly",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  func*             - prefix matching",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    };

    let paragraph =
        Paragraph::new(content).block(Block::default().borders(Borders::ALL).title(" Results "));

    frame.render_widget(paragraph, area);
}

fn render_no_results(frame: &mut Frame, query: &str, area: Rect) {
    let content = vec![
        Line::from(""),
        Line::from(format!("No results for \"{query}\"")),
        Line::from(""),
        Line::from(Span::styled(
            "Suggestions:",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  • Check spelling",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  • Try broader search terms",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "  • Use prefix matching: func*",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title(" Results "));

    frame.render_widget(paragraph, area);
}

fn render_results(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let style = if i == app.search_selected {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };

            let content = vec![
                Line::from(vec![
                    Span::styled(&result.repo_name, Style::default().fg(Color::Blue)),
                    Span::raw(":"),
                    Span::styled(
                        result.file_path.to_string_lossy().to_string(),
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                Line::from(Span::styled(
                    truncate_snippet(&result.snippet, area.width as usize - 4),
                    Style::default().fg(Color::DarkGray),
                )),
            ];

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(format!(
        " Results ({}) [p]review ",
        app.search_results.len()
    )));

    frame.render_widget(list, area);
}

fn render_results_with_preview(frame: &mut Frame, app: &App, area: Rect) {
    // Split horizontally: results on left, preview on right
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Render compact results list
    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let style = if i == app.search_selected {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };

            let content = Line::from(vec![
                Span::styled(&result.repo_name, Style::default().fg(Color::Blue)),
                Span::raw(":"),
                Span::styled(
                    result
                        .file_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    Style::default().fg(Color::Cyan),
                ),
            ]);

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Results ({}) ", app.search_results.len())),
    );

    frame.render_widget(list, chunks[0]);

    // Render preview pane
    let preview_content = app.preview_content.as_deref().unwrap_or("Loading...");
    let lines: Vec<Line> = preview_content
        .lines()
        .skip(app.preview_scroll)
        .take(area.height.saturating_sub(2) as usize)
        .enumerate()
        .map(|(i, line)| {
            let line_num = app.preview_scroll + i + 1;
            Line::from(vec![
                Span::styled(
                    format!("{line_num:4} "),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(line),
            ])
        })
        .collect();

    let selected_file = if app.search_results.is_empty() {
        String::new()
    } else {
        app.search_results[app.search_selected]
            .file_path
            .to_string_lossy()
            .to_string()
    };

    let preview = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Preview: {selected_file} "))
            .border_style(Style::default().fg(Color::Green)),
    );

    frame.render_widget(preview, chunks[1]);
}

fn truncate_snippet(snippet: &str, max_len: usize) -> String {
    let cleaned = snippet
        .replace(">>>", "")
        .replace("<<<", "")
        .replace('\n', " ")
        .trim()
        .to_string();

    if cleaned.len() > max_len {
        format!("{}...", &cleaned[..max_len.saturating_sub(3)])
    } else {
        cleaned
    }
}
