use crate::app::{App, AppState, PanelId};
use crate::layout::LayoutMode;
use crate::utils::helpers::{format_bytes, percentage_bar};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Sparkline, Table, Row, Cell, Wrap},
    Frame,
};

/// Main render entry point. Called each frame from the event loop.
pub fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Update layout based on current terminal size
    app.update_layout(size.width, size.height);

    let colors = app.theme.colors();

    // Set background color if theme specifies one
    if let Some(bg) = colors.bg {
        let bg_block = Block::default().style(Style::default().bg(bg));
        frame.render_widget(bg_block, size);
    }

    // 3 vertical chunks: top bar, panels, status bar
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    render_top_bar(frame, app, outer[0]);
    render_panels(frame, app, outer[1]);
    render_status_bar(frame, app, outer[2]);

    // Overlays on top
    match &app.state {
        AppState::LocationSearch => render_location_overlay(frame, app, size),
        AppState::Help => render_help_overlay(frame, app, size),
        _ => {}
    }
}

/// Render the top bar: "GLANCE" title + time + layout/theme info.
fn render_top_bar(frame: &mut Frame, app: &App, area: Rect) {
    let colors = app.theme.colors();

    let bar_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(area);

    let mut title_style = Style::default()
        .fg(colors.title)
        .add_modifier(Modifier::BOLD);
    if let Some(bg) = colors.bg {
        title_style = title_style.bg(bg);
    }

    let title = Paragraph::new(Line::from(vec![
        Span::styled(" GLANCE ", title_style),
    ]))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(colors.panel_border)),
    );

    let mut time_style = Style::default()
        .fg(colors.title)
        .add_modifier(Modifier::BOLD);
    if let Some(bg) = colors.bg {
        time_style = time_style.bg(bg);
    }
    let time = Paragraph::new(Line::from(vec![
        Span::styled(app.time_display(), time_style),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(colors.panel_border)),
    );

    let layout_info = format!(
        "{} {} {} ",
        app.icons.separator,
        app.layout.name(),
        app.theme.name()
    );
    let mut info_style = Style::default().fg(colors.dim);
    if let Some(bg) = colors.bg {
        info_style = info_style.bg(bg);
    }
    let info = Paragraph::new(Line::from(vec![Span::styled(layout_info, info_style)]))
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(colors.panel_border)),
        );

    frame.render_widget(title, bar_chunks[0]);
    frame.render_widget(time, bar_chunks[1]);
    frame.render_widget(info, bar_chunks[2]);
}

/// Dispatch to layout-specific panel rendering.
fn render_panels(frame: &mut Frame, app: &App, area: Rect) {
    match app.layout {
        LayoutMode::Wide => render_wide(frame, app, area),
        LayoutMode::Compact => render_compact(frame, app, area),
        LayoutMode::Tall => render_tall(frame, app, area),
        LayoutMode::Minimal => render_minimal(frame, app, area),
    }
}

/// Wide layout: 3 panels horizontal — 25% | 35% | 40%.
fn render_wide(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(35),
            Constraint::Percentage(40),
        ])
        .split(area);

    render_weather_panel(frame, app, chunks[0]);
    render_news_panel(frame, app, chunks[1]);
    render_system_panel(frame, app, chunks[2]);
}

/// Compact layout: 2 rows — [Weather 40% | News 60%] top, [System 100%] bottom.
fn render_compact(frame: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(rows[0]);

    render_weather_panel(frame, app, top[0]);
    render_news_panel(frame, app, top[1]);
    render_system_panel(frame, app, rows[1]);
}

/// Tall layout: 3 panels vertical — 20% | 40% | 40%.
fn render_tall(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(40),
            Constraint::Percentage(40),
        ])
        .split(area);

    render_weather_panel(frame, app, chunks[0]);
    render_news_panel(frame, app, chunks[1]);
    render_system_panel(frame, app, chunks[2]);
}

/// Minimal layout: shows only the current panel.
fn render_minimal(frame: &mut Frame, app: &App, area: Rect) {
    match app.current_panel {
        PanelId::Weather => render_weather_panel(frame, app, area),
        PanelId::News => render_news_panel(frame, app, area),
        PanelId::System => render_system_panel(frame, app, area),
    }
}

