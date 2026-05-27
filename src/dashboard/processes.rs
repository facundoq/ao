use crate::dashboard::app::{App, ProcessDetails, ProcessSort};
use crate::dashboard::utils::format_bytes;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

fn get_user_color(user: &str, app: &App) -> Color {
    let hash = user.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64));
    let colors = [
        app.theme.selected_color,
        [0, 255, 0],
        [0, 0, 255],
        [255, 0, 255],
        [0, 255, 255],
    ];
    Color::Rgb(
        colors[(hash as usize) % colors.len()][0],
        colors[(hash as usize) % colors.len()][1],
        colors[(hash as usize) % colors.len()][2],
    )
}

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    if app.use_tree_view && app.tree_expansion_depth > 0 {
        draw_process_tree(f, app, area);
    } else {
        draw_process_list(f, app, area);
    }
}

pub fn draw_full_details(f: &mut Frame, app: &App, details: &ProcessDetails, area: Rect) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(4),
            ratatui::layout::Constraint::Min(0),
        ])
        .split(area);

    let search_indicator = if details.is_searching {
        format!(" (SEARCHING: {}|)", details.filter)
    } else {
        format!(" (Filter: {}) ", details.filter)
    };

    let title = format!(
        " Process: {} (PID: {}) | [Esc: Close] [/: Search] {} ",
        details.name, details.pid, search_indicator
    );

    let stats_text = format!(
        "Executable: {}\nCPU: {:.1}% | Memory: {} | Disk Read: {}/s | Disk Write: {}/s",
        details.exe,
        details.cpu,
        crate::dashboard::utils::format_bytes(details.memory),
        crate::dashboard::utils::format_bytes(details.disk_read),
        crate::dashboard::utils::format_bytes(details.disk_write),
    );

    let stats_block = ratatui::widgets::Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(stats_block, chunks[0]);

    let items_per_page = chunks[1].height.saturating_sub(4) as usize;
    let total_items = details.filtered_resources.len();
    let total_pages = (total_items + items_per_page - 1).max(1) / items_per_page;
    let current_page = if total_items > 0 {
        (details.selected_index / items_per_page + 1).min(total_pages)
    } else {
        1
    };

    let header = Row::new(vec![
        Cell::from("Type").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Resource Name").style(Style::default().add_modifier(Modifier::BOLD)),
    ]);

    let rows = details
        .filtered_resources
        .iter()
        .enumerate()
        .skip(details.scroll_offset)
        .take(items_per_page)
        .map(|(i, r)| {
            let mut style = match r.resource_type.as_str() {
                "Socket" => Style::default().fg(Color::Magenta),
                "Pipe" => Style::default().fg(Color::Cyan),
                "File" => Style::default().fg(Color::White),
                _ => Style::default(),
            };

            if i + details.scroll_offset == details.selected_index {
                style = style
                    .bg(Color::Rgb(
                        app.theme.selected_color[0],
                        app.theme.selected_color[1],
                        app.theme.selected_color[2],
                    ))
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD);
            }

            Row::new(vec![
                Cell::from(r.resource_type.clone()),
                Cell::from(r.name.clone()),
            ])
            .style(style)
        });

    let table_title = format!("Files & Sockets [Page {}/{}]", current_page, total_pages);

    let table = Table::new(rows, [Constraint::Length(10), Constraint::Min(0)])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(table_title))
        .widths([Constraint::Length(10), Constraint::Min(0)]);

    f.render_widget(table, chunks[1]);
}

