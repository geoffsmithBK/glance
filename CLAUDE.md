# Glance — Terminal Dashboard

A terminal-based dashboard displaying weather, news headlines, and system metrics using Rust and Ratatui.

## Project Structure

```
src/
├── main.rs           # Entry point, tokio runtime, event loop, key dispatch
├── app.rs            # App struct, state management, per-panel scroll/selection
├── ui.rs             # Layout dispatch, status bar, overlays, theme-aware rendering
├── config.rs         # TOML config via XDG_CONFIG_HOME / ~/.config
├── system.rs         # CPU/RAM/disk/network metrics with history buffers
├── weather.rs        # Open-Meteo API client with TTL cache
├── news.rs           # RSS feed fetcher with TTL cache
├── layout.rs         # Layout enum, auto-selection by terminal size
├── theme.rs          # Color themes: Matte Black (default), Dark, Light, Dracula
├── icons.rs          # Nerd font / fallback icon sets, detection logic
├── location.rs       # Location search overlay, Open-Meteo geocoding client
├── browser.rs        # Platform-aware URL opener (open/xdg-open)
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
- **Config**: TOML file at `$XDG_CONFIG_HOME/glance/config.toml` or `~/.config/glance/config.toml`. Created with defaults on first run.
- **Layouts**: 4 responsive presets (Wide, Compact, Tall, Minimal) auto-selected by terminal dimensions.
- **Themes**: 4 color presets (Matte Black default, Dark, Light, Dracula). Cycled with `T`.
- **Icons**: Nerd font glyphs with auto-detection, env var, or config override. Falls back to Unicode/emoji.
- **Navigation**: Tab/Shift+Tab between panels, vim keys + arrows within panels, Enter to open URLs.

## Key Types

- `App` (app.rs) — Owns all state: config, system metrics, weather/news data, services
- `AppState` — enum: Running, LoadingWeather, LoadingNews, LocationSearch, Help, EditingConfig
- `PanelId` — enum: Weather, News, System
- `LayoutMode` (layout.rs) — enum: Wide, Compact, Tall, Minimal
- `Theme` (theme.rs) — enum: MatteBlack, Dark, Light, Dracula
- `Icons` (icons.rs) — Nerd font and fallback glyph sets
- `SystemMetrics` (system.rs) — Wraps sysinfo::System, Disks, Networks
- `WeatherService` / `NewsService` — Async fetchers with built-in caching

## Build & Test

```sh
cargo build
cargo test
cargo run
```

## Design Documents

- [TUI Overhaul Design](docs/plans/2026-03-09-tui-overhaul-design.md) — Responsive layouts, navigation, themes, nerd fonts, location search, system metrics enhancements

## Future Work

### UI / Display
- **Title bar time/date**: make bold, match brightness/lightness of the "GLANCE" app name on the left
- **3-day forecast mini-chart**: temp range bars per day (sparkline-style) instead of plain numbers
- **GPU metrics**: platform-specific (Metal on macOS, NVML for NVIDIA, ROCm for AMD)

### Weather
- **Weather panel nerd font icons**: wire `app.icons.weather_icon()` into the current conditions and forecast rows (currently using emoji fallback strings directly in weather.rs)

### Navigation / Config
- **Config editor UI**: `AppState::EditingConfig` is stubbed — implement in-TUI editing of feeds, location, theme, layout
- **Configurable RSS feeds via TUI**: add/remove feeds without touching the TOML file

### System Panel
- **Per-process memory breakdown**: top-N processes by RAM in system panel (sysinfo supports this)

### New Panels (wtfutil parity / leapfrog)
- **GitHub panel**: notifications, open PRs, CI status — useful for devs
- **Calendar panel**: next meeting name + countdown in title bar or dedicated panel
