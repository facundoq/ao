use crate::dashboard::app::{App, ProcessSort};
use crate::dashboard::utils::format_bytes;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

fn get_user_color(user: &str) -> Color {
    let hash = user.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64));
    let colors = [Color::Red, Color::Green, Color::Yellow, Color::Blue, Color::Magenta, Color::Cyan];
    colors[(hash as usize) % colors.len()]
}

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    if app.use_tree_view {
        draw_process_tree(f, app, area);
    } else {
        draw_process_list(f, app, area);
    }
}

fn draw_process_list(f: &mut Frame, app: &App, area: Rect) {
    let sort_indicator = if app.sort_descending { "▼" } else { "▲" };
    let header_cells: Vec<Cell> = [
        ("PID", app.process_sort == ProcessSort::Pid),
        ("User", app.process_sort == ProcessSort::User),
        ("CPU%", app.process_sort == ProcessSort::Cpu),
        ("RSS", app.process_sort == ProcessSort::Mem),
        ("Name", app.process_sort == ProcessSort::Name),
        ("Command", false),
    ].iter().map(|(h, active)| {
        let text = if *active { format!("{} {}", h, sort_indicator) } else { h.to_string() };
        Cell::from(text).style(Style::default().add_modifier(Modifier::BOLD))
    }).collect();

    let header = Row::new(header_cells).height(1).bottom_margin(1);
    let rows = app.sorted_processes.iter().skip(app.selected_index).take(area.height.saturating_sub(4) as usize).map(|p| {
        Row::new(vec![
            Cell::from(p.pid.to_string()),
            Cell::from(p.user.clone()).style(Style::default().fg(get_user_color(&p.user))),
            Cell::from(format!("{:.1}", p.cpu)),
            Cell::from(format_bytes(p.memory)),
            Cell::from(p.name.clone()),
            Cell::from(p.command.clone()),
        ])
    });

    let table = Table::new(rows, [Constraint::Length(8), Constraint::Length(12), Constraint::Length(10), Constraint::Length(10), Constraint::Length(20), Constraint::Min(0)])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" Process List "));
    f.render_widget(table, area);
}

fn draw_process_tree(_f: &mut Frame, _app: &App, _area: Rect) {
    // Tree drawing simplified...
}
