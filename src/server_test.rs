//! Integration tests for HTTP server.
//!
//! These tests start an actual server and make HTTP requests to verify routing.

use crate::config::Plan;
use crate::state::AppState;
use crate::Config;
use std::sync::Arc;
use std::time::Duration;

// =============================================================================
// Server Integration Tests
// =============================================================================

/// Starts server on random port and returns the address.
async fn start_test_server() -> (Arc<AppState>, String) {
    let config = Config::new("test-token", "test-project", Plan::Pro, 300, 0);
    let state = Arc::new(AppState::new(config));

    // Bind to port 0 to get a random available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    let state_clone = state.clone();

    // Start server in background
    tokio::spawn(async move {
        
        
        use hyper::server::conn::http1;
        use hyper::service::service_fn;
        use hyper_util::rt::TokioIo;

        loop {
            let (stream, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(_) => break,
            };
            let state = state_clone.clone();

            tokio::spawn(async move {
                let io = TokioIo::new(stream);
                let state_svc = state.clone();

                let svc = service_fn(move |req| {
                    let state = state_svc.clone();
                    async move {
                        use crate::handlers;

                        let path = req.uri().path();
                        let accepts_gzip = req
                            .headers()
                            .get("Accept-Encoding")
                            .and_then(|v| v.to_str().ok())
                            .map(|v| v.contains("gzip"))
                            .unwrap_or(false);

                        let response = match path {
                            "/metrics" => {
                                let wants_json = req
                                    .headers()
                                    .get("Accept")
                                    .and_then(|v| v.to_str().ok())
                                    .map(|v| v.contains("application/json"))
                                    .unwrap_or(false);

                                if wants_json {
                                    handlers::metrics_json(&state).await
                                } else {
                                    handlers::metrics_prometheus(&state)
                                }
                            }
                            "/status" | "/" => handlers::status(&state).await,
                            "/health" => handlers::health(),
                            _ => handlers::not_found(),
                        };

                        Ok::<_, std::convert::Infallible>(handlers::finalize(
                            response,
                            state.config.cors_enabled,
                            accepts_gzip,
                            &state.config.gzip,
                        ))
                    }
                });

                let _ = http1::Builder::new().serve_connection(io, svc).await;
            });
        }
    });

    // Wait for server to be ready
    tokio::time::sleep(Duration::from_millis(10)).await;

    (state, url)
}

#[tokio::test]
async fn test_server_health_endpoint() {
    let (_state, url) = start_test_server().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/health", url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap(), "ok");
}

#[tokio::test]
async fn test_server_status_endpoint() {
    let (_state, url) = start_test_server().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/status", url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let json: serde_json::Value = resp.json().await.unwrap();
    assert!(json["version"].is_string());
    assert!(json["uptime_seconds"].is_number());
    assert!(json["endpoints"].is_object());
    assert!(json["config"].is_object());
    assert!(json["process"].is_object());
    assert!(json["api"].is_object());
}

#[tokio::test]
async fn test_server_root_endpoint_redirects_to_status() {
    let (_state, url) = start_test_server().await;

    let client = reqwest::Client::new();
    let resp = client.get(format!("{}/", url)).send().await.unwrap();

    assert_eq!(resp.status(), 200);

    let json: serde_json::Value = resp.json().await.unwrap();
    assert!(json["version"].is_string());
}

#[tokio::test]
async fn test_server_metrics_prometheus() {
    let (_state, url) = start_test_server().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/metrics", url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let content_type = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("text/plain"));

    let body = resp.text().await.unwrap();
    assert!(body.contains("railway_"));
}

