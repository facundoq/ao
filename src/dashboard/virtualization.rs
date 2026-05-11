use crate::dashboard::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["ID", "Image", "Status", "Names"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let items_per_page = area.height.saturating_sub(4) as usize;
    let total_items = app.containers.len();
    let total_pages = (total_items + items_per_page - 1).max(1) / items_per_page;
    let current_page = (app.selected_index / items_per_page + 1).min(total_pages);

    let rows = app
        .containers
        .iter()
        .enumerate()
        .skip(app.scroll_offset)
        .take(items_per_page)
        .map(|(i, c)| {
            let mut style = if c.status.contains("Up") {
                Style::default().fg(Color::Green)
            } else if c.status.contains("Exited") {
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
                Cell::from(c.id.clone()),
                Cell::from(c.image.clone()),
                Cell::from(c.status.clone()),
                Cell::from(c.names.clone()),
            ])
            .style(style)
        });

    let title = format!(" Virtualization [Page {}/{}] ", current_page, total_pages);
    let table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Length(25),
            Constraint::Length(20),
            Constraint::Min(0),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(table, area);
}
