# TUI Overhaul Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform Glance from a rigid 3-panel dashboard into a responsive, navigable, theme-aware terminal app with nerd font support, location search, and enhanced system metrics.

**Architecture:** Bottom-up build order — theme and icons first (everything renders with colors/glyphs), then config path fix, then layout engine, then navigation, then feature modules (sparklines, location search, browser open), then status bar and help overlay last (they reference all keybindings).

**Tech Stack:** Rust, Ratatui 0.29, crossterm 0.28, tokio, sysinfo, reqwest, Open-Meteo geocoding API, serde, toml.

---

### Task 1: Add `fuzzy-matcher` dependency

We need fuzzy matching for the location search. Add it now so it's available.

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add dependency**

Add `fuzzy-matcher` to `Cargo.toml` under `[dependencies]`:

```toml
fuzzy-matcher = "0.3"
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles with no errors.

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add fuzzy-matcher dependency for location search"
```

---

### Task 2: Create `theme.rs` — Color theme system

The theme module defines 4 color presets and a `ThemeColors` struct used by all rendering code. Matte Black is the default.

**Files:**
- Create: `src/theme.rs`
- Modify: `src/lib.rs:1-7` (add `pub mod theme;`)

**Step 1: Write tests for theme**

Create `src/theme.rs` with tests at the bottom:

```rust
use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    MatteBlack,
    Dark,
    Light,
    Dracula,
}

impl Theme {
    pub fn next(self) -> Self {
        match self {
            Theme::MatteBlack => Theme::Dark,
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dracula,
            Theme::Dracula => Theme::MatteBlack,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Theme::MatteBlack => "matte-black",
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::Dracula => "dracula",
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "dark" => Theme::Dark,
            "light" => Theme::Light,
            "dracula" => Theme::Dracula,
            _ => Theme::MatteBlack,
        }
    }

    pub fn colors(self) -> ThemeColors {
        match self {
            Theme::MatteBlack => ThemeColors {
                bg: Some(Color::Rgb(0x12, 0x12, 0x12)),
                fg: Some(Color::Rgb(0xBE, 0xBE, 0xBE)),
                panel_border: Color::Rgb(0x33, 0x33, 0x33),
                active_border: Color::Rgb(0xFF, 0xC1, 0x07),
                weather_accent: Color::Rgb(0xE6, 0x8E, 0x0D),
                news_accent: Color::Rgb(0xFF, 0xC1, 0x07),
                system_accent: Color::Rgb(0xD3, 0x5F, 0x5F),
                highlight_fg: Color::Rgb(0xEA, 0xEA, 0xEA),
                highlight_bg: Color::Rgb(0x33, 0x33, 0x33),
                status_bar_bg: Color::Rgb(0x33, 0x33, 0x33),
                dim: Color::Rgb(0x8A, 0x8A, 0x8D),
                title: Color::Rgb(0xFF, 0xC1, 0x07),
            },
            Theme::Dark => ThemeColors {
                bg: None,
                fg: None,
                panel_border: Color::Gray,
                active_border: Color::Cyan,
                weather_accent: Color::Blue,
                news_accent: Color::Green,
                system_accent: Color::Yellow,
                highlight_fg: Color::White,
                highlight_bg: Color::Cyan,
                status_bar_bg: Color::DarkGray,
                dim: Color::DarkGray,
                title: Color::Cyan,
            },
            Theme::Light => ThemeColors {
                bg: None,
                fg: None,
                panel_border: Color::DarkGray,
                active_border: Color::Blue,
                weather_accent: Color::DarkBlue,
                news_accent: Color::DarkGreen,
                system_accent: Color::DarkYellow,
                highlight_fg: Color::Black,
                highlight_bg: Color::Blue,
                status_bar_bg: Color::Gray,
                dim: Color::Gray,
                title: Color::Blue,
            },
            Theme::Dracula => ThemeColors {
                bg: Some(Color::Rgb(0x28, 0x2A, 0x36)),
                fg: Some(Color::Rgb(0xF8, 0xF8, 0xF2)),
                panel_border: Color::Rgb(0x62, 0x72, 0xA4),
                active_border: Color::Rgb(0xBD, 0x93, 0xF9),
                weather_accent: Color::Rgb(0x8B, 0xE9, 0xFD),
                news_accent: Color::Rgb(0x50, 0xFA, 0x7B),
                system_accent: Color::Rgb(0xF1, 0xFA, 0x8C),
                highlight_fg: Color::White,
                highlight_bg: Color::Rgb(0x62, 0x72, 0xA4),
                status_bar_bg: Color::Rgb(0x44, 0x47, 0x5A),
                dim: Color::Rgb(0x62, 0x72, 0xA4),
                title: Color::Rgb(0xBD, 0x93, 0xF9),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub bg: Option<Color>,
    pub fg: Option<Color>,
    pub panel_border: Color,
    pub active_border: Color,
    pub weather_accent: Color,
    pub news_accent: Color,
    pub system_accent: Color,
    pub highlight_fg: Color,
    pub highlight_bg: Color,
    pub status_bar_bg: Color,
    pub dim: Color,
    pub title: Color,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_cycle() {
        assert_eq!(Theme::MatteBlack.next(), Theme::Dark);
        assert_eq!(Theme::Dark.next(), Theme::Light);
        assert_eq!(Theme::Light.next(), Theme::Dracula);
        assert_eq!(Theme::Dracula.next(), Theme::MatteBlack);
    }

    #[test]
    fn test_theme_name_roundtrip() {
        for theme in [Theme::MatteBlack, Theme::Dark, Theme::Light, Theme::Dracula] {
            assert_eq!(Theme::from_name(theme.name()), theme);
        }
    }

    #[test]
    fn test_theme_default_is_matte_black() {
        assert_eq!(Theme::default(), Theme::MatteBlack);
    }

    #[test]
    fn test_matte_black_colors() {
        let colors = Theme::MatteBlack.colors();
        assert_eq!(colors.bg, Some(Color::Rgb(0x12, 0x12, 0x12)));
        assert_eq!(colors.active_border, Color::Rgb(0xFF, 0xC1, 0x07));
    }

    #[test]
    fn test_unknown_name_defaults_to_matte_black() {
        assert_eq!(Theme::from_name("unknown"), Theme::MatteBlack);
        assert_eq!(Theme::from_name(""), Theme::MatteBlack);
    }
}
```

**Step 2: Register module in lib.rs**

Add `pub mod theme;` to `src/lib.rs`.

**Step 3: Register module in main.rs**

Add `mod theme;` to `src/main.rs:1-7` alongside the other mod declarations.

**Step 4: Run tests**

Run: `cargo test theme`
Expected: All 5 tests pass.

**Step 5: Commit**

```bash
git add src/theme.rs src/lib.rs src/main.rs
git commit -m "feat: add color theme system with 4 presets (Matte Black default)"
```

---

### Task 3: Create `icons.rs` — Nerd font and fallback icon sets

**Files:**
- Create: `src/icons.rs`
- Modify: `src/lib.rs` (add `pub mod icons;`)

**Step 1: Write icons module with tests**

Create `src/icons.rs`:

