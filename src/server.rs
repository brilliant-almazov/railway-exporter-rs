//! HTTP server and routing.

use crate::handlers;
use crate::state::AppState;
use futures_util::{SinkExt, StreamExt};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn};

/// Starts the HTTP server.
pub async fn start(state: Arc<AppState>) {
    let addr = SocketAddr::from(([0, 0, 0, 0], state.config.port));
    let listener = TcpListener::bind(addr).await.unwrap();

    info!("Listening on http://{}", addr);
    info!(
        "Endpoints: /metrics{}, /status, /health{}",
        if state.config.websocket_enabled {
            ", /ws"
        } else {
            ""
        },
        if state.config.cors_enabled {
            " (CORS enabled)"
        } else {
            ""
        }
    );

    loop {
        let (stream, _peer_addr) = listener.accept().await.unwrap();
        let state = state.clone();

        tokio::spawn(async move {
            // Peek at first bytes to detect WebSocket upgrade to /ws
            if state.config.websocket_enabled {
                let mut peek_buf = [0u8; 256];
                match stream.peek(&mut peek_buf).await {
                    Ok(n) => {
                        if n > 0 {
                            let req_start = String::from_utf8_lossy(&peek_buf[..n]);
                            // Check if this is a GET /ws request with WebSocket upgrade
                            let is_ws_upgrade = req_start.contains("GET /ws") && req_start.contains("Upgrade");
                            info!("Peek: {} bytes, is_ws={}, first_line={:?}",
                                  n, is_ws_upgrade, req_start.lines().next().unwrap_or(""));
                            if is_ws_upgrade {
                                info!("Routing to WebSocket handler");
                                handle_websocket(state, stream).await;
                                return;
                            }
                        } else {
                            warn!("Peek returned 0 bytes");
                        }
                    }
                    Err(e) => {
                        warn!("Peek error: {}", e);
                    }
                }
            }

            // Regular HTTP request
            let io = TokioIo::new(stream);
            let state_clone = state.clone();

            let svc = service_fn(move |req: Request<hyper::body::Incoming>| {
                let state = state_clone.clone();
                async move { route_request(req, state).await }
            });

            let _ = http1::Builder::new().serve_connection(io, svc).await;
        });
    }
}

/// Routes incoming HTTP requests to appropriate handlers.
async fn route_request(
    req: Request<hyper::body::Incoming>,
    state: Arc<AppState>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req.uri().path();

    // Check if client accepts gzip
    let accepts_gzip = req
        .headers()
        .get("Accept-Encoding")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("gzip"))
        .unwrap_or(false);

    // WebSocket is handled at TCP level before reaching here
    // Route to handlers, get (Builder, Bytes) tuple
    let response = match path {
        "/metrics" => {
            // Content negotiation: Accept header determines format
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
        "/status" => handlers::status(&state).await,
        "/health" => handlers::health(),
        _ if path.starts_with("/icons/services/") => {
            // Extract service name from path: /icons/services/{service_name}
            let service_name = &path["/icons/services/".len()..];
            if service_name.is_empty() {
                handlers::not_found()
            } else {
                // URL decode the service name
                let decoded = urlencoding::decode(service_name)
                    .unwrap_or_else(|_| service_name.into());
                handlers::icons(&state, &decoded).await
            }
        }
        // Static files: serve from /static directory (for combined image)
        // Falls back to index.html for SPA routing
        _ => handlers::static_file(path),
    };

    // Finalize: add CORS headers if enabled, gzip if configured, build response
    Ok(handlers::finalize(
        response,
        state.config.cors_enabled,
        accepts_gzip,
        &state.config.gzip,
    ))
}

/// Handles WebSocket connections.
pub async fn handle_websocket(state: Arc<AppState>, stream: tokio::net::TcpStream) {
    use crate::types::{ApiStatus, WsMessage, WsStatus};
    use crate::utils::ProcessInfoProvider;

    info!("Starting WebSocket handshake...");
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => {
            info!("WebSocket handshake successful!");
            ws
        }
        Err(e) => {
            warn!("WebSocket handshake failed: {}", e);
            return;
        }
    };

    // Track client count
    let client_count = state.ws_client_connect();
    info!("WebSocket client connected (total: {})", client_count);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Process info provider for memory/cpu stats
    let process_info = ProcessInfoProvider::new();

    // Helper to build WsStatus with current process stats
    let build_status = |state: &AppState, api_status: &crate::state::ApiStatusData, process_info: &ProcessInfoProvider| -> WsStatus {
        let proc_status = process_info.status();
        WsStatus {
            uptime_seconds: state.start_time.elapsed().as_secs(),
            memory_mb: proc_status.memory_mb,
            cpu_percent: proc_status.cpu_percent,
            api: ApiStatus {
                last_success: api_status.last_success,
                last_error: api_status.last_error.clone(),
                total_scrapes: api_status.total_scrapes,
                failed_scrapes: api_status.failed_scrapes,
            },
            ws_clients: state.ws_client_count(),
        }
    };

    // Send current metrics and status immediately
    {
        let api_status = state.api_status.read().await;

        // Send status first
        let status_msg = WsMessage::Status(build_status(&state, &api_status, &process_info));
        if let Ok(json) = serde_json::to_string(&status_msg) {
            info!("Sending initial status ({} bytes)", json.len());
            if let Err(e) = ws_sender.send(Message::Text(json.into())).await {
                warn!("Failed to send initial status: {}", e);
                state.ws_client_disconnect();
                return;
            }
        }

        // Then send metrics if available
        if let Some(metrics) = state.metrics_json.read().await.as_ref() {
            let metrics_msg = WsMessage::Metrics(metrics.clone());
            if let Ok(json) = serde_json::to_string(&metrics_msg) {
                info!("Sending initial metrics ({} bytes)", json.len());
                if let Err(e) = ws_sender.send(Message::Text(json.into())).await {
                    warn!("Failed to send initial metrics: {}", e);
                    state.ws_client_disconnect();
                    return;
                }
            }
        } else {
            info!("No metrics available yet");
        }
    }

    // Subscribe to updates
    let mut rx = state.ws_broadcast.subscribe();

    // Ticker for periodic status updates (every 5 seconds)
    let mut status_ticker = tokio::time::interval(std::time::Duration::from_secs(5));

    loop {
        tokio::select! {
            // Periodic status updates
            _ = status_ticker.tick() => {
                let api_status = state.api_status.read().await;
                let status_msg = WsMessage::Status(build_status(&state, &api_status, &process_info));
                if let Ok(json) = serde_json::to_string(&status_msg) {
                    info!("Sending periodic status update");
                    if ws_sender.send(Message::Text(json.into())).await.is_err() {
                        info!("Client disconnected during status send");
                        state.ws_client_disconnect();
                        break;
                    }
                }
            }
            // Handle incoming messages (for ping/pong)
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Ping(data))) => {
                        let _ = ws_sender.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        let remaining = state.ws_client_disconnect();
                        info!("WebSocket client disconnected (remaining: {})", remaining);
                        break;
                    }
                    _ => {}
                }
            }
            // Broadcast updates (already serialized WsMessage JSON)
            update = rx.recv() => {
                match update {
                    Ok(json) => {
                        if ws_sender.send(Message::Text(json.into())).await.is_err() {
                            state.ws_client_disconnect();
                            break;
                        }
                    }
                    Err(_) => {
                        state.ws_client_disconnect();
                        break;
                    }
                }
            }
        }
    }
}
