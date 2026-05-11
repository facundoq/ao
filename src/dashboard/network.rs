use crate::dashboard::app::App;
use crate::dashboard::utils::format_bytes;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = [
        "Interface",
        "Type",
        "State",
        "IPs",
        "Rx Speed",
        "Tx Speed",
        "Total Rx",
        "Total Tx",
    ]
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let items_per_page = area.height.saturating_sub(4) as usize;
    let total_items = app.interfaces.len();
    let total_pages = (total_items + items_per_page - 1).max(1) / items_per_page;
    let current_page = (app.selected_index / items_per_page + 1).min(total_pages);

    let rows = app
        .interfaces
        .iter()
        .enumerate()
        .skip(app.scroll_offset)
        .take(items_per_page)
        .map(|(idx, i)| {
            let mut style = match i.state.to_lowercase().as_str() {
                "up" => Style::default().fg(Color::Green),
                "down" => Style::default().fg(Color::Red),
                _ => Style::default().fg(Color::Yellow),
            };

            if idx == app.selected_index {
                style = style
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD);
            }

            let type_lower = i.interface_type.to_lowercase();
            let icon = if type_lower.contains("wifi") || type_lower.contains("wlan") {
                "📡"
            } else if type_lower.contains("ethernet")
                || type_lower.contains("eth")
                || type_lower.contains("enp")
            {
                "🔗"
            } else if type_lower.contains("loopback") || type_lower.contains("lo") {
                "🔄"
            } else {
                "🌐"
            }; // Virtual or other

            let (rx_speed, tx_speed) = app.network_speeds.get(&i.name).cloned().unwrap_or((0, 0));
            let total_rx = app
                .networks
                .iter()
                .find(|(name, _)| name == &&i.name)
                .map(|(_, n)| n.total_received())
                .unwrap_or(0);
            let total_tx = app
                .networks
                .iter()
                .find(|(name, _)| name == &&i.name)
                .map(|(_, n)| n.total_transmitted())
                .unwrap_or(0);

            Row::new(vec![
                Cell::from(format!("{} {}", icon, i.name)),
                Cell::from(i.interface_type.clone()),
                Cell::from(i.state.clone()),
                Cell::from(i.ips.join(", ")),
                Cell::from(format!("{}/s", format_bytes(rx_speed))),
                Cell::from(format!("{}/s", format_bytes(tx_speed))),
                Cell::from(format_bytes(total_rx)),
                Cell::from(format_bytes(total_tx)),
            ])
            .style(style)
        });

    let title = format!(
        " Network Monitoring [Page {}/{}] ",
        current_page, total_pages
    );
    let table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Min(20),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(table, area);
}