```rust
/// Icon sets for nerd font and Unicode/emoji fallback.
/// Nerd font detection: config override > NERD_FONT env var > auto-detect.

#[derive(Debug, Clone)]
pub struct Icons {
    pub weather_clear_day: &'static str,
    pub weather_clear_night: &'static str,
    pub weather_partly_cloudy: &'static str,
    pub weather_cloudy: &'static str,
    pub weather_fog: &'static str,
    pub weather_drizzle: &'static str,
    pub weather_rain: &'static str,
    pub weather_snow: &'static str,
    pub weather_thunderstorm: &'static str,
    pub weather_unknown: &'static str,
    pub panel_weather: &'static str,
    pub panel_news: &'static str,
    pub panel_system: &'static str,
    pub cpu: &'static str,
    pub ram: &'static str,
    pub disk: &'static str,
    pub net_up: &'static str,
    pub net_down: &'static str,
    pub sparkline: &'static str,
    pub separator: &'static str,
}

impl Icons {
    pub fn nerd() -> Self {
        Self {
            weather_clear_day: "\u{f0599}",   // 󰖙 nf-md-weather_sunny
            weather_clear_night: "\u{f0594}", // 󰖔 nf-md-weather_night
            weather_partly_cloudy: "\u{f0595}", // 󰖕 nf-md-weather_partly_cloudy
            weather_cloudy: "\u{f0590}",      // 󰖐 nf-md-weather_cloudy
            weather_fog: "\u{f0591}",         // 󰖑 nf-md-weather_fog
            weather_drizzle: "\u{f0597}",     // 󰖗 nf-md-weather_rainy
            weather_rain: "\u{f0597}",        // 󰖗 nf-md-weather_rainy
            weather_snow: "\u{f0598}",        // 󰖘 nf-md-weather_snowy
            weather_thunderstorm: "\u{f0596}", // 󰖖 nf-md-weather_lightning
            weather_unknown: "\u{f0599}",     // 󰖙
            panel_weather: "\u{f0599}",       // 󰖙
            panel_news: "\u{f0395}",          // 󰎕 nf-md-newspaper
            panel_system: "\u{f035b}",        // 󰍛 nf-md-memory
            cpu: "\u{f0ee0}",                 // 󰻠 nf-md-cpu_64_bit
            ram: "\u{f035b}",                 // 󰍛 nf-md-memory
            disk: "\u{f02ca}",                // 󰋊 nf-md-harddisk
            net_up: "\u{f005d}",              // 󰁝 nf-md-arrow_up_bold
            net_down: "\u{f0045}",            // 󰁅 nf-md-arrow_down_bold
            sparkline: "\u{f012d}",           // 󰄭 nf-md-chart_line
            separator: "\u{e0b0}",            //  nf-pl-left_hard_divider
        }
    }

    pub fn fallback() -> Self {
        Self {
            weather_clear_day: "☀️",
            weather_clear_night: "🌙",
            weather_partly_cloudy: "🌤️",
            weather_cloudy: "☁️",
            weather_fog: "🌫️",
            weather_drizzle: "🌦️",
            weather_rain: "🌧️",
            weather_snow: "❄️",
            weather_thunderstorm: "⚡",
            weather_unknown: "🌡️",
            panel_weather: "W",
            panel_news: "N",
            panel_system: "S",
            cpu: "CPU",
            ram: "RAM",
            disk: "DSK",
            net_up: "TX",
            net_down: "RX",
            sparkline: "~",
            separator: "│",
        }
    }

    /// Get the weather icon for a WMO weather code.
    pub fn weather_icon(&self, code: u16, is_day: bool) -> &'static str {
        match code {
            0 => {
                if is_day {
                    self.weather_clear_day
                } else {
                    self.weather_clear_night
                }
            }
            1..=3 => {
                if is_day {
                    self.weather_partly_cloudy
                } else {
                    self.weather_cloudy
                }
            }
            45..=48 => self.weather_fog,
            51..=67 => self.weather_drizzle,
            71..=77 => self.weather_snow,
            80..=82 => self.weather_rain,
            85..=86 => self.weather_snow,
            95..=99 => self.weather_thunderstorm,
            _ => {
                if is_day {
                    self.weather_clear_day
                } else {
                    self.weather_clear_night
                }
            }
        }
    }
}

/// Detect whether nerd fonts are available.
/// Priority: config override > NERD_FONT env var > false (safe default).
/// Auto-detection via terminal cursor query is fragile; we default to
/// checking env var and config. Users with nerd fonts can set NERD_FONT=1
/// or nerd_font=true in config.
pub fn detect_nerd_font(config_value: Option<bool>) -> bool {
    if let Some(val) = config_value {
        return val;
    }
    if let Ok(val) = std::env::var("NERD_FONT") {
        return val == "1" || val.eq_ignore_ascii_case("true");
    }
    false
}

pub fn icons_for_config(config_value: Option<bool>) -> Icons {
    if detect_nerd_font(config_value) {
        Icons::nerd()
    } else {
        Icons::fallback()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nerd_icons_populated() {
        let icons = Icons::nerd();
        assert!(!icons.cpu.is_empty());
        assert!(!icons.panel_weather.is_empty());
    }

    #[test]
    fn test_fallback_icons_ascii() {
        let icons = Icons::fallback();
        assert_eq!(icons.cpu, "CPU");
        assert_eq!(icons.panel_news, "N");
    }

    #[test]
    fn test_weather_icon_day_vs_night() {
        let icons = Icons::fallback();
        assert_eq!(icons.weather_icon(0, true), "☀️");
        assert_eq!(icons.weather_icon(0, false), "🌙");
    }

    #[test]
    fn test_weather_icon_codes() {
        let icons = Icons::fallback();
        assert_eq!(icons.weather_icon(45, true), "🌫️");
        assert_eq!(icons.weather_icon(71, true), "❄️");
        assert_eq!(icons.weather_icon(95, true), "⚡");
    }

    #[test]
    fn test_detect_nerd_font_config_override() {
        assert!(detect_nerd_font(Some(true)));
        assert!(!detect_nerd_font(Some(false)));
    }

    #[test]
    fn test_detect_nerd_font_no_config() {
        // Without env var set and no config, defaults to false
        // (env var may or may not be set in test env, so we test config override)
        assert!(!detect_nerd_font(Some(false)));
    }
}
```

**Step 2: Register module in lib.rs and main.rs**

Add `pub mod icons;` to `src/lib.rs` and `mod icons;` to `src/main.rs`.

**Step 3: Run tests**

Run: `cargo test icons`
Expected: All 6 tests pass.

**Step 4: Commit**

```bash
git add src/icons.rs src/lib.rs src/main.rs
git commit -m "feat: add nerd font icon system with detection and fallback"
```

---

### Task 4: Fix config path — XDG on all Unix, migrate macOS

**Files:**
- Modify: `src/config.rs:1-134`

**Step 1: Update config structs with new fields**

Add new fields to `UiConfig` in `src/config.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_refresh_rate")]
    pub refresh_rate_ms: u64,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_layout")]
    pub preferred_layout: String,
    #[serde(default)]
    pub nerd_font: Option<bool>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            refresh_rate_ms: default_refresh_rate(),
            theme: default_theme(),
            preferred_layout: default_layout(),
            nerd_font: None,
        }
    }
}

fn default_layout() -> String {
    "auto".to_string()
}
```

Change `default_theme()` to return `"matte-black"`.

**Step 2: Replace `config_path()` with XDG logic**

Replace the `config_path()` method:

```rust
fn config_path() -> Result<PathBuf> {
    let config_dir = Self::config_dir()?;
    Ok(config_dir.join("glance").join("config.toml"))
}

fn config_dir() -> Result<PathBuf> {
    // 1. Check XDG_CONFIG_HOME (works on all platforms)
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg);
        if path.is_absolute() {
            return Ok(path);
        }
    }

    // 2. Unix: always use ~/.config (not ~/Library/Application Support on macOS)
    #[cfg(unix)]
    {
        if let Some(home) = dirs::home_dir() {
            return Ok(home.join(".config"));
        }
    }

    // 3. Windows: use dirs::config_dir() (%APPDATA%)
    #[cfg(windows)]
    {
        if let Some(dir) = dirs::config_dir() {
            return Ok(dir);
        }
    }

    // 4. Fallback: current directory
    std::env::current_dir().context("Failed to get current directory")
}
```

**Step 3: Add migration logic in `load()`**

Update `Config::load()` to check for old macOS path:

```rust
pub fn load() -> Result<Self> {
    let config_path = Self::config_path()?;

    // Migrate from old macOS path if needed
    #[cfg(target_os = "macos")]
    if !config_path.exists() {
        Self::migrate_macos_config(&config_path)?;
    }

    if config_path.exists() {
        let contents = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config at {:?}", config_path))?;
        let config: Config =
            toml::from_str(&contents).with_context(|| "Failed to parse config file")?;
        Ok(config)
    } else {
        Self::create_default(&config_path)
    }
}

#[cfg(target_os = "macos")]
fn migrate_macos_config(new_path: &PathBuf) -> Result<()> {
    if let Some(old_dir) = dirs::config_dir() {
        // dirs::config_dir() on macOS returns ~/Library/Application Support
        let old_path = old_dir.join("glance").join("config.toml");
        if old_path.exists() {
            if let Some(parent) = new_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&old_path, new_path)?;
            eprintln!(
                "Migrated config from {:?} to {:?}",
                old_path, new_path
            );
        }
    }
    Ok(())
}
```

