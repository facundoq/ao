use crate::dashboard::app::App;
use crate::dashboard::utils::{format_bytes, make_bar};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let disk_rows = app.disks.iter().map(|d| {
        let name = d.name().to_string_lossy().to_string();
        let is_physical = name.starts_with("/dev/sd") || name.starts_with("/dev/nvme") || name.starts_with("/dev/vd");
        let icon = if is_physical { "💽" } else { "💾" };

        let total = d.total_space();
        let used = total - d.available_space();
        let percent = used.checked_mul(100).and_then(|v| v.checked_div(total)).unwrap_or(0);
        let bar = make_bar(percent as u16, 20);
        Row::new(vec![
            Cell::from(format!("{} {}", icon, name)),
            Cell::from(d.mount_point().to_string_lossy().to_string()),
            Cell::from(format_bytes(used)),
            Cell::from(format_bytes(total)),
            Cell::from(format!("{:>3}% {}", percent, bar)),
        ])
    });

    let disk_table = Table::new(
        disk_rows,
        [Constraint::Length(18), Constraint::Length(15), Constraint::Length(12), Constraint::Length(12), Constraint::Min(30)],
    )
    .header(Row::new(vec!["Device", "Mount", "Used", "Total", "Usage"]).style(Style::default().add_modifier(Modifier::BOLD)))
    .block(Block::default().borders(Borders::ALL).title(" Storage "));
    f.render_widget(disk_table, area);
}
