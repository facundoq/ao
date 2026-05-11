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

    // 3. Network RX Chart
    let max_rx = app.net_rx_history.iter().map(|(_, v)| *v).fold(0.0, f64::max).max(1024.0);
    let rx_dataset = Dataset::default().name("RX").marker(ratatui::symbols::Marker::Braille).graph_type(GraphType::Line).style(Style::default().fg(Color::Yellow)).data(&app.net_rx_history);
    f.render_widget(Chart::new(vec![rx_dataset]).block(Block::default().borders(Borders::ALL).title(format!(" Total Network RX (Max: {}/s) ", format_bytes(max_rx as u64)))).x_axis(Axis::default().bounds(x_bounds)).y_axis(Axis::default().bounds([0.0, max_rx])), bottom_chunks[0]);

    // 4. Network TX Chart
    let max_tx = app.net_tx_history.iter().map(|(_, v)| *v).fold(0.0, f64::max).max(1024.0);
    let tx_dataset = Dataset::default().name("TX").marker(ratatui::symbols::Marker::Braille).graph_type(GraphType::Line).style(Style::default().fg(Color::Blue)).data(&app.net_tx_history);
    f.render_widget(Chart::new(vec![tx_dataset]).block(Block::default().borders(Borders::ALL).title(format!(" Total Network TX (Max: {}/s) ", format_bytes(max_tx as u64)))).x_axis(Axis::default().bounds(x_bounds)).y_axis(Axis::default().bounds([0.0, max_tx])), bottom_chunks[1]);
}
