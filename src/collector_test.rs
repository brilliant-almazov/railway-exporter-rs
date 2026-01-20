//! Tests for metrics collector with mock Railway API.

use crate::client::Client;
use crate::collector::{collect_metrics, days_in_current_month};
use crate::config::Plan;
use crate::state::AppState;
use crate::Config;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::net::TcpListener;

// =============================================================================
// days_in_current_month Tests
// =============================================================================

#[test]
fn test_days_in_january() {
    assert_eq!(days_in_current_month(2024, 1), 31);
    assert_eq!(days_in_current_month(2025, 1), 31);
}

#[test]
fn test_days_in_february_regular() {
    assert_eq!(days_in_current_month(2023, 2), 28);
    assert_eq!(days_in_current_month(2025, 2), 28);
}

#[test]
fn test_days_in_february_leap() {
    assert_eq!(days_in_current_month(2024, 2), 29);
    assert_eq!(days_in_current_month(2020, 2), 29);
}

#[test]
fn test_days_in_april() {
    assert_eq!(days_in_current_month(2024, 4), 30);
}

#[test]
fn test_days_in_december() {
    assert_eq!(days_in_current_month(2024, 12), 31);
}

// =============================================================================
// Mock GraphQL Server
// =============================================================================

async fn start_mock_railway_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}/graphql", addr);

    tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);

            tokio::spawn(async move {
                let service = service_fn(|req: Request<hyper::body::Incoming>| async move {
                    // Read body to determine which query
                    use http_body_util::BodyExt;
                    let body_bytes = req.collect().await.unwrap().to_bytes();
                    let body = String::from_utf8_lossy(&body_bytes);

                    let response = if body.contains("project(id:") {
                        r#"{
                            "data": {
                                "project": {
                                    "name": "test-project",
                                    "services": {
                                        "edges": [
                                            { "node": { "id": "svc-1", "name": "api", "icon": null } },
                                            { "node": { "id": "svc-2", "name": "web", "icon": "🌐" } }
                                        ]
                                    }
                                }
                            }
                        }"#
                    } else if body.contains("usage(projectId:") {
                        r#"{
                            "data": {
                                "usage": [
                                    { "measurement": "CPU_USAGE", "value": 100.0, "tags": { "serviceId": "svc-1" } },
                                    { "measurement": "MEMORY_USAGE_GB", "value": 50.0, "tags": { "serviceId": "svc-1" } },
                                    { "measurement": "CPU_USAGE", "value": 200.0, "tags": { "serviceId": "svc-2" } }
                                ]
                            }
                        }"#
                    } else if body.contains("estimatedUsage(projectId:") {
                        r#"{
                            "data": {
                                "estimatedUsage": [
                                    { "measurement": "CPU_USAGE", "estimatedValue": 3000.0 },
                                    { "measurement": "MEMORY_USAGE_GB", "estimatedValue": 1500.0 }
                                ]
                            }
                        }"#
                    } else {
                        r#"{ "data": null }"#
                    };

                    Ok::<_, Infallible>(
                        Response::builder()
                            .header("content-type", "application/json")
                            .body(Full::new(Bytes::from(response)))
                            .unwrap(),
                    )
                });

                let _ = http1::Builder::new().serve_connection(io, service).await;
            });
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    url
}

// =============================================================================
// collect_metrics Tests
// =============================================================================

#[tokio::test]
async fn test_collect_metrics_success() {
    let api_url = start_mock_railway_server().await;

    let config = Config::new("test-token", "project-123", Plan::Pro, 300, 9090);
    let mut config_with_url = config;
    config_with_url.api_url = api_url.clone();

    let state = Arc::new(AppState::new(config_with_url.clone()));
    let client = Client::new("test-token", Some(&api_url));

    let result = collect_metrics(&client, &state).await;

    assert!(result.is_ok());

    // Check metrics were recorded
    let json = state.metrics_json.read().await;
    assert!(json.is_some());

    let metrics_json = json.as_ref().unwrap();
    assert_eq!(metrics_json.project.name, "test-project");
    assert_eq!(metrics_json.services.len(), 2);

    // Check API status was updated
    let status = state.api_status.read().await;
    assert!(status.last_success.is_some());
    assert!(status.last_error.is_none());
    assert_eq!(status.total_scrapes, 1);
    assert_eq!(status.failed_scrapes, 0);
}

#[tokio::test]
async fn test_collect_metrics_updates_prometheus_gauges() {
    let api_url = start_mock_railway_server().await;

    let config = Config::new("test-token", "project-123", Plan::Pro, 300, 9090);
    let mut config_with_url = config;
    config_with_url.api_url = api_url.clone();

    let state = Arc::new(AppState::new(config_with_url.clone()));
    let client = Client::new("test-token", Some(&api_url));

    collect_metrics(&client, &state).await.unwrap();

    // Verify Prometheus metrics were set
    // Note: We can't easily verify gauge values, but we can verify no panics occurred
}

#[tokio::test]
async fn test_collect_metrics_api_error() {
    // Use invalid URL to trigger error
    let config = Config::new("test-token", "project-123", Plan::Pro, 300, 9090);
    let mut config_with_url = config;
    config_with_url.api_url = "http://127.0.0.1:1/graphql".to_string();

    let state = Arc::new(AppState::new(config_with_url.clone()));
    let client = Client::new("test-token", Some("http://127.0.0.1:1/graphql"));

    let result = collect_metrics(&client, &state).await;

    assert!(result.is_err());

    // Check failed_scrapes was incremented
    let status = state.api_status.read().await;
    assert_eq!(status.total_scrapes, 1);
    assert_eq!(status.failed_scrapes, 1);
    assert!(status.last_error.is_some());
}

#[tokio::test]
async fn test_collect_metrics_broadcasts_to_websocket() {
    let api_url = start_mock_railway_server().await;

    let config = Config::new("test-token", "project-123", Plan::Pro, 300, 9090);
    let mut config_with_url = config;
    config_with_url.api_url = api_url.clone();

    let state = Arc::new(AppState::new(config_with_url.clone()));
    let client = Client::new("test-token", Some(&api_url));

    // Subscribe to broadcast before collection
    let mut rx = state.ws_broadcast.subscribe();

    collect_metrics(&client, &state).await.unwrap();

    // Should receive a message
    let msg = rx.try_recv();
    assert!(msg.is_ok());
    let json_str = msg.unwrap();
    assert!(json_str.contains("\"type\":\"metrics\""));
}

#[tokio::test]
async fn test_collect_metrics_with_service_groups() {
    let api_url = start_mock_railway_server().await;

    let mut config = Config::new("test-token", "project-123", Plan::Pro, 300, 9090);
    config.api_url = api_url.clone();
    config
        .service_groups
        .insert("backend".to_string(), vec!["api".to_string()]);

    let state = Arc::new(AppState::new(config));
    let client = Client::new("test-token", Some(&api_url));

    collect_metrics(&client, &state).await.unwrap();

    let json = state.metrics_json.read().await;
    let metrics_json = json.as_ref().unwrap();

    // api service should be in "backend" group
    let api_service = metrics_json
        .services
        .iter()
        .find(|s| s.name == "api")
        .unwrap();
    assert_eq!(api_service.group, "backend");

    // web service should be in "ungrouped" (default)
    let web_service = metrics_json
        .services
        .iter()
        .find(|s| s.name == "web")
        .unwrap();
    assert_eq!(web_service.group, "ungrouped");
}
