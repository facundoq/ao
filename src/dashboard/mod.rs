use crate::config::Config;
use crate::os::detector::DetectedSystem;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::time::{Duration, Instant};

pub mod app;
mod charts;
mod network;
mod overview;
mod processes;
pub mod rss;
mod sensors;
mod services;
mod storage;
pub mod ui;
mod users;
pub mod utils;
mod virtualization;

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
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    <B as ratatui::backend::Backend>::Error: std::error::Error + Send + Sync + 'static,
{
    let mut last_tick = Instant::now();
    loop {
        terminal
            .draw(|f| ui::draw(f, app))
            .map_err(|e| anyhow::anyhow!(e))?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            if let Some(ref mut details) = app.process_details {
                if details.is_searching {
                    match key.code {
                        KeyCode::Enter | KeyCode::Esc => {
                            details.is_searching = false;
                        }
                        KeyCode::Backspace => {
                            details.filter.pop();
                            app.update_details_filter();
                        }
                        KeyCode::Char(c) => {
                            details.filter.push(c);
                            app.update_details_filter();
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Esc => {
                            app.process_details = None;
                        }
                        KeyCode::Char('/') => {
                            details.is_searching = true;
                        }
                        KeyCode::Up | KeyCode::Char('k') => app.details_on_up(),
                        KeyCode::Down | KeyCode::Char('j') => app.details_on_down(),
                        _ => {}
                    }
                }
            } else if app.is_filtering {
                match key.code {
                    KeyCode::Enter | KeyCode::Esc => {
                        app.is_filtering = false;
                        app.refresh_process_data(true);
                        // Save config on filter exit
                        let mut config = Config::load().unwrap_or_default();
                        config.ui.process_filter = app.process_filter.clone();
                        let _ = config.save();
                    }
                    KeyCode::Backspace => {
                        app.process_filter.pop();
                        app.refresh_process_data(true);
                    }
                    KeyCode::Char(c) => {
                        app.process_filter.push(c);
                        app.refresh_process_data(true);
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Esc => {
                        app.process_details = None;
                    }
                    KeyCode::Enter if app.tab_index == 1 => {
                        app.fetch_process_details();
                    }
                    KeyCode::Char('/') if app.tab_index == 1 => {
                        app.is_filtering = true;
                    }
                    KeyCode::Tab | KeyCode::Right => app.next_tab(),
                    KeyCode::BackTab | KeyCode::Left => app.prev_tab(),

                    KeyCode::Char('t') => {
                        app.use_tree_view = !app.use_tree_view;
                        app.refresh_process_data(true);
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        app.tree_expansion_depth = c.to_digit(10).unwrap();
                        app.use_tree_view = true;
                        app.refresh_process_data(true);
                    }
                    KeyCode::Char('o') => {
                        app.show_only_current_user = !app.show_only_current_user;
                        app.refresh_process_data(true);
                    }
                    KeyCode::Char('H') | KeyCode::Char('h') => {
                        app.show_user_threads = !app.show_user_threads;
                        app.refresh_process_data(true);
                    }
                    KeyCode::Char('k') => {
                        if app.tab_index == 2 {
                            app.hide_kernel_processes = !app.hide_kernel_processes;
                            app.refresh_process_data(true);
                        } else {
                            app.on_up();
                        }
                    }

                    KeyCode::Up => app.on_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.on_down(),
                    KeyCode::PageUp => app.on_page_up(),
                    KeyCode::PageDown => app.on_page_down(),

                    KeyCode::Char('i') => {
                        app.process_sort = app::ProcessSort::Pid;
                        app.sort_descending = !app.sort_descending;
                        app.refresh_process_data(true);
                    }
                    KeyCode::Char('c') => {
                        app.process_sort = app::ProcessSort::Cpu;
                        app.sort_descending = !app.sort_descending;
                        app.refresh_process_data(true);
                    }
                    KeyCode::Char('m') => {
                        app.process_sort = app::ProcessSort::Mem;
                        app.sort_descending = !app.sort_descending;
                        app.refresh_process_data(true);
                    }
                    KeyCode::Char('n') => {
                        app.process_sort = app::ProcessSort::Name;
                        app.sort_descending = !app.sort_descending;
                        app.refresh_process_data(true);
                    }
                    KeyCode::Char('u') => {
                        app.process_sort = app::ProcessSort::User;
                        app.sort_descending = !app.sort_descending;
                        app.refresh_process_data(true);
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        app.tick_rate = app.tick_rate.saturating_add(Duration::from_millis(200));
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        let new_rate = app.tick_rate.saturating_sub(Duration::from_millis(200));
                        if new_rate >= Duration::from_millis(200) {
                            app.tick_rate = new_rate;
                        } else {
                            app.tick_rate = Duration::from_millis(200);
                        }
                    }
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= app.tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}
