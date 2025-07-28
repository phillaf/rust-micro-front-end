use super::{User, UserDatabase};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Cache entry with TTL
#[derive(Clone)]
struct CacheEntry {
    user: Option<User>,
    expires_at: Instant,
}

/// Database caching layer for improved performance
pub struct CachedUserDatabase {
    inner: Arc<dyn UserDatabase>,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    cache_ttl: Duration,
    cache_enabled: bool,
}

impl CachedUserDatabase {
    /// Create a new cached database wrapper
    pub fn new(inner: Arc<dyn UserDatabase>, cache_ttl: Duration, cache_enabled: bool) -> Self {
        info!(
            "Database cache initialized with TTL: {:?}, enabled: {}",
            cache_ttl, cache_enabled
        );

        Self {
            inner,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
            cache_enabled,
        }
    }

    /// Get user from cache if available and not expired
    fn get_cached_user(&self, username: &str) -> Option<Option<User>> {
        if !self.cache_enabled {
            return None;
        }

        let cache = self.cache.read().ok()?;

        if let Some(entry) = cache.get(username) {
            if entry.expires_at > Instant::now() {
                debug!("Database cache hit for user: {}", username);

                // Track cache hit metric
                if let Some(metrics) = crate::router::get_metrics_instance() {
                    crate::metrics::track_cache_hit(metrics, "user_cache");
                }

                return Some(entry.user.clone());
            } else {
                debug!("Database cache entry expired for user: {}", username);

                // Expired entries are effectively misses
                if let Some(metrics) = crate::router::get_metrics_instance() {
                    crate::metrics::track_cache_miss(metrics, "user_cache");
                }
            }
        } else {
            // Track cache miss metric
            if let Some(metrics) = crate::router::get_metrics_instance() {
                crate::metrics::track_cache_miss(metrics, "user_cache");
            }
        }

        None
    }

    /// Store user in cache
    fn cache_user(&self, username: &str, user: Option<User>) {
        if !self.cache_enabled {
            return;
        }

        if let Ok(mut cache) = self.cache.write() {
            // Simple eviction strategy: clear cache if it gets too large
            if cache.len() >= 1000 {
                cache.clear();
                debug!("Database cache cleared due to size limit");
            }

            let entry = CacheEntry {
                user,
                expires_at: Instant::now() + self.cache_ttl,
            };

            cache.insert(username.to_string(), entry);
            debug!("Database cache stored for user: {}", username);
        }
    }

    /// Invalidate cache entry for a specific user
    fn invalidate_user_cache(&self, username: &str) {
        if !self.cache_enabled {
            return;
        }

        if let Ok(mut cache) = self.cache.write() {
            cache.remove(username);
            debug!("Database cache invalidated for user: {}", username);
        }
    }

    /// Clear all cache entries
    #[allow(dead_code)]
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
            info!("Database cache cleared");
        }
    }

    /// Get cache statistics
    #[allow(dead_code)]
    pub fn cache_stats(&self) -> (usize, usize) {
        if let Ok(cache) = self.cache.read() {
            let total_entries = cache.len();
            let expired_entries = cache.values().filter(|entry| entry.expires_at <= Instant::now()).count();

            (total_entries, expired_entries)
        } else {
            (0, 0)
        }
    }
}

#[async_trait]
impl UserDatabase for CachedUserDatabase {
    async fn get_user(&self, username: &str) -> Result<Option<User>> {
        // Check cache first
        if let Some(cached_user) = self.get_cached_user(username) {
            return Ok(cached_user);
        }

        // Cache miss - fetch from database
        debug!("Database cache miss for user: {}", username);
        let user = self.inner.get_user(username).await?;

        // Cache the result
        self.cache_user(username, user.clone());

        Ok(user)
    }

    async fn update_user_display_name(&self, username: &str, display_name: &str) -> Result<()> {
        // Update in database
        self.inner.update_user_display_name(username, display_name).await?;

        // Invalidate cache to ensure fresh data on next read
        self.invalidate_user_cache(username);

        Ok(())
    }

    async fn health_check(&self) -> Result<String> {
        // Health check should always go to the database
        self.inner.health_check().await
    }
}
