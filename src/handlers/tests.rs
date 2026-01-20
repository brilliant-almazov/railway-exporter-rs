//! Handler tests for Railway Exporter.

use super::{
    finalize, health, metrics_json, metrics_prometheus, not_found, status, HandlerResponse,
};
use crate::config::{Config, GzipConfig, Plan};
use crate::state::AppState;
use hyper::body::Bytes;
use hyper::http::StatusCode;
use hyper::Response;
use std::sync::Arc;

// =============================================================================
// Test Helpers
// =============================================================================

fn create_test_config() -> Config {
    Config::new("test_token", "test_project", Plan::Hobby, 60, 8080)
}

fn create_test_state() -> Arc<AppState> {
    Arc::new(AppState::new(create_test_config()))
}

// =============================================================================
// Health Handler Tests
// =============================================================================

#[test]
fn test_health_handler_returns_ok() {
    let (builder, body) = health();
    let response = builder.body(body.clone()).unwrap();

    assert_eq!(body, Bytes::from("ok"));
    assert_eq!(response.status(), StatusCode::OK);
}

#[test]
fn test_health_handler_content_type() {
    let (builder, body) = health();
    let response = builder.body(body).unwrap();

    let content_type = response.headers().get("Content-Type").unwrap();
    assert_eq!(content_type, "text/plain");
}

// =============================================================================
// Not Found Handler Tests
// =============================================================================

#[test]
fn test_not_found_returns_404() {
    let (builder, body) = not_found();
    let response = builder.body(body.clone()).unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(body, Bytes::from("Not Found"));
}

// =============================================================================
// Finalize Function Tests
// =============================================================================

#[test]
fn test_finalize_without_cors() {
    let response: HandlerResponse = (
        Response::builder().header("Content-Type", "text/plain"),
        Bytes::from("test"),
    );
    let gzip = GzipConfig::default();

    let result = finalize(response, false, false, &gzip);

    assert!(result
        .headers()
        .get("Access-Control-Allow-Origin")
        .is_none());
}

#[test]
fn test_finalize_with_cors() {
    let response: HandlerResponse = (
        Response::builder().header("Content-Type", "text/plain"),
        Bytes::from("test"),
    );
    let gzip = GzipConfig::default();

    let result = finalize(response, true, false, &gzip);

    let cors = result.headers().get("Access-Control-Allow-Origin").unwrap();
    assert_eq!(cors, "*");
}

#[test]
fn test_finalize_gzip_disabled() {
    let large_body = "x".repeat(1000);
    let response: HandlerResponse = (
        Response::builder().header("Content-Type", "text/plain"),
        Bytes::from(large_body),
    );
    let gzip = GzipConfig {
        enabled: false,
        min_size: 256,
        level: 1,
    };

    let result = finalize(response, false, true, &gzip);

    assert!(result.headers().get("Content-Encoding").is_none());
}

#[test]
fn test_finalize_gzip_not_requested() {
    let large_body = "x".repeat(1000);
    let response: HandlerResponse = (
        Response::builder().header("Content-Type", "text/plain"),
        Bytes::from(large_body),
    );
    let gzip = GzipConfig::default();

    // gzip_requested = false
    let result = finalize(response, false, false, &gzip);

    assert!(result.headers().get("Content-Encoding").is_none());
}

#[test]
fn test_finalize_gzip_body_too_small() {
    let small_body = "x".repeat(100); // Less than min_size (256)
    let response: HandlerResponse = (
        Response::builder().header("Content-Type", "text/plain"),
        Bytes::from(small_body),
    );
    let gzip = GzipConfig::default();

    let result = finalize(response, false, true, &gzip);

    assert!(result.headers().get("Content-Encoding").is_none());
}

