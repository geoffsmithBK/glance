use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Simple TTL cache for weather and news data
#[derive(Debug, Clone)]
pub struct Cache<T> {
    data: Arc<RwLock<HashMap<String, (T, SystemTime)>>>,
    ttl: Duration,
}

impl<T: Clone> Cache<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    pub fn set(&self, key: String, value: T) {
        let now = SystemTime::now();
        self.data.write().insert(key, (value, now));
    }

    pub fn get(&self) -> Option<T> {
        let now = SystemTime::now();
        let data = self.data.read();

        for (_, (value, inserted_at)) in data.iter() {
            if now.duration_since(*inserted_at).unwrap_or(Duration::ZERO) < self.ttl {
                return Some(value.clone());
            }
        }

        None
    }

    pub fn clear(&self) {
        self.data.write().clear();
    }

    pub fn is_empty(&self) -> bool {
        self.data.read().is_empty()
    }
}

impl Default for Cache<String> {
    fn default() -> Self {
        Self::new(Duration::from_secs(300)) // 5 minutes default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_ttl() {
        let cache = Cache::new(Duration::from_secs(2));

        // Set initial value
        cache.set("key".to_string(), "value".to_string());

        // Should get value immediately
        assert_eq!(cache.get(), Some("value".to_string()));

        // Wait for TTL to expire
        std::thread::sleep(Duration::from_secs(3));

        // Should be expired
        assert_eq!(cache.get(), None);
    }

    #[test]
    fn test_cache_clear() {
        let cache = Cache::new(Duration::from_secs(60));

        cache.set("key1".to_string(), "value1".to_string());
        cache.set("key2".to_string(), "value2".to_string());

        assert!(!cache.is_empty());

        cache.clear();

        assert!(cache.is_empty());
    }
}
