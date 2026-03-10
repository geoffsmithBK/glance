use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GeoResult {
    pub name: String,
    pub admin1: Option<String>,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl GeoResult {
    pub fn display_label(&self) -> String {
        match &self.admin1 {
            Some(region) => format!("{}, {}, {}", self.name, region, self.country),
            None => format!("{}, {}", self.name, self.country),
        }
    }
}

#[derive(Deserialize)]
struct GeoResponse {
    #[serde(default)]
    results: Vec<GeoResult>,
}

pub struct LocationSearch {
    pub query: String,
    pub results: Vec<GeoResult>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    client: Client,
    matcher: SkimMatcherV2,
}

impl LocationSearch {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            results: Vec::new(),
            filtered: Vec::new(),
            selected: 0,
            client: Client::new(),
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn push_char(&mut self, c: char) {
        self.query.push(c);
        self.update_filter();
    }

    pub fn pop_char(&mut self) {
        self.query.pop();
        self.update_filter();
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.filtered.is_empty() && self.selected < self.filtered.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn selected_result(&self) -> Option<&GeoResult> {
        let &idx = self.filtered.get(self.selected)?;
        self.results.get(idx)
    }

    pub async fn fetch(&mut self) {
        if self.query.is_empty() {
            self.results.clear();
            self.update_filter();
            return;
        }

        let encoded = urlencoding(&self.query);
        let url = format!(
            "https://geocoding-api.open-meteo.com/v1/search?name={}&count=10&language=en",
            encoded
        );

        match self.client.get(&url).send().await {
            Ok(resp) => {
                if let Ok(geo) = resp.json::<GeoResponse>().await {
                    self.results = geo.results;
                }
            }
            Err(_) => {}
        }

        self.update_filter();
    }

    fn update_filter(&mut self) {
        if self.query.is_empty() {
            self.filtered = (0..self.results.len()).collect();
        } else {
            let mut scored: Vec<(usize, i64)> = self
                .results
                .iter()
                .enumerate()
                .filter_map(|(i, r)| {
                    self.matcher
                        .fuzzy_match(&r.display_label(), &self.query)
                        .map(|score| (i, score))
                })
                .collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered = scored.into_iter().map(|(i, _)| i).collect();
        }

        if self.selected >= self.filtered.len() && !self.filtered.is_empty() {
            self.selected = self.filtered.len() - 1;
        } else if self.filtered.is_empty() {
            self.selected = 0;
        }
    }
}

fn urlencoding(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => {
                result.push_str("%20");
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_result_display_label() {
        let result = GeoResult {
            name: "Austin".to_string(),
            admin1: Some("Texas".to_string()),
            country: "United States".to_string(),
            latitude: 30.2672,
            longitude: -97.7431,
        };
        assert_eq!(result.display_label(), "Austin, Texas, United States");
    }

    #[test]
    fn test_geo_result_display_no_region() {
        let result = GeoResult {
            name: "Singapore".to_string(),
            admin1: None,
            country: "Singapore".to_string(),
            latitude: 1.3521,
            longitude: 103.8198,
        };
        assert_eq!(result.display_label(), "Singapore, Singapore");
    }

    #[test]
    fn test_location_search_navigation() {
        let mut search = LocationSearch::new();
        search.results = vec![
            GeoResult {
                name: "A".to_string(),
                admin1: None,
                country: "X".to_string(),
                latitude: 0.0,
                longitude: 0.0,
            },
            GeoResult {
                name: "B".to_string(),
                admin1: None,
                country: "Y".to_string(),
                latitude: 0.0,
                longitude: 0.0,
            },
        ];
        search.filtered = vec![0, 1];
        search.selected = 0;

        // Move down
        search.move_down();
        assert_eq!(search.selected, 1);

        // Can't go past end
        search.move_down();
        assert_eq!(search.selected, 1);

        // Move up
        search.move_up();
        assert_eq!(search.selected, 0);

        // Can't go below 0
        search.move_up();
        assert_eq!(search.selected, 0);
    }

    #[test]
    fn test_location_search_selected_result() {
        let mut search = LocationSearch::new();

        // Empty — should return None
        assert!(search.selected_result().is_none());

        // Add a result
        search.results = vec![GeoResult {
            name: "London".to_string(),
            admin1: Some("England".to_string()),
            country: "United Kingdom".to_string(),
            latitude: 51.5074,
            longitude: -0.1278,
        }];
        search.filtered = vec![0];
        search.selected = 0;

        let result = search.selected_result().unwrap();
        assert_eq!(result.name, "London");
    }

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("hello world"), "hello%20world");
        assert_eq!(urlencoding("New York"), "New%20York");
    }

    #[test]
    fn test_push_pop_char() {
        let mut search = LocationSearch::new();
        search.push_char('a');
        search.push_char('b');
        assert_eq!(search.query, "ab");

        search.pop_char();
        assert_eq!(search.query, "a");

        search.pop_char();
        assert_eq!(search.query, "");

        // Pop on empty should not panic
        search.pop_char();
        assert_eq!(search.query, "");
    }
}