**Step 4: Update existing test and add new ones**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(
            config.weather.api_url,
            "https://api.open-meteo.com/v1/forecast"
        );
        assert!(!config.news.feeds.is_empty());
    }

    #[test]
    fn test_default_theme_is_matte_black() {
        let config = Config::default();
        assert_eq!(config.ui.theme, "matte-black");
    }

    #[test]
    fn test_default_layout_is_auto() {
        let config = Config::default();
        assert_eq!(config.ui.preferred_layout, "auto");
    }

    #[test]
    fn test_nerd_font_default_is_none() {
        let config = Config::default();
        assert!(config.ui.nerd_font.is_none());
    }
}
```

**Step 5: Run tests**

Run: `cargo test config`
Expected: All 4 tests pass.

**Step 6: Commit**

```bash
git add src/config.rs
git commit -m "fix: use XDG config path on macOS, add theme/layout/nerd_font config fields"
```

---

### Task 5: Create `layout.rs` — Responsive layout engine

**Files:**
- Create: `src/layout.rs`
- Modify: `src/lib.rs` (add `pub mod layout;`)

**Step 1: Write layout module with tests**

Create `src/layout.rs`:

```rust
use ratatui::layout::{Constraint, Direction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Wide,      // ≥120 cols: 3 panels side-by-side
    Compact,   // 80-119 cols: 2 panels top + 1 bottom
    Tall,      // <80 cols, ≥40 rows: stacked vertically
    Minimal,   // <80 cols, <40 rows: single panel at a time
}

impl LayoutMode {
    /// Auto-select layout based on terminal dimensions.
    pub fn auto_select(cols: u16, rows: u16) -> Self {
        if cols >= 120 {
            LayoutMode::Wide
        } else if cols >= 80 {
            LayoutMode::Compact
        } else if rows >= 40 {
            LayoutMode::Tall
        } else {
            LayoutMode::Minimal
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "wide" => Some(LayoutMode::Wide),
            "compact" => Some(LayoutMode::Compact),
            "tall" => Some(LayoutMode::Tall),
            "minimal" => Some(LayoutMode::Minimal),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            LayoutMode::Wide => "wide",
            LayoutMode::Compact => "compact",
            LayoutMode::Tall => "tall",
            LayoutMode::Minimal => "minimal",
        }
    }

    pub fn next(self) -> Self {
        match self {
            LayoutMode::Wide => LayoutMode::Compact,
            LayoutMode::Compact => LayoutMode::Tall,
            LayoutMode::Tall => LayoutMode::Minimal,
            LayoutMode::Minimal => LayoutMode::Wide,
        }
    }

    /// Main direction for panel arrangement.
    pub fn panel_direction(self) -> Direction {
        match self {
            LayoutMode::Wide => Direction::Horizontal,
            LayoutMode::Compact => Direction::Vertical,
            LayoutMode::Tall => Direction::Vertical,
            LayoutMode::Minimal => Direction::Vertical,
        }
    }
}

/// Describes how panels are arranged for a given layout mode.
/// For Compact: two rows — top_panels and bottom_panels.
/// For Wide/Tall: single row/column — top_panels only.
/// For Minimal: single panel (caller uses current_panel to pick).
pub struct LayoutSpec {
    pub mode: LayoutMode,
    /// Outer vertical split: [top_bar, panels_area, status_bar]
    pub outer_constraints: Vec<Constraint>,
}

