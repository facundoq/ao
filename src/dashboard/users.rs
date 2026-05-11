use crate::dashboard::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    // Sessions Table
    let header_cells = ["User", "TTY", "Host", "Start", "End"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let items_per_page = chunks[0].height.saturating_sub(4) as usize;
    let total_items = app.sessions.len();
    let total_pages = (total_items + items_per_page - 1).max(1) / items_per_page;
    let current_page = (app.selected_index / items_per_page + 1).min(total_pages);

    let rows = app
        .sessions
        .iter()
        .enumerate()
        .skip(app.scroll_offset)
        .take(items_per_page)
        .map(|(i, s)| {
            let mut style = if s.end == "still logged in" {
                Style::default().fg(Color::Green)
            } else if s.end == "crash" || s.end == "down" {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Yellow)
            };

            if i == app.selected_index {
                style = style
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD);
            }

            Row::new(vec![
                Cell::from(s.username.clone()),
                Cell::from(s.line.clone()),
                Cell::from(s.host.clone()),
                Cell::from(s.start.clone()),
                Cell::from(s.end.clone()),
            ])
            .style(style)
        });

    let title = format!(" Recent Sessions [Page {}/{}] ", current_page, total_pages);
    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(15),
            Constraint::Min(20),
            Constraint::Min(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(table, chunks[0]);

    // Users List (Unpaged in UI, just shows what fits)
    let user_list: Vec<Line> = app
        .users
        .iter()
        .take(chunks[1].height.saturating_sub(2) as usize)
        .map(|(u, is_system)| {
            let is_logged_in = app
                .sessions
                .iter()
                .any(|s| s.username == *u && s.end == "still logged in");
            let style = if is_logged_in {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if *is_system {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(vec![Span::styled(u.clone(), style)])
        })
        .collect();
    f.render_widget(
        Paragraph::new(user_list).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" System Users "),
        ),
        chunks[1],
    );
}
