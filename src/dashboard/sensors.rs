use crate::dashboard::app::App;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Device/Label", "Temperature", "Max", "Critical"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let mut sensors = app.sensors.clone();
    sensors.sort_by(|a, b| a.label.cmp(&b.label));

    let rows = sensors.iter().skip(app.selected_index).take(area.height.saturating_sub(4) as usize).map(|s| {
        let label_lower = s.label.to_lowercase();
        let icon = if label_lower.contains("nvme") || label_lower.contains("disk") { "💾" } 
        else if label_lower.contains("wifi") || label_lower.contains("iwl") { "🌐" } 
        else if label_lower.contains("cpu") || label_lower.contains("core") || label_lower.contains("k10") { "🖥️" } 
        else if label_lower.contains("gpu") || label_lower.contains("amdgpu") || label_lower.contains("nvidia") { "🎮" } 
        else { "🧩" };

        let max_temp = app.max_temps.get(&s.label).copied().unwrap_or(s.temperature);
        let temp_style = if let Some(crit) = s.critical {
            if s.temperature >= crit { Style::default().fg(Color::Red).add_modifier(Modifier::BOLD) } 
            else if s.temperature >= crit * 0.8 { Style::default().fg(Color::Yellow) } 
            else { Style::default().fg(Color::Green) }
        } else { Style::default().fg(Color::Green) };

        Row::new(vec![
            Cell::from(format!("{} {}", icon, s.label)),
            Cell::from(format!("{:.1}°C", s.temperature)).style(temp_style),
            Cell::from(format!("{:.1}°C", max_temp)),
            Cell::from(s.critical.map(|c| format!("{:.1}°C", c)).unwrap_or_else(|| "N/A".to_string())),
        ])
    });

    let table = Table::new(rows, [Constraint::Min(30), Constraint::Length(15), Constraint::Length(15), Constraint::Length(15)])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" Hardware Sensors "));
    f.render_widget(table, area);
}
