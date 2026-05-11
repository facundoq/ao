use crate::dashboard::app::App;
use crate::dashboard::utils::format_bytes;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Interface", "Type", "State", "IPs", "Rx Speed", "Tx Speed", "Total Rx", "Total Tx"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app.interfaces.iter().skip(app.selected_index).take(area.height.saturating_sub(4) as usize).map(|i| {
        let style = match i.state.to_lowercase().as_str() {
            "up" => Style::default().fg(Color::Green),
            "down" => Style::default().fg(Color::Red),
            _ => Style::default().fg(Color::Yellow),
        };

        let (rx_speed, tx_speed) = app.network_speeds.get(&i.name).cloned().unwrap_or((0, 0));
        let total_rx = app.networks.iter().find(|(name, _)| name == &&i.name).map(|(_, n)| n.total_received()).unwrap_or(0);
        let total_tx = app.networks.iter().find(|(name, _)| name == &&i.name).map(|(_, n)| n.total_transmitted()).unwrap_or(0);

        Row::new(vec![
            Cell::from(i.name.clone()),
            Cell::from(i.interface_type.clone()),
            Cell::from(i.state.clone()),
            Cell::from(i.ips.join(", ")),
            Cell::from(format!("{}/s", format_bytes(rx_speed))),
            Cell::from(format!("{}/s", format_bytes(tx_speed))),
            Cell::from(format_bytes(total_rx)),
            Cell::from(format_bytes(total_tx)),
        ]).style(style)
    });

    let table = Table::new(rows, [Constraint::Length(12), Constraint::Length(10), Constraint::Length(8), Constraint::Min(20), Constraint::Length(12), Constraint::Length(12), Constraint::Length(12), Constraint::Length(12)])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" Network Monitoring "));
    f.render_widget(table, area);
}
