use crate::dashboard::app::App;
use crate::dashboard::utils::format_bytes;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let x_bounds = if app.cpu_history.is_empty() {
        [0.0, 60.0]
    } else {
        [
            app.cpu_history[0].0,
            app.cpu_history
                .last()
                .unwrap()
                .0
                .max(app.cpu_history[0].0 + 60.0),
        ]
    };

    // 1. Unified System Resources Chart (CPU, MEM, SWAP)
    let cpu_dataset = Dataset::default()
        .name("CPU")
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Rgb(255, 200, 150)))
        .data(&app.cpu_history);

    let mem_dataset = Dataset::default()
        .name("Memory")
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Green))
        .data(&app.mem_history);

    let swap_dataset = Dataset::default()
        .name("Swap")
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Magenta))
        .data(&app.swap_history);

    let system_chart = Chart::new(vec![cpu_dataset, mem_dataset, swap_dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" System Resources (%) "),
        )
        .x_axis(Axis::default().bounds(x_bounds))
        .y_axis(Axis::default().bounds([0.0, 100.0]).labels(vec![
            Span::raw("0"),
            Span::raw("50"),
            Span::raw("100"),
        ]));
    f.render_widget(system_chart, chunks[0]);

    // 2. Unified Network Chart (RX, TX)
    let max_rx = app
        .net_rx_history
        .iter()
        .map(|(_, v)| *v)
        .fold(0.0, f64::max);
    let max_tx = app
        .net_tx_history
        .iter()
        .map(|(_, v)| *v)
        .fold(0.0, f64::max);
    let dynamic_max = max_rx.max(max_tx).max(1024.0); // At least 1KB/s

    let rx_dataset = Dataset::default()
        .name("RX")
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Yellow))
        .data(&app.net_rx_history);

    let tx_dataset = Dataset::default()
        .name("TX")
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Blue))
        .data(&app.net_tx_history);

    let network_chart = Chart::new(vec![rx_dataset, tx_dataset])
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Network Throughput (Max: {}/s) ",
            format_bytes(dynamic_max as u64)
        )))
        .x_axis(Axis::default().bounds(x_bounds))
        .y_axis(Axis::default().bounds([0.0, dynamic_max]).labels(vec![
            Span::raw("0"),
            Span::raw(format_bytes((dynamic_max / 2.0) as u64)),
            Span::raw(format_bytes(dynamic_max as u64)),
        ]));
    f.render_widget(network_chart, chunks[1]);
}