#[test]
fn test_finalize_gzip_compresses_large_body() {
    let large_body = "x".repeat(1000); // Larger than min_size
    let response: HandlerResponse = (
        Response::builder().header("Content-Type", "text/plain"),
        Bytes::from(large_body),
    );
    let gzip = GzipConfig::default();

    let result = finalize(response, false, true, &gzip);

    let encoding = result.headers().get("Content-Encoding");
    assert_eq!(encoding.unwrap(), "gzip");
}

#[test]
fn test_finalize_gzip_custom_level() {
    let large_body = "x".repeat(1000);
    let response: HandlerResponse = (
        Response::builder().header("Content-Type", "text/plain"),
        Bytes::from(large_body),
    );
    let gzip = GzipConfig {
        enabled: true,
        min_size: 256,
        level: 9, // Max compression
    };

    let result = finalize(response, false, true, &gzip);

    assert_eq!(result.headers().get("Content-Encoding").unwrap(), "gzip");
}

// =============================================================================
// Metrics Prometheus Handler Tests
// =============================================================================

#[test]
fn test_metrics_prometheus_content_type() {
    let state = create_test_state();
    let (builder, _body) = metrics_prometheus(&state);
    let response = builder.body(Bytes::new()).unwrap();

    let content_type = response.headers().get("Content-Type").unwrap();
    assert_eq!(content_type, "text/plain; version=0.0.4; charset=utf-8");
}

#[test]
fn test_metrics_prometheus_returns_data() {
    let state = create_test_state();
    let (_builder, body) = metrics_prometheus(&state);

    // Body should contain Prometheus metrics format
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("# HELP") || body_str.contains("# TYPE") || !body_str.is_empty());
}

// =============================================================================
// Metrics JSON Handler Tests
// =============================================================================

#[tokio::test]
async fn test_metrics_json_no_data() {
    let state = create_test_state();
    let (builder, body) = metrics_json(&state).await;
    let response = builder.body(Bytes::new()).unwrap();

    let content_type = response.headers().get("Content-Type").unwrap();
    assert_eq!(content_type, "application/json");

    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("No data yet"));
}

// =============================================================================
// Status Handler Tests
// =============================================================================

#[tokio::test]
async fn test_status_handler_content_type() {
    let state = create_test_state();
    let (builder, _body) = status(&state).await;
    let response = builder.body(Bytes::new()).unwrap();

    let content_type = response.headers().get("Content-Type").unwrap();
    assert_eq!(content_type, "application/json");
}

#[tokio::test]
async fn test_status_handler_version() {
    let state = create_test_state();
    let (_builder, body) = status(&state).await;

    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed["version"], env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn test_status_handler_endpoints() {
    let state = create_test_state();
    let (_builder, body) = status(&state).await;

    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed["endpoints"]["prometheus"], true);
    assert_eq!(parsed["endpoints"]["json"], true);
    assert_eq!(parsed["endpoints"]["health"], true);
    assert_eq!(parsed["endpoints"]["websocket"], true);
}

#[tokio::test]
async fn test_status_handler_config() {
    let state = create_test_state();
    let (_builder, body) = status(&state).await;

    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed["config"]["plan"], "hobby");
    assert_eq!(parsed["config"]["scrape_interval_seconds"], 60);
    assert_eq!(
        parsed["config"]["api_url"],
        "https://backboard.railway.app/graphql/v2"
    );
}

#[tokio::test]
async fn test_status_handler_process_info() {
    let state = create_test_state();
    let (_builder, body) = status(&state).await;

    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(parsed["process"]["pid"].is_u64());
    assert!(parsed["process"]["memory_mb"].is_f64());
    assert!(parsed["process"]["cpu_percent"].is_f64());
}

#[tokio::test]
async fn test_status_handler_api_status() {
    let state = create_test_state();
    let (_builder, body) = status(&state).await;

    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(parsed["api"]["total_scrapes"], 0);
    assert_eq!(parsed["api"]["failed_scrapes"], 0);
    assert!(parsed["api"]["last_success"].is_null());
    assert!(parsed["api"]["last_error"].is_null());
}
