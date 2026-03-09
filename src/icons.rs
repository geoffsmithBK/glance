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
            weather_clear_day: "\u{f0599}",
            weather_clear_night: "\u{f0594}",
            weather_partly_cloudy: "\u{f0595}",
            weather_cloudy: "\u{f0590}",
            weather_fog: "\u{f0591}",
            weather_drizzle: "\u{f0597}",
            weather_rain: "\u{f0597}",
            weather_snow: "\u{f0598}",
            weather_thunderstorm: "\u{f0596}",
            weather_unknown: "\u{f0599}",
            panel_weather: "\u{f0599}",
            panel_news: "\u{f0395}",
            panel_system: "\u{f035b}",
            cpu: "\u{f0ee0}",
            ram: "\u{f035b}",
            disk: "\u{f02ca}",
            net_up: "\u{f005d}",
            net_down: "\u{f0045}",
            sparkline: "\u{f012d}",
            separator: "\u{e0b0}",
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
        assert!(!icons.weather_clear_day.is_empty());
        assert!(!icons.cpu.is_empty());
        assert!(!icons.panel_weather.is_empty());
        assert!(!icons.separator.is_empty());
    }

    #[test]
    fn test_fallback_icons_ascii() {
        let icons = Icons::fallback();
        assert_eq!(icons.panel_weather, "W");
        assert_eq!(icons.panel_news, "N");
        assert_eq!(icons.panel_system, "S");
        assert_eq!(icons.cpu, "CPU");
        assert_eq!(icons.ram, "RAM");
        assert_eq!(icons.disk, "DSK");
        assert_eq!(icons.net_up, "TX");
        assert_eq!(icons.net_down, "RX");
        assert_eq!(icons.sparkline, "~");
        assert_eq!(icons.separator, "│");
    }

    #[test]
    fn test_weather_icon_day_vs_night() {
        let icons = Icons::nerd();
        let day = icons.weather_icon(0, true);
        let night = icons.weather_icon(0, false);
        assert_ne!(day, night);
        assert_eq!(day, icons.weather_clear_day);
        assert_eq!(night, icons.weather_clear_night);
    }

    #[test]
    fn test_weather_icon_codes() {
        let icons = Icons::nerd();
        assert_eq!(icons.weather_icon(45, true), icons.weather_fog);
        assert_eq!(icons.weather_icon(55, true), icons.weather_drizzle);
        assert_eq!(icons.weather_icon(73, true), icons.weather_snow);
        assert_eq!(icons.weather_icon(80, true), icons.weather_rain);
        assert_eq!(icons.weather_icon(95, true), icons.weather_thunderstorm);
        assert_eq!(icons.weather_icon(2, true), icons.weather_partly_cloudy);
        assert_eq!(icons.weather_icon(2, false), icons.weather_cloudy);
    }

    #[test]
    fn test_detect_nerd_font_config_override() {
        assert!(detect_nerd_font(Some(true)));
        assert!(!detect_nerd_font(Some(false)));
    }

    #[test]
    fn test_detect_nerd_font_no_config() {
        // With no config and no env var, should return false
        std::env::remove_var("NERD_FONT");
        assert!(!detect_nerd_font(None));
    }
}
