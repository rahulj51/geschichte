use lru::LruCache;
use std::num::NonZeroUsize;

pub struct DiffCache {
    cache: LruCache<String, String>,
}

impl DiffCache {
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(50).unwrap());
        Self {
            cache: LruCache::new(capacity),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&String> {
        self.cache.get(key)
    }

    pub fn put(&mut self, key: String, value: String) {
        self.cache.put(key, value);
    }

    #[allow(dead_code)]
    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains(key)
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}