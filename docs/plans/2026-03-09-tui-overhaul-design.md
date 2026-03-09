# Glance TUI Overhaul — Design Document

**Date:** 2026-03-09

## Overview

Comprehensive overhaul of the Glance terminal dashboard to make it responsive, navigable, visually polished, and more functional. Touches every module.

---

## 1. Responsive Layouts

Four layout presets, auto-selected by terminal size, overridable with `L` key.

**Wide Horizontal (≥120 cols):** `[Weather 25%] [News 35%] [System 40%]` side-by-side with bottom status bar.

**Compact Horizontal (80–119 cols):** Two-row — `[Weather | News]` top, `[System full-width]` bottom. Reduced padding/labels.

**Tall Vertical (≥40 rows, <80 cols):** Stacked top-to-bottom: Weather (small), News (medium), System (large).

**Minimal Vertical (<80 cols, <40 rows):** Single panel at a time. Tab to switch. Status bar shows `● ○ ○` indicator.

**Implementation:**
- New `layout.rs`: `Layout` enum + `select_layout(cols, rows) -> Layout`
- `ui.rs` dispatches to layout-specific render functions with their own `Constraint` splits
- Crossterm resize events trigger layout re-selection
- Config: `preferred_layout = "auto" | "wide" | "compact" | "tall" | "minimal"`

---

## 2. Navigation & Interaction

**Panel navigation:**
- `Tab` / `Shift+Tab` cycle forward/backward through panels
- Active panel: highlighted border (bold + accent color)
- Inactive panels: dim border

**Intra-panel navigation (vim keys + arrows):**
- `j`/`↓` and `k`/`↑` move selection cursor within active panel
- News panel: scrollable headline list with visible highlight bar
- System panel: scroll through disks/interfaces if overflow
- Weather panel: no scrolling (reserved for future forecast nav)

**Actions:**
- `Enter` on news headline opens link via `open` (macOS) / `xdg-open` (Linux)
- `L` cycles layout presets
- `T` cycles color themes
- `q` / `Ctrl+Q` quits
- `?` toggles help overlay with all keybindings
- `/` opens location search

**Implementation:**
- `app.rs`: `scroll_offset: HashMap<PanelId, usize>`, `selected_index: HashMap<PanelId, usize>`
- Key dispatch: global bindings first (Tab, q, L, T, ?, /), then route to active panel handler
- New `browser.rs` util: `open_url(url)` wrapping platform `open`/`xdg-open`

---

## 3. Nerd Font Detection & Glyphs

**Detection (layered):**
1. Config: `nerd_font = true | false | "auto"` (default: `"auto"`)
2. Env var: `NERD_FONT=1` forces on
3. Auto-detect: render known nerd font glyph, query cursor position for width=1. Timeout 50ms, fall back to Unicode/emoji.

**Icon sets:**

| Element | Nerd Font | Fallback |
|---------|-----------|----------|
| Clear day | `󰖙` nf-md-weather_sunny | ☀️ |
| Rain | `󰖗` nf-md-weather_rainy | 🌧️ |
| Cloud | `󰖐` nf-md-weather_cloudy | ☁️ |
| Panel: Weather | `󰖙` | `W` |
| Panel: News | `󰎕` nf-md-newspaper | `N` |
| Panel: System | `󰍛` nf-md-memory | `S` |
| CPU | `󰻠` nf-md-cpu | `CPU` |
| RAM | `󰍛` nf-md-memory | `RAM` |
| Disk | `󰋊` nf-md-harddisk | `DSK` |
| Network ↑ | `󰁝` nf-md-arrow_up | `TX` |
| Network ↓ | `󰁅` nf-md-arrow_down | `RX` |
| Sparkline | `󰄭` nf-md-chart_line | `~` |
| Status separator | `` nf-pl-left_hard | `│` |

**Implementation:**
- New `icons.rs`: `Icons` struct with both sets, `Icons::new(nerd_font: bool)`
- All render code uses `app.icons.*` instead of hardcoded strings
- Weather code → icon mapping moves from `weather.rs` to `icons.rs`

---

## 4. Config Path Fix & Location Search

**Config path (macOS fix):**
- Check `XDG_CONFIG_HOME` first, fall back to `~/.config` on all Unix
- Windows: keep `dirs::config_dir()` (`%APPDATA%`)
- Migrate existing `~/Library/Application Support/glance/config.toml` to `~/.config/glance/config.toml` on first run, print notice

**Location search (when no location configured):**
1. Detect `config.weather.location` is None on startup
2. Enter `AppState::LocationSearch` — centered overlay with text input
3. Debounced 300ms queries to Open-Meteo geocoding API: `geocoding-api.open-meteo.com/v1/search?name={query}&count=10`
4. Results as fzf-style filtered list: `City, Region, Country (lat, lon)`
5. `j`/`k`/arrows to navigate, `Enter` to confirm (saves to config, fetches weather)
6. `Esc` to skip — weather panel shows "No location configured (press / to search)"
7. `/` key re-opens location search anytime

