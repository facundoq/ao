use crate::dashboard::app::App;
use crate::dashboard::utils::format_bytes;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Gauge, Row, Table},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
        .split(area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // RAM
            Constraint::Length(3), // Swap
            Constraint::Length(3), // Global CPU
            Constraint::Min(0),    // CPU Cores
        ])
        .split(main_chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Top CPU Processes
            Constraint::Percentage(50), // Top Mem Processes
        ])
        .split(main_chunks[1]);

    // 1. RAM
    let mem_used = app.system_info.used_memory();
    let mem_total = app.system_info.total_memory();
    let mem_percent = mem_used.checked_mul(100).and_then(|v| v.checked_div(mem_total)).unwrap_or(0) as u16;
    let ram_title = format!(" RAM ({}) ", app.ram_config);
    let mem_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(ram_title))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(mem_percent)
        .label(format!("{} / {} ({}%)", format_bytes(mem_used), format_bytes(mem_total), mem_percent));
    f.render_widget(mem_gauge, left_chunks[0]);

    // 2. Swap
    let swap_used = app.system_info.used_swap();
    let swap_total = app.system_info.total_swap();
    let swap_percent = swap_used.checked_mul(100).and_then(|v| v.checked_div(swap_total)).unwrap_or(0) as u16;
    let swap_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Swap "))
        .gauge_style(Style::default().fg(Color::Magenta))
        .percent(swap_percent)
        .label(format!("{} / {} ({}%)", format_bytes(swap_used), format_bytes(swap_total), swap_percent));
    f.render_widget(swap_gauge, left_chunks[1]);

    // 3. Global CPU
    let cpu_usage = app.system_info.global_cpu_usage();
    let cpu_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" CPU "))
        .gauge_style(Style::default().fg(Color::Yellow))
        .percent(cpu_usage as u16);
    f.render_widget(cpu_gauge, left_chunks[2]);

    // 4. CPU Cores - Placeholder for brevity
    let cores_block = Block::default().borders(Borders::ALL).title(" CPU Cores ");
    f.render_widget(cores_block, left_chunks[3]);

    // 1. Top CPU Processes
    let top_cpu = &app.top_cpu_processes;
    let cpu_proc_rows: Vec<Row> = top_cpu.iter().map(|p| {
        Row::new(vec![
            Cell::from(p.pid.to_string()),
            Cell::from(p.user.clone()),
            Cell::from(p.name.clone()),
            Cell::from(format!("{:.1}%", p.cpu)),
            Cell::from(format_bytes(p.memory)),
        ])
    }).collect();
    let cpu_proc_table = Table::new(cpu_proc_rows, [Constraint::Length(8), Constraint::Length(12), Constraint::Min(0), Constraint::Length(8), Constraint::Length(10)])
        .header(Row::new(vec!["PID", "User", "Name", "CPU%", "RSS"]).style(Style::default().add_modifier(Modifier::BOLD)))
        .block(Block::default().borders(Borders::ALL).title(" Top 10 CPU Processes "));
    f.render_widget(cpu_proc_table, right_chunks[0]);

    // 2. Top Mem Processes
    let top_mem = &app.top_mem_processes;
    let mem_proc_rows: Vec<Row> = top_mem.iter().map(|p| {
        Row::new(vec![
            Cell::from(p.pid.to_string()),
            Cell::from(p.user.clone()),
            Cell::from(p.name.clone()),
            Cell::from(format!("{:.1}%", p.cpu)),
            Cell::from(format_bytes(p.memory)),
        ])
    }).collect();
    let mem_proc_table = Table::new(mem_proc_rows, [Constraint::Length(8), Constraint::Length(12), Constraint::Min(0), Constraint::Length(8), Constraint::Length(10)])
        .header(Row::new(vec!["PID", "User", "Name", "CPU%", "RSS"]).style(Style::default().add_modifier(Modifier::BOLD)))
        .block(Block::default().borders(Borders::ALL).title(" Top 10 Mem Processes "));
    f.render_widget(mem_proc_table, right_chunks[1]);
}
