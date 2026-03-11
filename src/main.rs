mod app;
mod browser;
mod config;
mod digest;
mod icons;
mod layout;
mod location;
mod news;
mod system;
mod theme;
mod ui;
mod utils;
mod weather;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::time::{Duration, Instant};

use app::{App, AppState};
use ui::render;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Setup app and load initial data (unless location search is needed first)
    let mut app = App::new()?;
    if app.state != AppState::LocationSearch {
        app.load_data().await;
    }

    // Main loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    let mut last_data_refresh = Instant::now();
    let data_refresh_interval = Duration::from_secs(300); // Refresh weather/news every 5 min
    let mut location_debounce: Option<Instant> = None;

    loop {
        // Draw UI
        terminal.draw(|f| render(f, app))?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if !handle_key(app, key.code, key.modifiers, &mut location_debounce).await {
                    break;
                }

                // If location was just confirmed, fetch weather/news immediately
                // (handled inside handle_key via confirm_location + load_data)
            }
        }

        // Location search debouncing: fetch results after 300ms of inactivity
        if let Some(debounce_start) = location_debounce {
            if debounce_start.elapsed() >= Duration::from_millis(300) {
                if let Some(ref mut ls) = app.location_search {
                    ls.fetch().await;
                }
                location_debounce = None;
            }
        }

        // Update system metrics every tick
        app.update_metrics();

        // Periodically refresh weather/news
        if last_data_refresh.elapsed() >= data_refresh_interval {
            app.load_data().await;
            last_data_refresh = Instant::now();
        }
    }

    Ok(())
}

/// Returns false to signal quit.
async fn handle_key(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
    location_debounce: &mut Option<Instant>,
) -> bool {
    // Ctrl+Q always quits
    if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('q') {
        return false;
    }

    match app.state {
        AppState::LocationSearch => {
            handle_location_key(app, code, location_debounce).await
        }
        AppState::Help => {
            match code {
                KeyCode::Esc | KeyCode::Char('?') => app.toggle_help(),
                KeyCode::Enter => app.show_help_modal(),
                KeyCode::Char('q') => return false,
                _ => {}
            }
            true
        }
        AppState::HelpModal => {
            match code {
                KeyCode::Esc | KeyCode::Char('?') => app.toggle_help(),
                _ => {}
            }
            true
        }
        AppState::LoadingArticle => {
            // Ignore inputs while loading
            true
        }
        AppState::ReadingArticle { .. } => {
            match code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char(' ') => app.state = AppState::Running,
                KeyCode::Char('j') | KeyCode::Down => {
                    if let AppState::ReadingArticle { scroll: s, title, content, url } = &app.state {
                        app.state = AppState::ReadingArticle { scroll: s.saturating_add(1), title: title.clone(), content: content.clone(), url: url.clone() };
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if let AppState::ReadingArticle { scroll: s, title, content, url } = &app.state {
                        app.state = AppState::ReadingArticle { scroll: s.saturating_sub(1), title: title.clone(), content: content.clone(), url: url.clone() };
                    }
                }
                KeyCode::Enter => {
                    if let AppState::ReadingArticle { url, .. } = &app.state {
                        let _ = browser::open_url(url);
                    }
                }
                _ => {}
            }
            true
        }
        _ => {
            // Normal mode (Running, LoadingWeather, LoadingNews, EditingConfig)
            match code {
                KeyCode::Char('q') => return false,
                KeyCode::Tab => app.cycle_panels(),
                KeyCode::BackTab => app.cycle_panels_back(),
                KeyCode::Char('j') | KeyCode::Down => app.move_down(),
                KeyCode::Char('k') | KeyCode::Up => app.move_up(),
                KeyCode::Char('o') => {
                    if let Some(url) = app.selected_headline_url() {
                        let url = url.to_string();
                        let _ = browser::open_url(&url);
                    }
                }
                KeyCode::Char(' ') | KeyCode::Enter => {
                    if app.current_panel == app::PanelId::News {
                        if let Some((title, url)) = app.selected_headline() {
                            let title = title.to_string();
                            let url = url.to_string();
                            // We need to spawn a background task so we don't block the UI
                            // But since `app.load_article` is async and takes `&mut app`, 
                            // we'll set the state to LoadingArticle here, and we need a way to 
                            // process the request.
                            // Actually, let's just use `tokio::spawn` here and use an mpsc channel 
                            // or something. Wait! `handle_key` is async! We can just `await` it.
                            app.load_article(&url, &title).await;
                        }
                    }
                }
                KeyCode::Char('l' | 'L') => app.cycle_layout(),
                KeyCode::Char('t' | 'T') => app.cycle_theme(),
                KeyCode::Char('/') => app.start_location_search(),
                KeyCode::Char('?') => app.toggle_help(),
                KeyCode::Char('m') => app.toggle_ampm(),
                KeyCode::Char('z') => app.toggle_utc(),
                KeyCode::Char('p' | 'P') => app.toggle_processes(),
                _ => {}
            }
            true
        }
    }
}

/// Handle keys while in location search mode. Returns false to quit.
async fn handle_location_key(
    app: &mut App,
    code: KeyCode,
    location_debounce: &mut Option<Instant>,
) -> bool {
    match code {
        KeyCode::Esc => {
            app.cancel_location_search();
            *location_debounce = None;
        }
        KeyCode::Enter => {
            if app.confirm_location() {
                *location_debounce = None;
                app.load_data().await;
            }
        }
        KeyCode::Backspace => {
            if let Some(ref mut ls) = app.location_search {
                ls.pop_char();
                *location_debounce = Some(Instant::now());
            }
        }
        KeyCode::Up => {
            if let Some(ref mut ls) = app.location_search {
                ls.move_up();
            }
        }
        KeyCode::Down => {
            if let Some(ref mut ls) = app.location_search {
                ls.move_down();
            }
        }
        KeyCode::Char(c) => {
            if let Some(ref mut ls) = app.location_search {
                ls.push_char(c);
                *location_debounce = Some(Instant::now());
            }
        }
        _ => {}
    }
    true
}