/// Create a themed panel border block.
/// Active panel gets `active_border` color + bold title, inactive gets `panel_border`.
fn panel_block<'a>(app: &'a App, panel: PanelId, title: &'a str) -> Block<'a> {
    let colors = app.theme.colors();
    let is_active = app.current_panel == panel;

    let (border_color, title_modifier) = if is_active {
        (colors.active_border, Modifier::BOLD)
    } else {
        (colors.panel_border, Modifier::empty())
    };

    let mut block = Block::default()
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(border_color).add_modifier(title_modifier),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    if let Some(bg) = colors.bg {
        block = block.style(Style::default().bg(bg));
    }

    block
}

/// Format a "HH:MM" time string into 12h (h:MM AM/PM) or leave as-is for 24h.
fn fmt_time(time_str: &str, use_12h: bool) -> String {
    if !use_12h || time_str.is_empty() {
        return time_str.to_string();
    }
    let mut parts = time_str.splitn(2, ':');
    if let (Some(h), Some(m)) = (parts.next(), parts.next()) {
        if let (Ok(h), Ok(m)) = (h.parse::<u32>(), m.parse::<u32>()) {
            let (period, h12) = match h {
                0 => ("AM", 12u32),
                1..=11 => ("AM", h),
                12 => ("PM", 12),
                _ => ("PM", h - 12),
            };
            return format!("{}:{:02} {}", h12, m, period);
        }
    }
    time_str.to_string()
}