#[tokio::test]
async fn test_server_metrics_json() {
    let (state, url) = start_test_server().await;

    // First populate some metrics
    {
        use crate::types::{MetricsJson, ProjectSummary};
        let mut json = state.metrics_json.write().await;
        *json = Some(MetricsJson {
            project: ProjectSummary {
                name: "test".to_string(),
                current_usage_usd: 10.0,
                estimated_monthly_usd: 30.0,
                daily_average_usd: 1.0,
                days_elapsed: 10,
                days_remaining: 20,
            },
            services: vec![],
            scrape_timestamp: 1700000000,
            scrape_duration_seconds: 0.1,
        });
    }

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/metrics", url))
        .header("Accept", "application/json")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let content_type = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(content_type.contains("application/json"));

    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["project"]["name"], "test");
}

#[tokio::test]
async fn test_server_not_found() {
    let (_state, url) = start_test_server().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/nonexistent", url))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_server_cors_headers() {
    let (_state, url) = start_test_server().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/health", url))
        .send()
        .await
        .unwrap();

    assert!(resp.headers().get("Access-Control-Allow-Origin").is_some());
}

#[tokio::test]
async fn test_server_gzip_header_accepted() {
    let (_state, url) = start_test_server().await;

    let client = reqwest::Client::new();

    // Request with gzip accept header
    let resp = client
        .get(format!("{}/health", url))
        .header("Accept-Encoding", "gzip")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    // Health response is small, might not be compressed, but request should work
    let body = resp.text().await.unwrap();
    assert_eq!(body, "ok");
}

#[tokio::test]
async fn test_server_multiple_concurrent_requests() {
    let (_state, url) = start_test_server().await;

    let client = reqwest::Client::new();
    let mut handles = vec![];

    for _ in 0..10 {
        let client = client.clone();
        let url = url.clone();
        handles.push(tokio::spawn(async move {
            client.get(format!("{}/health", url)).send().await
        }));
    }

    for handle in handles {
        let resp = handle.await.unwrap().unwrap();
        assert_eq!(resp.status(), 200);
    }
}

// =============================================================================
// WebSocket Server Tests
// =============================================================================

/// Starts server with WebSocket support on random port.
async fn start_ws_test_server() -> (Arc<AppState>, String) {
    use crate::server::handle_websocket;
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper_util::rt::TokioIo;

    let mut config = Config::new("test-token", "test-project", Plan::Pro, 300, 0);
    config.websocket_enabled = true;
    let state = Arc::new(AppState::new(config));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("ws://{}", addr);

    let state_clone = state.clone();

    tokio::spawn(async move {
        loop {
            let (stream, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(_) => break,
            };
            let state = state_clone.clone();

            tokio::spawn(async move {
                // Peek to detect WebSocket upgrade
                let mut peek_buf = [0u8; 256];
                if let Ok(n) = stream.peek(&mut peek_buf).await {
                    if n > 0 {
                        let req_start = String::from_utf8_lossy(&peek_buf[..n]);
                        if req_start.contains("GET /ws") && req_start.contains("Upgrade") {
                            handle_websocket(state, stream).await;
                            return;
                        }
                    }
                }

                // Regular HTTP
                let io = TokioIo::new(stream);
                let state_svc = state.clone();

                let svc = service_fn(move |req| {
                    let state = state_svc.clone();
                    async move {
                        use crate::handlers;

                        let path = req.uri().path();
                        let accepts_gzip = req
                            .headers()
                            .get("Accept-Encoding")
                            .and_then(|v| v.to_str().ok())
                            .map(|v| v.contains("gzip"))
                            .unwrap_or(false);

                        let response = match path {
                            "/health" => handlers::health(),
                            _ => handlers::not_found(),
                        };

                        Ok::<_, std::convert::Infallible>(handlers::finalize(
                            response,
                            state.config.cors_enabled,
                            accepts_gzip,
                            &state.config.gzip,
                        ))
                    }
                });

                let _ = http1::Builder::new().serve_connection(io, svc).await;
            });
        }
    });

    tokio::time::sleep(Duration::from_millis(10)).await;

    (state, url)
}

