use crate::config::WeatherConfig;
use crate::utils::cache::Cache;
use chrono::NaiveDate;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DayForecast {
    pub date: String,
    pub temp_max: f32,
    pub temp_min: f32,
    pub weather_code: u16,
}

#[derive(Debug, Clone)]
pub struct WeatherData {
    pub weather_code: u16,
    pub is_day: bool,
    pub temp: f32,
    pub unit: String,
    pub condition: String,
    pub humidity: String,
    pub wind: String,
    pub wind_unit: Option<String>,
    pub sunrise: String,
    pub sunset: String,
    pub forecast: Vec<DayForecast>,
    pub day_summary: String,
}

impl Default for WeatherData {
    fn default() -> Self {
        Self {
            weather_code: 0,
            is_day: true,
            temp: 0.0,
            unit: "C".to_string(),
            condition: "Unknown".to_string(),
            humidity: String::new(),
            wind: String::new(),
            wind_unit: None,
            sunrise: String::new(),
            sunset: String::new(),
            forecast: Vec::new(),
            day_summary: String::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct HourlyWeather {
    weather_code: Vec<u16>,
}

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    current: CurrentWeather,
    #[serde(default)]
    daily: Option<DailyWeather>,
    #[serde(default)]
    hourly: Option<HourlyWeather>,
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

#[derive(Debug, Deserialize)]
struct DailyWeather {
    time: Vec<String>,
    temperature_2m_max: Vec<f32>,
    temperature_2m_min: Vec<f32>,
    weather_code: Vec<u16>,
    sunrise: Vec<String>,
    sunset: Vec<String>,
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

        let temp_unit = &self.config.temperature_unit;
        let wind_unit = if temp_unit == "fahrenheit" { "mph" } else { "kmh" };
        let url = format!(
            "{}?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m,is_day&daily=temperature_2m_max,temperature_2m_min,weather_code,sunrise,sunset&hourly=weather_code&temperature_unit={}&wind_speed_unit={}&timezone=auto&forecast_days=7",
            self.config.api_url, location.lat, location.lon, temp_unit, wind_unit
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

        let humidity = current
            .relative_humidity
            .map(|h| h.to_string())
            .unwrap_or_default();

        let wind_unit_label = if self.config.temperature_unit == "fahrenheit" { "mph" } else { "km/h" };
        let (wind, wind_unit) = if let Some(windspeed) = current.windspeed {
            (
                format!("{:.0} {}", windspeed, wind_unit_label),
                Some(wind_unit_label.to_string()),
            )
        } else {
            (String::new(), None)
        };

        let unit = if self.config.temperature_unit == "fahrenheit" {
            "F".to_string()
        } else {
            "C".to_string()
        };

        let (sunrise, sunset) = if let Some(ref daily) = data.daily {
            let sr = daily
                .sunrise
                .first()
                .map(|s| s.split('T').nth(1).unwrap_or(s).to_string())
                .unwrap_or_default();
            let ss = daily
                .sunset
                .first()
                .map(|s| s.split('T').nth(1).unwrap_or(s).to_string())
                .unwrap_or_default();
            (sr, ss)
        } else {
            (String::new(), String::new())
        };

        let forecast = data
            .daily
            .as_ref()
            .map(|d| self.parse_forecast(d))
            .unwrap_or_default();

        let day_summary = data
            .hourly
            .as_ref()
            .map(|h| self.build_day_summary(h))
            .unwrap_or_default();

        WeatherData {
            weather_code: current.weathercode,
            is_day: is_day == 1,
            temp: current.temperature,
            unit,
            condition: self.get_weather_condition(current.weathercode),
            humidity,
            wind,
            wind_unit,
            sunrise,
            sunset,
            forecast,
            day_summary,
        }
    }

    fn parse_forecast(&self, daily: &DailyWeather) -> Vec<DayForecast> {
        let mut forecast = Vec::new();
        for i in 0..daily.time.len().min(7) {
            let day_name =
                if let Ok(date) = NaiveDate::parse_from_str(&daily.time[i], "%Y-%m-%d") {
                    date.format("%a").to_string()
                } else {
                    daily.time[i].clone()
                };

            forecast.push(DayForecast {
                date: day_name,
                temp_max: daily.temperature_2m_max[i],
                temp_min: daily.temperature_2m_min[i],
                weather_code: daily.weather_code[i],
            });
        }
        forecast
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

    fn build_day_summary(&self, hourly: &HourlyWeather) -> String {
        // Today's hours: indices 0..24
        let today: Vec<u16> = hourly.weather_code.iter().take(24).copied().collect();
        if today.len() < 24 {
            return String::new();
        }

        let periods = [
            ("Morning", &today[6..12]),
            ("Afternoon", &today[12..18]),
            ("Evening", &today[18..24]),
            ("Overnight", &today[0..6]),
        ];

        let mut parts: Vec<String> = Vec::new();
        let mut last_desc = String::new();

        for (name, hours) in &periods {
            let code = Self::dominant_code(hours);
            let desc = Self::short_condition(code);
            if desc != last_desc {
                parts.push(format!("{} {}", desc, name.to_lowercase()));
                last_desc = desc;
            }
        }

        // Capitalize first letter
        if let Some(first) = parts.first_mut() {
            if let Some(c) = first.get(0..1) {
                *first = format!("{}{}", c.to_uppercase(), &first[1..]);
            }
        }

        parts.join(", ")
    }

    fn dominant_code(hours: &[u16]) -> u16 {
        // Find the most common weather code in this period
        let mut counts = std::collections::HashMap::new();
        for &code in hours {
            *counts.entry(code).or_insert(0u32) += 1;
        }
        // Return the code with highest count; on tie, prefer worse weather (higher code)
        counts
            .into_iter()
            .max_by_key(|&(code, count)| (count, code))
            .map(|(code, _)| code)
            .unwrap_or(0)
    }

    fn short_condition(code: u16) -> String {
        match code {
            0 => "clear".to_string(),
            1 => "mostly clear".to_string(),
            2 => "partly cloudy".to_string(),
            3 => "overcast".to_string(),
            45 | 48 => "foggy".to_string(),
            51 | 53 => "light drizzle".to_string(),
            55 => "drizzle".to_string(),
            56 | 57 => "freezing drizzle".to_string(),
            61 => "light rain".to_string(),
            63 => "rain".to_string(),
            65 => "heavy rain".to_string(),
            66 | 67 => "freezing rain".to_string(),
            71 => "light snow".to_string(),
            73 => "snow".to_string(),
            75 => "heavy snow".to_string(),
            77 => "snow grains".to_string(),
            80 => "light showers".to_string(),
            81 => "showers".to_string(),
            82 => "heavy showers".to_string(),
            85 => "light snow showers".to_string(),
            86 => "heavy snow showers".to_string(),
            95 => "thunderstorm".to_string(),
            96 | 99 => "thunderstorm with hail".to_string(),
            _ => "clear".to_string(),
        }
    }
}