/// Render the weather panel with themed colors.
fn render_weather_panel(frame: &mut Frame, app: &App, area: Rect) {
    let colors = app.theme.colors();
    let title = format!("{} Weather", app.icons.panel_weather);
    let block = panel_block(app, PanelId::Weather, &title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.config.weather.location.is_none() {
        let msg = Paragraph::new(Line::from(vec![Span::styled(
            "No location configured. Press / to search.",
            Style::default().fg(colors.dim),
        )]))
        .wrap(Wrap { trim: false });
        frame.render_widget(msg, inner);
        return;
    }

    let weather = &app.weather_data;

    // Build top section lines
    let mut top_lines: Vec<Line> = Vec::new();

    // Location name
    if let Some(ref name) = app.config.weather.location_name {
        top_lines.push(Line::from(vec![Span::styled(
            name.as_str(),
            Style::default().fg(colors.dim),
        )]));
    }

    // Temperature
    top_lines.push(Line::from(vec![Span::styled(
        format!("{} {}°{}", app.icons.weather_icon(weather.weather_code, weather.is_day), weather.temp, weather.unit),
        Style::default()
            .fg(colors.weather_accent)
            .add_modifier(Modifier::BOLD),
    )]));

    // Condition
    top_lines.push(Line::from(vec![Span::styled(
        &weather.condition,
        Style::default().fg(colors.fg.unwrap_or(Color::White)),
    )]));

    top_lines.push(Line::raw(""));

    // Humidity
    if !weather.humidity.is_empty() {
        top_lines.push(Line::from(vec![
            Span::styled("Humidity: ", Style::default().fg(colors.dim)),
            Span::styled(
                format!("{}%", weather.humidity),
                Style::default().fg(colors.fg.unwrap_or(Color::White)),
            ),
        ]));
    }

    // Wind
    if !weather.wind.is_empty() {
        top_lines.push(Line::from(vec![
            Span::styled("Wind: ", Style::default().fg(colors.dim)),
            Span::styled(
                &weather.wind,
                Style::default().fg(colors.fg.unwrap_or(Color::White)),
            ),
        ]));
    }

    // Sunrise / Sunset
    if !weather.sunrise.is_empty() || !weather.sunset.is_empty() {
        top_lines.push(Line::from(vec![
            Span::styled(
                format!("{}↑ {}", app.icons.weather_clear_day, fmt_time(&weather.sunrise, app.use_12h)),
                Style::default().fg(colors.weather_accent),
            ),
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("{}↓ {}", app.icons.weather_clear_day, fmt_time(&weather.sunset, app.use_12h)),
                Style::default().fg(colors.dim),
            ),
        ]));
    }

    // Day summary
    if !weather.day_summary.is_empty() {
        top_lines.push(Line::raw(""));
        // Word-wrap the summary manually into lines that fit the panel width
        let summary = &weather.day_summary;
        let max_width = inner.width as usize;
        let mut current_line = String::new();
        for word in summary.split_whitespace() {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                top_lines.push(Line::from(Span::styled(
                    current_line.clone(),
                    Style::default().fg(colors.fg.unwrap_or(Color::White)),
                )));
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            top_lines.push(Line::from(Span::styled(
                current_line,
                Style::default().fg(colors.fg.unwrap_or(Color::White)),
            )));
        }
    }

    // Build forecast lines (bottom-aligned)
    let mut forecast_lines: Vec<Line> = Vec::new();
    if !weather.forecast.is_empty() {
        let col_width = 5usize;
        let max_days = (inner.width as usize / col_width).min(weather.forecast.len()).min(7);

        if max_days > 0 {
            // Day names
            let day_spans: Vec<Span> = weather.forecast.iter().take(max_days).map(|d| {
                Span::styled(
                    format!("{:<width$}", d.date, width = col_width),
                    Style::default().fg(colors.fg.unwrap_or(Color::White)).add_modifier(Modifier::BOLD),
                )
            }).collect();
            forecast_lines.push(Line::from(day_spans));

            // Icons
            let icon_spans: Vec<Span> = weather.forecast.iter().take(max_days).map(|d| {
                Span::styled(
                    format!("{:<width$}", app.icons.weather_icon(d.weather_code, true), width = col_width),
                    Style::default().fg(colors.fg.unwrap_or(Color::White)),
                )
            }).collect();
            forecast_lines.push(Line::from(icon_spans));

            // High temps
            let high_spans: Vec<Span> = weather.forecast.iter().take(max_days).map(|d| {
                Span::styled(
                    format!("{:<width$}", format!("{:.0}°", d.temp_max), width = col_width),
                    Style::default().fg(colors.weather_accent),
                )
            }).collect();
            forecast_lines.push(Line::from(high_spans));

            // Low temps
            let low_spans: Vec<Span> = weather.forecast.iter().take(max_days).map(|d| {
                Span::styled(
                    format!("{:<width$}", format!("{:.0}°", d.temp_min), width = col_width),
                    Style::default().fg(colors.dim),
                )
            }).collect();
            forecast_lines.push(Line::from(low_spans));
        }
    }

    // Render top section
    let forecast_height = forecast_lines.len() as u16;
    let top_height = inner.height.saturating_sub(forecast_height).saturating_sub(1); // -1 for gap

    if top_height > 0 {
        let top_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: top_height,
        };
        let top_para = Paragraph::new(top_lines).wrap(Wrap { trim: false });
        frame.render_widget(top_para, top_area);
    }

    // Render forecast at bottom
    if forecast_height > 0 && inner.height > forecast_height {
        let forecast_y = inner.y + inner.height - forecast_height;
        let forecast_area = Rect {
            x: inner.x,
            y: forecast_y,
            width: inner.width,
            height: forecast_height,
        };
        let forecast_para = Paragraph::new(forecast_lines);
        frame.render_widget(forecast_para, forecast_area);
    }
}

