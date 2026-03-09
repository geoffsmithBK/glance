# Glance

A terminal-based dashboard that displays weather, news headlines, and system metrics (CPU, RAM, disk, network) in a single view. Built with Rust and [Ratatui](https://ratatui.rs).

```
╭──────────────────────────────────────────────────────────────╮
│  GLANCE 12:34:56  [WEATHER] [ENTER] [Ctrl+Q]                │
╞══════════════════╤══════════════════╤════════════════════════╡
│ WEATHER          │ NEWS             │ SYSTEM                 │
│ ☀️ 23°C          │ AI Breakthrough  │ CPU  ████████░░ 85.0%  │
│ Clear sky        │ DeepMind annou...│ RAM  █████░░░░░ 45.2%  │
│ Humidity: 65%    │                  │ DISK ███████░░░ 70.1%  │
│ Wind: 12 km/h    │ Rust 2.0 Released│                        │
│                  │ The Rust team ...│ NET  rx: 1.2 GB  tx:...│
╰──────────────────┴──────────────────┴════════════════════════╯
```

## Features

- **System monitoring** — CPU usage, RAM usage, disk usage with ASCII progress bars, network I/O totals
- **Weather** — Current temperature, conditions, humidity, and wind via [Open-Meteo](https://open-meteo.com) (free, no API key required)
- **News** — Headlines from configurable RSS feeds (defaults to Hacker News and TechCrunch)
- **Caching** — Weather and news data cached with 5-minute TTL to avoid excessive API calls
- **Configurable** — TOML config file for location, RSS feeds, refresh rate, and theme

## Install

Requires [Rust](https://rustup.rs) 1.70+.

```sh
git clone https://github.com/geoffsmithBK/glance.git
cd glance
cargo build --release
```

The binary will be at `target/release/glance`.

## Usage

```sh
cargo run
# or
./target/release/glance
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `q` / `Ctrl+Q` | Quit |
| `Tab` | Cycle panel focus |
| `Enter` | Toggle config editor |

## Configuration

On first run, a default config file is created at:

- **macOS**: `~/Library/Application Support/glance/config.toml`
- **Linux**: `~/.config/glance/config.toml`
- **Windows**: `%APPDATA%\glance\config.toml`

### Example config

```toml
[weather]
api_url = "https://api.open-meteo.com/v1/forecast"

[weather.location]
lat = 51.5074
lon = -0.1278

[news]
feeds = [
    "https://hnrss.org/frontpage",
    "https://techcrunch.com/feed/",
]

[ui]
refresh_rate_ms = 500
theme = "default"
```

To get weather data, you must set `[weather.location]` with your latitude and longitude.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` + `crossterm` | Terminal UI rendering |
| `tokio` | Async runtime for network I/O |
| `sysinfo` | CPU, RAM, disk, and network metrics |
| `reqwest` | HTTP client for weather API |
| `rss` | RSS feed parsing |
| `chrono` | Date/time handling |
| `serde` + `toml` | Config serialization |
| `parking_lot` | Thread-safe TTL cache |
| `dirs` | Platform-independent config paths |
| `anyhow` | Error handling |

## Platform Support

- **macOS** — Full support (CPU, RAM, disk, network, weather, news)
- **Linux** — Full support
- **Windows** — Basic support (CPU, RAM, disk via sysinfo)

## License

MIT
