use std::collections::HashMap;

use crate::config::{Config, Location};
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
    LoadingArticle,
    ReadingArticle { title: String, content: String, scroll: u16, url: String },
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
    pub use_12h: bool,
    pub use_utc: bool,
    pub show_processes: bool,
}

impl App {
    pub fn new() -> Result<Self, anyhow::Error> {
        let config = Config::load()?;
        let weather_service = WeatherService::new(config.weather.clone());
        let news_service = NewsService::new(config.news.clone());
        let theme = Theme::from_name(&config.ui.theme);
        let icons = icons::icons_for_config(config.ui.nerd_font);
        let layout_override = LayoutMode::from_name(&config.ui.preferred_layout);

        let mut selected = HashMap::new();
        selected.insert(PanelId::Weather, 0);
        selected.insert(PanelId::News, 0);
        selected.insert(PanelId::System, 0);

        let mut scroll_offset = HashMap::new();
        scroll_offset.insert(PanelId::Weather, 0);
        scroll_offset.insert(PanelId::News, 0);
        scroll_offset.insert(PanelId::System, 0);

        let needs_location = config.weather.location.is_none();

        let show_processes = config.ui.show_processes;
        
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
            layout: LayoutMode::Wide,
            layout_override,
            selected,
            scroll_offset,
            location_search: None,
            use_12h: false,
            use_utc: false,
            show_processes,
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

    pub async fn load_article(&mut self, url: &str, title: &str) {
        self.state = AppState::LoadingArticle;
        
        let jina_url = format!("https://r.jina.ai/{}", url);
        if let Ok(res) = reqwest::get(&jina_url).await {
            if res.status().is_success() {
                if let Ok(content) = res.text().await {
                    self.state = AppState::ReadingArticle { 
                        title: title.to_string(), 
                        content, 
                        scroll: 0,
                        url: url.to_string()
                    };
                    return;
                }
            }
        }
        
        // If it failed, just go back to running state
        self.state = AppState::Running;
    }

    pub fn update_metrics(&mut self) {
        self.system.refresh();
    }

    pub fn update_layout(&mut self, cols: u16, rows: u16) {
        self.layout = self
            .layout_override
            .unwrap_or_else(|| LayoutMode::auto_select(cols, rows));
    }

    pub fn cycle_layout(&mut self) {
        let next = self.layout.next();
        self.layout = next;
        self.layout_override = Some(next);
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

    pub fn max_items_for_panel(&self, panel: PanelId) -> usize {
        match panel {
            PanelId::News => self.news_data.headlines.len(),
            PanelId::System => 3 + self.system.disk_info().len(),
            PanelId::Weather => 1,
        }
    }

    pub fn selected_headline(&self) -> Option<(&str, &str)> {
        if self.current_panel != PanelId::News {
            return None;
        }
        let idx = self.news_selected();
        let headline = self.news_data.headlines.get(idx)?;
        if headline.link.is_empty() {
            None
        } else {
            Some((&headline.title, &headline.link))
        }
    }

    pub fn selected_headline_url(&self) -> Option<&str> {
        self.selected_headline().map(|(_, url)| url)
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
        let result = self
            .location_search
            .as_ref()
            .and_then(|ls| ls.selected_result().cloned());

        if let Some(geo) = result {
            self.config.weather.location = Some(Location {
                lat: geo.latitude,
                lon: geo.longitude,
            });

            // Auto-detect temperature unit from country code
            let fahrenheit_countries = ["US", "BS", "KY", "LR", "PW", "FM", "MH"];
            let temp_unit = if let Some(ref cc) = geo.country_code {
                if fahrenheit_countries.contains(&cc.as_str()) {
                    "fahrenheit"
                } else {
                    "celsius"
                }
            } else {
                "celsius"
            };
            self.config.weather.temperature_unit = temp_unit.to_string();
            self.config.weather.location_name = Some(geo.display_label());

            let _ = self.config.save();
            self.weather_service = WeatherService::new(self.config.weather.clone());
            self.state = AppState::Running;
            self.location_search = None;
            true
        } else {
            false
        }
    }

    pub fn toggle_help(&mut self) {
        self.state = match self.state {
            AppState::Help => AppState::Running,
            AppState::Running => AppState::Help,
            _ => AppState::Running,
        };
    }

    pub fn toggle_config(&mut self) {
        self.state = match self.state {
            AppState::EditingConfig => AppState::Running,
            AppState::Running => AppState::EditingConfig,
            _ => AppState::Running,
        };
    }

    pub fn toggle_ampm(&mut self) {
        self.use_12h = !self.use_12h;
    }

    pub fn toggle_utc(&mut self) {
        self.use_utc = !self.use_utc;
    }

    pub fn toggle_processes(&mut self) {
        self.show_processes = !self.show_processes;
    }

    pub fn time_display(&self) -> String {
        use chrono::Utc;

        let now_local = Local::now();
        let (time_str, date_str, week_str) = if self.use_utc {
            let now = Utc::now();
            let time = if self.use_12h {
                now.format("%I:%M:%S %p").to_string()
            } else {
                now.format("%H:%M:%S").to_string()
            };
            let date = now.format("%d %B").to_string();
            let week = now.format("W%V").to_string();
            (time, date, format!("{} UTC", week))
        } else {
            let time = if self.use_12h {
                now_local.format("%I:%M:%S %p").to_string()
            } else {
                now_local.format("%H:%M:%S").to_string()
            };
            let date = now_local.format("%d %B").to_string();
            let week = now_local.format("W%V").to_string();
            (time, date, week)
        };
        format!("{} \\\\ {} {}", time_str, date_str, week_str)
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
        let panel = PanelId::Weather;
        let panel = panel.next();
        assert_eq!(panel, PanelId::News);
        let panel = panel.next();
        assert_eq!(panel, PanelId::System);
        let panel = panel.next();
        assert_eq!(panel, PanelId::Weather);
    }

    #[test]
    fn test_panel_cycle_backward() {
        let panel = PanelId::Weather;
        let panel = panel.prev();
        assert_eq!(panel, PanelId::System);
        let panel = panel.prev();
        assert_eq!(panel, PanelId::News);
        let panel = panel.prev();
        assert_eq!(panel, PanelId::Weather);
    }

    #[test]
    fn test_app_state_variants() {
        let _running = AppState::Running;
        let _loading_weather = AppState::LoadingWeather;
        let _loading_news = AppState::LoadingNews;
        let _location_search = AppState::LocationSearch;
        let _help = AppState::Help;
        let _editing_config = AppState::EditingConfig;
    }
}
