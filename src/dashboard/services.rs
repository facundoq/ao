use crate::dashboard::app::App;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Service", "Loaded", "Active", "Status", "Description"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app.services.iter().skip(app.selected_index).take(area.height.saturating_sub(4) as usize).map(|s| {
        let style = if s.active == "active" { Style::default().fg(Color::Green) } 
        else if s.active == "failed" { Style::default().fg(Color::Red) } 
        else { Style::default().fg(Color::Yellow) };

        Row::new(vec![
            Cell::from(s.name.clone()),
            Cell::from(s.loaded.clone()),
            Cell::from(s.active.clone()),
            Cell::from(s.status.clone()),
            Cell::from(s.description.clone()),
        ]).style(style)
    });

    let table = Table::new(rows, [Constraint::Length(30), Constraint::Length(10), Constraint::Length(10), Constraint::Length(10), Constraint::Min(0)])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" System Services "));
    f.render_widget(table, area);
}
