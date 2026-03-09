use crate::config::WeatherConfig;
use crate::utils::cache::Cache;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct WeatherData {
    pub icon: String,
    pub temp: f32,
    pub unit: String,
    pub condition: String,
    pub humidity: String,
    pub wind: String,
    pub wind_unit: Option<String>,
}

impl Default for WeatherData {
    fn default() -> Self {
        Self {
            icon: "🌡️".to_string(),
            temp: 0.0,
            unit: "C".to_string(),
            condition: "Unknown".to_string(),
            humidity: String::new(),
            wind: String::new(),
            wind_unit: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    current: CurrentWeather,
}

#[derive(Debug, Deserialize)]
struct CurrentWeather {
    #[serde(alias = "temperature_2m")]
    temperature: f32,
    #[serde(alias = "weather_code", default)]
    weathercode: u16,
    #[serde(alias = "relative_humidity_2m")]
    relative_humidity: Option<u8>,
    #[serde(alias = "wind_speed_10m")]
    windspeed: Option<f32>,
    is_day: Option<u8>,
}

pub struct WeatherService {
    client: Client,
    config: WeatherConfig,
    cache: Cache<WeatherData>,
}

impl WeatherService {
    pub fn new(config: WeatherConfig) -> Self {
        Self {
            client: Client::new(),
            config,
            cache: Cache::new(Duration::from_secs(300)),
        }
    }

    pub async fn fetch(&self) -> WeatherData {
        if let Some(cached) = self.cache.get() {
            return cached;
        }

        let location = match &self.config.location {
            Some(loc) => loc,
            None => return WeatherData::default(),
        };

        let url = format!(
            "{}?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m,is_day",
            self.config.api_url, location.lat, location.lon
        );

        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                match response.json::<WeatherResponse>().await {
                    Ok(data) => {
                        let weather = self.parse_weather(data);
                        self.cache.set("current".to_string(), weather.clone());
                        weather
                    }
                    Err(_) => WeatherData::default(),
                }
            }
            _ => WeatherData::default(),
        }
    }

    fn parse_weather(&self, data: WeatherResponse) -> WeatherData {
        let current = data.current;
        let is_day = current.is_day.unwrap_or(1);

        let icon = if is_day == 1 {
            self.get_day_icon(current.weathercode)
        } else {
            self.get_night_icon(current.weathercode)
        };

        let humidity = current
            .relative_humidity
            .map(|h| h.to_string())
            .unwrap_or_default();

        let (wind, wind_unit) = if let Some(windspeed) = current.windspeed {
            (
                format!("{:.0} km/h", windspeed),
                Some("km/h".to_string()),
            )
        } else {
            (String::new(), None)
        };

        WeatherData {
            icon,
            temp: current.temperature,
            unit: "C".to_string(),
            condition: self.get_weather_condition(current.weathercode),
            humidity,
            wind,
            wind_unit,
        }
    }

    fn get_weather_condition(&self, code: u16) -> String {
        match code {
            0 => "Clear sky".to_string(),
            1..=3 => "Partly cloudy".to_string(),
            45..=48 => "Fog".to_string(),
            51..=67 => "Drizzle".to_string(),
            71..=77 => "Snow".to_string(),
            80..=82 => "Rain showers".to_string(),
            85..=86 => "Snow showers".to_string(),
            95..=99 => "Thunderstorm".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    fn get_day_icon(&self, code: u16) -> String {
        match code {
            0 => "☀️".to_string(),
            1..=2 => "🌤️".to_string(),
            3 => "⛅".to_string(),
            45..=48 => "🌫️".to_string(),
            51..=67 => "🌦️".to_string(),
            71..=77 => "❄️".to_string(),
            80..=82 => "🌧️".to_string(),
            85..=86 => "🌨️".to_string(),
            95..=99 => "⚡".to_string(),
            _ => "☀️".to_string(),
        }
    }

    fn get_night_icon(&self, code: u16) -> String {
        match code {
            0 => "🌙".to_string(),
            1..=2 => "🌥️".to_string(),
            3 => "☁️".to_string(),
            45..=48 => "🌑".to_string(),
            51..=67 => "🌫️".to_string(),
            71..=77 => "🌜".to_string(),
            80..=82 => "🌧️".to_string(),
            85..=86 => "🌨️".to_string(),
            95..=99 => "⛈️".to_string(),
            _ => "🌙".to_string(),
        }
    }
}
