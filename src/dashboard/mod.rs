use crate::os::detector::DetectedSystem;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
};
use std::io;
use std::time::{Duration, Instant};

mod app;
mod ui;

use app::App;

pub fn run(system: &DetectedSystem) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let tick_rate = Duration::from_millis(1000);
    let app = App::new(system);
    let res = run_app(&mut terminal, app, tick_rate);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend<Error = io::Error>>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Tab | KeyCode::Right => app.next_tab(),
                KeyCode::BackTab | KeyCode::Left => app.prev_tab(),

                // Numeric keys for tree depth (1-9), 0 for infinite
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    if let Some(mut depth) = c.to_digit(10) {
                        if depth == 0 {
                            depth = 99; // Infinite-ish depth
                        }
                        app.tree_expansion_depth = depth;
                        app.use_tree_view = true;
                    }
                }
                KeyCode::Char('t') => app.use_tree_view = !app.use_tree_view,

                KeyCode::Up | KeyCode::Char('k') => app.on_up(),
                KeyCode::Down | KeyCode::Char('j') => app.on_down(),
                KeyCode::PageUp => app.on_page_up(),
                KeyCode::PageDown => app.on_page_down(),

                KeyCode::Char('p') => {
                    app.process_sort = crate::dashboard::app::ProcessSort::Pid;
                    app.sort_descending = !app.sort_descending;
                }
                KeyCode::Char('c') => {
                    app.process_sort = crate::dashboard::app::ProcessSort::Cpu;
                    app.sort_descending = !app.sort_descending;
                }
                KeyCode::Char('m') => {
                    app.process_sort = crate::dashboard::app::ProcessSort::Mem;
                    app.sort_descending = !app.sort_descending;
                }
                KeyCode::Char('n') => {
                    app.process_sort = crate::dashboard::app::ProcessSort::Name;
                    app.sort_descending = !app.sort_descending;
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}
