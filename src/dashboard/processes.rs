use crate::dashboard::app::{App, ProcessDetails, ProcessSort};
use crate::dashboard::utils::format_bytes;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
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
    let mut main_area = area;

    if app.is_filtering || !app.process_filter.is_empty() {
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Min(0),
            ])
            .split(area);

        let filter_title = if app.is_filtering {
            " Filter (Editing) "
        } else {
            " Filter "
        };
        let mut filter_text = app.process_filter.clone();
        if app.is_filtering {
            filter_text.push('|');
        }

        let filter_block = ratatui::widgets::Paragraph::new(filter_text)
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red))
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Red))
                    .title(filter_title),
            );
        f.render_widget(filter_block, chunks[0]);
        main_area = chunks[1];
    }

    if app.use_tree_view && app.tree_expansion_depth > 0 {
        draw_process_tree(f, app, main_area);
    } else {
        draw_process_list(f, app, main_area);
    }
}

pub fn draw_full_details(f: &mut Frame, app: &App, details: &ProcessDetails, area: Rect) {
    let mut main_area = area;

    if details.is_searching || !details.filter.is_empty() {
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Min(0),
            ])
            .split(area);

        let filter_title = if details.is_searching {
            " Filter (Editing) "
        } else {
            " Filter "
        };
        let mut filter_text = details.filter.clone();
        if details.is_searching {
            filter_text.push('|');
        }

        let filter_block = ratatui::widgets::Paragraph::new(filter_text)
            .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red))
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Red))
                    .title(filter_title),
            );
        f.render_widget(filter_block, chunks[0]);
        main_area = chunks[1];
    }

    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(4),
            ratatui::layout::Constraint::Min(0),
        ])
        .split(main_area);

    let title = format!(
        " Process: {} (PID: {}) | [Esc: Close] [/: Search] ",
        details.name, details.pid
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
                Cell::from(r.resource_type.as_str()),
                Cell::from(r.name.as_str()),
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

fn make_header_cell(
    label: &str,
    active: bool,
    sortable: bool,
    sort_indicator: &str,
) -> Cell<'static> {
    if sortable && !label.is_empty() {
        let first_char = label.chars().next().unwrap().to_string();
        let rest: String = label.chars().skip(1).collect();
        let mut spans = vec![
            Span::styled(
                first_char,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(rest, Style::default().add_modifier(Modifier::BOLD)),
        ];
        if active {
            spans.push(Span::styled(
                format!(" {}", sort_indicator),
                Style::default().add_modifier(Modifier::BOLD),
            ));
        }
        Cell::from(Line::from(spans))
    } else {
        let text = if active {
            format!("{} {}", label, sort_indicator)
        } else {
            label.to_string()
        };
        Cell::from(text).style(Style::default().add_modifier(Modifier::BOLD))
    }
}

fn draw_process_list(f: &mut Frame, app: &App, area: Rect) {
    let sort_indicator = if app.sort_descending { "▼" } else { "▲" };
    let mut header_data = vec![
        ("PID", app.process_sort == ProcessSort::Pid, true),
        ("User", app.process_sort == ProcessSort::User, true),
        ("CPU%", app.process_sort == ProcessSort::Cpu, true),
        ("VIRT", false, false),
        ("RSS", app.process_sort == ProcessSort::Mem, true),
        ("SHR", false, false),
    ];
    if !app.show_user_threads {
        header_data.push(("Threads", false, false));
    }
    header_data.push(("Name", app.process_sort == ProcessSort::Name, true));
    header_data.push(("Command", false, false));

    let header_cells: Vec<Cell> = header_data
        .iter()
        .map(|(h, active, sortable)| make_header_cell(h, *active, *sortable, sort_indicator))
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
                Cell::from(p.user.as_str()).style(if i == app.selected_index {
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
            row_cells.push(Cell::from(p.name.as_str()));
            row_cells.push(Cell::from(p.command.as_str()));
            Row::new(row_cells).style(style)
        });

    // When a filter is active but no processes match, show a clear "no results" message
    // instead of an empty table (which looks like a broken or loading state).
    if total_items == 0 && !app.process_filter.is_empty() {
        let msg = format!(" No processes matching '{}'  ", app.process_filter);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Process List ");
        let para = Paragraph::new(Line::from(msg))
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(para, area);
        return;
    }

    let title = format!(" Process List [Page {}/{}] ", current_page, total_pages);

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
    let mut header_data = vec![
        ("PID", app.process_sort == ProcessSort::Pid, true),
        ("User", app.process_sort == ProcessSort::User, true),
        ("CPU%", app.process_sort == ProcessSort::Cpu, true),
        ("VIRT", false, false),
        ("RSS", app.process_sort == ProcessSort::Mem, true),
        ("SHR", false, false),
    ];
    if !app.show_user_threads {
        header_data.push(("Threads", false, false));
    }
    header_data.push(("Name", app.process_sort == ProcessSort::Name, true));

    let header_cells: Vec<Cell> = header_data
        .iter()
        .map(|(h, active, sortable)| make_header_cell(h, *active, *sortable, sort_indicator))
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
                Cell::from(node.pid.as_str()),
                Cell::from(node.user.as_str()).style(if i == app.selected_index {
                    Style::default()
                } else {
                    Style::default().fg(get_user_color(&node.user, app))
                }),
                Cell::from(node.cpu.as_str()),
                Cell::from(node.virt.as_str()),
                Cell::from(node.mem.as_str()),
                Cell::from(node.shared.as_str()),
            ];
            if !app.show_user_threads {
                row_cells.push(Cell::from(node.threads.as_str()));
            }
            row_cells.push(Cell::from(node.name.as_str()));
            Row::new(row_cells).style(style)
        });

    // When a filter is active but no processes match, show a clear "no results" message.
    if total_items == 0 && !app.process_filter.is_empty() {
        let msg = format!(" No processes matching '{}'  ", app.process_filter);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Process Tree ");
        let para = Paragraph::new(Line::from(msg))
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(para, area);
        return;
    }

    let title = format!(
        " Process Tree (Depth {}) [Page {}/{}] ",
        app.tree_expansion_depth, current_page, total_pages
    );

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
