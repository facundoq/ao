use crate::dashboard::app::{App, ProcessDetails, ProcessSort};
use crate::dashboard::utils::format_bytes;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

fn get_user_color(user: &str) -> Color {
    let hash = user.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64));
    let colors = [
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
    ];
    colors[(hash as usize) % colors.len()]
}

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    if app.use_tree_view && app.tree_expansion_depth > 0 {
        draw_process_tree(f, app, area);
    } else {
        draw_process_list(f, app, area);
    }
}

pub fn draw_full_details(f: &mut Frame, _app: &App, details: &ProcessDetails, area: Rect) {
    let search_indicator = if details.is_searching {
        format!(" (SEARCHING: {}|)", details.filter)
    } else {
        format!(" (Filter: {}) ", details.filter)
    };

    let title = format!(
        " Process: {} (PID: {}) | Files & Sockets [Esc: Close] [/: Search] {} ",
        details.name, details.pid, search_indicator
    );

    let items_per_page = area.height.saturating_sub(4) as usize;
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
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD);
            }

            Row::new(vec![
                Cell::from(r.resource_type.clone()),
                Cell::from(r.name.clone()),
            ])
            .style(style)
        });

    let table_title = format!("{} [Page {}/{}]", title, current_page, total_pages);

    let table = Table::new(rows, [Constraint::Length(10), Constraint::Min(0)])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(table_title))
        .widths([Constraint::Length(10), Constraint::Min(0)]);

    f.render_widget(table, area);
}

fn draw_process_list(f: &mut Frame, app: &App, area: Rect) {
    let sort_indicator = if app.sort_descending { "▼" } else { "▲" };
    let header_cells: Vec<Cell> = [
        ("PID", app.process_sort == ProcessSort::Pid),
        ("User", app.process_sort == ProcessSort::User),
        ("CPU%", app.process_sort == ProcessSort::Cpu),
        ("VIRT", false),
        ("RSS", app.process_sort == ProcessSort::Mem),
        ("SHR", false),
        ("Name", app.process_sort == ProcessSort::Name),
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
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(p.pid.to_string()),
                Cell::from(p.user.clone()).style(if i == app.selected_index {
                    Style::default()
                } else {
                    Style::default().fg(get_user_color(&p.user))
                }),
                Cell::from(format!("{:.1}%", p.cpu)),
                Cell::from(format_bytes(p.virt_mem)),
                Cell::from(format_bytes(p.memory)),
                Cell::from(format_bytes(p.shared_mem)),
                Cell::from(p.name.clone()),
                Cell::from(p.command.clone()),
            ])
            .style(style)
        });

    let mut title = format!(" Process List [Page {}/{}] ", current_page, total_pages);
    if !app.process_filter.is_empty() {
        title.push_str(&format!("(Filter: {}) ", app.process_filter));
    }
    if app.is_filtering {
        title.push_str(" [EDITING FILTER] ");
    }

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(20),
            Constraint::Min(0),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(table, area);
}

fn draw_process_tree(f: &mut Frame, app: &App, area: Rect) {
    let sort_indicator = if app.sort_descending { "▼" } else { "▲" };
    let header_cells: Vec<Cell> = [
        ("PID", app.process_sort == ProcessSort::Pid),
        ("User", app.process_sort == ProcessSort::User),
        ("CPU%", app.process_sort == ProcessSort::Cpu),
        ("VIRT", false),
        ("RSS", app.process_sort == ProcessSort::Mem),
        ("SHR", false),
        ("Process Tree", app.process_sort == ProcessSort::Name),
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
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD);
            }
            Row::new(vec![
                Cell::from(node.pid.clone()),
                Cell::from(node.user.clone()).style(if i == app.selected_index {
                    Style::default()
                } else {
                    Style::default().fg(get_user_color(&node.user))
                }),
                Cell::from(node.cpu.clone()),
                Cell::from(node.virt.clone()),
                Cell::from(node.mem.clone()),
                Cell::from(node.shared.clone()),
                Cell::from(node.name.clone()),
            ])
            .style(style)
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

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Min(0),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(table, area);
}
