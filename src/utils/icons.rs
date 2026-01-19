//! Icon caching service.
//!
//! Fetches icons from URLs and converts them to Base64 data URLs.
//! Uses LRU cache to limit memory usage - stores raw bytes, encodes on demand.
//!
//! Cache capacity is configured via `icon_cache.max_count` in config.yaml.

use base64::{engine::general_purpose::STANDARD, Engine};
use lru::LruCache;
use reqwest::Client;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// Cached icon data: content type and raw bytes.
#[derive(Clone)]
pub struct CachedIcon {
    pub content_type: String,
    pub data: Vec<u8>,
}

impl CachedIcon {
    /// Converts to Base64 data URL.
    fn to_data_url(&self) -> String {
        let base64_data = STANDARD.encode(&self.data);
        format!("data:{};base64,{}", self.content_type, base64_data)
    }

    /// Returns the size in bytes (for monitoring).
    #[allow(dead_code)]
    fn size_bytes(&self) -> usize {
        self.content_type.len() + self.data.len()
    }
}

/// Icon cache service - fetches and caches icons as raw bytes.
/// Uses LRU eviction to limit memory usage.
pub struct IconCache {
    /// LRU cache: service_name -> (content_type, raw bytes)
    cache: Mutex<LruCache<String, CachedIcon>>,
    /// HTTP client for fetching icons
    client: Client,
}

impl IconCache {
    /// Creates a new icon cache with specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of icons to cache. Must be > 0.
    ///
    /// # Panics
    ///
    /// Panics if capacity is 0.
    pub fn new(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity).expect("IconCache capacity must be > 0");
        Self {
            cache: Mutex::new(LruCache::new(cap)),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Gets icon as Base64 data URL, fetching and caching if needed.
    ///
    /// Returns the data URL if successful, or the original URL as fallback.
    pub async fn get_icon(&self, service_name: &str, icon_url: &str) -> String {
        // Empty URL -> return empty
        if icon_url.is_empty() {
            return String::new();
        }

        // Already a data URL -> return as-is
        if icon_url.starts_with("data:") {
            return icon_url.to_string();
        }

        // Check cache first
        {
            let mut cache = self.cache.lock().await;
            if let Some(cached) = cache.get(service_name) {
                return cached.to_data_url();
            }
        }

        // Fetch and convert
        match self.fetch_icon(icon_url).await {
            Ok(icon) => {
                let data_url = icon.to_data_url();
                // Cache it
                let mut cache = self.cache.lock().await;
                cache.put(service_name.to_string(), icon);
                debug!(
                    service = service_name,
                    "Cached icon ({} bytes)",
                    data_url.len()
                );
                data_url
            }
            Err(e) => {
                warn!(service = service_name, error = %e, "Failed to fetch icon, using URL");
                // Return original URL as fallback
                icon_url.to_string()
            }
        }
    }

    /// Fetches icon from URL and returns raw bytes with content type.
    async fn fetch_icon(&self, url: &str) -> Result<CachedIcon, String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("HTTP error: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()));
        }

        // Get content type (default to image/png)
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/png")
            .to_string();

        // Get bytes
        let data = response
            .bytes()
            .await
            .map_err(|e| format!("Body read error: {}", e))?
            .to_vec();

        Ok(CachedIcon { content_type, data })
    }

    /// Pre-fetches icons for multiple services in parallel.
    pub async fn prefetch_icons(&self, services: &[(String, String)]) {
        use futures_util::future::join_all;

        let futures: Vec<_> = services
            .iter()
            .filter(|(_, url)| !url.is_empty() && !url.starts_with("data:"))
            .map(|(name, url)| async move {
                self.get_icon(name, url).await;
            })
            .collect();

        join_all(futures).await;
    }

    /// Gets raw icon data from cache (for serving via /static/icons endpoint).
    /// Returns None if not cached - caller should trigger fetch first.
    pub async fn get_raw(&self, service_name: &str) -> Option<CachedIcon> {
        let mut cache = self.cache.lock().await;
        cache.get(service_name).cloned()
    }

    /// Ensures icon is cached, fetching if needed. Returns true if cached/fetched successfully.
    pub async fn ensure_cached(&self, service_name: &str, icon_url: &str) -> bool {
        // Already cached?
        {
            let cache = self.cache.lock().await;
            if cache.contains(service_name) {
                return true;
            }
        }

        // Try to fetch
        if icon_url.is_empty() || icon_url.starts_with("data:") {
            return false;
        }

        match self.fetch_icon(icon_url).await {
            Ok(icon) => {
                let mut cache = self.cache.lock().await;
                cache.put(service_name.to_string(), icon);
                true
            }
            Err(_) => false,
        }
    }

    /// Returns the number of cached icons.
    pub async fn cache_size(&self) -> usize {
        self.cache.lock().await.len()
    }

    /// Returns estimated memory usage in bytes.
    pub async fn memory_usage(&self) -> usize {
        let cache = self.cache.lock().await;
        cache
            .iter()
            .map(|(k, v)| k.len() + v.content_type.len() + v.data.len())
            .sum()
    }

    /// Returns cache statistics: count, total bytes, min/max/median/avg icon size.
    pub async fn stats(&self) -> IconCacheStats {
        let cache = self.cache.lock().await;

        if cache.is_empty() {
            return IconCacheStats::default();
        }

        let mut sizes: Vec<usize> = cache.iter().map(|(_, v)| v.data.len()).collect();
        sizes.sort_unstable();

        let count = sizes.len();
        let total_bytes: usize = sizes.iter().sum();
        let min_bytes = *sizes.first().unwrap_or(&0);
        let max_bytes = *sizes.last().unwrap_or(&0);
        let avg_bytes = total_bytes / count;

        // Median
        let median_bytes = if count % 2 == 0 {
            (sizes[count / 2 - 1] + sizes[count / 2]) / 2
        } else {
            sizes[count / 2]
        };

        IconCacheStats {
            count,
            total_bytes,
            min_bytes,
            max_bytes,
            median_bytes,
            avg_bytes,
        }
    }
}

/// Icon cache statistics.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct IconCacheStats {
    /// Number of cached icons.
    pub count: usize,
    /// Total cache size in bytes.
    pub total_bytes: usize,
    /// Minimum icon size in bytes.
    pub min_bytes: usize,
    /// Maximum icon size in bytes.
    pub max_bytes: usize,
    /// Median icon size in bytes.
    pub median_bytes: usize,
    /// Average icon size in bytes.
    pub avg_bytes: usize,
}

/// Shared icon cache type.
pub type SharedIconCache = Arc<IconCache>;

/// Creates a new shared icon cache with specified capacity.
///
/// # Arguments
///
/// * `capacity` - Maximum number of icons to cache (LRU eviction when exceeded)
pub fn create_icon_cache(capacity: usize) -> SharedIconCache {
    Arc::new(IconCache::new(capacity))
}
