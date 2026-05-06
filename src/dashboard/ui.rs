use crate::dashboard::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, Tabs},
};
use sysinfo::System;

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_tabs(f, app, chunks[1]);
    draw_content(f, app, chunks[2]);
    draw_footer(f, app, chunks[3]);
}

fn draw_header(f: &mut Frame, _app: &App, area: Rect) {
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
    let uptime = format_uptime(System::uptime());

    // Get distro info from the detected system directly for now
    let distro_name = if let Ok(distro) = crate::os::detector::Distro::detect() {
        format!("{:?}", distro)
    } else {
        "Linux".to_string()
    };

    let header_content = vec![Line::from(vec![
        Span::styled(
            " ao dashboard ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(" v{} ", env!("CARGO_PKG_VERSION"))),
        Span::raw(" | "),
        Span::styled(
            format!(" OS: {} ", distro_name),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(" Host: {} ", hostname),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(" Uptime: {} ", uptime),
            Style::default().fg(Color::Yellow),
        ),
    ])];

    let block = Block::default().borders(Borders::ALL);
    let paragraph = Paragraph::new(header_content).block(block);
    f.render_widget(paragraph, area);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = app.tabs.iter().cloned().map(Line::from).collect();
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Domains "))
        .select(app.tab_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, area);
}

fn draw_content(f: &mut Frame, app: &mut App, area: Rect) {
    match app.tab_index {
        0 => draw_overview(f, app, area),
        1 => draw_processes(f, app, area),
        2 => draw_users(f, app, area),
        3 => draw_network(f, app, area),
        4 => draw_services(f, app, area),
        5 => draw_virtualization(f, app, area),
        _ => {
            let block = Block::default().borders(Borders::ALL);
            let paragraph = Paragraph::new("Coming Soon...").block(block);
            f.render_widget(paragraph, area);
        }
    }
}

fn draw_overview(f: &mut Frame, app: &App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
        .split(area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Global CPU
            Constraint::Min(0),    // CPU Cores
            Constraint::Length(3), // RAM
            Constraint::Length(3), // Swap
        ])
        .split(main_chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30), // Top CPU Processes
            Constraint::Percentage(30), // Top Mem Processes
            Constraint::Min(0),         // Disks
        ])
        .split(main_chunks[1]);

    // --- LEFT COLUMN ---

    // 1. Global CPU Gauge
    let cpu_usage = app.system_info.global_cpu_usage();
    let cpu_gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Global CPU Usage "),
        )
        .gauge_style(Style::default().fg(Color::Rgb(255, 200, 150))) // "Tan"-like color
        .percent(cpu_usage as u16);
    f.render_widget(cpu_gauge, left_chunks[0]);

    // 2. CPU Cores Area
    let core_count = app.system_info.cpus().len();
    let cores_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" CPU Cores ({}) ", core_count));
    let cores_inner = cores_block.inner(left_chunks[1]);
    f.render_widget(cores_block, left_chunks[1]);

    // Sub-layout for cores inside the block. Scale to fit.
    let core_constraints = vec![Constraint::Ratio(1, core_count as u32); core_count];
    let core_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(core_constraints)
        .split(cores_inner);

    for (i, cpu) in app.system_info.cpus().iter().enumerate() {
        if i < core_chunks.len() {
            let usage = cpu.cpu_usage();
            let chunk = core_chunks[i];

            if chunk.height >= 3 {
                let row_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(4), Constraint::Min(0)])
                    .split(chunk);

                let label_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ])
                    .split(row_chunks[0]);

                let core_label = Paragraph::new(format!("C{:02}", i)).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                );
                f.render_widget(core_label, label_chunks[1]);

                let core_gauge = Gauge::default()
                    .block(Block::default().borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::Rgb(150, 255, 255)))
                    .percent(usage as u16)
                    .label(format!("{:.1}%", usage));
                f.render_widget(core_gauge, row_chunks[1]);
            } else if chunk.height > 0 {
                // Compact view for many cores
                let label = format!("C{:02} ", i);
                let bar_width = chunk.width.saturating_sub(12); // Space for label and percentage
                let bar = make_bar(usage as u16, bar_width);
                let content = format!("{}{:>5.1}% {}", label, usage, bar);
                let p =
                    Paragraph::new(content).style(Style::default().fg(Color::Rgb(150, 255, 255)));
                f.render_widget(p, chunk);
            }
        }
    }

    // 3. RAM
    let mem_used = app.system_info.used_memory();
    let mem_total = app.system_info.total_memory();
    let mem_percent = mem_used
        .checked_mul(100)
        .and_then(|v| v.checked_div(mem_total))
        .unwrap_or(0) as u16;
    let mem_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" RAM "))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(mem_percent)
        .label(format!(
            "{} / {} ({}%)",
            format_bytes(mem_used),
            format_bytes(mem_total),
            mem_percent
        ));
    f.render_widget(mem_gauge, left_chunks[2]);

    // 4. Swap
    let swap_used = app.system_info.used_swap();
    let swap_total = app.system_info.total_swap();
    let swap_percent = swap_used
        .checked_mul(100)
        .and_then(|v| v.checked_div(swap_total))
        .unwrap_or(0) as u16;
    let swap_gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Swap "))
        .gauge_style(Style::default().fg(Color::Magenta))
        .percent(swap_percent)
        .label(format!(
            "{} / {} ({}%)",
            format_bytes(swap_used),
            format_bytes(swap_total),
            swap_percent
        ));
    f.render_widget(swap_gauge, left_chunks[3]);

    // --- RIGHT COLUMN ---

    // 1. Top CPU Processes
    let top_cpu = app.get_top_cpu_processes(5);
    let cpu_proc_rows: Vec<Row> = top_cpu
        .iter()
        .map(|p| {
            let user_name = p
                .user_id()
                .and_then(|uid| app.users_list.get_user_by_id(uid))
                .map(|u| u.name().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            Row::new(vec![
                Cell::from(p.pid().to_string()),
                Cell::from(user_name),
                Cell::from(p.name().to_string_lossy().into_owned()),
                Cell::from(format!("{:.1}%", p.cpu_usage())),
            ])
        })
        .collect();
    let cpu_proc_table = Table::new(
        cpu_proc_rows,
        [
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Min(0),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["PID", "User", "Name", "CPU%"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Top 5 CPU Processes "),
    );
    f.render_widget(cpu_proc_table, right_chunks[0]);

    // 2. Top Mem Processes
    let top_mem = app.get_top_mem_processes(5);
    let mem_proc_rows: Vec<Row> = top_mem
        .iter()
        .map(|p| {
            let user_name = p
                .user_id()
                .and_then(|uid| app.users_list.get_user_by_id(uid))
                .map(|u| u.name().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            Row::new(vec![
                Cell::from(p.pid().to_string()),
                Cell::from(user_name),
                Cell::from(p.name().to_string_lossy().into_owned()),
                Cell::from(format_bytes(p.memory())),
            ])
        })
        .collect();
    let mem_proc_table = Table::new(
        mem_proc_rows,
        [
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Min(0),
            Constraint::Length(12),
        ],
    )
    .header(
        Row::new(vec!["PID", "User", "Name", "Mem"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Top 5 Mem Processes "),
    );
    f.render_widget(mem_proc_table, right_chunks[1]);

    // 3. Disks
    let disk_rows = app.disks.iter().map(|d| {
        let total = d.total_space();
        let available = d.available_space();
        let used = total - available;
        let percent = used
            .checked_mul(100)
            .and_then(|v| v.checked_div(total))
            .unwrap_or(0);
        let bar = make_bar(percent as u16, 15);
        Row::new(vec![
            Cell::from(d.name().to_string_lossy().to_string()),
            Cell::from(d.mount_point().to_string_lossy().to_string()),
            Cell::from(format_bytes(used)),
            Cell::from(format_bytes(total)),
            Cell::from(format!("{:>3}% {}", percent, bar)),
        ])
    });
    let disk_table = Table::new(
        disk_rows,
        [
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Min(20),
        ],
    )
    .header(
        Row::new(vec!["Dev", "Mount", "Used", "Total", "Usage"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Physical Disks "),
    );
    f.render_widget(disk_table, right_chunks[2]);
}

fn make_bar(percent: u16, width: u16) -> String {
    let filled = (percent as f32 / 100.0 * width as f32).round() as u16;
    let mut bar = String::from("[");
    for i in 0..width {
        if i < filled {
            bar.push('■');
        } else {
            bar.push(' ');
        }
    }
    bar.push(']');
    bar
}

fn draw_processes(f: &mut Frame, app: &App, area: Rect) {
    if app.use_tree_view {
        draw_process_tree(f, app, area);
    } else {
        draw_process_list(f, app, area);
    }
}

fn draw_process_list(f: &mut Frame, app: &App, area: Rect) {
    let sort_indicator = if app.sort_descending { "▼" } else { "▲" };

    let header_cells: Vec<Cell> = [
        (
            "PID",
            app.process_sort == crate::dashboard::app::ProcessSort::Pid,
        ),
        ("User", false),
        (
            "CPU%",
            app.process_sort == crate::dashboard::app::ProcessSort::Cpu,
        ),
        (
            "MEM%",
            app.process_sort == crate::dashboard::app::ProcessSort::Mem,
        ),
        (
            "Name",
            app.process_sort == crate::dashboard::app::ProcessSort::Name,
        ),
        ("Command", false),
    ]
    .iter()
    .map(|(h, active)| {
        let text = if *active {
            format!("{} {}", h, sort_indicator)
        } else {
            h.to_string()
        };
        Cell::from(text).style(Style::default().add_modifier(Modifier::BOLD))
    })
    .collect();

    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let processes = app.get_sorted_processes();
    let items_per_page = area.height.saturating_sub(4) as usize;
    let total_items = processes.len();
    let total_pages = (total_items + items_per_page - 1).max(1) / items_per_page;
    let current_page = (app.selected_index / items_per_page + 1).min(total_pages);

    let start = app.selected_index;
    let rows = processes.iter().skip(start).take(items_per_page).map(|p| {
        let user_name = p
            .user_id()
            .and_then(|uid| app.users_list.get_user_by_id(uid))
            .map(|u| u.name().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Row::new(vec![
            Cell::from(p.pid().to_string()),
            Cell::from(user_name),
            Cell::from(format!("{:.1}", p.cpu_usage())),
            Cell::from(format_bytes(p.memory())),
            Cell::from(p.name().to_string_lossy().into_owned()),
            Cell::from(
                p.exe()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_else(|| "".to_string()),
            ),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(20),
            Constraint::Min(0),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Process List [{}/{}] ", current_page, total_pages)),
    );
    f.render_widget(table, area);
}

fn draw_process_tree(f: &mut Frame, app: &App, area: Rect) {
    let tree_roots = app.get_process_tree();
    let mut flat_rows = Vec::new();

    fn flatten_tree(
        node: &crate::dashboard::app::ProcessTreeNode,
        max_depth: u32,
        rows: &mut Vec<(String, String, String, String, String, u32)>,
    ) {
        let indent = "  ".repeat(node.depth as usize);
        let prefix = if node.depth > 0 { "└─ " } else { "" };

        let cpu_str = if node.depth < max_depth && !node.children.is_empty() {
            format!("{:.1} ({:.1})", node.cpu_usage, node.total_cpu)
        } else {
            format!("{:.1}", node.cpu_usage)
        };

        let mem_str = if node.depth < max_depth && !node.children.is_empty() {
            format!(
                "{} ({})",
                format_bytes(node.memory),
                format_bytes(node.total_mem)
            )
        } else {
            format_bytes(node.memory)
        };

        rows.push((
            node.pid.to_string(),
            node.user.clone(),
            cpu_str,
            mem_str,
            format!("{}{}{}", indent, prefix, node.name),
            node.depth,
        ));

        if node.depth + 1 < max_depth {
            for child in &node.children {
                flatten_tree(child, max_depth, rows);
            }
        }
    }

    for root in tree_roots {
        flatten_tree(&root, app.tree_expansion_depth, &mut flat_rows);
    }

    let items_per_page = area.height.saturating_sub(4) as usize;
    let total_items = flat_rows.len();
    let total_pages = (total_items + items_per_page - 1).max(1) / items_per_page;
    let current_page = (app.selected_index / items_per_page + 1).min(total_pages);

    let start = app.selected_index;
    let rows: Vec<Row> = flat_rows
        .iter()
        .skip(start)
        .take(items_per_page)
        .map(|(pid, user, cpu, mem, name, depth)| {
            let style = if *depth == 0 {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(pid.clone()),
                Cell::from(user.clone()),
                Cell::from(cpu.clone()),
                Cell::from(mem.clone()),
                Cell::from(name.clone()),
            ])
            .style(style)
        })
        .collect();

    let header = Row::new(vec![
        Cell::from("PID").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("User").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("CPU% (Total)").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Mem (Total)").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Process Tree").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .height(1)
    .bottom_margin(1);

    let depth_str = if app.tree_expansion_depth >= 99 {
        "∞".to_string()
    } else {
        app.tree_expansion_depth.to_string()
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(15),
            Constraint::Length(20),
            Constraint::Min(0),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(format!(
        " Process Tree (Depth {}) [{}/{}] ",
        depth_str, current_page, total_pages
    )));
    f.render_widget(table, area);
}

fn draw_network(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = [
        "Interface",
        "Type",
        "State",
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

    let start = app.selected_index;
    let rows = app
        .interfaces
        .iter()
        .skip(start)
        .take(items_per_page)
        .map(|i| {
            let style = match i.state.to_lowercase().as_str() {
                "up" => Style::default().fg(Color::Green),
                "down" => Style::default().fg(Color::Red),
                _ => Style::default().fg(Color::Yellow),
            };

            let (rx_speed, tx_speed) = app.network_speeds.get(&i.name).cloned().unwrap_or((0, 0));
            let network_data = app.networks.get(&i.name);
            let total_rx = network_data.map(|n| n.received()).unwrap_or(0);
            let total_tx = network_data.map(|n| n.transmitted()).unwrap_or(0);

            Row::new(vec![
                Cell::from(i.name.clone()),
                Cell::from(i.interface_type.clone()),
                Cell::from(i.state.clone()),
                Cell::from(format!("{}/s", format_bytes(rx_speed))),
                Cell::from(format!("{}/s", format_bytes(tx_speed))),
                Cell::from(format_bytes(total_rx)),
                Cell::from(format_bytes(total_tx)),
            ])
            .style(style)
        });

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(format!(
        " Network Monitoring [{}/{}] ",
        current_page, total_pages
    )));
    f.render_widget(table, area);
}

fn draw_users(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    // Sessions Table
    let header_cells = ["User", "TTY", "Host", "Start", "End"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let items_per_page = chunks[0].height.saturating_sub(4) as usize;
    let total_sessions = app.sessions.len();
    let total_session_pages = (total_sessions + items_per_page - 1).max(1) / items_per_page;
    let current_session_page = (app.selected_index / items_per_page + 1).min(total_session_pages);

    let start = app.selected_index;
    let rows = app
        .sessions
        .iter()
        .skip(start)
        .take(items_per_page)
        .map(|s| {
            let style = if s.end == "still logged in" {
                Style::default().fg(Color::Green)
            } else if s.end == "crash" || s.end == "down" {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Yellow)
            };

            Row::new(vec![
                Cell::from(s.username.clone()),
                Cell::from(s.line.clone()),
                Cell::from(s.host.clone()),
                Cell::from(s.start.clone()),
                Cell::from(s.end.clone()),
            ])
            .style(style)
        });

    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(15),
            Constraint::Min(20),
            Constraint::Min(20),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(format!(
        " Recent Sessions [{}/{}] ",
        current_session_page, total_session_pages
    )));
    f.render_widget(table, chunks[0]);

    // Users List
    let users_items_per_page = chunks[1].height.saturating_sub(2) as usize;
    let total_users = app.users.len();
    let total_users_pages = (total_users + users_items_per_page - 1).max(1) / users_items_per_page;
    let current_users_page = (app.selected_index / users_items_per_page + 1).min(total_users_pages);

    let user_list: Vec<Line> = app
        .users
        .iter()
        .skip(start)
        .take(users_items_per_page)
        .map(|(u, is_system)| {
            let style = if *is_system {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(vec![Span::styled(u.clone(), style)])
        })
        .collect();
    let users_block = Block::default().borders(Borders::ALL).title(format!(
        " System Users [{}/{}] ",
        current_users_page, total_users_pages
    ));
    let users_paragraph = Paragraph::new(user_list).block(users_block);
    f.render_widget(users_paragraph, chunks[1]);
}

fn draw_services(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Service", "Loaded", "Active", "Status", "Description"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let items_per_page = area.height.saturating_sub(4) as usize;
    let total_items = app.services.len();
    let total_pages = (total_items + items_per_page - 1).max(1) / items_per_page;
    let current_page = (app.selected_index / items_per_page + 1).min(total_pages);

    let start = app.selected_index;
    let rows = app
        .services
        .iter()
        .skip(start)
        .take(items_per_page)
        .map(|s| {
            let style = if s.active == "active" {
                Style::default().fg(Color::Green)
            } else if s.active == "failed" {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Yellow)
            };

            Row::new(vec![
                Cell::from(s.name.clone()),
                Cell::from(s.loaded.clone()),
                Cell::from(s.active.clone()),
                Cell::from(s.status.clone()),
                Cell::from(s.description.clone()),
            ])
            .style(style)
        });

    let table = Table::new(
        rows,
        [
            Constraint::Length(30),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Min(0),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(format!(
        " System Service (Systemd) [{}/{}] ",
        current_page, total_pages
    )));
    f.render_widget(table, area);
}

fn draw_virtualization(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["ID", "Image", "Status", "Names"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let items_per_page = area.height.saturating_sub(4) as usize;
    let total_items = app.containers.len();
    let total_pages = (total_items + items_per_page - 1).max(1) / items_per_page;
    let current_page = (app.selected_index / items_per_page + 1).min(total_pages);

    let start = app.selected_index;
    let rows = app
        .containers
        .iter()
        .skip(start)
        .take(items_per_page)
        .map(|c| {
            let style = if c.status.contains("Up") {
                Style::default().fg(Color::Green)
            } else if c.status.contains("Exited") {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Yellow)
            };

            Row::new(vec![
                Cell::from(c.id.clone()),
                Cell::from(c.image.clone()),
                Cell::from(c.status.clone()),
                Cell::from(c.names.clone()),
            ])
            .style(style)
        });

    let table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Length(25),
            Constraint::Length(20),
            Constraint::Min(0),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(format!(
        " Virtualization (Docker Containers) [{}/{}] ",
        current_page, total_pages
    )));
    f.render_widget(table, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![
        Span::styled("[q]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Quit | "),
        Span::styled(
            "[Tab/Arrows]",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Tabs"),
    ];

    if app.tab_index != 0 {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(
            "[PgUp/PgDn]",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" Scroll"));
    }

    if app.tab_index == 1 {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled(
            "[p/c/m/n]",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" Sort | "));
        spans.push(Span::styled(
            "[t]",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" Tree | "));
        spans.push(Span::styled(
            "[0-9]",
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" Depth"));
    }

    let help_text = vec![Line::from(spans)];
    let block = Block::default().borders(Borders::ALL);
    let paragraph = Paragraph::new(help_text).block(block);
    f.render_widget(paragraph, area);
}

// Helper functions (could be moved to a common place if needed)
fn format_uptime(seconds: u64) -> String {
    let d = seconds / 86400;
    let h = (seconds % 86400) / 3600;
    let m = (seconds % 3600) / 60;
    if d > 0 {
        format!("{}d {}h {}m", d, h, m)
    } else if h > 0 {
        format!("{}h {}m", h, m)
    } else {
        format!("{}m", m)
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
