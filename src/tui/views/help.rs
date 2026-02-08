use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, area: Rect) {
    // Calculate centered popup
    let popup_width = 60.min(area.width.saturating_sub(4));
    let popup_height = 20.min(area.height.saturating_sub(4));

    let popup_area = centered_rect(popup_width, popup_height, area);

    // Clear background
    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Global",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  ?           Toggle this help"),
        Line::from("  q           Quit application"),
        Line::from("  Ctrl+C      Force quit"),
        Line::from("  Tab         Switch between views"),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  ↑/k         Move up"),
        Line::from("  ↓/j         Move down"),
        Line::from("  Enter       Select / Open file"),
        Line::from("  Esc         Clear / Go back"),
        Line::from(""),
        Line::from(Span::styled(
            "Search View",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Type        Start searching"),
        Line::from("  Ctrl+U      Clear search"),
        Line::from("  o           Open in editor"),
        Line::from(""),
        Line::from(Span::styled(
            "Repos View",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  d           Delete from index"),
        Line::from("  r           Refresh list"),
        Line::from(""),
        Line::from(Span::styled(
            "Press ? or Esc to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let help = Paragraph::new(help_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(help, popup_area);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((area.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(vertical[1]);

    horizontal[1]
}
