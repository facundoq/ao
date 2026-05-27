use crate::dashboard::app::App;
use crate::dashboard::utils::{format_bytes, make_bar};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let items_per_page = area.height.saturating_sub(4) as usize;
    let total_items = app.disks.len();
    let total_pages = (total_items + items_per_page - 1).max(1) / items_per_page;
    let current_page = (app.selected_index / items_per_page + 1).min(total_pages);

    let disk_rows = app
        .disks
        .iter()
        .enumerate()
        .skip(app.scroll_offset)
        .take(items_per_page)
        .map(|(i, d)| {
            let name = d.name().to_string_lossy().to_string();
            let is_physical = name.starts_with("/dev/sd")
                || name.starts_with("/dev/nvme")
                || name.starts_with("/dev/vd");
            let icon = if is_physical { "💽" } else { "💾" };

            let total = d.total_space();
            let used = total - d.available_space();
            let percent = used
                .checked_mul(100)
                .and_then(|v| v.checked_div(total))
                .unwrap_or(0);
            let bar = make_bar(percent as u16, 20);

            let style = if i == app.selected_index {
                Style::default()
                    .bg(Color::Rgb(
                        app.theme.selected_color[0],
                        app.theme.selected_color[1],
                        app.theme.selected_color[2],
                    ))
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let mut bar_style = Style::default();
            if i == app.selected_index {
                bar_style = bar_style
                    .bg(Color::Rgb(
                        app.theme.selected_color[0],
                        app.theme.selected_color[1],
                        app.theme.selected_color[2],
                    ))
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD);
            } else if percent >= 90 {
                bar_style = bar_style.fg(Color::Rgb(
                    app.theme.storage_color_high[0],
                    app.theme.storage_color_high[1],
                    app.theme.storage_color_high[2],
                ));
            } else if percent >= 50 {
                bar_style = bar_style.fg(Color::Rgb(
                    app.theme.storage_color_med[0],
                    app.theme.storage_color_med[1],
                    app.theme.storage_color_med[2],
                ));
            } else {
                bar_style = bar_style.fg(Color::Rgb(
                    app.theme.storage_color_low[0],
                    app.theme.storage_color_low[1],
                    app.theme.storage_color_low[2],
                ));
            }

            Row::new(vec![
                Cell::from(format!("{} {}", icon, name)),
                Cell::from(d.mount_point().to_string_lossy().to_string()),
                Cell::from(format_bytes(used)),
                Cell::from(format_bytes(total)),
                Cell::from(format!("{:>3}% {}", percent, bar)).style(bar_style),
            ])
            .style(style)
        });

    let title = format!(" Storage [Page {}/{}] ", current_page, total_pages);
    let disk_table = Table::new(
        disk_rows,
        [
            Constraint::Length(18),
            Constraint::Length(15),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Min(30),
        ],
    )
    .header(
        Row::new(vec!["Device", "Mount", "Used", "Total", "Usage"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(disk_table, area);
}
