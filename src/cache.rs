//! AST caching to avoid re-parsing unchanged files
//!
//! This module provides an LRU cache for parsed AST modules, keyed by file path
//! and modification time. This dramatically improves performance for repeated
//! parsing of the same files (e.g., in LSP, watch mode, or CI/CD pipelines).

use crate::ast::Module;
use anyhow::Result;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::PathBuf;
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
pub struct AstCache {
    cache: Mutex<LruCache<CacheKey, Arc<Module>>>,
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
        let capacity = NonZeroUsize::new(capacity).expect("Cache capacity must be greater than 0");

        Self {
            cache: Mutex::new(LruCache::new(capacity)),
        }
    }

    /// Create a new cache with default capacity (100 entries)
    pub fn with_default_capacity() -> Self {
        Self::new(100)
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
                return Ok(Arc::clone(cached_module));
            }
        }

        // Cache miss - parse the file
        let module = parse_fn(path)?;
        let module_arc = Arc::new(module);

        // Store in cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(key, Arc::clone(&module_arc));
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
        let cache = self.cache.lock().unwrap();
        cache.cap().get()
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
        let cache = AstCache::default();
        assert_eq!(cache.capacity(), 100);
        assert!(cache.is_empty());
    }

    #[test]
    #[should_panic(expected = "Cache capacity must be greater than 0")]
    fn test_zero_capacity_panics() {
        AstCache::new(0);
    }
}
