# Glance — Terminal Dashboard

A terminal-based dashboard displaying weather, news headlines, and system metrics using Rust and Ratatui.

## Project Structure

```
src/
├── main.rs           # Entry point, tokio runtime, event loop
├── app.rs            # App struct, state management, panel cycling
├── ui.rs             # Ratatui layout and rendering (3-panel layout)
├── config.rs         # TOML config loading/saving via dirs crate
├── system.rs         # CPU/RAM/disk/network metrics via sysinfo
├── weather.rs        # Open-Meteo API client with TTL cache
├── news.rs           # RSS feed fetcher with TTL cache
├── lib.rs            # Public module re-exports
└── utils/
    ├── mod.rs
    ├── cache.rs      # Generic TTL cache using parking_lot::RwLock
    └── helpers.rs    # format_bytes, percentage_bar, truncate_str
```

## Architecture

- **Async**: Tokio runtime for weather/news HTTP fetching. System metrics are synchronous (sysinfo).
- **Event loop**: 100ms poll via crossterm. System metrics refresh every tick, weather/news every 5 minutes.
- **Caching**: `utils::cache::Cache<T>` — generic TTL cache backed by `parking_lot::RwLock<HashMap>`.
- **Config**: TOML file at `dirs::config_dir()/glance/config.toml`. Created with defaults on first run.

## Key Types

- `App` (app.rs) — Owns all state: config, system metrics, weather/news data, services
- `AppState` — enum: Running, LoadingWeather, LoadingNews, EditingConfig
- `PanelId` — enum: Weather, News, System
- `SystemMetrics` (system.rs) — Wraps sysinfo::System, Disks, Networks
- `WeatherService` / `NewsService` — Async fetchers with built-in caching

## Build & Test

```sh
cargo build
cargo test
cargo run
```

## Future Work

- GPU metrics (platform-specific)
- Theme/color customization
- Config editor UI (AppState::EditingConfig is stubbed)
- Per-process memory breakdown
- 3-day weather forecast mini-chart
