mod test {

    #[test]
    fn test_lru_cache_eviction() {
        let mut cache = geschichte::cache::DiffCache::new(2);
        
        cache.put("key1".to_string(), "diff1".to_string());
        cache.put("key2".to_string(), "diff2".to_string());
        cache.put("key3".to_string(), "diff3".to_string());
        
        assert!(cache.get("key1").is_none());
        assert!(cache.get("key2").is_some());
        assert!(cache.get("key3").is_some());
    }
    
    #[test]
    fn test_range_diff_caching() {
        let mut cache = geschichte::cache::DiffCache::new(10);
        let key = "abc123..def456";
        let diff = "sample diff content";
        
        cache.put(key.to_string(), diff.to_string());
        assert_eq!(cache.get(key).unwrap(), diff);
    }
    
    #[test]
    fn test_cache_hit_updates_order() {
        let mut cache = geschichte::cache::DiffCache::new(2);
        
        cache.put("key1".to_string(), "diff1".to_string());
        cache.put("key2".to_string(), "diff2".to_string());
        
        cache.get("key1");
        
        cache.put("key3".to_string(), "diff3".to_string());
        
        assert!(cache.get("key1").is_some());
        assert!(cache.get("key2").is_none());
        assert!(cache.get("key3").is_some());
    }
    
    #[test]
    fn test_cache_capacity_zero() {
        let mut cache = geschichte::cache::DiffCache::new(0);
        
        cache.put("key1".to_string(), "diff1".to_string());
        // With zero capacity, the cache should still work with minimum capacity of 1
        assert!(cache.get("key1").is_some());
    }
    
    #[test]
    fn test_cache_overwrite_same_key() {
        let mut cache = geschichte::cache::DiffCache::new(2);
        
        cache.put("key1".to_string(), "diff1".to_string());
        cache.put("key1".to_string(), "diff1_updated".to_string());
        
        assert_eq!(cache.get("key1").unwrap(), "diff1_updated");
        assert_eq!(cache.len(), 1);
    }
}