/// Render the news panel as a scrollable list with highlighted selection.
fn render_news_panel(frame: &mut Frame, app: &App, area: Rect) {
    let colors = app.theme.colors();
    let title = format!("{} News", app.icons.panel_news);
    let block = panel_block(app, PanelId::News, &title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.news_data.headlines.is_empty() {
        let msg = Paragraph::new(Line::from(vec![Span::styled(
            "No news available",
            Style::default().fg(colors.dim),
        )]));
        frame.render_widget(msg, inner);
        return;
    }

    let selected = app.news_selected();
    let visible_height = inner.height as usize;

    // Each headline takes 2 lines (title + summary), compute scroll
    let item_height = 2;
    let items_visible = visible_height / item_height;
    let scroll_offset = if selected >= items_visible {
        selected - items_visible + 1
    } else {
        0
    };

    let items: Vec<ListItem> = app
        .news_data
        .headlines
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .map(|(i, headline)| {
            let is_selected = i == selected && app.current_panel == PanelId::News;

            let title_style = if is_selected {
                Style::default()
                    .fg(colors.highlight_fg)
                    .bg(colors.highlight_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(colors.news_accent)
                    .add_modifier(Modifier::BOLD)
            };

            let summary: String = headline.summary.chars().take(80).collect();
            let summary_style = Style::default().fg(colors.dim);

            ListItem::new(vec![
                Line::from(Span::styled(&headline.title, title_style)),
                Line::from(Span::styled(
                    if summary.is_empty() {
                        String::new()
                    } else {
                        format!("  {}", summary)
                    },
                    summary_style,
                )),
            ])
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

/// Render the system panel with metrics, bars, trends, and optional sparklines.
fn render_system_panel(frame: &mut Frame, app: &App, area: Rect) {
    let colors = app.theme.colors();
    let title = format!("{} System{}", app.icons.panel_system, if app.show_processes { " (Processes)" } else { " (p to toggle)" });
    let block = panel_block(app, PanelId::System, &title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.show_processes {
        let mut rows = Vec::new();
        for proc in &app.system.top_processes {
            let style = if proc.cpu_usage > 50.0 {
                Style::default().fg(Color::Red)
            } else if proc.cpu_usage > 20.0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(colors.fg.unwrap_or(Color::White))
            };

            rows.push(Row::new(vec![
                Cell::from(proc.pid.to_string()).style(Style::default().fg(colors.dim)),
                Cell::from(proc.name.clone()).style(Style::default()),
                Cell::from(format!("{:.1}%", proc.cpu_usage)).style(style),
                Cell::from(format_bytes(proc.mem_bytes)).style(Style::default()),
            ]));
        }

        let widths = [
            Constraint::Length(6),
            Constraint::Min(10),
            Constraint::Length(7),
            Constraint::Length(8),
        ];

        let table = Table::new(rows, widths)
            .header(Row::new(vec!["PID", "Name", "CPU", "MEM"]).style(Style::default().add_modifier(Modifier::BOLD).fg(colors.system_accent)))
            .column_spacing(1);

        frame.render_widget(table, inner);
        return;
    }

    let sys = &app.system;
    let bar_width = 20;
    let mut lines: Vec<Line> = Vec::new();

    // CPU
    let cpu = sys.cpu_usage();
    let cpu_trend = sys.cpu_trend();
    lines.push(Line::from(vec![
        Span::styled(
            format!("{} CPU  ", app.icons.cpu),
            Style::default().fg(colors.system_accent),
        ),
        Span::styled(
            percentage_bar(cpu, bar_width),
            Style::default().fg(colors.system_accent),
        ),
        Span::styled(
            format!(" {:.1}% {}", cpu, cpu_trend),
            Style::default().fg(colors.fg.unwrap_or(Color::White)),
        ),
    ]));

    // RAM
    let mem_pct = sys.memory_usage_pct();
    let used_mem = sys.total_memory() - sys.available_memory();
    let ram_trend = sys.ram_trend();
    lines.push(Line::from(vec![
        Span::styled(
            format!("{} RAM  ", app.icons.ram),
            Style::default().fg(colors.system_accent),
        ),
        Span::styled(
            percentage_bar(mem_pct, bar_width),
            Style::default().fg(colors.system_accent),
        ),
        Span::styled(
            format!(
                " {:.1}% {} ({}/{})",
                mem_pct,
                ram_trend,
                format_bytes(used_mem),
                format_bytes(sys.total_memory())
            ),
            Style::default().fg(colors.fg.unwrap_or(Color::White)),
        ),
    ]));

    // Disk
    let disk_pct = sys.disk_usage_pct();
    lines.push(Line::from(vec![
        Span::styled(
            format!("{} DISK ", app.icons.disk),
            Style::default().fg(colors.system_accent),
        ),
        Span::styled(
            percentage_bar(disk_pct, bar_width),
            Style::default().fg(colors.system_accent),
        ),
        Span::styled(
            format!(" {:.1}%", disk_pct),
            Style::default().fg(colors.fg.unwrap_or(Color::White)),
        ),
    ]));

    // Per-disk breakdown
    for disk in sys.disk_info() {
        let used = disk.total - disk.available;
        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("{} {}/{}", disk.mount_point, format_bytes(used), format_bytes(disk.total)),
                Style::default().fg(colors.dim),
            ),
        ]));
    }

    // Network
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled(
            format!("{} ", app.icons.net_down),
            Style::default().fg(colors.system_accent),
        ),
        Span::styled(
            format!("{}/s", format_bytes(sys.net_rx_rate as u64)),
            Style::default().fg(colors.fg.unwrap_or(Color::White)),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("{} ", app.icons.net_up),
            Style::default().fg(colors.system_accent),
        ),
        Span::styled(
            format!("{}/s", format_bytes(sys.net_tx_rate as u64)),
            Style::default().fg(colors.fg.unwrap_or(Color::White)),
        ),
    ]));

    // Calculate how much space text takes
    let text_lines = lines.len() as u16;
    let remaining_height = inner.height.saturating_sub(text_lines);
    let show_sparklines = inner.height >= 15 && remaining_height >= 5;

    if show_sparklines {
        // Add a blank separator
        lines.push(Line::raw(""));

        let text_height = lines.len() as u16;

        // Render text portion
        let text_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: text_height.min(inner.height),
        };
        let para = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(para, text_area);

        // Sparkline area below the text
        let spark_y = inner.y + text_height;
        let spark_height = inner.height.saturating_sub(text_height);

        if spark_height >= 3 {
            // Split sparkline area into two side-by-side
            let spark_area = Rect {
                x: inner.x,
                y: spark_y,
                width: inner.width,
                height: spark_height,
            };

            let spark_cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(spark_area);

            // CPU sparkline
            let cpu_data: Vec<u64> = sys.cpu_history.iter().map(|v| *v as u64).collect();
            let cpu_spark = Sparkline::default()
                .block(
                    Block::default()
                        .title(Span::styled(
                            format!(" {} CPU History ", app.icons.sparkline),
                            Style::default().fg(colors.dim),
                        ))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(colors.panel_border)),
                )
                .data(&cpu_data)
                .max(100)
                .style(Style::default().fg(colors.system_accent));
            frame.render_widget(cpu_spark, spark_cols[0]);

            // RAM sparkline
            let ram_data: Vec<u64> = sys.ram_history.iter().map(|v| *v as u64).collect();
            let ram_spark = Sparkline::default()
                .block(
                    Block::default()
                        .title(Span::styled(
                            format!(" {} RAM History ", app.icons.sparkline),
                            Style::default().fg(colors.dim),
                        ))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(colors.panel_border)),
                )
                .data(&ram_data)
                .max(100)
                .style(Style::default().fg(colors.system_accent));
            frame.render_widget(ram_spark, spark_cols[1]);
        }
    } else {
        let para = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(para, inner);
    }
}

