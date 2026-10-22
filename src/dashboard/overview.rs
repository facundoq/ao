use crate::dashboard::app::App;
use crate::dashboard::utils::format_bytes;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table},
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
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
    let mem_total = app.system_info.total_memory();
    let mem_available = app.system_info.available_memory();
    let mem_used = mem_total.saturating_sub(mem_available);
    let mem_ratio = if mem_total > 0 {
        mem_used as f64 / mem_total as f64
    } else {
        0.0
    };
    let mem_total_gb = mem_total as f64 / 1_073_741_824.0;
    let mem_gauge = Gauge::default()
        .block(
            Block::default().borders(Borders::ALL).title(" RAM ").title(
                ratatui::text::Line::from(format!(" {:.1} GB ", mem_total_gb))
                    .alignment(ratatui::layout::Alignment::Right),
            ),
        )
        .gauge_style(Style::default().fg(Color::Rgb(
            app.theme.ram_color[0],
            app.theme.ram_color[1],
            app.theme.ram_color[2],
        )))
        .ratio(mem_ratio)
        .label(format!("{:.1}%", mem_ratio * 100.0));
    f.render_widget(mem_gauge, left_chunks[0]);

    // 2. Swap
    let swap_used = app.system_info.used_swap();
    let swap_total = app.system_info.total_swap();
    let swap_ratio = if swap_total > 0 {
        swap_used as f64 / swap_total as f64
    } else {
        0.0
    };
    let swap_total_gb = swap_total as f64 / 1_073_741_824.0;
    let swap_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Swap ")
                .title(
                    ratatui::text::Line::from(format!(" {:.1} GB ", swap_total_gb))
                        .alignment(ratatui::layout::Alignment::Right),
                ),
        )
        .gauge_style(Style::default().fg(Color::Rgb(
            app.theme.swap_color[0],
            app.theme.swap_color[1],
            app.theme.swap_color[2],
        )))
        .ratio(swap_ratio)
        .label(format!("{:.1}%", swap_ratio * 100.0));
    f.render_widget(swap_gauge, left_chunks[1]);

    // 3. Global CPU
    let cpu_usage = app.system_info.global_cpu_usage();
    let title = if app.global_cpu_frequency > 0 {
        format!(
            " CPU ({:.1} GHz) ",
            app.global_cpu_frequency as f64 / 1000.0
        )
    } else {
        " CPU ".to_string()
    };
    let cpu_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(title))
        .gauge_style(
            Style::default()
                .fg(Color::Rgb(
                    app.theme.cpu_color[0],
                    app.theme.cpu_color[1],
                    app.theme.cpu_color[2],
                ))
                .bg(Color::Rgb(30, 30, 30)),
        )
        .ratio(cpu_usage as f64 / 100.0)
        .label(format!("{:.1}%", cpu_usage));
    f.render_widget(cpu_gauge, left_chunks[2]);

    // 4. CPU Cores
    let core_count = app.system_info.cpus().len();
    let cores_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Cores ({}) ", core_count));
    let cores_inner = cores_block.inner(left_chunks[3]);
    f.render_widget(cores_block, left_chunks[3]);

    let mut core_constraints = vec![Constraint::Length(1); core_count];
    core_constraints.push(Constraint::Min(0));
    let core_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(core_constraints)
        .split(cores_inner);

    for (i, cpu) in app.system_info.cpus().iter().enumerate() {
        if i < core_chunks.len() {
            let usage = cpu.cpu_usage();
            let chunk = core_chunks[i];

            let row_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(5), Constraint::Min(0)])
                .split(chunk);

            f.render_widget(
                Paragraph::new(format!("C{:02}:", i)).style(Style::default().fg(Color::Cyan)),
                row_chunks[0],
            );

            let core_gauge = Gauge::default()
                .gauge_style(
                    Style::default()
                        .fg(Color::Rgb(
                            app.theme.cpu_color[0],
                            app.theme.cpu_color[1],
                            app.theme.cpu_color[2],
                        ))
                        .bg(Color::Rgb(30, 30, 30)),
                )
                .ratio(usage as f64 / 100.0)
                .label(format!("{:>5.1}%", usage));
            f.render_widget(core_gauge, row_chunks[1]);
        }
    }

    // --- RIGHT COLUMN ---

    // 1. Top CPU Processes
    let top_cpu = &app.top_cpu_processes;
    let cpu_proc_rows: Vec<Row> = top_cpu
        .iter()
        .map(|p| {
            Row::new(vec![
                Cell::from(p.pid.to_string()),
                Cell::from(p.name.as_str()),
                Cell::from(format!("{:>6.1}%", p.cpu)),
                Cell::from(format_bytes(p.memory)),
                Cell::from(p.executable.as_str()),
            ])
        })
        .collect();
    let cpu_proc_table = Table::new(
        cpu_proc_rows,
        [
            Constraint::Length(7),
            Constraint::Length(15),
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Min(20),
        ],
    )
    .header(
        Row::new(vec!["PID", "Name", "CPU%", "RSS", "Executable"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Top CPU Processes "),
    );
    f.render_widget(cpu_proc_table, right_chunks[0]);

    // 2. Top Mem Processes
    let top_mem = &app.top_mem_processes;
    let mem_proc_rows: Vec<Row> = top_mem
        .iter()
        .map(|p| {
            Row::new(vec![
                Cell::from(p.pid.to_string()),
                Cell::from(p.name.as_str()),
                Cell::from(format!("{:>6.1}%", p.cpu)),
                Cell::from(format_bytes(p.memory)),
                Cell::from(p.executable.as_str()),
            ])
        })
        .collect();
    let mem_proc_table = Table::new(
        mem_proc_rows,
        [
            Constraint::Length(7),
            Constraint::Length(15),
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Min(20),
        ],
    )
    .header(
        Row::new(vec!["PID", "Name", "CPU%", "RSS", "Executable"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Top Mem Processes "),
    );
    f.render_widget(mem_proc_table, right_chunks[1]);
}
