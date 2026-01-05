use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

pub struct CacheEntry<T> {
    data: T,
    created_at: SystemTime,
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    pub fn is_valid(&self) -> bool {
        self.created_at.elapsed().unwrap_or(Duration::MAX) < self.ttl
    }
}

pub struct Cache<K, V> {
    data: Arc<Mutex<HashMap<K, CacheEntry<V>>>>,
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone> Cache<K, V> {
    pub fn new() -> Self {
        Cache {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let data = self.data.lock().ok()?;
        let entry = data.get(key)?;

        if entry.is_valid() {
            Some(entry.data.clone())
        } else {
            None
        }
    }

    pub fn set(&self, key: K, value: V, ttl: Duration) -> Result<(), String> {
        let mut data = self.data.lock().map_err(|e| e.to_string())?;
        data.insert(
            key,
            CacheEntry {
                data: value,
                created_at: SystemTime::now(),
                ttl,
            },
        );
        Ok(())
    }

    pub fn clear(&self) -> Result<(), String> {
        self.data.lock().map_err(|e| e.to_string())?.clear();
        Ok(())
    }
}

impl<K: std::hash::Hash + Eq, V> Clone for Cache<K, V> {
    fn clone(&self) -> Self {
        Cache {
            data: Arc::clone(&self.data),
        }
    }
}
