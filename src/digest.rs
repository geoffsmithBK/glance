use crate::app::App;
use chrono::{DateTime, Local, NaiveTime, TimeZone, Utc};
use unicode_width::UnicodeWidthStr;

const CPU_WARN_THRESHOLD: f32 = 75.0;
const MEMORY_WARN_THRESHOLD: f32 = 85.0;
const DISK_WARN_THRESHOLD: f32 = 90.0;
const NETWORK_WARN_THRESHOLD: f64 = 20.0 * 1024.0 * 1024.0;
const FRESH_NEWS_WINDOW_HOURS: i64 = 8;
const WEATHER_EVENT_WINDOW_MINUTES: i64 = 90;

#[derive(Debug, Clone, Default)]
pub struct DigestState {
    pub clauses: Vec<String>,
    pub text: String,
}

impl DigestState {
    pub fn from_app(app: &App) -> Self {
        Self::from_inputs(DigestInputs::from_app(app))
    }

    fn from_inputs(inputs: DigestInputs) -> Self {
        let mut clauses = Vec::new();

        if let Some(system) = build_system_clause(&inputs) {
            clauses.push(system);
        }

        if let Some(weather) = build_weather_clause(&inputs) {
            clauses.push(weather);
        }

        if let Some(news) = build_news_clause(&inputs) {
            clauses.push(news);
        }

        if clauses.is_empty() {
            clauses.push("System steady".to_string());
        }

        let text = compose_clauses(&clauses);
        Self { clauses, text }
    }

    pub fn render(&self, width: usize) -> String {
        if width == 0 {
            return String::new();
        }

        for end in (1..=self.clauses.len()).rev() {
            let candidate = compose_clauses(&self.clauses[..end]);
            if display_width(&candidate) <= width {
                return candidate;
            }
        }

        truncate_display_width(&self.clauses[0], width)
    }
}

#[derive(Debug, Clone, Default)]
struct DigestInputs {
    cpu_usage: f32,
    memory_usage_pct: f32,
    disks: Vec<DiskDigest>,
    net_rx_rate: f64,
    net_tx_rate: f64,
    top_processes: Vec<ProcessDigest>,
    weather: WeatherDigest,
    headlines: Vec<HeadlineDigest>,
    use_12h: bool,
}

#[derive(Debug, Clone)]
struct DiskDigest {
    mount_point: String,
    usage_pct: f32,
}

#[derive(Debug, Clone)]
struct ProcessDigest {
    name: String,
    cpu_usage: f32,
    mem_bytes: u64,
}

#[derive(Debug, Clone, Default)]
struct WeatherDigest {
    condition: String,
    day_summary: String,
    sunrise: String,
    sunset: String,
}

#[derive(Debug, Clone)]
struct HeadlineDigest {
    published: Option<DateTime<Utc>>,
}

impl DigestInputs {
    fn from_app(app: &App) -> Self {
        let disks = app
            .system
            .disk_info()
            .into_iter()
            .map(|disk| {
                let usage_pct = if disk.total == 0 {
                    0.0
                } else {
                    (disk.total - disk.available) as f32 / disk.total as f32 * 100.0
                };
                DiskDigest {
                    mount_point: disk.mount_point,
                    usage_pct,
                }
            })
            .collect();

        let top_processes = app
            .system
            .top_processes
            .iter()
            .map(|proc| ProcessDigest {
                name: proc.name.clone(),
                cpu_usage: proc.cpu_usage,
                mem_bytes: proc.mem_bytes,
            })
            .collect();

        let weather = WeatherDigest {
            condition: app.weather_data.condition.clone(),
            day_summary: app.weather_data.day_summary.clone(),
            sunrise: app.weather_data.sunrise.clone(),
            sunset: app.weather_data.sunset.clone(),
        };

        let headlines = app
            .news_data
            .headlines
            .iter()
            .map(|headline| HeadlineDigest {
                published: headline.published,
            })
            .collect();

        Self {
            cpu_usage: app.system.cpu_usage(),
            memory_usage_pct: app.system.memory_usage_pct(),
            disks,
            net_rx_rate: app.system.net_rx_rate,
            net_tx_rate: app.system.net_tx_rate,
            top_processes,
            weather,
            headlines,
            use_12h: app.use_12h,
        }
    }
}