impl LayoutSpec {
    pub fn new(mode: LayoutMode) -> Self {
        Self {
            mode,
            outer_constraints: vec![
                Constraint::Length(2),  // top bar
                Constraint::Min(0),    // panels
                Constraint::Length(1),  // status bar
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_select_wide() {
        assert_eq!(LayoutMode::auto_select(120, 40), LayoutMode::Wide);
        assert_eq!(LayoutMode::auto_select(200, 50), LayoutMode::Wide);
    }

    #[test]
    fn test_auto_select_compact() {
        assert_eq!(LayoutMode::auto_select(80, 40), LayoutMode::Compact);
        assert_eq!(LayoutMode::auto_select(119, 30), LayoutMode::Compact);
    }

    #[test]
    fn test_auto_select_tall() {
        assert_eq!(LayoutMode::auto_select(79, 40), LayoutMode::Tall);
        assert_eq!(LayoutMode::auto_select(60, 60), LayoutMode::Tall);
    }

    #[test]
    fn test_auto_select_minimal() {
        assert_eq!(LayoutMode::auto_select(79, 39), LayoutMode::Minimal);
        assert_eq!(LayoutMode::auto_select(40, 20), LayoutMode::Minimal);
    }

    #[test]
    fn test_layout_cycle() {
        assert_eq!(LayoutMode::Wide.next(), LayoutMode::Compact);
        assert_eq!(LayoutMode::Compact.next(), LayoutMode::Tall);
        assert_eq!(LayoutMode::Tall.next(), LayoutMode::Minimal);
        assert_eq!(LayoutMode::Minimal.next(), LayoutMode::Wide);
    }

    #[test]
    fn test_layout_name_roundtrip() {
        for mode in [LayoutMode::Wide, LayoutMode::Compact, LayoutMode::Tall, LayoutMode::Minimal] {
            assert_eq!(LayoutMode::from_name(mode.name()), Some(mode));
        }
    }

    #[test]
    fn test_layout_spec_outer_constraints() {
        let spec = LayoutSpec::new(LayoutMode::Wide);
        assert_eq!(spec.outer_constraints.len(), 3);
    }
}
```

**Step 2: Register module**

Add `pub mod layout;` to `src/lib.rs` and `mod layout;` to `src/main.rs`.

**Step 3: Run tests**

Run: `cargo test layout`
Expected: All 7 tests pass.

**Step 4: Commit**

```bash
git add src/layout.rs src/lib.rs src/main.rs
git commit -m "feat: add responsive layout engine with 4 modes"
```

---

### Task 6: Create `browser.rs` — Platform URL opener

**Files:**
- Create: `src/browser.rs`
- Modify: `src/lib.rs` (add `pub mod browser;`)

**Step 1: Write browser module**

Create `src/browser.rs`:

```rust
use std::process::Command;

/// Open a URL in the system default browser.
/// Returns Ok(()) if the command was spawned (does not wait for browser).
pub fn open_url(url: &str) -> Result<(), std::io::Error> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(url).spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/C", "start", url]).spawn()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // open_url spawns a real browser, so we only test compilation
    // and that the function signature is correct.
    use super::open_url;

    #[test]
    fn test_open_url_exists() {
        // Verify function is callable (don't actually open a browser in tests)
        let _ = open_url as fn(&str) -> Result<(), std::io::Error>;
    }
}
```

**Step 2: Register module**

Add `pub mod browser;` to `src/lib.rs` and `mod browser;` to `src/main.rs`.

**Step 3: Run tests**

Run: `cargo test browser`
Expected: 1 test passes.

**Step 4: Commit**

```bash
git add src/browser.rs src/lib.rs src/main.rs
git commit -m "feat: add platform-aware browser URL opener"
```

---

### Task 7: Create `location.rs` — Geocoding search

**Files:**
- Create: `src/location.rs`
- Modify: `src/lib.rs` (add `pub mod location;`)

**Step 1: Write location module**

Create `src/location.rs`:

```rust
use reqwest::Client;
use serde::Deserialize;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

#[derive(Debug, Clone, Deserialize)]
pub struct GeoResult {
    pub name: String,
    #[serde(default)]
    pub admin1: Option<String>,   // Region/state
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl GeoResult {
    pub fn display_label(&self) -> String {
        match &self.admin1 {
            Some(region) => format!("{}, {}, {}", self.name, region, self.country),
            None => format!("{}, {}", self.name, self.country),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GeoResponse {
    #[serde(default)]
    results: Vec<GeoResult>,
}

/// State for the location search overlay.
pub struct LocationSearch {
    pub query: String,
    pub results: Vec<GeoResult>,
    pub filtered: Vec<usize>, // indices into results
    pub selected: usize,
    client: Client,
    matcher: SkimMatcherV2,
}

impl LocationSearch {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            filtered: Vec::new(),
            selected: 0,
            client: Client::new(),
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn push_char(&mut self, c: char) {
        self.query.push(c);
        self.update_filter();
    }

    pub fn pop_char(&mut self) {
        self.query.pop();
        self.update_filter();
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.filtered.is_empty() && self.selected < self.filtered.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn selected_result(&self) -> Option<&GeoResult> {
        self.filtered
            .get(self.selected)
            .and_then(|&idx| self.results.get(idx))
    }

    /// Fetch results from Open-Meteo geocoding API.
    pub async fn fetch(&mut self) {
        if self.query.trim().is_empty() {
            self.results.clear();
            self.filtered.clear();
            return;
        }

        let url = format!(
            "https://geocoding-api.open-meteo.com/v1/search?name={}&count=10&language=en",
            urlencoding(&self.query)
        );

        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(data) = resp.json::<GeoResponse>().await {
                    self.results = data.results;
                    self.update_filter();
                }
            }
            _ => {}
        }
    }

    fn update_filter(&mut self) {
        if self.query.is_empty() {
            self.filtered = (0..self.results.len()).collect();
        } else {
            let mut scored: Vec<(usize, i64)> = self
                .results
                .iter()
                .enumerate()
                .filter_map(|(i, r)| {
                    self.matcher
                        .fuzzy_match(&r.display_label(), &self.query)
                        .map(|score| (i, score))
                })
                .collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered = scored.into_iter().map(|(i, _)| i).collect();
        }
        // Reset selection if out of bounds
        if self.selected >= self.filtered.len() {
            self.selected = 0;
        }
    }
}

/// Simple percent-encoding for URL query params.
fn urlencoding(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(b as char);
            }
            b' ' => result.push_str("%20"),
            _ => {
                result.push_str(&format!("%{:02X}", b));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_result_display_label() {
        let result = GeoResult {
            name: "Portland".to_string(),
            admin1: Some("Oregon".to_string()),
            country: "United States".to_string(),
            latitude: 45.5,
            longitude: -122.7,
        };
        assert_eq!(result.display_label(), "Portland, Oregon, United States");
    }

    #[test]
    fn test_geo_result_display_no_region() {
        let result = GeoResult {
            name: "Tokyo".to_string(),
            admin1: None,
            country: "Japan".to_string(),
            latitude: 35.7,
            longitude: 139.7,
        };
        assert_eq!(result.display_label(), "Tokyo, Japan");
    }

    #[test]
    fn test_location_search_navigation() {
        let mut search = LocationSearch::new();
        search.results = vec![
            GeoResult {
                name: "A".to_string(),
                admin1: None,
                country: "X".to_string(),
                latitude: 0.0,
                longitude: 0.0,
            },
            GeoResult {
                name: "B".to_string(),
                admin1: None,
                country: "Y".to_string(),
                latitude: 1.0,
                longitude: 1.0,
            },
        ];
        search.filtered = vec![0, 1];
        search.selected = 0;

        search.move_down();
        assert_eq!(search.selected, 1);
        search.move_down(); // should not go past end
        assert_eq!(search.selected, 1);
        search.move_up();
        assert_eq!(search.selected, 0);
        search.move_up(); // should not go below 0
        assert_eq!(search.selected, 0);
    }

    #[test]
    fn test_location_search_selected_result() {
        let mut search = LocationSearch::new();
        assert!(search.selected_result().is_none());

        search.results = vec![GeoResult {
            name: "London".to_string(),
            admin1: Some("England".to_string()),
            country: "UK".to_string(),
            latitude: 51.5,
            longitude: -0.1,
        }];
        search.filtered = vec![0];
        assert_eq!(search.selected_result().unwrap().name, "London");
    }

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("hello world"), "hello%20world");
        assert_eq!(urlencoding("New York"), "New%20York");
        assert_eq!(urlencoding("abc123"), "abc123");
    }

    #[test]
    fn test_push_pop_char() {
        let mut search = LocationSearch::new();
        search.push_char('a');
        search.push_char('b');
        assert_eq!(search.query, "ab");
        search.pop_char();
        assert_eq!(search.query, "a");
        search.pop_char();
        assert_eq!(search.query, "");
        search.pop_char(); // should not panic on empty
        assert_eq!(search.query, "");
    }
}
```

**Step 2: Register module**

Add `pub mod location;` to `src/lib.rs` and `mod location;` to `src/main.rs`.

**Step 3: Run tests**

Run: `cargo test location`
Expected: All 6 tests pass.

**Step 4: Commit**

```bash
git add src/location.rs src/lib.rs src/main.rs
git commit -m "feat: add location search with Open-Meteo geocoding and fuzzy matching"
```

---

### Task 8: Enhance `system.rs` — History buffers and throughput

**Files:**
- Modify: `src/system.rs:1-111`

**Step 1: Add history buffers and throughput fields**

Add imports and new fields to `SystemMetrics`:

```rust
use std::collections::VecDeque;
use sysinfo::{Disks, Networks, System};

const HISTORY_CAP: usize = 60;

pub struct SystemMetrics {
    system: System,
    disks: Disks,
    networks: Networks,
    pub cpu_history: VecDeque<f64>,
    pub ram_history: VecDeque<f64>,
    pub net_rx_rate: f64,  // bytes per second
    pub net_tx_rate: f64,  // bytes per second
    pub net_rx_history: VecDeque<u64>,  // stored as u64 for Sparkline
    pub net_tx_history: VecDeque<u64>,
    prev_rx: u64,
    prev_tx: u64,
    last_refresh: std::time::Instant,
}
```

**Step 2: Update `new()` and `refresh()`**

```rust
impl SystemMetrics {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_cpu_all();
        system.refresh_memory();
        let networks = Networks::new_with_refreshed_list();

        let rx: u64 = networks.iter().map(|(_, n)| n.total_received()).sum();
        let tx: u64 = networks.iter().map(|(_, n)| n.total_transmitted()).sum();

        Self {
            system,
            disks: Disks::new_with_refreshed_list(),
            networks,
            cpu_history: VecDeque::with_capacity(HISTORY_CAP),
            ram_history: VecDeque::with_capacity(HISTORY_CAP),
            net_rx_rate: 0.0,
            net_tx_rate: 0.0,
            net_rx_history: VecDeque::with_capacity(HISTORY_CAP),
            net_tx_history: VecDeque::with_capacity(HISTORY_CAP),
            prev_rx: rx,
            prev_tx: tx,
            last_refresh: std::time::Instant::now(),
        }
    }

    pub fn refresh(&mut self) {
        let elapsed = self.last_refresh.elapsed().as_secs_f64();
        self.last_refresh = std::time::Instant::now();

        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.disks.refresh();
        self.networks.refresh();

        // CPU history
        let cpu = self.cpu_usage() as f64;
        if self.cpu_history.len() >= HISTORY_CAP {
            self.cpu_history.pop_front();
        }
        self.cpu_history.push_back(cpu);

        // RAM history
        let ram = self.memory_usage_pct() as f64;
        if self.ram_history.len() >= HISTORY_CAP {
            self.ram_history.pop_front();
        }
        self.ram_history.push_back(ram);

        // Network throughput
        let rx: u64 = self.networks.iter().map(|(_, n)| n.total_received()).sum();
        let tx: u64 = self.networks.iter().map(|(_, n)| n.total_transmitted()).sum();

        if elapsed > 0.0 {
            self.net_rx_rate = (rx.saturating_sub(self.prev_rx)) as f64 / elapsed;
            self.net_tx_rate = (tx.saturating_sub(self.prev_tx)) as f64 / elapsed;
        }

        if self.net_rx_history.len() >= HISTORY_CAP {
            self.net_rx_history.pop_front();
        }
        self.net_rx_history.push_back(self.net_rx_rate as u64);

        if self.net_tx_history.len() >= HISTORY_CAP {
            self.net_tx_history.pop_front();
        }
        self.net_tx_history.push_back(self.net_tx_rate as u64);

        self.prev_rx = rx;
        self.prev_tx = tx;
    }

    /// Trend arrow based on last 5 samples: ↑ ↓ or →
    pub fn cpu_trend(&self) -> &'static str {
        Self::trend(&self.cpu_history)
    }

    pub fn ram_trend(&self) -> &'static str {
        Self::trend(&self.ram_history)
    }

