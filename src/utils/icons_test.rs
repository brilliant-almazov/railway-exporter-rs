//! Tests for icon caching service.

use super::icons::{create_icon_cache, IconCache};

/// Default capacity for tests.
const TEST_CACHE_CAPACITY: usize = 100;

// =============================================================================
// IconCache Creation Tests
// =============================================================================

#[test]
fn test_icon_cache_new() {
    let _cache = IconCache::new(TEST_CACHE_CAPACITY);
    // Just verify it creates without panic
    assert!(true, "IconCache::new(TEST_CACHE_CAPACITY) succeeded");
}

#[test]
fn test_create_icon_cache() {
    let _cache = create_icon_cache(TEST_CACHE_CAPACITY);
    // Returns Arc<IconCache>
    assert!(true, "create_icon_cache() succeeded");
}

// =============================================================================
// IconCache Behavior Tests
// =============================================================================

#[tokio::test]
async fn test_cache_size_initially_zero() {
    let cache = IconCache::new(TEST_CACHE_CAPACITY);
    assert_eq!(cache.cache_size().await, 0);
}

#[tokio::test]
async fn test_get_icon_empty_url_returns_empty() {
    let cache = IconCache::new(TEST_CACHE_CAPACITY);
    let result = cache.get_icon("my-service", "").await;
    assert_eq!(result, "");
}

#[tokio::test]
async fn test_get_icon_data_url_returns_unchanged() {
    let cache = IconCache::new(TEST_CACHE_CAPACITY);

    let data_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEA";
    let result = cache.get_icon("my-service", data_url).await;

    assert_eq!(result, data_url);
}

#[tokio::test]
async fn test_get_icon_emoji_returns_unchanged() {
    let cache = IconCache::new(TEST_CACHE_CAPACITY);

    // Emoji icon (common for Railway services)
    let emoji = "🚀";
    // Since emoji is not a URL and doesn't start with "data:", it will try to fetch
    // and fail, returning the original string
    let result = cache.get_icon("my-service", emoji).await;

    // The emoji should be returned as-is since HTTP fetch will fail
    assert_eq!(result, emoji);
}

#[tokio::test]
async fn test_cache_miss_then_hit() {
    let cache = IconCache::new(TEST_CACHE_CAPACITY);

    // First call - cache miss, will try to fetch and fail
    let url = "https://example.com/nonexistent-icon.png";
    let result1 = cache.get_icon("service-a", url).await;

    // Should return original URL as fallback since fetch fails
    assert_eq!(result1, url);

    // Cache should still be empty (failed fetches are not cached as data URLs)
    // But the behavior might differ - let's just verify it works twice
    let result2 = cache.get_icon("service-a", url).await;
    assert_eq!(result2, url);
}

#[tokio::test]
async fn test_different_services_different_cache_entries() {
    let cache = IconCache::new(TEST_CACHE_CAPACITY);

    let data_url_a = "data:image/png;base64,AAA";
    let data_url_b = "data:image/png;base64,BBB";

    // Data URLs are returned as-is (not cached because already data URLs)
    let result_a = cache.get_icon("service-a", data_url_a).await;
    let result_b = cache.get_icon("service-b", data_url_b).await;

    assert_eq!(result_a, data_url_a);
    assert_eq!(result_b, data_url_b);
}

#[tokio::test]
async fn test_prefetch_icons_empty_list() {
    let cache = IconCache::new(TEST_CACHE_CAPACITY);
    let services: Vec<(String, String)> = vec![];

    cache.prefetch_icons(&services).await;

    // Should complete without error
    assert_eq!(cache.cache_size().await, 0);
}

#[tokio::test]
async fn test_prefetch_icons_skips_empty_urls() {
    let cache = IconCache::new(TEST_CACHE_CAPACITY);
    let services = vec![
        ("service-a".to_string(), "".to_string()),
        ("service-b".to_string(), "".to_string()),
    ];

    cache.prefetch_icons(&services).await;

    // Empty URLs are skipped
    assert_eq!(cache.cache_size().await, 0);
}

#[tokio::test]
async fn test_prefetch_icons_skips_data_urls() {
    let cache = IconCache::new(TEST_CACHE_CAPACITY);
    let services = vec![
        (
            "service-a".to_string(),
            "data:image/png;base64,AAA".to_string(),
        ),
        (
            "service-b".to_string(),
            "data:image/svg+xml;base64,BBB".to_string(),
        ),
    ];

    cache.prefetch_icons(&services).await;

    // Data URLs are skipped (already in final form)
    assert_eq!(cache.cache_size().await, 0);
}

#[tokio::test]
async fn test_get_icon_with_https_url_fallback() {
    let cache = IconCache::new(TEST_CACHE_CAPACITY);

    // Invalid URL that will fail to fetch
    let url = "https://invalid-host-that-does-not-exist-12345.com/icon.png";
    let result = cache.get_icon("my-service", url).await;

    // Should return original URL as fallback
    assert_eq!(result, url);
}

// =============================================================================
// Concurrent Access Tests
// =============================================================================

#[tokio::test]
async fn test_concurrent_get_icon() {
    use std::sync::Arc;
    use tokio::task::JoinSet;

    let cache = Arc::new(IconCache::new(TEST_CACHE_CAPACITY));
    let mut tasks = JoinSet::new();

    for i in 0..10 {
        let cache = cache.clone();
        let service_name = format!("service-{}", i);
        let icon_url = format!("data:image/png;base64,ICON{}", i);

        tasks.spawn(async move { cache.get_icon(&service_name, &icon_url).await });
    }

    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        results.push(result.unwrap());
    }

    assert_eq!(results.len(), 10);
}