fn build_system_clause(inputs: &DigestInputs) -> Option<String> {
    let process_label = primary_process_label(&inputs.top_processes);
    let mut candidates: Vec<(u8, String)> = Vec::new();

    if inputs.cpu_usage >= CPU_WARN_THRESHOLD {
        let clause = if let Some(label) = process_label.clone() {
            format!("CPU pressure from {}", label)
        } else {
            "CPU pressure detected".to_string()
        };
        let severity = if inputs.cpu_usage >= 90.0 { 4 } else { 3 };
        candidates.push((severity, clause));
    }

    if inputs.memory_usage_pct >= MEMORY_WARN_THRESHOLD {
        let clause = if let Some(label) = process_label.clone() {
            format!("Memory climbing for {}", label)
        } else {
            "Memory pressure detected".to_string()
        };
        let severity = if inputs.memory_usage_pct >= 92.0 { 4 } else { 3 };
        candidates.push((severity, clause));
    }

    if let Some(disk) = hottest_disk(&inputs.disks) {
        if disk.usage_pct >= DISK_WARN_THRESHOLD {
            let mount = if disk.mount_point.is_empty() {
                "disk".to_string()
            } else {
                disk.mount_point.clone()
            };
            let severity = if disk.usage_pct >= 97.0 { 5 } else { 4 };
            candidates.push((severity, format!("Disk pressure on {}", mount)));
        }
    }

    let net_peak = inputs.net_rx_rate.max(inputs.net_tx_rate);
    if net_peak >= NETWORK_WARN_THRESHOLD && inputs.cpu_usage < CPU_WARN_THRESHOLD {
        let direction = if inputs.net_tx_rate > inputs.net_rx_rate {
            "TX"
        } else {
            "RX"
        };
        candidates.push((2, format!("Network spike on {}", direction)));
    }

    candidates
        .into_iter()
        .max_by_key(|(severity, _)| *severity)
        .map(|(_, clause)| clause)
}

