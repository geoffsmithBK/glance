use crate::config::Config;
use crate::news::{NewsData, NewsService};
use crate::system::SystemMetrics;
use crate::weather::{WeatherData, WeatherService};
use chrono::Local;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    Running,
    LoadingWeather,
    LoadingNews,
    EditingConfig,
}

pub struct App {
    state: AppState,
    current_panel: PanelId,
    config: Config,
    system: SystemMetrics,
    weather_data: WeatherData,
    news_data: NewsData,
    weather_service: WeatherService,
    news_service: NewsService,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanelId {
    #[default]
    Weather,
    News,
    System,
}

impl App {
    pub fn new() -> Result<Self, anyhow::Error> {
        let config = Config::load()?;
        let weather_service = WeatherService::new(config.weather.clone());
        let news_service = NewsService::new(config.news.clone());

        Ok(Self {
            state: AppState::Running,
            current_panel: PanelId::Weather,
            config,
            system: SystemMetrics::new(),
            weather_data: WeatherData::default(),
            news_data: NewsData::default(),
            weather_service,
            news_service,
        })
    }

    pub async fn load_data(&mut self) {
        self.state = AppState::LoadingWeather;
        self.weather_data = self.weather_service.fetch().await;

        self.state = AppState::LoadingNews;
        self.news_data = self.news_service.fetch().await;

        self.state = AppState::Running;
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn current_panel(&self) -> PanelId {
        self.current_panel
    }

    pub fn system(&self) -> &SystemMetrics {
        &self.system
    }

    pub fn weather(&self) -> &WeatherData {
        &self.weather_data
    }

    pub fn news(&self) -> &NewsData {
        &self.news_data
    }

    pub fn update_metrics(&mut self) {
        self.system.refresh();
    }

    pub fn cycle_panels(&mut self) {
        self.current_panel = match self.current_panel {
            PanelId::Weather => PanelId::News,
            PanelId::News => PanelId::System,
            PanelId::System => PanelId::Weather,
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
}