    fn trend(history: &VecDeque<f64>) -> &'static str {
        if history.len() < 5 {
            return "→";
        }
        let recent: Vec<f64> = history.iter().rev().take(5).copied().collect();
        let avg_recent = (recent[0] + recent[1]) / 2.0;
        let avg_older = (recent[3] + recent[4]) / 2.0;
        let diff = avg_recent - avg_older;
        if diff > 2.0 {
            "↑"
        } else if diff < -2.0 {
            "↓"
        } else {
            "→"
        }
    }

    // ... keep all existing pub fn methods unchanged (cpu_usage, total_memory, etc.)
}
```

Keep the existing `cpu_usage()`, `total_memory()`, `available_memory()`, `memory_usage_pct()`, `disk_info()`, `disk_usage_pct()`, `network_received()`, `network_transmitted()` methods exactly as they are. Also keep `DiskInfo` unchanged.

**Step 3: Update tests**

Add new tests while keeping existing ones:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_metrics_basic() {
        let mut metrics = SystemMetrics::new();
        metrics.refresh();
        assert!(metrics.cpu_usage() >= 0.0);
        assert!(metrics.memory_usage_pct() >= 0.0);
        assert!(metrics.disk_usage_pct() >= 0.0);
    }

    #[test]
    fn test_memory_calculation() {
        let metrics = SystemMetrics::new();
        let total = metrics.total_memory();
        let available = metrics.available_memory();
        assert!(total >= available);
    }

    #[test]
    fn test_history_accumulates() {
        let mut metrics = SystemMetrics::new();
        for _ in 0..5 {
            metrics.refresh();
        }
        assert_eq!(metrics.cpu_history.len(), 5);
        assert_eq!(metrics.ram_history.len(), 5);
    }

    #[test]
    fn test_history_caps_at_60() {
        let mut metrics = SystemMetrics::new();
        for _ in 0..70 {
            metrics.refresh();
        }
        assert_eq!(metrics.cpu_history.len(), HISTORY_CAP);
        assert_eq!(metrics.ram_history.len(), HISTORY_CAP);
    }

    #[test]
    fn test_trend_stable() {
        let history: VecDeque<f64> = vec![50.0, 50.0, 50.0, 50.0, 50.0].into();
        assert_eq!(SystemMetrics::trend(&history), "→");
    }

    #[test]
    fn test_trend_insufficient_data() {
        let history: VecDeque<f64> = vec![50.0, 50.0].into();
        assert_eq!(SystemMetrics::trend(&history), "→");
    }
}
```

**Step 4: Run tests**

Run: `cargo test system`
Expected: All 6 tests pass.

**Step 5: Commit**

```bash
git add src/system.rs
git commit -m "feat: add CPU/RAM history buffers, network throughput rate, trend arrows"
```

---

### Task 9: Update `app.rs` — New state, navigation, and module integration

This is the biggest refactor. `App` gains theme, icons, layout, per-panel selection state, and new `AppState` variants.

**Files:**
- Modify: `src/app.rs:1-105`

**Step 1: Rewrite app.rs**

Replace the entire file:

```rust
use std::collections::HashMap;

use crate::config::Config;
use crate::icons::{self, Icons};
use crate::layout::LayoutMode;
use crate::location::LocationSearch;
use crate::news::{NewsData, NewsService};
use crate::system::SystemMetrics;
use crate::theme::Theme;
use crate::weather::{WeatherData, WeatherService};
use chrono::Local;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    Running,
    LoadingWeather,
    LoadingNews,
    LocationSearch,
    Help,
    EditingConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PanelId {
    #[default]
    Weather,
    News,
    System,
}

impl PanelId {
    pub fn all() -> &'static [PanelId] {
        &[PanelId::Weather, PanelId::News, PanelId::System]
    }

    pub fn next(self) -> Self {
        match self {
            PanelId::Weather => PanelId::News,
            PanelId::News => PanelId::System,
            PanelId::System => PanelId::Weather,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            PanelId::Weather => PanelId::System,
            PanelId::News => PanelId::Weather,
            PanelId::System => PanelId::News,
        }
    }
}

pub struct App {
    pub state: AppState,
    pub current_panel: PanelId,
    pub config: Config,
    pub system: SystemMetrics,
    pub weather_data: WeatherData,
    pub news_data: NewsData,
    pub weather_service: WeatherService,
    pub news_service: NewsService,
    pub theme: Theme,
    pub icons: Icons,
    pub layout: LayoutMode,
    pub layout_override: Option<LayoutMode>,
    pub selected: HashMap<PanelId, usize>,
    pub scroll_offset: HashMap<PanelId, usize>,
    pub location_search: Option<LocationSearch>,
}

impl App {
    pub fn new() -> Result<Self, anyhow::Error> {
        let config = Config::load()?;
        let weather_service = WeatherService::new(config.weather.clone());
        let news_service = NewsService::new(config.news.clone());
        let theme = Theme::from_name(&config.ui.theme);
        let icons = icons::icons_for_config(config.ui.nerd_font);
        let layout_override = LayoutMode::from_name(&config.ui.preferred_layout);

        let needs_location = config.weather.location.is_none();

        let mut app = Self {
            state: AppState::Running,
            current_panel: PanelId::Weather,
            config,
            system: SystemMetrics::new(),
            weather_data: WeatherData::default(),
            news_data: NewsData::default(),
            weather_service,
            news_service,
            theme,
            icons,
            layout: LayoutMode::Wide, // will be updated on first render
            layout_override,
            selected: HashMap::new(),
            scroll_offset: HashMap::new(),
            location_search: None,
        };

        if needs_location {
            app.start_location_search();
        }

        Ok(app)
    }

    pub async fn load_data(&mut self) {
        self.state = AppState::LoadingWeather;
        self.weather_data = self.weather_service.fetch().await;

        self.state = AppState::LoadingNews;
        self.news_data = self.news_service.fetch().await;

        self.state = AppState::Running;
    }

    pub fn update_metrics(&mut self) {
        self.system.refresh();
    }

    pub fn update_layout(&mut self, cols: u16, rows: u16) {
        self.layout = self.layout_override.unwrap_or_else(|| LayoutMode::auto_select(cols, rows));
    }

    pub fn cycle_layout(&mut self) {
        let next = self.layout.next();
        self.layout_override = Some(next);
        self.layout = next;
    }

    pub fn cycle_theme(&mut self) {
        self.theme = self.theme.next();
        self.config.ui.theme = self.theme.name().to_string();
        let _ = self.config.save();
    }

    pub fn cycle_panels(&mut self) {
        self.current_panel = self.current_panel.next();
    }

    pub fn cycle_panels_back(&mut self) {
        self.current_panel = self.current_panel.prev();
    }

    pub fn move_up(&mut self) {
        let sel = self.selected.entry(self.current_panel).or_insert(0);
        if *sel > 0 {
            *sel -= 1;
        }
    }

    pub fn move_down(&mut self) {
        let max = self.max_items_for_panel(self.current_panel);
        let sel = self.selected.entry(self.current_panel).or_insert(0);
        if max > 0 && *sel < max - 1 {
            *sel += 1;
        }
    }

    fn max_items_for_panel(&self, panel: PanelId) -> usize {
        match panel {
            PanelId::News => self.news_data.headlines.len(),
            PanelId::System => {
                // CPU + RAM + Disk header + disks + blank + Net = variable
                3 + self.system.disk_info().len()
            }
            PanelId::Weather => 1, // single view
        }
    }

    pub fn selected_headline_url(&self) -> Option<&str> {
        if self.current_panel != PanelId::News {
            return None;
        }
        let idx = self.selected.get(&PanelId::News).copied().unwrap_or(0);
        self.news_data
            .headlines
            .get(idx)
            .map(|h| h.link.as_str())
            .filter(|l| !l.is_empty())
    }

    pub fn start_location_search(&mut self) {
        self.state = AppState::LocationSearch;
        self.location_search = Some(LocationSearch::new());
    }

    pub fn cancel_location_search(&mut self) {
        self.state = AppState::Running;
        self.location_search = None;
    }

    pub fn confirm_location(&mut self) -> bool {
        if let Some(ref search) = self.location_search {
            if let Some(result) = search.selected_result() {
                self.config.weather.location = Some(crate::config::Location {
                    lat: result.latitude,
                    lon: result.longitude,
                });
                let _ = self.config.save();
                self.weather_service = WeatherService::new(self.config.weather.clone());
                self.state = AppState::Running;
                self.location_search = None;
                return true;
            }
        }
        false
    }

    pub fn toggle_help(&mut self) {
        self.state = match self.state {
            AppState::Help => AppState::Running,
            AppState::Running => AppState::Help,
            _ => self.state.clone(),
        };
    }

    pub fn toggle_config(&mut self) {
        self.state = match self.state {
            AppState::Running => AppState::EditingConfig,
            AppState::EditingConfig => AppState::Running,
            _ => AppState::Running,
        };
    }

    pub fn time_display(&self) -> String {
        Local::now().format("%H:%M:%S").to_string()
    }

    pub fn news_selected(&self) -> usize {
        self.selected.get(&PanelId::News).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_cycle_forward() {
        assert_eq!(PanelId::Weather.next(), PanelId::News);
        assert_eq!(PanelId::News.next(), PanelId::System);
        assert_eq!(PanelId::System.next(), PanelId::Weather);
    }

    #[test]
    fn test_panel_cycle_backward() {
        assert_eq!(PanelId::Weather.prev(), PanelId::System);
        assert_eq!(PanelId::News.prev(), PanelId::Weather);
        assert_eq!(PanelId::System.prev(), PanelId::News);
    }

    #[test]
    fn test_app_state_variants() {
        // Ensure all variants exist
        let states = vec![
            AppState::Running,
            AppState::LoadingWeather,
            AppState::LoadingNews,
            AppState::LocationSearch,
            AppState::Help,
            AppState::EditingConfig,
        ];
        assert_eq!(states.len(), 6);
    }
}
```

