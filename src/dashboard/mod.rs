use crate::os::detector::DetectedSystem;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};

pub mod app;
mod charts;
mod storage;
mod network;
mod overview;
mod processes;
mod sensors;
mod services;
mod ui;
mod users;
pub mod utils;
mod virtualization;
pub mod rss;

use app::App;

pub fn run(system: &DetectedSystem) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(system);
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    res
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> 
where
    <B as ratatui::backend::Backend>::Error: std::error::Error + Send + Sync + 'static 
{
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui::draw(f, app)).map_err(|e| anyhow::anyhow!(e))?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Tab | KeyCode::Right => app.next_tab(),
                    KeyCode::BackTab | KeyCode::Left => app.prev_tab(),
                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.tab_index == 2 {
                             app.hide_kernel_processes = !app.hide_kernel_processes;
                        } else {
                             app.on_up();
                        }
                    },
                    KeyCode::Down | KeyCode::Char('j') => app.on_down(),
                    KeyCode::PageUp => app.on_page_up(),
                    KeyCode::PageDown => app.on_page_down(),
                    KeyCode::Char('i') => {
                        app.process_sort = app::ProcessSort::Pid;
                        app.sort_descending = !app.sort_descending;
                    }
                    KeyCode::Char('c') => {
                        app.process_sort = app::ProcessSort::Cpu;
                        app.sort_descending = !app.sort_descending;
                    }
                    KeyCode::Char('m') => {
                        app.process_sort = app::ProcessSort::Mem;
                        app.sort_descending = !app.sort_descending;
                    }
                    KeyCode::Char('n') => {
                        app.process_sort = app::ProcessSort::Name;
                        app.sort_descending = !app.sort_descending;
                    }
                    KeyCode::Char('u') => {
                        app.process_sort = app::ProcessSort::User;
                        app.sort_descending = !app.sort_descending;
                    }
                    KeyCode::Char('o') => app.show_only_current_user = !app.show_only_current_user,
                    _ => {}
                }
            }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}