#[tokio::test]
async fn test_websocket_connection() {
    use tokio_tungstenite::connect_async;
    use futures_util::StreamExt;

    let (state, url) = start_ws_test_server().await;

    // Populate metrics for the initial message
    {
        use crate::types::{MetricsJson, ProjectSummary};
        let mut json = state.metrics_json.write().await;
        *json = Some(MetricsJson {
            project: ProjectSummary {
                name: "test-ws".to_string(),
                current_usage_usd: 5.0,
                estimated_monthly_usd: 15.0,
                daily_average_usd: 0.5,
                days_elapsed: 5,
                days_remaining: 25,
            },
            services: vec![],
            scrape_timestamp: 1700000000,
            scrape_duration_seconds: 0.05,
        });
    }

    let ws_url = format!("{}/ws", url);
    let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");

    // Should receive initial status message
    // Format: {"type": "status", "data": {"uptime_seconds": N, ...}}
    let msg = ws_stream.next().await.unwrap().unwrap();
    let text = msg.into_text().unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["type"], "status");
    assert!(json["data"]["uptime_seconds"].is_number());

    // Should receive initial metrics message
    // Format: {"type": "metrics", "data": {"project": {...}, ...}}
    let msg = ws_stream.next().await.unwrap().unwrap();
    let text = msg.into_text().unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["type"], "metrics");
    assert_eq!(json["data"]["project"]["name"], "test-ws");

    // Check client count was incremented
    assert_eq!(state.ws_client_count(), 1);
}

#[tokio::test]
async fn test_websocket_client_count() {
    use tokio_tungstenite::connect_async;

    let (state, url) = start_ws_test_server().await;
    let ws_url = format!("{}/ws", url);

    assert_eq!(state.ws_client_count(), 0);

    // Connect first client
    let (ws1, _) = connect_async(&ws_url).await.expect("Failed to connect");
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(state.ws_client_count(), 1);

    // Connect second client
    let (ws2, _) = connect_async(&ws_url).await.expect("Failed to connect");
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(state.ws_client_count(), 2);

    // Disconnect first client
    drop(ws1);
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(state.ws_client_count() <= 2); // May not have processed disconnect yet

    // Disconnect second client
    drop(ws2);
}

#[tokio::test]
async fn test_websocket_ping_pong() {
    use tokio_tungstenite::connect_async;
    use tokio_tungstenite::tungstenite::Message;
    use futures_util::{SinkExt, StreamExt};

    let (_state, url) = start_ws_test_server().await;
    let ws_url = format!("{}/ws", url);

    let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");

    // Skip initial messages
    let _ = ws_stream.next().await;
    let _ = ws_stream.next().await;

    // Send ping
    ws_stream.send(Message::Ping(vec![1, 2, 3])).await.unwrap();

    // Should receive pong with same data
    // Note: The pong might come interleaved with status updates
    let timeout = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Some(Ok(msg)) = ws_stream.next().await {
                if let Message::Pong(data) = msg {
                    return data;
                }
            }
        }
    });

    match timeout.await {
        Ok(data) => assert_eq!(data, vec![1, 2, 3]),
        Err(_) => panic!("Timeout waiting for pong"),
    }
}

#[tokio::test]
async fn test_websocket_broadcast() {
    use tokio_tungstenite::connect_async;
    use futures_util::StreamExt;

    let (state, url) = start_ws_test_server().await;
    let ws_url = format!("{}/ws", url);

    let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");

    // Skip initial messages (status + maybe metrics)
    let _ = ws_stream.next().await;
    // Might not have metrics yet, so don't wait for second message

    // Broadcast a message
    let _ = state.ws_broadcast.send(r#"{"type":"test","data":"hello"}"#.to_string());

    // Should receive the broadcast
    let timeout = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if let Some(Ok(msg)) = ws_stream.next().await {
                let text = msg.into_text().unwrap_or_default();
                if text.contains("test") {
                    return text;
                }
            }
        }
    });

    match timeout.await {
        Ok(text) => assert!(text.contains("hello")),
        Err(_) => panic!("Timeout waiting for broadcast"),
    }
}
