use ao_cli::dashboard::app::App;
use ao_cli::os::detector::detect_system;
use ratatui::{Terminal, backend::TestBackend};

#[test]
fn test_dashboard_rendering() {
    let system = Box::leak(Box::new(detect_system().unwrap()));
    let mut app = App::new(system);
    let backend = TestBackend::new(80, 24);
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
    terminal
        .draw(|f| ao_cli::dashboard::ui::draw(f, &mut app))
        .unwrap();
    let content = format!("{:?}", terminal.backend().buffer());
    assert!(content.contains("Charts") || content.contains("System Resources"));
}
