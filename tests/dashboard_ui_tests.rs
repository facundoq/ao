use ao_cli::dashboard::app::App;
use ao_cli::os::detector::detect_system;
use ratatui::{Terminal, backend::TestBackend};

#[test]
fn test_dashboard_rendering() {
    let system = Box::leak(Box::new(detect_system().unwrap()));
    let mut app = App::new(system);
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    // Test Overview Tab
    app.tab_index = 0;
    terminal
        .draw(|f| ao_cli::dashboard::ui::draw(f, &mut app))
        .unwrap();
    let content = format!("{:?}", terminal.backend().buffer());
    assert!(content.contains("Overview"));
    assert!(content.contains("ao dashboard"));

    // Test Process Tab
    app.tab_index = 1;
    terminal
        .draw(|f| ao_cli::dashboard::ui::draw(f, &mut app))
        .unwrap();
    let content = format!("{:?}", terminal.backend().buffer());
    assert!(content.contains("Process"));
    // Since we are in the process tab, it should show headers
    assert!(content.contains("PID"));
    assert!(content.contains("CPU%"));

    // Test Storage Tab
    app.tab_index = 2;
    terminal
        .draw(|f| ao_cli::dashboard::ui::draw(f, &mut app))
        .unwrap();
    let content = format!("{:?}", terminal.backend().buffer());
    assert!(content.contains("Storage"));

    // Test User Tab
    app.tab_index = 3;
    terminal
        .draw(|f| ao_cli::dashboard::ui::draw(f, &mut app))
        .unwrap();
    let content = format!("{:?}", terminal.backend().buffer());
    assert!(content.contains("User"));

    // Test Network Tab
    app.tab_index = 4;
    terminal
        .draw(|f| ao_cli::dashboard::ui::draw(f, &mut app))
        .unwrap();
    let content = format!("{:?}", terminal.backend().buffer());
    // It might show "Loading..." if interfaces is empty
    assert!(content.contains("Network") || content.contains("Loading..."));

    // Test Service Tab
    app.tab_index = 5;
    terminal
        .draw(|f| ao_cli::dashboard::ui::draw(f, &mut app))
        .unwrap();
    let content = format!("{:?}", terminal.backend().buffer());
    assert!(content.contains("Service") || content.contains("Loading..."));

    // Test Virtualization Tab
    app.tab_index = 6;
    terminal
        .draw(|f| ao_cli::dashboard::ui::draw(f, &mut app))
        .unwrap();
    let content = format!("{:?}", terminal.backend().buffer());
    assert!(content.contains("Virtualization") || content.contains("Loading..."));

    // Test Sensors Tab
    app.tab_index = 7;
    terminal
        .draw(|f| ao_cli::dashboard::ui::draw(f, &mut app))
        .unwrap();
    let content = format!("{:?}", terminal.backend().buffer());
    assert!(content.contains("Sensors") || content.contains("Loading..."));

    // Test Charts Tab
    app.tab_index = 8;
    app.interfaces = vec![ao_cli::os::NetInterfaceInfo {
        name: "eth0".to_string(),
        state: "up".to_string(),
        mtu: 1500,
        mac: "00:11:22:33:44:55".to_string(),
        interface_type: "Ethernet".to_string(),
        ips: vec!["192.168.1.2".to_string()],
    }];
    app.selected_index = 0;
    app.network_speeds
        .insert("eth0".to_string(), (128000, 31457280)); // 125 KB/s, 30 MB/s
    app.net_history_per_adapter.insert(
        "eth0".to_string(),
        (vec![(0.0, 128000.0)], vec![(0.0, 31457280.0)]),
    );
    app.cpu_history = vec![(0.0, 42.5)];
    app.mem_history = vec![(0.0, 68.1)];
    app.swap_history = vec![(0.0, 5.0)];

    terminal
        .draw(|f| ao_cli::dashboard::ui::draw(f, &mut app))
        .unwrap();
    let content = format!("{:?}", terminal.backend().buffer());
    assert!(content.contains("Charts") || content.contains("System Resources"));
    assert!(content.contains("RX:     125.0 KB/s"));
    assert!(content.contains("TX:      30.0 MB/s"));
    assert!(content.contains("CPU:  (42.5%)"));
    assert!(content.contains("RAM:  (68.1%)"));
    assert!(content.contains("Swap: ( 5.0%)"));
}
