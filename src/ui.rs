use crate::app::{App, AppState, PanelId};
use crate::utils::helpers::{format_bytes, percentage_bar};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Main layout: top bar + panels
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Top bar
            Constraint::Min(0),   // Panels
        ])
        .split(size);

    // Top bar
    let bar_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(chunks[0]);

    let title = Paragraph::new(" GLANCE")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM));

    let time = Paragraph::new(app.time_display())
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::BOTTOM));

    let nav = Paragraph::new(format!(
        "[{}] [{}] [Ctrl+Q]",
        panel_label(app.current_panel),
        if app.state == AppState::EditingConfig {
            "ESC"
        } else {
            "ENTER"
        }
    ))
    .style(Style::default().fg(Color::Yellow))
    .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(title, bar_chunks[0]);
    frame.render_widget(time, bar_chunks[1]);
    frame.render_widget(nav, bar_chunks[2]);

    // Panel layout
    let panel_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(50),
        ])
        .split(chunks[1]);

    frame.render_widget(Clear, panel_chunks[0]);
    frame.render_widget(
        weather_panel(app),
        panel_chunks[0].inner(Margin {
            horizontal: 1,
            vertical: 0,
        }),
    );

    frame.render_widget(Clear, panel_chunks[1]);
    frame.render_widget(
        news_panel(app),
        panel_chunks[1].inner(Margin {
            horizontal: 1,
            vertical: 0,
        }),
    );

    frame.render_widget(Clear, panel_chunks[2]);
    frame.render_widget(
        system_panel(app),
        panel_chunks[2].inner(Margin {
            horizontal: 1,
            vertical: 0,
        }),
    );
}

fn panel_label(id: PanelId) -> &'static str {
    match id {
        PanelId::Weather => "WEATHER",
        PanelId::News => "NEWS",
        PanelId::System => "SYSTEM",
    }
}

fn weather_panel<'a>(app: &'a App) -> Paragraph<'a> {
    let weather = &app.weather_data;
    let mut lines: Vec<Line> = vec![
        Line::raw(format!("{} {}°{}  ", weather.icon, weather.temp, weather.unit)),
        Line::raw(format!("{}  ", weather.condition)),
    ];

    if !weather.humidity.is_empty() {
        lines.push(Line::raw(format!("Humidity: {}%", weather.humidity)));
    }
    if !weather.wind.is_empty() {
        lines.push(Line::raw(format!("Wind: {}", weather.wind)));
    }

    Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(Block::default().title("WEATHER").borders(Borders::ALL))
        .style(Style::default().fg(Color::Blue))
}

fn news_panel<'a>(app: &'a App) -> Paragraph<'a> {
    let news = &app.news_data;
    let mut lines = Vec::new();

    if news.headlines.is_empty() {
        lines.push(Line::raw("No news available"));
    } else {
        for (i, headline) in news.headlines.iter().take(5).enumerate() {
            if i > 0 {
                lines.push(Line::raw(""));
            }
            lines.push(Line::styled(
                headline.title.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            ));
            if !headline.summary.is_empty() {
                let summary: String = headline.summary.chars().take(60).collect();
                lines.push(Line::raw(format!("{}...", summary)));
            }
        }
    }

    Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(Block::default().title("NEWS").borders(Borders::ALL))
        .style(Style::default().fg(Color::Green))
}

fn system_panel<'a>(app: &'a App) -> Paragraph<'a> {
    let sys = &app.system;
    let bar_width = 20;
    let mut lines = Vec::new();

    // CPU
    let cpu = sys.cpu_usage();
    lines.push(Line::raw(format!(
        "CPU  {} {:.1}%",
        percentage_bar(cpu, bar_width),
        cpu
    )));

    // RAM
    let mem_pct = sys.memory_usage_pct();
    let used_mem = sys.total_memory() - sys.available_memory();
    lines.push(Line::raw(format!(
        "RAM  {} {:.1}%  ({}/{})",
        percentage_bar(mem_pct, bar_width),
        mem_pct,
        format_bytes(used_mem),
        format_bytes(sys.total_memory())
    )));

    // Disk
    let disk_pct = sys.disk_usage_pct();
    lines.push(Line::raw(format!(
        "DISK {} {:.1}%",
        percentage_bar(disk_pct, bar_width),
        disk_pct
    )));

    // Per-disk breakdown
    for disk in sys.disk_info() {
        let used = disk.total - disk.available;
        lines.push(Line::raw(format!(
            "  {} {}/{}",
            disk.mount_point,
            format_bytes(used),
            format_bytes(disk.total)
        )));
    }

    // Network
    lines.push(Line::raw(""));
    lines.push(Line::raw(format!(
        "NET  rx: {}  tx: {}",
        format_bytes(sys.network_received()),
        format_bytes(sys.network_transmitted())
    )));

    Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(Block::default().title("SYSTEM").borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow))
}
