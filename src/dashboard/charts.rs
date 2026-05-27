use crate::dashboard::app::App;
use crate::dashboard::utils::format_bytes;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Axis, Block, Borders, Cell, Chart, Dataset, GraphType, LegendPosition, Row, Table},
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(area);

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(main_chunks[0]);

    let charts_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(horizontal_chunks[0]);

    let now_sec = app.history_start.elapsed().as_secs_f64();
    let x_bounds = [now_sec - 300.0, now_sec];

    // 1. Unified System Resources Chart (CPU, MEM, SWAP)
    let cpu_dataset = Dataset::default()
        .name("CPU")
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Rgb(
            app.theme.cpu_color[0],
            app.theme.cpu_color[1],
            app.theme.cpu_color[2],
        )))
        .data(&app.cpu_history);

    let mem_dataset = Dataset::default()
        .name("Memory")
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Rgb(
            app.theme.ram_color[0],
            app.theme.ram_color[1],
            app.theme.ram_color[2],
        )))
        .data(&app.mem_history);

    let swap_dataset = Dataset::default()
        .name("Swap")
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Rgb(
            app.theme.swap_color[0],
            app.theme.swap_color[1],
            app.theme.swap_color[2],
        )))
        .data(&app.swap_history);

    let system_chart = Chart::new(vec![cpu_dataset, mem_dataset, swap_dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" System Resources (%) "),
        )
        .legend_position(Some(LegendPosition::TopRight))
        .hidden_legend_constraints((Constraint::Min(0), Constraint::Min(0)))
        .x_axis(Axis::default().bounds(x_bounds).labels(vec![
            Span::raw("-5m"),
            Span::raw("-4m"),
            Span::raw("-3m"),
            Span::raw("-2m"),
            Span::raw("-1m"),
            Span::raw("0"),
        ]))
        .y_axis(Axis::default().bounds([0.0, 100.0]).labels(vec![
            Span::raw("0"),
            Span::raw("50"),
            Span::raw("100"),
        ]));
    f.render_widget(system_chart, charts_chunks[0]);

    let selected_interface_name = if !app.interfaces.is_empty() {
        app.interfaces[app.selected_index.min(app.interfaces.len() - 1)]
            .name
            .clone()
    } else {
        "".to_string()
    };

    let empty_history = (Vec::new(), Vec::new());
    let (rx_history, tx_history) = app
        .net_history_per_adapter
        .get(&selected_interface_name)
        .unwrap_or(&empty_history);

    // 2. Unified Network Chart (RX, TX)
    let max_rx = rx_history.iter().map(|(_, v)| *v).fold(0.0, f64::max);
    let max_tx = tx_history.iter().map(|(_, v)| *v).fold(0.0, f64::max);
    let max_rx_remembered = app
        .max_rx_throughput
        .get(&selected_interface_name)
        .copied()
        .unwrap_or(0);
    let max_tx_remembered = app
        .max_tx_throughput
        .get(&selected_interface_name)
        .copied()
        .unwrap_or(0);
    let dynamic_max = max_rx
        .max(max_tx)
        .max(max_rx_remembered as f64)
        .max(max_tx_remembered as f64); // Remembered peaks

    let rx_dataset = Dataset::default()
        .name("RX")
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Yellow))
        .data(rx_history);

    let tx_dataset = Dataset::default()
        .name("TX")
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Blue))
        .data(tx_history);

    let network_chart = Chart::new(vec![rx_dataset, tx_dataset])
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Network Throughput: {} (Max: {}/s) ",
            selected_interface_name,
            format_bytes(dynamic_max as u64)
        )))
        .legend_position(Some(LegendPosition::TopRight))
        .hidden_legend_constraints((Constraint::Min(0), Constraint::Min(0)))
        .x_axis(Axis::default().bounds(x_bounds).labels(vec![
            Span::raw("-5m"),
            Span::raw("-4m"),
            Span::raw("-3m"),
            Span::raw("-2m"),
            Span::raw("-1m"),
            Span::raw("0"),
        ]))
        .y_axis(Axis::default().bounds([0.0, dynamic_max]).labels(vec![
            Span::raw("0"),
            Span::raw(format_bytes((dynamic_max / 2.0) as u64)),
            Span::raw(format_bytes(dynamic_max as u64)),
        ]));
    f.render_widget(network_chart, charts_chunks[1]);

    // 3. Network Adapters List
    let rows = app.interfaces.iter().enumerate().map(|(idx, i)| {
        let mut style = Style::default();
        if idx == app.selected_index {
            style = style
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD);
        } else if i.state.to_lowercase() == "up" {
            style = style.fg(Color::Green);
        } else {
            style = style.fg(Color::DarkGray);
        }
        Row::new(vec![
            Cell::from(i.name.clone()),
            Cell::from(i.state.clone()),
        ])
        .style(style)
    });

    let adapters_table = Table::new(
        rows,
        [Constraint::Percentage(70), Constraint::Percentage(30)],
    )
    .header(Row::new(vec!["Adapter", "State"]).style(Style::default().add_modifier(Modifier::BOLD)))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Network Adapters "),
    );

    f.render_widget(adapters_table, horizontal_chunks[1]);
}