/// Render the status bar at the bottom of the screen.
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let colors = app.theme.colors();

    let content = match &app.state {
        AppState::LocationSearch => {
            " \u{2191}\u{2193}: select | Enter: confirm | Esc: cancel".to_string()
        }
        AppState::Help => " Esc/?: close help".to_string(),
        _ => {
            let mut s = String::new();
            // In minimal layout, prepend panel indicator dots
            if app.layout == LayoutMode::Minimal {
                for panel in PanelId::all() {
                    if *panel == app.current_panel {
                        s.push_str("\u{25CF} "); // ●
                    } else {
                        s.push_str("\u{25CB} "); // ○
                    }
                }
                s.push_str(" ");
            }
            s.push_str("Tab: panels | \u{2191}\u{2193}/jk: scroll | Enter: open | L: layout | T: theme | /: location | ?: help | q: quit");
            format!(" {}", s)
        }
    };

    let bar = Paragraph::new(Line::from(vec![Span::styled(
        content,
        Style::default()
            .fg(colors.fg.unwrap_or(Color::White))
            .bg(colors.status_bar_bg),
    )]))
    .style(Style::default().bg(colors.status_bar_bg));

    frame.render_widget(bar, area);
}

/// Render the location search overlay as a centered popup.
fn render_location_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let colors = app.theme.colors();
    let popup = centered_rect(60, 16, area);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(Span::styled(
            " Location Search ",
            Style::default()
                .fg(colors.title)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.active_border))
        .style(Style::default().bg(colors.bg.unwrap_or(Color::Black)));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    if let Some(search) = &app.location_search {
        let mut lines: Vec<Line> = Vec::new();

        // Input line
        lines.push(Line::from(vec![
            Span::styled("> ", Style::default().fg(colors.title)),
            Span::styled(
                &search.query,
                Style::default().fg(colors.fg.unwrap_or(Color::White)),
            ),
            Span::styled("_", Style::default().fg(colors.dim)),
        ]));

        lines.push(Line::raw(""));

        // Results
        if search.filtered.is_empty() {
            if search.query.is_empty() {
                lines.push(Line::from(Span::styled(
                    "Type to search for a city...",
                    Style::default().fg(colors.dim),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    "No results found",
                    Style::default().fg(colors.dim),
                )));
            }
        } else {
            for (display_idx, &result_idx) in search.filtered.iter().enumerate() {
                if let Some(geo) = search.results.get(result_idx) {
                    let is_selected = display_idx == search.selected;
                    let style = if is_selected {
                        Style::default()
                            .fg(colors.highlight_fg)
                            .bg(colors.highlight_bg)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(colors.fg.unwrap_or(Color::White))
                    };

                    let prefix = if is_selected { "> " } else { "  " };
                    lines.push(Line::from(Span::styled(
                        format!("{}{}", prefix, geo.display_label()),
                        style,
                    )));
                }
            }
        }

        let para = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(para, inner);
    }
}

