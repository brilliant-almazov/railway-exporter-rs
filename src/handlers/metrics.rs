//! Metrics handlers (Prometheus and JSON formats).

use super::HandlerResponse;
use crate::state::AppState;
use hyper::body::Bytes;
use hyper::Response;

/// GET /metrics - Prometheus format.
pub fn handle_prometheus(state: &AppState) -> HandlerResponse {
    state.metrics.update_process_metrics();

    (
        Response::builder().header("Content-Type", "text/plain; version=0.0.4; charset=utf-8"),
        Bytes::from(state.metrics.encode()),
    )
}

/// GET /metrics with Accept: application/json - JSON format.
pub async fn handle_json(state: &AppState) -> HandlerResponse {
    let json = state.metrics_json.read().await;
    let body = match json.as_ref() {
        Some(m) => serde_json::to_string(m).unwrap_or_else(|_| "{}".to_string()),
        None => r#"{"error": "No data yet"}"#.to_string(),
    };

    (
        Response::builder().header("Content-Type", "application/json"),
        Bytes::from(body),
    )
}
