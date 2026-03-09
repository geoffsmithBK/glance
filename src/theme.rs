use ratatui::style::Color;

/// Available color themes for the dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    MatteBlack,
    Dark,
    Light,
    Dracula,
}

/// Color palette for a theme.
#[derive(Debug, Clone, Copy)]
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

impl Theme {
    /// Cycle to the next theme: MatteBlack -> Dark -> Light -> Dracula -> MatteBlack.
    pub fn next(self) -> Theme {
        match self {
            Theme::MatteBlack => Theme::Dark,
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dracula,
            Theme::Dracula => Theme::MatteBlack,
        }
    }

    /// Return the string name for this theme.
    pub fn name(self) -> &'static str {
        match self {
            Theme::MatteBlack => "matte-black",
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::Dracula => "dracula",
        }
    }

    /// Parse a theme name. Unknown names default to MatteBlack.
    pub fn from_name(name: &str) -> Theme {
        match name {
            "matte-black" => Theme::MatteBlack,
            "dark" => Theme::Dark,
            "light" => Theme::Light,
            "dracula" => Theme::Dracula,
            _ => Theme::MatteBlack,
        }
    }

    /// Return the color palette for this theme.
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
                weather_accent: Color::Rgb(0x00, 0x00, 0x8B), // DarkBlue
                news_accent: Color::Rgb(0x00, 0x64, 0x00),    // DarkGreen
                system_accent: Color::Rgb(0x8B, 0x8B, 0x00),  // DarkYellow
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_cycle() {
        let theme = Theme::MatteBlack;
        let theme = theme.next();
        assert_eq!(theme, Theme::Dark);
        let theme = theme.next();
        assert_eq!(theme, Theme::Light);
        let theme = theme.next();
        assert_eq!(theme, Theme::Dracula);
        let theme = theme.next();
        assert_eq!(theme, Theme::MatteBlack);
    }

    #[test]
    fn test_theme_name_roundtrip() {
        for theme in [Theme::MatteBlack, Theme::Dark, Theme::Light, Theme::Dracula] {
            let name = theme.name();
            let parsed = Theme::from_name(name);
            assert_eq!(theme, parsed);
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
        assert_eq!(Theme::from_name("neon"), Theme::MatteBlack);
        assert_eq!(Theme::from_name(""), Theme::MatteBlack);
        assert_eq!(Theme::from_name("solarized"), Theme::MatteBlack);
    }
}