**Step 2: Run tests**

Run: `cargo test app`
Expected: All 3 tests pass.

**Step 3: Commit**

```bash
git add src/app.rs
git commit -m "feat: expand App with theme, icons, layout, navigation, location search state"
```

---

### Task 10: Rewrite `ui.rs` — Theme-aware, layout-responsive rendering

This is the largest file change. It replaces the hardcoded layout with the responsive system and adds status bar, panel highlighting, sparklines, and overlays.

**Files:**
- Modify: `src/ui.rs:1-210`

**Step 1: Rewrite ui.rs**

Replace the entire file with the new rendering logic. This is long but each function is focused:

```rust
use crate::app::{App, AppState, PanelId};
use crate::layout::LayoutMode;
use crate::utils::helpers::{format_bytes, percentage_bar};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Sparkline, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();
    let colors = app.theme.colors();

    // Update layout based on current terminal size
    app.update_layout(size.width, size.height);

    // Set background if theme specifies one
    if let Some(bg) = colors.bg {
        let bg_block = Block::default().style(Style::default().bg(bg));
        frame.render_widget(bg_block, size);
    }

    // Outer: top bar | panels | status bar
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // top bar
            Constraint::Min(0),    // panels
            Constraint::Length(1),  // status bar
        ])
        .split(size);

    render_top_bar(frame, app, outer[0]);
    render_panels(frame, app, outer[1]);
    render_status_bar(frame, app, outer[2]);

    // Overlays (rendered last, on top)
    match &app.state {
        AppState::LocationSearch => render_location_overlay(frame, app, size),
        AppState::Help => render_help_overlay(frame, app, size),
        _ => {}
    }
}

fn render_top_bar(frame: &mut Frame, app: &App, area: Rect) {
    let colors = app.theme.colors();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let title = Paragraph::new(format!(" {} GLANCE", app.icons.panel_weather))
        .style(
            Style::default()
                .fg(colors.title)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(colors.panel_border)));

    let time = Paragraph::new(app.time_display())
        .alignment(Alignment::Center)
        .style(Style::default().fg(colors.fg.unwrap_or(Color::White)))
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(colors.panel_border)));

    let layout_name = app.layout.name();
    let theme_name = app.theme.name();
    let nav = Paragraph::new(format!(" {} {} ", layout_name, theme_name))
        .alignment(Alignment::Right)
        .style(Style::default().fg(colors.dim))
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(colors.panel_border)));

    frame.render_widget(title, chunks[0]);
    frame.render_widget(time, chunks[1]);
    frame.render_widget(nav, chunks[2]);
}

fn render_panels(frame: &mut Frame, app: &App, area: Rect) {
    match app.layout {
        LayoutMode::Wide => render_wide(frame, app, area),
        LayoutMode::Compact => render_compact(frame, app, area),
        LayoutMode::Tall => render_tall(frame, app, area),
        LayoutMode::Minimal => render_minimal(frame, app, area),
    }
}

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

fn render_compact(frame: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(rows[0]);

    render_weather_panel(frame, app, top[0]);
    render_news_panel(frame, app, top[1]);
    render_system_panel(frame, app, rows[1]);
}

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

fn render_minimal(frame: &mut Frame, app: &App, area: Rect) {
    // Show only the active panel
    match app.current_panel {
        PanelId::Weather => render_weather_panel(frame, app, area),
        PanelId::News => render_news_panel(frame, app, area),
        PanelId::System => render_system_panel(frame, app, area),
    }
}

fn panel_block<'a>(app: &App, panel: PanelId, title: &'a str) -> Block<'a> {
    let colors = app.theme.colors();
    let is_active = app.current_panel == panel;
    let border_color = if is_active {
        colors.active_border
    } else {
        colors.panel_border
    };
    let border_modifier = if is_active {
        Modifier::BOLD
    } else {
        Modifier::empty()
    };

    Block::default()
        .title(title)
        .title_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color).add_modifier(border_modifier))
}

fn render_weather_panel(frame: &mut Frame, app: &App, area: Rect) {
    let colors = app.theme.colors();
    let weather = &app.weather_data;
    let block = panel_block(app, PanelId::Weather, &format!(" {} Weather ", app.icons.panel_weather));
    let inner = block.inner(area);

    frame.render_widget(block, area);

    if app.config.weather.location.is_none() {
        let msg = Paragraph::new("No location configured\nPress / to search")
            .style(Style::default().fg(colors.dim))
            .alignment(Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(
                format!("{} {}°{}", weather.icon, weather.temp, weather.unit),
                Style::default()
                    .fg(colors.weather_accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(Span::styled(
            &weather.condition,
            Style::default().fg(colors.fg.unwrap_or(Color::White)),
        )),
    ];

    if !weather.humidity.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            format!("Humidity: {}%", weather.humidity),
            Style::default().fg(colors.dim),
        )));
    }
    if !weather.wind.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("Wind: {}", weather.wind),
            Style::default().fg(colors.dim),
        )));
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, inner);
}

fn render_news_panel(frame: &mut Frame, app: &App, area: Rect) {
    let colors = app.theme.colors();
    let news = &app.news_data;
    let block = panel_block(app, PanelId::News, &format!(" {} News ", app.icons.panel_news));
    let inner = block.inner(area);

    frame.render_widget(block, area);

    if news.headlines.is_empty() {
        let msg = Paragraph::new("No news available")
            .style(Style::default().fg(colors.dim));
        frame.render_widget(msg, inner);
        return;
    }

    let selected = app.news_selected();
    let visible_height = inner.height as usize;

    // Compute scroll offset to keep selection visible
    let scroll = {
        // Each headline takes 2 lines (title + summary) + 1 blank = 3, except last
        let item_height = 2;
        let start = selected.saturating_sub(visible_height.saturating_sub(item_height) / item_height);
        start
    };

    let items: Vec<ListItem> = news
        .headlines
        .iter()
        .enumerate()
        .skip(scroll)
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

            let summary: String = headline.summary.chars().take(60).collect();
            let summary_str = if summary.len() < headline.summary.len() {
                format!("{}...", summary)
            } else {
                summary
            };

            ListItem::new(vec![
                Line::from(Span::styled(&headline.title, title_style)),
                Line::from(Span::styled(summary_str, Style::default().fg(colors.dim))),
            ])
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_system_panel(frame: &mut Frame, app: &App, area: Rect) {
    let colors = app.theme.colors();
    let sys = &app.system;
    let block = panel_block(app, PanelId::System, &format!(" {} System ", app.icons.panel_system));
    let inner = block.inner(area);

    frame.render_widget(block, area);

    // Split inner area: metrics text on top, sparklines below (if room)
    let has_sparkline_room = inner.height >= 15;
    let chunks = if has_sparkline_room {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),
                Constraint::Length(6), // sparklines
            ])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(inner)
    };

    // Text metrics
    let bar_width = (chunks[0].width as usize).saturating_sub(20).min(25).max(10);
    let cpu = sys.cpu_usage();
    let mem_pct = sys.memory_usage_pct();
    let used_mem = sys.total_memory() - sys.available_memory();
    let disk_pct = sys.disk_usage_pct();

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled(
                format!("{} ", app.icons.cpu),
                Style::default().fg(colors.system_accent),
            ),
            Span::raw(format!(
                "{} {:.1}% {}",
                percentage_bar(cpu, bar_width),
                cpu,
                sys.cpu_trend()
            )),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{} ", app.icons.ram),
                Style::default().fg(colors.system_accent),
            ),
            Span::raw(format!(
                "{} {:.1}% {} ({}/{})",
                percentage_bar(mem_pct, bar_width),
                mem_pct,
                sys.ram_trend(),
                format_bytes(used_mem),
                format_bytes(sys.total_memory())
            )),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{} ", app.icons.disk),
                Style::default().fg(colors.system_accent),
            ),
            Span::raw(format!(
                "{} {:.1}%",
                percentage_bar(disk_pct, bar_width),
                disk_pct,
            )),
        ]),
    ];

    // Per-disk breakdown
    for disk in sys.disk_info() {
        let used = disk.total - disk.available;
        lines.push(Line::from(Span::styled(
            format!("  {} {}/{}", disk.mount_point, format_bytes(used), format_bytes(disk.total)),
            Style::default().fg(colors.dim),
        )));
    }

    // Network throughput
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled(
            format!("{} ", app.icons.net_down),
            Style::default().fg(colors.system_accent),
        ),
        Span::raw(format!("{}/s  ", format_bytes(sys.net_rx_rate as u64))),
        Span::styled(
            format!("{} ", app.icons.net_up),
            Style::default().fg(colors.system_accent),
        ),
        Span::raw(format!("{}/s", format_bytes(sys.net_tx_rate as u64))),
    ]));

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, chunks[0]);

    // Sparklines
    if has_sparkline_room && chunks.len() > 1 {
        let spark_area = chunks[1];
        let spark_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(spark_area);

        let cpu_data: Vec<u64> = sys.cpu_history.iter().map(|&v| v as u64).collect();
        let cpu_spark = Sparkline::default()
            .block(Block::default().title(format!("{} CPU", app.icons.sparkline)).title_style(Style::default().fg(colors.dim)))
            .data(&cpu_data)
            .max(100)
            .style(Style::default().fg(colors.system_accent));
        frame.render_widget(cpu_spark, spark_cols[0]);

        let ram_data: Vec<u64> = sys.ram_history.iter().map(|&v| v as u64).collect();
        let ram_spark = Sparkline::default()
            .block(Block::default().title(format!("{} RAM", app.icons.sparkline)).title_style(Style::default().fg(colors.dim)))
            .data(&ram_data)
            .max(100)
            .style(Style::default().fg(colors.system_accent));
        frame.render_widget(ram_spark, spark_cols[1]);
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let colors = app.theme.colors();
    let sep = app.icons.separator;

    let hints = match app.state {
        AppState::LocationSearch => format!(
            " ↑↓: select {sep} Enter: confirm {sep} Esc: cancel"
        ),
        AppState::Help => format!(
            " Esc/?: close help"
        ),
        _ => {
            let panel_indicator = match app.layout {
                LayoutMode::Minimal => {
                    let dots: String = PanelId::all()
                        .iter()
                        .map(|&p| if p == app.current_panel { "●" } else { "○" })
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!(" {dots} {sep}")
                }
                _ => String::new(),
            };
            format!(
                "{panel_indicator} Tab: panels {sep} ↑↓/jk: scroll {sep} Enter: open {sep} L: layout {sep} T: theme {sep} /: location {sep} ?: help {sep} q: quit"
            )
        }
    };

    let bar = Paragraph::new(hints)
        .style(
            Style::default()
                .fg(colors.fg.unwrap_or(Color::White))
                .bg(colors.status_bar_bg),
        );
    frame.render_widget(bar, area);
}

fn render_location_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let colors = app.theme.colors();

    // Centered popup: 60 wide, 16 tall (or smaller if terminal is small)
    let popup_width = area.width.min(60);
    let popup_height = area.height.min(16);
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Search Location ")
        .title_style(Style::default().fg(colors.active_border).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.active_border))
        .style(Style::default().bg(colors.status_bar_bg));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    if let Some(ref search) = app.location_search {
        // Input line
        let input_area = Rect::new(inner.x, inner.y, inner.width, 1);
        let input = Paragraph::new(format!("> {}_", search.query))
            .style(Style::default().fg(colors.fg.unwrap_or(Color::White)));
        frame.render_widget(input, input_area);

        // Results
        let results_area = Rect::new(inner.x, inner.y + 2, inner.width, inner.height.saturating_sub(2));

        let items: Vec<ListItem> = search
            .filtered
            .iter()
            .enumerate()
            .take(results_area.height as usize)
            .map(|(i, &idx)| {
                let result = &search.results[idx];
                let label = result.display_label();
                let coord = format!("  ({:.2}, {:.2})", result.latitude, result.longitude);
                let is_selected = i == search.selected;
                let style = if is_selected {
                    Style::default()
                        .fg(colors.highlight_fg)
                        .bg(colors.highlight_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(colors.fg.unwrap_or(Color::White))
                };
                ListItem::new(vec![
                    Line::from(Span::styled(label, style)),
                    Line::from(Span::styled(coord, Style::default().fg(colors.dim))),
                ])
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, results_area);
    }
}

fn render_help_overlay(frame: &mut Frame, app: &App, area: Rect) {
    let colors = app.theme.colors();

    let popup_width = area.width.min(50);
    let popup_height = area.height.min(18);
    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Keybindings ")
        .title_style(Style::default().fg(colors.active_border).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.active_border))
        .style(Style::default().bg(colors.status_bar_bg));
    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let help_text = vec![
        Line::styled("Global", Style::default().fg(colors.active_border).add_modifier(Modifier::BOLD)),
        Line::styled("  Tab / Shift+Tab   Switch panels", Style::default().fg(colors.fg.unwrap_or(Color::White))),
        Line::styled("  L                 Cycle layout", Style::default().fg(colors.fg.unwrap_or(Color::White))),
        Line::styled("  T                 Cycle theme", Style::default().fg(colors.fg.unwrap_or(Color::White))),
        Line::styled("  /                 Search location", Style::default().fg(colors.fg.unwrap_or(Color::White))),
        Line::styled("  ?                 Toggle this help", Style::default().fg(colors.fg.unwrap_or(Color::White))),
        Line::styled("  q / Ctrl+Q        Quit", Style::default().fg(colors.fg.unwrap_or(Color::White))),
        Line::raw(""),
        Line::styled("Panel Navigation", Style::default().fg(colors.active_border).add_modifier(Modifier::BOLD)),
        Line::styled("  j / ↓             Move down", Style::default().fg(colors.fg.unwrap_or(Color::White))),
        Line::styled("  k / ↑             Move up", Style::default().fg(colors.fg.unwrap_or(Color::White))),
        Line::raw(""),
        Line::styled("News Panel", Style::default().fg(colors.active_border).add_modifier(Modifier::BOLD)),
        Line::styled("  Enter             Open in browser", Style::default().fg(colors.fg.unwrap_or(Color::White))),
    ];

    let para = Paragraph::new(help_text);
    frame.render_widget(para, inner);
}
```

