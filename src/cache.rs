//! AST caching to avoid re-parsing unchanged files
//!
//! This module provides an LRU cache for parsed AST modules, keyed by file path
//! and modification time. This dramatically improves performance for repeated
//! parsing of the same files (e.g., in LSP, watch mode, or CI/CD pipelines).
//!
//! # Configuration
//!
//! Cache size can be configured via the `JCL_CACHE_SIZE` environment variable:
//!
//! ```bash
//! # Use larger cache for CI/CD
//! export JCL_CACHE_SIZE=5000
//!
//! # Disable cache for debugging
//! export JCL_CACHE_SIZE=0
//! ```

use crate::ast::Module;
use anyhow::Result;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Cache key combining file path and modification time
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Canonicalized file path
    pub path: PathBuf,
    /// File modification time (for invalidation)
    pub mtime: SystemTime,
}

impl CacheKey {
    /// Create a new cache key from a file path
    ///
    /// Returns None if the file doesn't exist or metadata can't be read
    pub fn from_path(path: &std::path::Path) -> Result<Self> {
        let canonical_path = path.canonicalize()?;
        let metadata = std::fs::metadata(&canonical_path)?;
        let mtime = metadata.modified()?;

        Ok(CacheKey {
            path: canonical_path,
            mtime,
        })
    }
}

/// Cache metrics for observability
///
/// Tracks cache hits, misses, and evictions to help tune cache size
/// and understand cache effectiveness.
#[derive(Debug, Default)]
pub struct CacheMetrics {
    /// Number of cache hits
    pub hits: AtomicU64,
    /// Number of cache misses
    pub misses: AtomicU64,
    /// Number of cache evictions (LRU)
    pub evictions: AtomicU64,
}

