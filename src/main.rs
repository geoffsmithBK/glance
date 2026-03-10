mod app;
mod browser;
mod config;
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
use std::time::Duration;

use app::App;
use ui::render;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Setup app and load initial data
    let mut app = App::new()?;
    app.load_data().await;

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
    let mut last_data_refresh = std::time::Instant::now();
    let data_refresh_interval = Duration::from_secs(300); // Refresh weather/news every 5 min

    loop {
        // Draw UI
        terminal.draw(|f| render(f, app))?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match (key.modifiers, key.code) {
                    (KeyModifiers::CONTROL, KeyCode::Char('q')) => break,
                    (_, KeyCode::Char('q')) => break,
                    (_, KeyCode::Tab) => app.cycle_panels(),
                    (_, KeyCode::Enter) => app.toggle_config(),
                    _ => {}
                }
            }
        }

        // Update system metrics every tick
        app.update_metrics();

        // Periodically refresh weather/news
        if last_data_refresh.elapsed() >= data_refresh_interval {
            app.load_data().await;
            last_data_refresh = std::time::Instant::now();
        }
    }

    Ok(())
}