**Step 2: Run compile check**

Run: `cargo check`
Expected: Compiles (may have warnings about unused imports from removed code — that's fine).

**Step 3: Commit**

```bash
git add src/ui.rs
git commit -m "feat: rewrite UI with responsive layouts, theme colors, sparklines, overlays"
```

---

### Task 11: Rewrite `main.rs` — Full key dispatch and event handling

**Files:**
- Modify: `src/main.rs:1-89`

**Step 1: Rewrite main.rs**

Replace the entire file:

```rust
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
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::time::{Duration, Instant};

use app::{App, AppState};
use ui::render;

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let mut app = App::new()?;

    // Only load data if we're not in location search (no location to fetch weather for)
    if app.state != AppState::LocationSearch {
        app.load_data().await;
    }

    let result = run_app(&mut terminal, &mut app).await;

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
    let data_refresh_interval = Duration::from_secs(300);
    let mut location_debounce: Option<Instant> = None;

    loop {
        terminal.draw(|f| render(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if !handle_key(app, key).await {
                        break;
                    }

                    // Debounce location search API calls
                    if app.state == AppState::LocationSearch {
                        location_debounce = Some(Instant::now());
                    }
                }
                Event::Resize(_, _) => {
                    // Layout will auto-update on next render via update_layout()
                }
                _ => {}
            }
        }

        // Fire debounced location search
        if let Some(debounce_start) = location_debounce {
            if debounce_start.elapsed() >= Duration::from_millis(300) {
                if let Some(ref mut search) = app.location_search {
                    search.fetch().await;
                }
                location_debounce = None;
            }
        }

        app.update_metrics();

        if last_data_refresh.elapsed() >= data_refresh_interval {
            app.load_data().await;
            last_data_refresh = Instant::now();
        }
    }

    Ok(())
}

/// Returns false if the app should quit.
async fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    // Location search mode has its own key handling
    if app.state == AppState::LocationSearch {
        return handle_location_key(app, key);
    }

    // Help mode: Esc or ? to dismiss
    if app.state == AppState::Help {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') => app.toggle_help(),
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => return false,
            _ => {}
        }
        return true;
    }

    // Normal mode key handling
    match (key.modifiers, key.code) {
        // Quit
        (KeyModifiers::CONTROL, KeyCode::Char('q')) => return false,
        (_, KeyCode::Char('q')) => return false,

        // Panel navigation
        (_, KeyCode::Tab) => app.cycle_panels(),
        (KeyModifiers::SHIFT, KeyCode::BackTab) => app.cycle_panels_back(),

        // Intra-panel navigation
        (_, KeyCode::Char('j')) | (_, KeyCode::Down) => app.move_down(),
        (_, KeyCode::Char('k')) | (_, KeyCode::Up) => app.move_up(),

        // Actions
        (_, KeyCode::Enter) => {
            if let Some(url) = app.selected_headline_url() {
                let _ = browser::open_url(url);
            }
        }

        // Layout cycle
        (_, KeyCode::Char('L')) | (_, KeyCode::Char('l')) => app.cycle_layout(),

        // Theme cycle
        (_, KeyCode::Char('T')) | (_, KeyCode::Char('t')) => app.cycle_theme(),

        // Location search
        (_, KeyCode::Char('/')) => app.start_location_search(),

        // Help
        (_, KeyCode::Char('?')) => app.toggle_help(),

        _ => {}
    }

    true
}

fn handle_location_key(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => app.cancel_location_search(),
        KeyCode::Enter => {
            if app.confirm_location() {
                // Trigger weather fetch after confirming location
                // (will happen on next data refresh, or we could force it)
                tokio::spawn({
                    // We can't easily await here since confirm_location is sync,
                    // but the next 5-min refresh will pick it up. For immediate
                    // feedback we'll handle it in the main loop.
                });
            }
        }
        KeyCode::Backspace => {
            if let Some(ref mut search) = app.location_search {
                search.pop_char();
            }
        }
        KeyCode::Up | KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(ref mut search) = app.location_search {
                search.move_up();
            }
        }
        KeyCode::Down | KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(ref mut search) = app.location_search {
                search.move_down();
            }
        }
        KeyCode::Up => {
            if let Some(ref mut search) = app.location_search {
                search.move_up();
            }
        }
        KeyCode::Down => {
            if let Some(ref mut search) = app.location_search {
                search.move_down();
            }
        }
        KeyCode::Char(c) => {
            if let Some(ref mut search) = app.location_search {
                search.push_char(c);
            }
        }
        _ => {}
    }
    true
}
```

**Step 2: Handle immediate weather fetch after location confirm**

After `confirm_location()` returns true in the main loop, we need to trigger an immediate data load. Add this to `run_app()` after the key handling block:

In `run_app()`, after the `handle_key` call, add a check:

```rust
// In the Event::Key branch, after handle_key:
// If location was just confirmed, fetch weather immediately
if app.state == AppState::Running && app.config.weather.location.is_some() && app.weather_data.condition == "Unknown" {
    app.load_data().await;
    last_data_refresh = Instant::now();
}
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully.

**Step 4: Run all tests**

Run: `cargo test`
Expected: All tests pass.

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: full key dispatch with vim nav, layout/theme cycling, location search, browser open"
```

---

### Task 12: Update `lib.rs` — Register all new modules

**Files:**
- Modify: `src/lib.rs`

**Step 1: Update lib.rs**

```rust
pub mod app;
pub mod browser;
pub mod config;
pub mod icons;
pub mod layout;
pub mod location;
pub mod news;
pub mod system;
pub mod theme;
pub mod ui;
pub mod utils;
pub mod weather;
```

**Step 2: Commit**

```bash
git add src/lib.rs
git commit -m "chore: register all new modules in lib.rs"
```

---

### Task 13: Integration — Build, fix, and test

At this point all code is in place. This task is about making it compile and work together.

**Step 1: Run cargo check and fix any compilation errors**

Run: `cargo check 2>&1`

Fix issues iteratively. Common things to watch for:
- The `render()` function in `ui.rs` takes `&mut App` now (for `update_layout`), so the `terminal.draw()` closure in `main.rs` needs `app` as `&mut`
- Import paths may need adjusting
- The `tokio::spawn` in `handle_location_key` can be removed (it's a no-op placeholder)

**Step 2: Run cargo test**

Run: `cargo test`
Expected: All tests pass.

**Step 3: Run the app**

Run: `cargo run`
Expected: App starts, shows Matte Black theme, status bar visible, location search popup appears if no location configured.

**Step 4: Test manually**
- Tab/Shift+Tab between panels
- j/k to scroll news
- L to cycle layouts
- T to cycle themes
- / to open location search
- ? for help overlay
- Enter on news headline opens browser
- Resize terminal to see layout auto-switch

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: complete TUI overhaul — responsive layouts, themes, navigation, sparklines"
```

---

### Task 14: Update Cargo.toml and CLAUDE.md with final state

**Files:**
- Modify: `Cargo.toml` (verify all deps present)
- Modify: `CLAUDE.md` (update if any changes from plan)

**Step 1: Verify Cargo.toml has all needed deps**

Should already have: `ratatui`, `crossterm`, `tokio`, `sysinfo`, `reqwest`, `serde`, `serde_json`, `rss`, `chrono`, `toml`, `dirs`, `anyhow`, `thiserror`, `unicode-width`, `parking_lot`, `fuzzy-matcher`.

**Step 2: Final CLAUDE.md update if needed**

Ensure the project structure, key types, and architecture sections match reality.

**Step 3: Commit**

```bash
git add Cargo.toml CLAUDE.md
git commit -m "docs: finalize Cargo.toml deps and CLAUDE.md for TUI overhaul"
```