**Implementation:**
- New `location.rs`: `LocationSearchState` (query, results, selected_index, debounce_timer)
- Geocoding API response struct matching Open-Meteo format
- `ui.rs`: `render_location_overlay()` using `Clear` + centered `Block` popup

---

## 5. System Metrics Enhancements

**Sparkline history:**
- Rolling buffer of 60 samples for CPU and RAM (one per 500ms tick = 30s window)
- Ratatui `Sparkline` widget rendered above percentage bars
- Label: current value + trend arrow (↑/↓/→ from last 5 samples)

**Network throughput:**
- Compute rx/tx delta per second between ticks
- Display as `↓ 1.2 MB/s  ↑ 340 KB/s` instead of cumulative totals
- Optional sparkline for throughput if panel height ≥15 rows

**Implementation:**
- `system.rs`: `cpu_history: VecDeque<f64>`, `ram_history: VecDeque<f64>`, `net_history: VecDeque<(f64, f64)>` — all capped at 60
- `net_throughput: (f64, f64)` bytes/sec from refresh delta
- `ui.rs`: conditional sparkline rendering based on available height

---

## 6. Color Themes

**Four presets, `T` key cycles. Default: Matte Black.**

| Token | Matte Black (default) | Dark | Light | Dracula |
|-------|----------------------|------|-------|---------|
| Background | `#121212` | terminal default | terminal default | `#282a36` |
| Foreground | `#BEBEBE` | terminal default | terminal default | `#f8f8f2` |
| Panel border | `#333333` | `Gray` | `DarkGray` | `#6272a4` |
| Active border | `#FFC107` (gold) | `Cyan` | `Blue` | `#bd93f9` |
| Weather accent | `#E68E0D` (amber) | `Blue` | `DarkBlue` | `#8be9fd` |
| News accent | `#FFC107` (gold) | `Green` | `DarkGreen` | `#50fa7b` |
| System accent | `#D35F5F` (muted red) | `Yellow` | `DarkYellow` | `#f1fa8c` |
| Highlight bar | `#EAEAEA on #333333` | `White on Cyan` | `Black on Blue` | `White on #6272a4` |
| Status bar bg | `#333333` | `DarkGray` | `LightGray` | `#44475a` |
| Dim text | `#8A8A8D` | `DarkGray` | `Gray` | `#6272a4` |

Matte Black palette sourced from [tahayvr/matte-black-theme](https://github.com/tahayvr/matte-black-theme) Ghostty config.

**Implementation:**
- New `theme.rs`: `Theme` enum + `ThemeColors` struct
- `App` holds current theme, passed to all render functions
- Saved to config as `theme = "matte-black" | "dark" | "light" | "dracula"`

---

## 7. Status Bar & Help Overlay

**Status bar (bottom row):**
- Persistent single-line bar showing contextual keybindings
- Default: `Tab: panels │ ↑↓/jk: scroll │ Enter: open │ L: layout │ T: theme │ ?: help │ q: quit`
- Location search: `↑↓: select │ Enter: confirm │ Esc: cancel`
- Uses nerd font separators when available

**Help overlay (`?`):**
- Centered popup listing all keybindings grouped by context (Global, News, System, etc.)
- `Esc` or `?` to dismiss

**Implementation:**
- `ui.rs`: 1-row bottom chunk from outermost vertical split
- `render_status_bar()` reads `AppState` + `PanelId` to select hint text
- `render_help_overlay()` for the `?` popup

---

## New Modules Summary

| File | Purpose |
|------|---------|
| `src/layout.rs` | Layout enum, auto-selection, constraint definitions |
| `src/theme.rs` | Theme enum, color token structs, preset definitions |
| `src/icons.rs` | Nerd font / fallback icon sets, detection logic |
| `src/location.rs` | Location search state, geocoding API client |
| `src/browser.rs` | Platform-aware URL opener |

## Modified Modules

| File | Changes |
|------|---------|
| `src/app.rs` | New states (LocationSearch, Help), per-panel scroll/selection, theme/layout/icons ownership |
| `src/ui.rs` | Layout dispatch, status bar, overlays, theme-aware rendering, sparklines |
| `src/config.rs` | XDG path fix, migration, new fields (preferred_layout, theme, nerd_font) |
| `src/system.rs` | History buffers, throughput calculation |
| `src/weather.rs` | Icon delegation to `icons.rs` |
| `src/main.rs` | Expanded key dispatch, resize handling, new app states |
| `src/lib.rs` | New module re-exports |
