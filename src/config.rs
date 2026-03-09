use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use dirs::config_dir;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Config {
    #[serde(default)]
    pub weather: WeatherConfig,
    #[serde(default)]
    pub news: NewsConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherConfig {
    #[serde(default = "default_open_meteo")]
    pub api_url: String,
    #[serde(default)]
    pub location: Option<Location>,
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            api_url: default_open_meteo(),
            location: None,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Location {
    #[serde(default)]
    pub lat: f64,
    #[serde(default)]
    pub lon: f64,
}

fn default_open_meteo() -> String {
    "https://api.open-meteo.com/v1/forecast".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsConfig {
    #[serde(default = "default_feeds")]
    pub feeds: Vec<String>,
}

impl Default for NewsConfig {
    fn default() -> Self {
        Self {
            feeds: default_feeds(),
        }
    }
}

fn default_feeds() -> Vec<String> {
    vec![
        "https://hnrss.org/frontpage".to_string(),
        "https://techcrunch.com/feed/".to_string(),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_refresh_rate")]
    pub refresh_rate_ms: u64,
    #[serde(default = "default_theme")]
    pub theme: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            refresh_rate_ms: default_refresh_rate(),
            theme: default_theme(),
        }
    }
}

fn default_refresh_rate() -> u64 {
    500
}

fn default_theme() -> String {
    "default".to_string()
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

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

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let contents = toml::to_string_pretty(self)?;
        fs::write(&config_path, contents)
            .with_context(|| format!("Failed to write config at {:?}", config_path))?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        match config_dir() {
            Some(dir) => Ok(dir.join("glance").join("config.toml")),
            None => Ok(std::env::current_dir()
                .context("Failed to get current directory")?
                .join("config.toml")),
        }
    }

    fn create_default(config_path: &PathBuf) -> Result<Self> {
        let config = Config::default();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(config_path, toml::to_string_pretty(&config)?)?;
        Ok(config)
    }
}

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
}
