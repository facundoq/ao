use crate::dashboard::app::App;
use crate::dashboard::utils::format_bytes;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let x_bounds = if app.cpu_history.is_empty() { [0.0, 60.0] } else {
        [app.cpu_history[0].0, app.cpu_history.last().unwrap().0.max(app.cpu_history[0].0 + 60.0)]
    };

    // 1. CPU Chart
    let cpu_dataset = Dataset::default().name("CPU %").marker(ratatui::symbols::Marker::Braille).graph_type(GraphType::Line).style(Style::default().fg(Color::Cyan)).data(&app.cpu_history);
    f.render_widget(Chart::new(vec![cpu_dataset]).block(Block::default().borders(Borders::ALL).title(" CPU Usage (%) ")).x_axis(Axis::default().bounds(x_bounds)).y_axis(Axis::default().bounds([0.0, 100.0]).labels(vec![Span::raw("0"), Span::raw("50"), Span::raw("100")])), top_chunks[0]);

    // 2. Memory Chart
    let mem_dataset = Dataset::default().name("Mem %").marker(ratatui::symbols::Marker::Braille).graph_type(GraphType::Line).style(Style::default().fg(Color::Green)).data(&app.mem_history);
    let swap_dataset = Dataset::default().name("Swap %").marker(ratatui::symbols::Marker::Braille).graph_type(GraphType::Line).style(Style::default().fg(Color::Magenta)).data(&app.swap_history);
    f.render_widget(Chart::new(vec![mem_dataset, swap_dataset]).block(Block::default().borders(Borders::ALL).title(" Memory & Swap (%) ")).x_axis(Axis::default().bounds(x_bounds)).y_axis(Axis::default().bounds([0.0, 100.0]).labels(vec![Span::raw("0"), Span::raw("50"), Span::raw("100")])), top_chunks[1]);

    // 3. Network RX/TX - Simplified
    f.render_widget(Block::default().borders(Borders::ALL).title(" Network History "), bottom_chunks[0]);
    f.render_widget(Block::default().borders(Borders::ALL).title(" More Stats "), bottom_chunks[1]);
}
