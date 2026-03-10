# Glance — TODOs & Ideas

Planned work and brainstorming. Roughly ordered by effort within each section.

---

## Weather Panel

- **3-day forecast mini-chart**: replace the plain high/low numbers in the forecast row with sparkline-style temp range bars (one bar per day showing the high/low spread)
- **Precipitation display**: add today's precipitation probability / accumulation from the Open-Meteo daily response (already in the API, just not requested)

## System Panel

- **Per-process memory breakdown**: top-N processes by RAM usage, rendered as a sub-list below the RAM bar (`sysinfo` exposes `processes()`, just needs wiring + UI space)
- **GPU metrics**: platform-specific — Metal Performance Shaders counters on macOS, NVML for NVIDIA, ROCm/HSA for AMD

## Config / Navigation

- **Config editor UI**: `AppState::EditingConfig` is stubbed in `app.rs` — implement an in-TUI editor for the most common fields (RSS feeds, location, theme, layout, nerd_font toggle) so users never have to touch the TOML file directly
- **Configurable RSS feeds via TUI**: add/remove feeds interactively (likely part of the config editor above)

## New Panels

- **GitHub panel**: open PRs, CI status, unread notifications — requires a personal access token in config
- **Calendar panel**: next upcoming event name + countdown, either as a title-bar widget or a full panel; likely via CalDAV or a local `ical` file

## Housekeeping

- **`git config` author identity**: the committer name/email is auto-configured from hostname; set `user.name` and `user.email` in git config to silence the warning on every commit
