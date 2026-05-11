use crate::dashboard::app::App;
use crate::dashboard::utils::format_uptime;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
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
    let distro_name = if let Ok(distro) = crate::os::detector::Distro::detect() {
        format!("{:?}", distro)
    } else {
        "Linux".to_string()
    };

    let header_content = vec![Line::from(vec![
        Span::styled(" ao dashboard ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(format!(" v{} ", env!("CARGO_PKG_VERSION"))),
        Span::raw(" | "),
        Span::styled(format!(" OS: {} ", distro_name), Style::default().fg(Color::Magenta)),
        Span::raw(" | "),
        Span::styled(format!(" Host: {} ", hostname), Style::default().fg(Color::Green)),
        Span::raw(" | "),
        Span::styled(format!(" Uptime: {} ", uptime), Style::default().fg(Color::Yellow)),
    ])];

    f.render_widget(Paragraph::new(header_content).block(Block::default().borders(Borders::ALL)), area);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let icons = ["🏠", "💾", "⚙", "👤", "🌐", "🛠", "🐳", "🌡", "📈"];
    let titles: Vec<Line> = app.tabs.iter().enumerate().map(|(i, t)| {
        Line::from(vec![
            Span::raw(format!("{} ", icons.get(i).unwrap_or(&""))),
            Span::raw(*t),
        ])
    }).collect();
    
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Domains "))
        .select(app.tab_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, area);
}

fn draw_content(f: &mut Frame, app: &mut App, area: Rect) {
    if app.get_current_list_len() == 0 && app.tab_index != 0 && app.tab_index != 8 {
        f.render_widget(Paragraph::new("Loading...").block(Block::default().borders(Borders::ALL).title(" Data ")), area);
        return;
    }

    match app.tab_index {
        0 => super::overview::draw(f, app, area),
        1 => super::storage::draw(f, app, area),
        2 => super::processes::draw(f, app, area),
        3 => super::users::draw(f, app, area),
        4 => super::network::draw(f, app, area),
        5 => super::services::draw(f, app, area),
        6 => super::virtualization::draw(f, app, area),
        7 => super::sensors::draw(f, app, area),
        8 => super::charts::draw(f, app, area),
        _ => f.render_widget(Paragraph::new("Coming Soon...").block(Block::default().borders(Borders::ALL)), area),
    }
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![
        Span::styled("[q]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Quit | "),
        Span::styled("[Tab/Arrows]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Tabs"),
    ];

    if app.tab_index != 0 {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled("[PgUp/PgDn]", Style::default().add_modifier(Modifier::BOLD)));
        spans.push(Span::raw(" Scroll"));
    }

    if app.tab_index == 2 {
        spans.push(Span::raw(" | "));
        spans.push(Span::styled("[i/c/m/n/u]", Style::default().add_modifier(Modifier::BOLD)));
        spans.push(Span::raw(" Sort | "));
        spans.push(Span::styled("[o]", Style::default().add_modifier(Modifier::BOLD)));
        spans.push(Span::raw(" Own | "));
        spans.push(Span::styled("[k]", Style::default().add_modifier(Modifier::BOLD)));
        spans.push(Span::raw(" Kernel"));
    }

    f.render_widget(Paragraph::new(vec![Line::from(spans)]).block(Block::default().borders(Borders::ALL)), area);
}