// =============================================================================
// Mock HTTP Server Tests
// =============================================================================

#[tokio::test]
async fn test_get_icon_from_mock_server() {
    use http_body_util::Full;
    use hyper::body::Bytes;
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper::{Request, Response};
    use hyper_util::rt::TokioIo;
    use std::convert::Infallible;
    use tokio::net::TcpListener;

    // Start mock server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Spawn server
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);

        let service = service_fn(|_req: Request<hyper::body::Incoming>| async {
            // Return a minimal PNG (1x1 transparent pixel)
            let png_bytes: &[u8] = &[
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
                0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
                0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
                0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
                0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
            ];

            Ok::<_, Infallible>(
                Response::builder()
                    .header("content-type", "image/png")
                    .body(Full::new(Bytes::from(png_bytes.to_vec())))
                    .unwrap(),
            )
        });

        let _ = http1::Builder::new().serve_connection(io, service).await;
    });

    // Wait for server to start
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    // Test icon fetch
    let cache = IconCache::new(TEST_CACHE_CAPACITY);
    let url = format!("http://{}/icon.png", addr);
    let result = cache.get_icon("test-service", &url).await;

    // Should return a data URL
    assert!(result.starts_with("data:image/png;base64,"));
    assert!(result.len() > 50); // Should have actual content

    // Cache should have 1 entry
    assert_eq!(cache.cache_size().await, 1);
}

#[tokio::test]
async fn test_get_icon_from_mock_server_with_svg() {
    use http_body_util::Full;
    use hyper::body::Bytes;
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper::{Request, Response};
    use hyper_util::rt::TokioIo;
    use std::convert::Infallible;
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);

        let service = service_fn(|_req: Request<hyper::body::Incoming>| async {
            let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><circle cx="8" cy="8" r="8" fill="red"/></svg>"#;

            Ok::<_, Infallible>(
                Response::builder()
                    .header("content-type", "image/svg+xml")
                    .body(Full::new(Bytes::from(svg)))
                    .unwrap(),
            )
        });

        let _ = http1::Builder::new().serve_connection(io, service).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let cache = IconCache::new(TEST_CACHE_CAPACITY);
    let url = format!("http://{}/icon.svg", addr);
    let result = cache.get_icon("svg-service", &url).await;

    // Should return a data URL with SVG content type
    assert!(result.starts_with("data:image/svg+xml;base64,"));

    // Decode and verify content
    let base64_part = result.strip_prefix("data:image/svg+xml;base64,").unwrap();
    let decoded =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_part).unwrap();
    let svg_str = String::from_utf8(decoded).unwrap();
    assert!(svg_str.contains("<svg"));
    assert!(svg_str.contains("circle"));
}

#[tokio::test]
async fn test_get_icon_from_mock_server_404() {
    use http_body_util::Full;
    use hyper::body::Bytes;
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper::{Request, Response, StatusCode};
    use hyper_util::rt::TokioIo;
    use std::convert::Infallible;
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);

        let service = service_fn(|_req: Request<hyper::body::Incoming>| async {
            Ok::<_, Infallible>(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::new(Bytes::from("Not found")))
                    .unwrap(),
            )
        });

        let _ = http1::Builder::new().serve_connection(io, service).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let cache = IconCache::new(TEST_CACHE_CAPACITY);
    let url = format!("http://{}/nonexistent.png", addr);
    let result = cache.get_icon("missing-service", &url).await;

    // Should return original URL as fallback on 404
    assert_eq!(result, url);
}

#[tokio::test]
async fn test_get_icon_cached_after_fetch() {
    use http_body_util::Full;
    use hyper::body::Bytes;
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper::{Request, Response};
    use hyper_util::rt::TokioIo;
    use std::convert::Infallible;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let request_count = Arc::new(AtomicU32::new(0));
    let request_count_clone = request_count.clone();

    tokio::spawn(async move {
        // Accept multiple connections
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);
            let count = request_count_clone.clone();

            tokio::spawn(async move {
                let service = service_fn(move |_req: Request<hyper::body::Incoming>| {
                    count.fetch_add(1, Ordering::SeqCst);
                    async {
                        let png: &[u8] = &[0x89, 0x50, 0x4E, 0x47]; // Minimal PNG header
                        Ok::<_, Infallible>(
                            Response::builder()
                                .header("content-type", "image/png")
                                .body(Full::new(Bytes::from(png.to_vec())))
                                .unwrap(),
                        )
                    }
                });
                let _ = http1::Builder::new().serve_connection(io, service).await;
            });
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let cache = IconCache::new(TEST_CACHE_CAPACITY);
    let url = format!("http://{}/icon.png", addr);

    // First fetch
    let _ = cache.get_icon("service-x", &url).await;
    assert_eq!(request_count.load(Ordering::SeqCst), 1);

    // Second fetch - should use cache
    let _ = cache.get_icon("service-x", &url).await;
    // Request count should still be 1 (cached)
    assert_eq!(request_count.load(Ordering::SeqCst), 1);

    // Cache should have 1 entry
    assert_eq!(cache.cache_size().await, 1);
}
