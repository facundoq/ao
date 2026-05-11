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

    let rows = app
        .containers
        .iter()
        .skip(app.selected_index)
        .take(area.height.saturating_sub(4) as usize)
        .map(|c| {
            let style = if c.status.contains("Up") {
                Style::default().fg(Color::Green)
            } else if c.status.contains("Exited") {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Yellow)
            };

            Row::new(vec![
                Cell::from(c.id.clone()),
                Cell::from(c.image.clone()),
                Cell::from(c.status.clone()),
                Cell::from(c.names.clone()),
            ])
            .style(style)
        });

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
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Virtualization "),
    );
    f.render_widget(table, area);
}
