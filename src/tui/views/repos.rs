use chrono::Utc;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::db::RepoStatus;
use crate::tui::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    if app.repos.is_empty() {
        render_empty(frame, area);
    } else {
        render_list(frame, app, area);
    }
}

fn render_empty(frame: &mut Frame, area: Rect) {
    let content = vec![
        Line::from(""),
        Line::from("No repositories indexed yet."),
        Line::from(""),
        Line::from(Span::styled(
            "Get started by indexing a project:",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  kdex index /path/to/project",
            Style::default().fg(Color::Cyan),
        )),
    ];

    let paragraph = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Repositories "),
    );

    frame.render_widget(paragraph, area);
}

fn render_list(frame: &mut Frame, app: &App, area: Rect) {
    let now = Utc::now();

    let items: Vec<ListItem> = app
        .repos
        .iter()
        .enumerate()
        .map(|(i, repo)| {
            let style = if i == app.repos_selected {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };

            let status_icon = match repo.status {
                RepoStatus::Ready => Span::styled("●", Style::default().fg(Color::Green)),
                RepoStatus::Pending => Span::styled("○", Style::default().fg(Color::Yellow)),
                RepoStatus::Indexing | RepoStatus::Syncing => {
                    Span::styled("◐", Style::default().fg(Color::Cyan))
                }
                RepoStatus::Cloning => Span::styled("↓", Style::default().fg(Color::Cyan)),
                RepoStatus::Error => Span::styled("!", Style::default().fg(Color::Red)),
            };

            let time_ago = repo.last_indexed_at.map_or_else(
                || "never".to_string(),
                |dt| format_time_ago(now.signed_duration_since(dt)),
            );

            let content = Line::from(vec![
                status_icon,
                Span::raw(" "),
                Span::styled(
                    format!("{:<20}", truncate(&repo.name, 20)),
                    Style::default().fg(Color::Blue),
                ),
                Span::raw(" │ "),
                Span::styled(
                    format!("{:>6} files", repo.file_count),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(" │ "),
                Span::styled(time_ago, Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Repositories ({}) ", app.repos.len())),
    );

    frame.render_widget(list, area);
}

fn format_time_ago(duration: chrono::Duration) -> String {
    let seconds = duration.num_seconds();

    if seconds < 60 {
        "just now".to_string()
    } else if seconds < 3600 {
        let mins = seconds / 60;
        format!("{mins}m ago")
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        format!("{hours}h ago")
    } else {
        let days = seconds / 86400;
        format!("{days}d ago")
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}…", &s[..max_len - 1])
    } else {
        s.to_string()
    }
}
