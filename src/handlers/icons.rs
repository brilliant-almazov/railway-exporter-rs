//! Cached icons endpoint handler.
//!
//! Serves cached service icons with browser caching headers.
//! GET /icons/services/{service_name}

use super::HandlerResponse;
use crate::state::AppState;
use hyper::body::Bytes;
use hyper::{Response, StatusCode};

/// GET /icons/services/{service_name} - Serve cached icon.
///
/// Returns the icon image with proper Content-Type and caching headers:
/// - Cache-Control: public, max-age={configured TTL}
/// - ETag: hash of service name (for cache validation)
///
/// Returns 404 if icon is not cached.
pub async fn handle(state: &AppState, service_name: &str) -> HandlerResponse {
    // Get icon from cache
    match state.icon_cache.get_raw(service_name).await {
        Some(icon) => {
            let etag = format!("\"{}\"", simple_hash(service_name));
            let cache_control = format!("public, max-age={}", state.config.icon_cache.max_age);

            (
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", &icon.content_type)
                    .header("Cache-Control", cache_control)
                    .header("ETag", etag),
                Bytes::from(icon.data),
            )
        }
        None => (
            Response::builder().status(StatusCode::NOT_FOUND),
            Bytes::from("Icon not found"),
        ),
    }
}

/// Simple hash for ETag generation.
fn simple_hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}