impl CacheMetrics {
    /// Calculate cache hit rate as a percentage (0.0 to 1.0)
    ///
    /// Returns 0.0 if no requests have been made yet.
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Reset all metrics to zero
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
    }

    /// Clone the current metrics values (for reporting)
    pub fn snapshot(&self) -> CacheMetricsSnapshot {
        CacheMetricsSnapshot {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of cache metrics at a point in time
#[derive(Debug, Clone, Copy)]
pub struct CacheMetricsSnapshot {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

impl CacheMetricsSnapshot {
    /// Calculate hit rate as a percentage (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Total number of cache requests
    pub fn total_requests(&self) -> u64 {
        self.hits + self.misses
    }
}

/// Thread-safe LRU cache for parsed AST modules
///
/// The cache uses `Arc<Module>` to allow cheap cloning of cached ASTs.
/// The entire cache is wrapped in a `Mutex` to ensure thread safety.
///
/// # Example
///
/// ```
/// use jcl::cache::AstCache;
/// use std::path::Path;
/// use std::sync::Arc;
/// use tempfile::NamedTempFile;
/// use std::io::Write;
///
/// // Create a temporary JCL file
/// let mut temp_file = NamedTempFile::new().unwrap();
/// writeln!(temp_file, "x = 42").unwrap();
/// temp_file.flush().unwrap();
///
/// let cache = AstCache::new(100);
///
/// // First parse (cache miss)
/// let module1 = cache.get_or_parse(temp_file.path(), |path| {
///     jcl::parse_file(path)
/// }).unwrap();
///
/// // Second parse (cache hit - near instant)
/// let module2 = cache.get_or_parse(temp_file.path(), |path| {
///     jcl::parse_file(path)
/// }).unwrap();
///
/// // Both modules point to the same underlying AST
/// assert!(Arc::ptr_eq(&module1, &module2));
/// ```
#[derive(Clone)]
pub struct AstCache {
    cache: Arc<Mutex<LruCache<CacheKey, Arc<Module>>>>,
    metrics: Arc<CacheMetrics>,
    capacity: Arc<AtomicUsize>,
}

impl AstCache {
    /// Create a new cache with the specified capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of ASTs to cache (LRU eviction)
    ///
    /// # Panics
    ///
    /// Panics if capacity is 0
    pub fn new(capacity: usize) -> Self {
        let capacity_val =
            NonZeroUsize::new(capacity).expect("Cache capacity must be greater than 0");

        Self {
            cache: Arc::new(Mutex::new(LruCache::new(capacity_val))),
            metrics: Arc::new(CacheMetrics::default()),
            capacity: Arc::new(AtomicUsize::new(capacity)),
        }
    }

    /// Create a new cache with default capacity (1000 entries)
    ///
    /// Can be overridden with `JCL_CACHE_SIZE` environment variable
    pub fn with_default_capacity() -> Self {
        let capacity = std::env::var("JCL_CACHE_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000);

        Self::new(capacity)
    }

    /// Get a cached AST or parse the file if not cached
    ///
    /// This is the primary method for interacting with the cache.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JCL file
    /// * `parse_fn` - Function to parse the file on cache miss
    ///
    /// # Returns
    ///
    /// Returns `Arc<Module>` on success, or an error if:
    /// - File doesn't exist or can't be read
    /// - Parsing fails
    ///
    /// # Cache Behavior
    ///
    /// - **Cache Hit**: Returns existing AST in <1µs (cheap Arc clone)
    /// - **Cache Miss**: Parses file and stores in cache
    /// - **Invalidation**: Automatic if file modification time changes
    pub fn get_or_parse<F>(&self, path: &std::path::Path, parse_fn: F) -> Result<Arc<Module>>
    where
        F: FnOnce(&std::path::Path) -> Result<Module>,
    {
        // Create cache key from path + mtime
        let key = CacheKey::from_path(path)?;

        // Try to get from cache
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached_module) = cache.get(&key) {
                // Cache hit!
                self.metrics.hits.fetch_add(1, Ordering::Relaxed);
                return Ok(Arc::clone(cached_module));
            }
        }

        // Cache miss - parse the file
        self.metrics.misses.fetch_add(1, Ordering::Relaxed);
        let module = parse_fn(path)?;
        let module_arc = Arc::new(module);

        // Store in cache
        {
            let mut cache = self.cache.lock().unwrap();
            let old_len = cache.len();
            cache.put(key, Arc::clone(&module_arc));

            // Track eviction if cache was full
            if old_len == cache.cap().get() && cache.len() == cache.cap().get() {
                self.metrics.evictions.fetch_add(1, Ordering::Relaxed);
            }
        }

        Ok(module_arc)
    }

    /// Get a cached AST without parsing
    ///
    /// Returns None if the file is not in cache or has been modified
    pub fn get(&self, path: &std::path::Path) -> Option<Arc<Module>> {
        let key = CacheKey::from_path(path).ok()?;
        let mut cache = self.cache.lock().unwrap();
        cache.get(&key).cloned()
    }

    /// Clear all cached ASTs
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// Get cache metrics
    pub fn metrics(&self) -> &CacheMetrics {
        &self.metrics
    }

    /// Get a snapshot of current metrics
    pub fn metrics_snapshot(&self) -> CacheMetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Reset cache metrics
    pub fn reset_metrics(&self) {
        self.metrics.reset();
    }

    /// Get the number of cached ASTs
    pub fn len(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        let cache = self.cache.lock().unwrap();
        cache.is_empty()
    }

    /// Get cache capacity
    pub fn capacity(&self) -> usize {
        self.capacity.load(Ordering::Relaxed)
    }
}

impl Default for AstCache {
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::thread;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    #[test]
    fn test_cache_key_from_path() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "x = 42").unwrap();
        temp_file.flush().unwrap();

        let key1 = CacheKey::from_path(temp_file.path()).unwrap();
        let key2 = CacheKey::from_path(temp_file.path()).unwrap();

        // Same file, same mtime -> equal keys
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_invalidation_on_modification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "x = 42").unwrap();
        temp_file.flush().unwrap();

        let key1 = CacheKey::from_path(temp_file.path()).unwrap();

        // Wait to ensure mtime changes
        thread::sleep(Duration::from_millis(10));

        // Modify file
        writeln!(temp_file, "y = 100").unwrap();
        temp_file.flush().unwrap();

        let key2 = CacheKey::from_path(temp_file.path()).unwrap();

        // Different mtime -> different keys
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cache_hit() {
        let cache = AstCache::new(10);
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "x = 42").unwrap();
        temp_file.flush().unwrap();

        let path = temp_file.path();

        // First access - cache miss
        let module1 = cache.get_or_parse(path, |p| crate::parse_file(p)).unwrap();

        // Second access - cache hit
        let module2 = cache.get_or_parse(path, |p| crate::parse_file(p)).unwrap();

        // Should return the same Arc (pointer equality)
        assert!(Arc::ptr_eq(&module1, &module2));
    }

    #[test]
    fn test_cache_invalidation_on_file_change() {
        let cache = AstCache::new(10);
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "x = 42").unwrap();
        temp_file.flush().unwrap();

        let path = temp_file.path().to_path_buf();

        // First parse
        let module1 = cache.get_or_parse(&path, |p| crate::parse_file(p)).unwrap();

        assert_eq!(cache.len(), 1);

        // Wait and modify file
        thread::sleep(Duration::from_millis(10));
        writeln!(temp_file, "y = 100").unwrap();
        temp_file.flush().unwrap();

        // Second parse after modification - should be cache miss
        let module2 = cache.get_or_parse(&path, |p| crate::parse_file(p)).unwrap();

        // Different Arc instances (different parse)
        assert!(!Arc::ptr_eq(&module1, &module2));

        // Cache now has 2 entries (old and new)
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_lru_eviction() {
        let cache = AstCache::new(2); // Small capacity

        // Create 3 temp files
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        let mut file3 = NamedTempFile::new().unwrap();

        writeln!(file1, "x = 1").unwrap();
        writeln!(file2, "x = 2").unwrap();
        writeln!(file3, "x = 3").unwrap();

        file1.flush().unwrap();
        file2.flush().unwrap();
        file3.flush().unwrap();

        // Parse all 3 files
        cache
            .get_or_parse(file1.path(), |p| crate::parse_file(p))
            .unwrap();
        cache
            .get_or_parse(file2.path(), |p| crate::parse_file(p))
            .unwrap();
        cache
            .get_or_parse(file3.path(), |p| crate::parse_file(p))
            .unwrap();

        // Cache should only contain 2 entries (file1 was evicted)
        assert_eq!(cache.len(), 2);

        // file1 should be evicted
        assert!(cache.get(file1.path()).is_none());

        // file2 and file3 should still be cached
        assert!(cache.get(file2.path()).is_some());
        assert!(cache.get(file3.path()).is_some());
    }

    #[test]
    fn test_cache_clear() {
        let cache = AstCache::new(10);
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "x = 42").unwrap();
        temp_file.flush().unwrap();

        cache
            .get_or_parse(temp_file.path(), |p| crate::parse_file(p))
            .unwrap();

        assert_eq!(cache.len(), 1);

        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_capacity() {
        let cache = AstCache::new(50);
        assert_eq!(cache.capacity(), 50);
    }

    #[test]
    fn test_default_cache() {
        // Unset env var for this test
        std::env::remove_var("JCL_CACHE_SIZE");
        let cache = AstCache::default();
        assert_eq!(cache.capacity(), 1000);
        assert!(cache.is_empty());
    }

    #[test]
    #[should_panic(expected = "Cache capacity must be greater than 0")]
    fn test_zero_capacity_panics() {
        AstCache::new(0);
    }

    #[test]
    fn test_cache_metrics_hits_and_misses() {
        let cache = AstCache::new(10);
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "x = 42").unwrap();
        temp_file.flush().unwrap();

        let path = temp_file.path();

        // First access - cache miss
        cache.get_or_parse(path, |p| crate::parse_file(p)).unwrap();

        let metrics = cache.metrics_snapshot();
        assert_eq!(metrics.hits, 0);
        assert_eq!(metrics.misses, 1);
        assert_eq!(metrics.hit_rate(), 0.0);

        // Second access - cache hit
        cache.get_or_parse(path, |p| crate::parse_file(p)).unwrap();

        let metrics = cache.metrics_snapshot();
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 1);
        assert_eq!(metrics.hit_rate(), 0.5);

        // Third access - cache hit
        cache.get_or_parse(path, |p| crate::parse_file(p)).unwrap();

        let metrics = cache.metrics_snapshot();
        assert_eq!(metrics.hits, 2);
        assert_eq!(metrics.misses, 1);
        assert!((metrics.hit_rate() - 0.6666).abs() < 0.01);
    }

    #[test]
    fn test_cache_metrics_evictions() {
        let cache = AstCache::new(2); // Small capacity

        // Create 3 temp files
        let mut file1 = NamedTempFile::new().unwrap();
        let mut file2 = NamedTempFile::new().unwrap();
        let mut file3 = NamedTempFile::new().unwrap();

        writeln!(file1, "x = 1").unwrap();
        writeln!(file2, "x = 2").unwrap();
        writeln!(file3, "x = 3").unwrap();

        file1.flush().unwrap();
        file2.flush().unwrap();
        file3.flush().unwrap();

        // Parse first two files (fills cache)
        cache
            .get_or_parse(file1.path(), |p| crate::parse_file(p))
            .unwrap();
        cache
            .get_or_parse(file2.path(), |p| crate::parse_file(p))
            .unwrap();

        let metrics = cache.metrics_snapshot();
        assert_eq!(metrics.evictions, 0);

        // Parse third file (causes eviction)
        cache
            .get_or_parse(file3.path(), |p| crate::parse_file(p))
            .unwrap();

        let metrics = cache.metrics_snapshot();
        assert_eq!(metrics.evictions, 1);
    }

    #[test]
    fn test_cache_metrics_reset() {
        let cache = AstCache::new(10);
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "x = 42").unwrap();
        temp_file.flush().unwrap();

        // Generate some metrics
        cache
            .get_or_parse(temp_file.path(), |p| crate::parse_file(p))
            .unwrap();
        cache
            .get_or_parse(temp_file.path(), |p| crate::parse_file(p))
            .unwrap();

        let metrics = cache.metrics_snapshot();
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 1);

        // Reset metrics
        cache.reset_metrics();

        let metrics = cache.metrics_snapshot();
        assert_eq!(metrics.hits, 0);
        assert_eq!(metrics.misses, 0);
    }

    #[test]
    fn test_env_var_cache_size() {
        // Save current env var state
        let original = std::env::var("JCL_CACHE_SIZE").ok();

        // Set env var
        std::env::set_var("JCL_CACHE_SIZE", "500");
        let cache = AstCache::with_default_capacity();
        assert_eq!(cache.capacity(), 500);

        // Restore original state
        match original {
            Some(val) => std::env::set_var("JCL_CACHE_SIZE", val),
            None => std::env::remove_var("JCL_CACHE_SIZE"),
        }
    }
}
