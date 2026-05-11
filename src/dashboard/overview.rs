use crate::dashboard::app::App;
use crate::dashboard::utils::{format_bytes, make_bar};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table},
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

    // --- LEFT COLUMN ---

    // 1. RAM
    let mem_used = app.system_info.used_memory();
    let mem_total = app.system_info.total_memory();
    let mem_percent = mem_used.checked_mul(100).and_then(|v| v.checked_div(mem_total)).unwrap_or(0) as u16;
    let ram_title = format!(" RAM ({}) ", app.ram_config);
    let mem_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(ram_title))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(mem_percent)
        .label(format!(
            "{} / {} ({}%) | RSS (TOTAL): {}",
            format_bytes(mem_used),
            format_bytes(mem_total),
            mem_percent,
            format_bytes(app.total_rss)
        ));
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
        .gauge_style(Style::default().fg(Color::Rgb(255, 200, 150)))
        .percent(cpu_usage as u16);
    f.render_widget(cpu_gauge, left_chunks[2]);

    // 4. CPU Cores
    let core_count = app.system_info.cpus().len();
    let cores_block = Block::default().borders(Borders::ALL).title(format!(" CPU Cores ({}) ", core_count));
    let cores_inner = cores_block.inner(left_chunks[3]);
    f.render_widget(cores_block, left_chunks[3]);

    let core_height = if cores_inner.height >= (core_count as u16 * 3) { 3 } else { 1 };
    let mut core_constraints = vec![Constraint::Length(core_height); core_count];
    core_constraints.push(Constraint::Min(0));
    let core_chunks = Layout::default().direction(Direction::Vertical).constraints(core_constraints).split(cores_inner);

    for (i, cpu) in app.system_info.cpus().iter().enumerate() {
        if i < core_chunks.len() {
            let usage = cpu.cpu_usage();
            let chunk = core_chunks[i];
            if core_height >= 3 {
                let row_chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Length(4), Constraint::Min(0)]).split(chunk);
                let core_label = Paragraph::new(format!("C{:02}", i)).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
                f.render_widget(core_label, Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)]).split(row_chunks[0])[1]);
                let core_gauge = Gauge::default().block(Block::default().borders(Borders::ALL)).gauge_style(Style::default().fg(Color::Rgb(255, 200, 150))).percent(usage as u16).label(format!("{:.1}%", usage));
                f.render_widget(core_gauge, row_chunks[1]);
            } else if chunk.height > 0 {
                let label = format!("C{:02} ", i);
                let bar_width = chunk.width.saturating_sub(12);
                let bar = make_bar(usage as u16, bar_width);
                let content = format!("{}{:>5.1}% {}", label, usage, bar);
                f.render_widget(Paragraph::new(content).style(Style::default().fg(Color::Rgb(255, 200, 150))), chunk);
            }
        }
    }

    // --- RIGHT COLUMN ---

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
