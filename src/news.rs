use crate::config::NewsConfig;
use crate::utils::cache::Cache;
use chrono::{DateTime, Utc};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Headline {
    pub title: String,
    pub summary: String,
    pub link: String,
    pub published: Option<DateTime<Utc>>,
}

impl Default for Headline {
    fn default() -> Self {
        Self {
            title: String::new(),
            summary: String::new(),
            link: String::new(),
            published: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewsData {
    pub headlines: Vec<Headline>,
}

impl Default for NewsData {
    fn default() -> Self {
        Self {
            headlines: Vec::new(),
        }
    }
}

pub struct NewsService {
    config: NewsConfig,
    cache: Cache<NewsData>,
}

impl NewsService {
    pub fn new(config: NewsConfig) -> Self {
        Self {
            config,
            cache: Cache::new(Duration::from_secs(300)),
        }
    }

    pub async fn fetch(&self) -> NewsData {
        if let Some(cached) = self.cache.get() {
            return cached;
        }

        let mut all_headlines: Vec<Headline> = Vec::new();

        for feed_url in &self.config.feeds {
            match self.fetch_feed(feed_url).await {
                Ok(feed) => {
                    all_headlines.extend(self.parse_headlines(feed));
                }
                Err(_) => {}
            }
        }

        // Sort by published date (newest first) and limit to top 10
        all_headlines.sort_by(|a, b| b.published.cmp(&a.published));
        all_headlines.truncate(10);

        let news_data = NewsData {
            headlines: all_headlines,
        };
        self.cache.set("news".to_string(), news_data.clone());
        news_data
    }

    async fn fetch_feed(
        &self,
        url: &str,
    ) -> Result<rss::Channel, Box<dyn std::error::Error + Send + Sync>> {
        let response = reqwest::get(url).await?;
        let content = response.text().await?;
        Ok(rss::Channel::read_from(content.as_bytes())?)
    }

    fn parse_headlines(&self, feed: rss::Channel) -> Vec<Headline> {
        feed.items()
            .iter()
            .map(|item| {
                let title = item.title().unwrap_or("").to_string();
                let link = item.link().unwrap_or("").to_string();

                let summary = item
                    .description()
                    .map(|s| strip_html_tags(s))
                    .unwrap_or_default();

                let published = item
                    .pub_date()
                    .and_then(|dt| DateTime::parse_from_rfc2822(dt).ok())
                    .map(|dt| dt.with_timezone(&Utc));

                Headline {
                    title,
                    summary,
                    link,
                    published,
                }
            })
            .collect()
    }
}

/// Simple HTML tag stripper
fn strip_html_tags(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    result.trim().to_string()
}
