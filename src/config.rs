use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
    #[serde(default = "default_temp_unit")]
    pub temperature_unit: String,
    #[serde(default)]
    pub location_name: Option<String>,
}

fn default_temp_unit() -> String {
    "celsius".to_string()
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            api_url: default_open_meteo(),
            location: None,
            temperature_unit: default_temp_unit(),
            location_name: None,
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
    #[serde(default = "default_preferred_layout")]
    pub preferred_layout: String,
    #[serde(default)]
    pub nerd_font: Option<bool>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            refresh_rate_ms: default_refresh_rate(),
            theme: default_theme(),
            preferred_layout: default_preferred_layout(),
            nerd_font: None,
        }
    }
}

fn default_refresh_rate() -> u64 {
    500
}

fn default_theme() -> String {
    "matte-black".to_string()
}

fn default_preferred_layout() -> String {
    "auto".to_string()
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        #[cfg(target_os = "macos")]
        {
            if !config_path.exists() {
                if let Some(old_dir) = dirs::config_dir() {
                    let old_path = old_dir.join("glance").join("config.toml");
                    if old_path.exists() {
                        if let Some(parent) = config_path.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        fs::copy(&old_path, &config_path)?;
                        eprintln!(
                            "Notice: migrated config from {:?} to {:?}",
                            old_path, config_path
                        );
                    }
                }
            }
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

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let contents = toml::to_string_pretty(self)?;
        fs::write(&config_path, contents)
            .with_context(|| format!("Failed to write config at {:?}", config_path))?;
        Ok(())
    }

    fn config_dir() -> Result<PathBuf> {
        // Check XDG_CONFIG_HOME first (must be absolute)
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            let path = PathBuf::from(&xdg);
            if path.is_absolute() {
                return Ok(path);
            }
        }

        // Platform-specific fallback
        #[cfg(unix)]
        {
            if let Some(home) = dirs::home_dir() {
                return Ok(home.join(".config"));
            }
        }

        #[cfg(windows)]
        {
            if let Some(dir) = dirs::config_dir() {
                return Ok(dir);
            }
        }

        // Final fallback: current working directory
        std::env::current_dir().context("Failed to get current directory")
    }

    fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("glance").join("config.toml"))
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