fn hottest_disk(disks: &[DiskDigest]) -> Option<&DiskDigest> {
    disks.iter().max_by(|a, b| {
        a.usage_pct
            .partial_cmp(&b.usage_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

fn primary_process_label(processes: &[ProcessDigest]) -> Option<String> {
    let top = processes.iter().max_by(|a, b| {
        let a_score = (a.cpu_usage, a.mem_bytes);
        let b_score = (b.cpu_usage, b.mem_bytes);
        a_score
            .partial_cmp(&b_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    })?;

    let raw = top.name.trim();
    if raw.is_empty() {
        return None;
    }

    let lower = raw.to_ascii_lowercase();
    let label = if lower.contains("rustc") || lower.contains("cargo") {
        "Rust build".to_string()
    } else if lower.contains("docker") {
        "Docker".to_string()
    } else if lower.contains("node")
        || lower.contains("npm")
        || lower.contains("pnpm")
        || lower.contains("yarn")
    {
        "Node toolchain".to_string()
    } else if lower.contains("git") {
        "Git".to_string()
    } else if lower.contains("chrome")
        || lower.contains("firefox")
        || lower.contains("safari")
        || lower.contains("arc")
    {
        "browser".to_string()
    } else if lower.contains("code") || lower.contains("cursor") || lower.contains("zed") {
        "editor".to_string()
    } else {
        raw.to_string()
    };

    Some(label)
}

fn build_weather_clause(inputs: &DigestInputs) -> Option<String> {
    let day_summary = inputs.weather.day_summary.trim();
    if !day_summary.is_empty() {
        let lowered = day_summary.to_ascii_lowercase();
        if contains_notable_weather(&lowered) {
            return Some(sentence_case(day_summary));
        }
    }

    let condition = inputs.weather.condition.trim();
    if !condition.is_empty() {
        let lowered = condition.to_ascii_lowercase();
        let clause = if lowered.contains("thunder") {
            Some("Thunderstorms now".to_string())
        } else if lowered.contains("snow") {
            Some("Snow now".to_string())
        } else if lowered.contains("rain") || lowered.contains("showers") {
            Some("Rain now".to_string())
        } else if lowered.contains("drizzle") {
            Some("Drizzle now".to_string())
        } else if lowered.contains("fog") {
            Some("Fog now".to_string())
        } else {
            None
        };
        if clause.is_some() {
            return clause;
        }
    }

    upcoming_weather_event("Sunrise", &inputs.weather.sunrise, inputs.use_12h)
        .or_else(|| upcoming_weather_event("Sunset", &inputs.weather.sunset, inputs.use_12h))
}

fn contains_notable_weather(s: &str) -> bool {
    ["rain", "showers", "snow", "drizzle", "fog", "thunder"]
        .iter()
        .any(|needle| s.contains(needle))
}

fn upcoming_weather_event(label: &str, time_str: &str, use_12h: bool) -> Option<String> {
    let time = NaiveTime::parse_from_str(time_str, "%H:%M").ok()?;
    let now = Local::now();
    let date = now.date_naive();
    let event = Local
        .from_local_datetime(&date.and_time(time))
        .single()?;
    let minutes = event.signed_duration_since(now).num_minutes();
    if !(0..=WEATHER_EVENT_WINDOW_MINUTES).contains(&minutes) {
        return None;
    }

    Some(format!("{} at {}", label, format_clock_time(time, use_12h)))
}

fn format_clock_time(time: NaiveTime, use_12h: bool) -> String {
    if use_12h {
        time.format("%-I:%M %p").to_string()
    } else {
        time.format("%H:%M").to_string()
    }
}

fn build_news_clause(inputs: &DigestInputs) -> Option<String> {
    let now = Utc::now();
    let fresh_count = inputs
        .headlines
        .iter()
        .filter_map(|headline| headline.published)
        .filter(|published| now.signed_duration_since(*published).num_hours() <= FRESH_NEWS_WINDOW_HOURS)
        .count();

    match fresh_count {
        0 => None,
        1 => Some("1 fresh headline".to_string()),
        n => Some(format!("{} fresh headlines", n)),
    }
}

fn compose_clauses(clauses: &[String]) -> String {
    let mut text = clauses.join(". ");
    if !text.ends_with('.') {
        text.push('.');
    }
    text
}

fn sentence_case(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut chars = trimmed.chars();
    let first = chars.next().unwrap().to_uppercase().to_string();
    format!("{}{}", first, chars.as_str())
}

fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

fn truncate_display_width(s: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    if display_width(s) <= max_width {
        return s.to_string();
    }

    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let mut out = String::new();
    for ch in s.chars() {
        let next = format!("{}{}...", out, ch);
        if display_width(&next) > max_width {
            break;
        }
        out.push(ch);
    }
    format!("{}...", out.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn fresh_headline(hours_ago: i64) -> HeadlineDigest {
        HeadlineDigest {
            published: Some(Utc::now() - Duration::hours(hours_ago)),
        }
    }

    #[test]
    fn cpu_pressure_prefers_process_attribution() {
        let digest = DigestState::from_inputs(DigestInputs {
            cpu_usage: 88.0,
            top_processes: vec![ProcessDigest {
                name: "rustc".to_string(),
                cpu_usage: 88.0,
                mem_bytes: 1024,
            }],
            ..DigestInputs::default()
        });

        assert_eq!(digest.clauses[0], "CPU pressure from Rust build");
    }

    #[test]
    fn memory_pressure_wins_over_lower_priority_clauses() {
        let digest = DigestState::from_inputs(DigestInputs {
            memory_usage_pct: 91.0,
            top_processes: vec![ProcessDigest {
                name: "Docker Desktop".to_string(),
                cpu_usage: 10.0,
                mem_bytes: 1024,
            }],
            weather: WeatherDigest {
                day_summary: "rain in the afternoon".to_string(),
                ..WeatherDigest::default()
            },
            headlines: vec![fresh_headline(1), fresh_headline(2)],
            ..DigestInputs::default()
        });

        assert_eq!(digest.clauses[0], "Memory climbing for Docker");
    }

    #[test]
    fn notable_weather_leads_when_system_is_quiet() {
        let digest = DigestState::from_inputs(DigestInputs {
            weather: WeatherDigest {
                day_summary: "showers in the afternoon".to_string(),
                ..WeatherDigest::default()
            },
            ..DigestInputs::default()
        });

        assert_eq!(digest.clauses[0], "Showers in the afternoon");
    }

    #[test]
    fn news_clause_drops_first_when_width_is_constrained() {
        let digest = DigestState::from_inputs(DigestInputs {
            cpu_usage: 82.0,
            top_processes: vec![ProcessDigest {
                name: "rustc".to_string(),
                cpu_usage: 80.0,
                mem_bytes: 1024,
            }],
            weather: WeatherDigest {
                day_summary: "rain in the afternoon".to_string(),
                ..WeatherDigest::default()
            },
            headlines: vec![fresh_headline(1), fresh_headline(2), fresh_headline(3)],
            ..DigestInputs::default()
        });

        let rendered = digest.render(54);
        assert!(rendered.contains("CPU pressure from Rust build."));
        assert!(rendered.contains("Rain in the afternoon."));
        assert!(!rendered.contains("fresh headlines"));
    }

    #[test]
    fn narrow_width_truncates_highest_priority_clause() {
        let digest = DigestState::from_inputs(DigestInputs {
            cpu_usage: 82.0,
            top_processes: vec![ProcessDigest {
                name: "rustc".to_string(),
                cpu_usage: 80.0,
                mem_bytes: 1024,
            }],
            ..DigestInputs::default()
        });

        assert_eq!(digest.render(12), "CPU press...");
    }

    #[test]
    fn quiet_state_has_stable_fallback() {
        let digest = DigestState::from_inputs(DigestInputs::default());
        assert_eq!(digest.text, "System steady.");
    }
}