fn draw_process_list(f: &mut Frame, app: &App, area: Rect) {
    let sort_indicator = if app.sort_descending { "▼" } else { "▲" };
    let mut header_data: Vec<(&str, bool)> = vec![
        ("PID", app.process_sort == ProcessSort::Pid),
        ("User", app.process_sort == ProcessSort::User),
        ("CPU%", app.process_sort == ProcessSort::Cpu),
        ("VIRT", false),
        ("RSS", app.process_sort == ProcessSort::Mem),
        ("SHR", false),
    ];
    if !app.show_user_threads {
        header_data.push(("Threads", false));
    }
    header_data.push(("Name", app.process_sort == ProcessSort::Name));
    header_data.push(("Command", false));

    let header_cells: Vec<Cell> = header_data
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
    let items_per_page = area.height.saturating_sub(4) as usize;
    let total_items = app.sorted_processes.len();
    let total_pages = (total_items + items_per_page - 1).max(1) / items_per_page;
    let current_page = (app.selected_index / items_per_page + 1).min(total_pages);

    let rows = app
        .sorted_processes
        .iter()
        .enumerate()
        .skip(app.scroll_offset)
        .take(items_per_page)
        .map(|(i, p)| {
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
            let mut row_cells = vec![
                Cell::from(p.pid.to_string()),
                Cell::from(p.user.clone()).style(if i == app.selected_index {
                    Style::default()
                } else {
                    Style::default().fg(get_user_color(&p.user, app))
                }),
                Cell::from(format!("{:>6.1}%", p.cpu)),
                Cell::from(format_bytes(p.virt_mem)),
                Cell::from(format_bytes(p.memory)),
                Cell::from(format_bytes(p.shared_mem)),
            ];
            if !app.show_user_threads {
                row_cells.push(Cell::from(p.thread_count.to_string()));
            }
            row_cells.push(Cell::from(p.name.clone()));
            row_cells.push(Cell::from(p.command.clone()));
            Row::new(row_cells).style(style)
        });

    let mut title = format!(" Process List [Page {}/{}] ", current_page, total_pages);
    if !app.process_filter.is_empty() {
        title.push_str(&format!("(Filter: {}) ", app.process_filter));
    }
    if app.is_filtering {
        title.push_str(" [EDITING FILTER] ");
    }

    let mut widths = vec![
        Constraint::Length(7),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(9),
        Constraint::Length(9),
        Constraint::Length(9),
    ];
    if !app.show_user_threads {
        widths.push(Constraint::Length(8));
    }
    widths.push(Constraint::Length(20));
    widths.push(Constraint::Min(0));

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(table, area);
}

fn draw_process_tree(f: &mut Frame, app: &App, area: Rect) {
    let sort_indicator = if app.sort_descending { "▼" } else { "▲" };
    let mut header_data: Vec<(&str, bool)> = vec![
        ("PID", app.process_sort == ProcessSort::Pid),
        ("User", app.process_sort == ProcessSort::User),
        ("CPU%", app.process_sort == ProcessSort::Cpu),
        ("VIRT", false),
        ("RSS", app.process_sort == ProcessSort::Mem),
        ("SHR", false),
    ];
    if !app.show_user_threads {
        header_data.push(("Threads", false));
    }
    header_data.push(("Process Tree", app.process_sort == ProcessSort::Name));

    let header_cells: Vec<Cell> = header_data
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
    let items_per_page = area.height.saturating_sub(4) as usize;
    let total_items = app.flattened_tree.len();
    let total_pages = (total_items + items_per_page - 1).max(1) / items_per_page;
    let current_page = (app.selected_index / items_per_page + 1).min(total_pages);

    let rows = app
        .flattened_tree
        .iter()
        .enumerate()
        .skip(app.scroll_offset)
        .take(items_per_page)
        .map(|(i, node)| {
            let mut style = if node.depth == 0 {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            if i == app.selected_index {
                style = style
                    .bg(Color::Rgb(
                        app.theme.selected_color[0],
                        app.theme.selected_color[1],
                        app.theme.selected_color[2],
                    ))
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD);
            }
            let mut row_cells = vec![
                Cell::from(node.pid.clone()),
                Cell::from(node.user.clone()).style(if i == app.selected_index {
                    Style::default()
                } else {
                    Style::default().fg(get_user_color(&node.user, app))
                }),
                Cell::from(node.cpu.clone()),
                Cell::from(node.virt.clone()),
                Cell::from(node.mem.clone()),
                Cell::from(node.shared.clone()),
            ];
            if !app.show_user_threads {
                row_cells.push(Cell::from(node.threads.clone()));
            }
            row_cells.push(Cell::from(node.name.clone()));
            Row::new(row_cells).style(style)
        });

    let mut title = format!(
        " Process Tree (Depth {}) [Page {}/{}] ",
        app.tree_expansion_depth, current_page, total_pages
    );
    if !app.process_filter.is_empty() {
        title.push_str(&format!("(Filter: {}) ", app.process_filter));
    }
    if app.is_filtering {
        title.push_str(" [EDITING FILTER] ");
    }

    let mut widths = vec![
        Constraint::Length(7),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(9),
        Constraint::Length(9),
        Constraint::Length(9),
    ];
    if !app.show_user_threads {
        widths.push(Constraint::Length(8));
    }
    widths.push(Constraint::Min(0));

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(table, area);
}