/// Render the help overlay as a centered popup.
fn render_help_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let colors = app.theme.colors();
    let popup = centered_rect(50, 18, area);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(Span::styled(
            " Help ",
            Style::default()
                .fg(colors.title)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.active_border))
        .style(Style::default().bg(colors.bg.unwrap_or(Color::Black)));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let header_style = Style::default()
        .fg(colors.title)
        .add_modifier(Modifier::BOLD);
    let key_style = Style::default().fg(colors.fg.unwrap_or(Color::White));
    let desc_style = Style::default().fg(colors.dim);

    let lines = vec![
        Line::from(Span::styled("Global", header_style)),
        Line::from(vec![
            Span::styled("  q           ", key_style),
            Span::styled("Quit", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  T           ", key_style),
            Span::styled("Cycle theme", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  L           ", key_style),
            Span::styled("Cycle layout", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  /           ", key_style),
            Span::styled("Location search", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  ?           ", key_style),
            Span::styled("Toggle help", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  m           ", key_style),
            Span::styled("Toggle 12h/24h time", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  z           ", key_style),
            Span::styled("Toggle local/UTC", desc_style),
        ]),
        Line::raw(""),
        Line::from(Span::styled("Panel Navigation", header_style)),
        Line::from(vec![
            Span::styled("  Tab         ", key_style),
            Span::styled("Next panel", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Shift+Tab   ", key_style),
            Span::styled("Previous panel", desc_style),
        ]),
        Line::raw(""),
        Line::from(Span::styled("News Panel", header_style)),
        Line::from(vec![
            Span::styled("  \u{2191}/\u{2193}, j/k   ", key_style),
            Span::styled("Scroll headlines", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Enter       ", key_style),
            Span::styled("Open in browser", desc_style),
        ]),
    ];

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, inner);
}

/// Compute a centered rectangle with a maximum width and height.
fn centered_rect(max_width: u16, max_height: u16, area: Rect) -> Rect {
    let width = max_width.min(area.width);
    let height = max_height.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}
