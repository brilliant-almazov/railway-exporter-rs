//! Tests for application state management.

use crate::config::Plan;
use crate::state::{ApiStatusData, AppState};
use crate::Config;

// =============================================================================
// ApiStatusData Tests
// =============================================================================

#[test]
fn test_api_status_data_default() {
    let status = ApiStatusData::default();
    assert!(status.last_success.is_none());
    assert!(status.last_error.is_none());
    assert_eq!(status.total_scrapes, 0);
    assert_eq!(status.failed_scrapes, 0);
}

#[test]
fn test_api_status_data_with_values() {
    let status = ApiStatusData {
        last_success: Some(1700000000),
        last_error: Some("Connection refused".to_string()),
        total_scrapes: 100,
        failed_scrapes: 5,
    };

    assert_eq!(status.last_success, Some(1700000000));
    assert_eq!(status.last_error.as_deref(), Some("Connection refused"));
    assert_eq!(status.total_scrapes, 100);
    assert_eq!(status.failed_scrapes, 5);
}

// =============================================================================
// AppState Tests
// =============================================================================

#[test]
fn test_app_state_new() {
    let config = Config::new("token", "project-id", Plan::Pro, 300, 9090);
    let state = AppState::new(config);

    assert_eq!(state.config.project_id, "project-id");
    assert_eq!(state.config.plan, Plan::Pro);
    assert_eq!(state.ws_client_count(), 0);
}

#[test]
fn test_ws_client_connect() {
    let config = Config::new("token", "project-id", Plan::Hobby, 300, 9090);
    let state = AppState::new(config);

    // First connect returns 1
    let count = state.ws_client_connect();
    assert_eq!(count, 1);
    assert_eq!(state.ws_client_count(), 1);

    // Second connect returns 2
    let count = state.ws_client_connect();
    assert_eq!(count, 2);
    assert_eq!(state.ws_client_count(), 2);
}

#[test]
fn test_ws_client_disconnect() {
    let config = Config::new("token", "project-id", Plan::Hobby, 300, 9090);
    let state = AppState::new(config);

    // Connect 2 clients
    state.ws_client_connect();
    state.ws_client_connect();
    assert_eq!(state.ws_client_count(), 2);

    // Disconnect one
    let count = state.ws_client_disconnect();
    assert_eq!(count, 1);
    assert_eq!(state.ws_client_count(), 1);

    // Disconnect another
    let count = state.ws_client_disconnect();
    assert_eq!(count, 0);
    assert_eq!(state.ws_client_count(), 0);
}

#[test]
fn test_ws_client_count() {
    let config = Config::new("token", "project-id", Plan::Pro, 300, 9090);
    let state = AppState::new(config);

    assert_eq!(state.ws_client_count(), 0);

    state.ws_client_connect();
    assert_eq!(state.ws_client_count(), 1);

    state.ws_client_connect();
    state.ws_client_connect();
    assert_eq!(state.ws_client_count(), 3);
}

#[tokio::test]
async fn test_app_state_metrics_json_initially_none() {
    let config = Config::new("token", "project-id", Plan::Pro, 300, 9090);
    let state = AppState::new(config);

    let json = state.metrics_json.read().await;
    assert!(json.is_none());
}

#[tokio::test]
async fn test_app_state_api_status_initially_default() {
    let config = Config::new("token", "project-id", Plan::Pro, 300, 9090);
    let state = AppState::new(config);

    let status = state.api_status.read().await;
    assert!(status.last_success.is_none());
    assert!(status.last_error.is_none());
    assert_eq!(status.total_scrapes, 0);
    assert_eq!(status.failed_scrapes, 0);
}

#[tokio::test]
async fn test_app_state_api_status_update() {
    let config = Config::new("token", "project-id", Plan::Pro, 300, 9090);
    let state = AppState::new(config);

    // Update status
    {
        let mut status = state.api_status.write().await;
        status.total_scrapes = 10;
        status.last_success = Some(1700000000);
    }

    // Verify update
    let status = state.api_status.read().await;
    assert_eq!(status.total_scrapes, 10);
    assert_eq!(status.last_success, Some(1700000000));
}

#[test]
fn test_app_state_start_time_is_recent() {
    let config = Config::new("token", "project-id", Plan::Pro, 300, 9090);
    let state = AppState::new(config);

    // Start time should be very recent (within 1 second)
    let elapsed = state.start_time.elapsed();
    assert!(elapsed.as_secs() < 1);
}

#[test]
fn test_app_state_ws_broadcast_channel() {
    let config = Config::new("token", "project-id", Plan::Pro, 300, 9090);
    let state = AppState::new(config);

    // Should be able to subscribe
    let _rx = state.ws_broadcast.subscribe();

    // Should be able to send (even with no receivers it won't fail with error we care about)
    let _ = state.ws_broadcast.send("test message".to_string());
}
