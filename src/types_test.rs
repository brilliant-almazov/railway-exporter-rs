//! Tests for types module - serialization/deserialization.

use crate::config::{NetworkPricing, PriceValues};
use crate::types::{
    ApiStatus, ConfigStatus, EndpointStatus, EstimatedData, EstimatedItem, GraphQLRequest,
    GraphQLResponse, IconCacheStatusConfig, MetricsJson, ProcessStatus, Project, ProjectData,
    ProjectSummary, ServerStatus, ServiceData, UsageData, UsageItem, WsMessage, WsStatus,
};

// =============================================================================
// ServiceData Tests
// =============================================================================

#[test]
fn test_service_data_serialize() {
    let service = ServiceData {
        id: "svc-123".to_string(),
        name: "web".to_string(),
        icon: "🌐".to_string(),
        group: "frontend".to_string(),
        cpu_usage: 123.45,
        memory_usage: 256.0,
        disk_usage: 1024.0,
        network_tx: 100.0,
        cost_usd: 1.23,
        estimated_monthly_usd: 45.67,
        is_deleted: false,
    };

    let json = serde_json::to_string(&service).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["id"], "svc-123");
    assert_eq!(parsed["name"], "web");
    assert_eq!(parsed["icon"], "🌐");
    assert_eq!(parsed["group"], "frontend");
    assert_eq!(parsed["cpu_usage"], 123.45);
    assert_eq!(parsed["memory_usage"], 256.0);
    assert_eq!(parsed["disk_usage"], 1024.0);
    assert_eq!(parsed["network_tx"], 100.0);
    assert_eq!(parsed["cost_usd"], 1.23);
    assert_eq!(parsed["estimated_monthly_usd"], 45.67);
    assert_eq!(parsed["isDeleted"], false); // Note: serde rename
}

#[test]
fn test_service_data_deleted_flag_rename() {
    let service = ServiceData {
        id: "svc-456".to_string(),
        name: "deleted-service".to_string(),
        icon: "".to_string(),
        group: "default".to_string(),
        cpu_usage: 0.0,
        memory_usage: 0.0,
        disk_usage: 0.0,
        network_tx: 0.0,
        cost_usd: 0.0,
        estimated_monthly_usd: 0.0,
        is_deleted: true,
    };

    let json = serde_json::to_string(&service).unwrap();

    // Should use camelCase "isDeleted" not snake_case
    assert!(json.contains("\"isDeleted\":true"));
    assert!(!json.contains("is_deleted"));
}

// =============================================================================
// ProjectSummary Tests
// =============================================================================

#[test]
fn test_project_summary_serialize() {
    let summary = ProjectSummary {
        name: "my-project".to_string(),
        current_usage_usd: 12.34,
        estimated_monthly_usd: 56.78,
        daily_average_usd: 1.89,
        days_elapsed: 15,
        days_remaining: 15,
    };

    let json = serde_json::to_string(&summary).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["name"], "my-project");
    assert_eq!(parsed["current_usage_usd"], 12.34);
    assert_eq!(parsed["estimated_monthly_usd"], 56.78);
    assert_eq!(parsed["daily_average_usd"], 1.89);
    assert_eq!(parsed["days_elapsed"], 15);
    assert_eq!(parsed["days_remaining"], 15);
}

// =============================================================================
// MetricsJson Tests
// =============================================================================

#[test]
fn test_metrics_json_serialize() {
    let metrics = MetricsJson {
        project: ProjectSummary {
            name: "test".to_string(),
            current_usage_usd: 10.0,
            estimated_monthly_usd: 30.0,
            daily_average_usd: 1.0,
            days_elapsed: 10,
            days_remaining: 20,
        },
        services: vec![ServiceData {
            id: "svc-1".to_string(),
            name: "api".to_string(),
            icon: "🚀".to_string(),
            group: "backend".to_string(),
            cpu_usage: 50.0,
            memory_usage: 128.0,
            disk_usage: 512.0,
            network_tx: 10.0,
            cost_usd: 0.5,
            estimated_monthly_usd: 15.0,
            is_deleted: false,
        }],
        scrape_timestamp: 1700000000,
        scrape_duration_seconds: 0.123,
    };

    let json = serde_json::to_string(&metrics).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["project"]["name"], "test");
    assert_eq!(parsed["services"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["services"][0]["name"], "api");
    assert_eq!(parsed["scrape_timestamp"], 1700000000);
    assert_eq!(parsed["scrape_duration_seconds"], 0.123);
}

// =============================================================================
// WsMessage Tests
// =============================================================================

#[test]
fn test_ws_message_metrics_serialize() {
    let metrics = MetricsJson {
        project: ProjectSummary {
            name: "ws-test".to_string(),
            current_usage_usd: 5.0,
            estimated_monthly_usd: 15.0,
            daily_average_usd: 0.5,
            days_elapsed: 10,
            days_remaining: 20,
        },
        services: vec![],
        scrape_timestamp: 1700000000,
        scrape_duration_seconds: 0.1,
    };

    let msg = WsMessage::Metrics(metrics);
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Tagged enum with "type" field
    assert_eq!(parsed["type"], "metrics");
    assert!(parsed["data"]["project"].is_object());
}

#[test]
fn test_ws_message_status_serialize() {
    let status = WsStatus {
        uptime_seconds: 3600,
        memory_mb: 64.5,
        cpu_percent: 12.3,
        api: ApiStatus {
            last_success: Some(1700000000),
            last_error: None,
            total_scrapes: 100,
            failed_scrapes: 2,
        },
        ws_clients: 3,
    };

    let msg = WsMessage::Status(status);
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["type"], "status");
    assert_eq!(parsed["data"]["uptime_seconds"], 3600);
    assert_eq!(parsed["data"]["memory_mb"], 64.5);
    assert_eq!(parsed["data"]["cpu_percent"], 12.3);
    assert_eq!(parsed["data"]["ws_clients"], 3);
}

// =============================================================================
// ServerStatus Tests
// =============================================================================

#[test]
fn test_server_status_serialize() {
    let status = ServerStatus {
        version: "0.1.0",
        project_name: "test-project".to_string(),
        uptime_seconds: 7200,
        endpoints: EndpointStatus {
            prometheus: true,
            json: true,
            websocket: true,
            health: true,
        },
        config: ConfigStatus {
            plan: "pro".to_string(),
            scrape_interval_seconds: 300,
            api_url: "https://api.railway.app".to_string(),
            service_groups: vec!["monitoring".to_string(), "database".to_string()],
            prices: PriceValues {
                cpu: Some(0.000231),
                memory: Some(0.000116),
                disk: Some(0.000021),
                network: Some(NetworkPricing { tx: Some(0.10) }),
            },
            gzip: crate::config::GzipConfig::default(),
            icon_cache: IconCacheStatusConfig {
                enabled: true,
                mode: crate::config::IconMode::Base64,
                max_count: Some(200),
                max_age: None,
                base_url: None,
            },
        },
        process: ProcessStatus {
            pid: 12345,
            memory_mb: 32.5,
            cpu_percent: 5.0,
        },
        api: ApiStatus {
            last_success: Some(1700000000),
            last_error: None,
            total_scrapes: 50,
            failed_scrapes: 1,
        },
        icon_cache: Some(crate::utils::IconCacheStats::default()),
    };

    let json = serde_json::to_string(&status).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["version"], "0.1.0");
    assert_eq!(parsed["project_name"], "test-project");
    assert_eq!(parsed["uptime_seconds"], 7200);
    assert_eq!(parsed["endpoints"]["prometheus"], true);
    assert_eq!(parsed["endpoints"]["websocket"], true);
    assert_eq!(parsed["config"]["plan"], "pro");
    assert_eq!(parsed["config"]["scrape_interval_seconds"], 300);
    assert_eq!(
        parsed["config"]["service_groups"].as_array().unwrap().len(),
        2
    );
    assert_eq!(parsed["config"]["gzip"]["enabled"], true);
    assert_eq!(parsed["config"]["gzip"]["min_size"], 256);
    assert_eq!(parsed["config"]["gzip"]["level"], 1);
    assert_eq!(parsed["config"]["icon_cache"]["enabled"], true);
    assert_eq!(parsed["config"]["icon_cache"]["max_count"], 200);
    assert_eq!(parsed["process"]["pid"], 12345);
    assert_eq!(parsed["api"]["total_scrapes"], 50);
}

// =============================================================================
// GraphQL Types Tests
// =============================================================================

#[test]
fn test_graphql_request_serialize() {
    let req = GraphQLRequest {
        query: "query { project { name } }".to_string(),
    };

    let json = serde_json::to_string(&req).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["query"], "query { project { name } }");
}

#[test]
fn test_graphql_response_deserialize_success() {
    let json = r#"{"data": {"project": {"name": "my-project", "services": {"edges": []}}}}"#;
    let response: GraphQLResponse<ProjectData> = serde_json::from_str(json).unwrap();

    assert!(response.data.is_some());
    assert!(response.errors.is_none());
    assert_eq!(response.data.unwrap().project.name, "my-project");
}

#[test]
fn test_graphql_response_deserialize_error() {
    let json = r#"{"data": null, "errors": [{"message": "Not found"}]}"#;
    let response: GraphQLResponse<ProjectData> = serde_json::from_str(json).unwrap();

    assert!(response.data.is_none());
    assert!(response.errors.is_some());
    assert_eq!(response.errors.unwrap()[0].message, "Not found");
}

#[test]
fn test_project_deserialize() {
    let json = r#"{
        "name": "test-project",
        "services": {
            "edges": [
                {"node": {"id": "svc-1", "name": "web", "icon": "🌐"}},
                {"node": {"id": "svc-2", "name": "api", "icon": null}}
            ]
        }
    }"#;

    let project: Project = serde_json::from_str(json).unwrap();

    assert_eq!(project.name, "test-project");
    assert_eq!(project.services.edges.len(), 2);
    assert_eq!(project.services.edges[0].node.id, "svc-1");
    assert_eq!(project.services.edges[0].node.name, "web");
    assert_eq!(project.services.edges[0].node.icon, Some("🌐".to_string()));
    assert_eq!(project.services.edges[1].node.icon, None);
}

#[test]
fn test_usage_item_deserialize() {
    let json = r#"{
        "measurement": "CPU_USAGE",
        "value": 123.456,
        "tags": {"serviceId": "svc-123"}
    }"#;

    let item: UsageItem = serde_json::from_str(json).unwrap();

    assert_eq!(item.measurement, "CPU_USAGE");
    assert_eq!(item.value, 123.456);
    assert_eq!(item.tags.service_id, "svc-123");
}

#[test]
fn test_usage_data_deserialize() {
    let json = r#"{
        "usage": [
            {"measurement": "CPU_USAGE", "value": 100.0, "tags": {"serviceId": "svc-1"}},
            {"measurement": "MEMORY_USAGE", "value": 256.0, "tags": {"serviceId": "svc-1"}}
        ]
    }"#;

    let data: UsageData = serde_json::from_str(json).unwrap();

    assert_eq!(data.usage.len(), 2);
    assert_eq!(data.usage[0].measurement, "CPU_USAGE");
    assert_eq!(data.usage[1].measurement, "MEMORY_USAGE");
}

#[test]
fn test_estimated_item_deserialize() {
    let json = r#"{
        "measurement": "CPU_USAGE",
        "estimatedValue": 500.0
    }"#;

    let item: EstimatedItem = serde_json::from_str(json).unwrap();

    assert_eq!(item.measurement, "CPU_USAGE");
    assert_eq!(item.estimated_value, 500.0);
}

#[test]
fn test_estimated_data_deserialize() {
    let json = r#"{
        "estimatedUsage": [
            {"measurement": "CPU_USAGE", "estimatedValue": 500.0},
            {"measurement": "MEMORY_USAGE", "estimatedValue": 1000.0}
        ]
    }"#;

    let data: EstimatedData = serde_json::from_str(json).unwrap();

    assert_eq!(data.estimated_usage.len(), 2);
    assert_eq!(data.estimated_usage[0].estimated_value, 500.0);
    assert_eq!(data.estimated_usage[1].estimated_value, 1000.0);
}

// =============================================================================
// ApiStatus Tests
// =============================================================================

#[test]
fn test_api_status_with_error() {
    let status = ApiStatus {
        last_success: Some(1700000000),
        last_error: Some("Connection timeout".to_string()),
        total_scrapes: 100,
        failed_scrapes: 5,
    };

    let json = serde_json::to_string(&status).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["last_success"], 1700000000);
    assert_eq!(parsed["last_error"], "Connection timeout");
    assert_eq!(parsed["total_scrapes"], 100);
    assert_eq!(parsed["failed_scrapes"], 5);
}

#[test]
fn test_api_status_no_error() {
    let status = ApiStatus {
        last_success: Some(1700000000),
        last_error: None,
        total_scrapes: 100,
        failed_scrapes: 0,
    };

    let json = serde_json::to_string(&status).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["last_error"], serde_json::Value::Null);
    assert_eq!(parsed["failed_scrapes"], 0);
}

// =============================================================================
// EndpointStatus Tests
// =============================================================================

#[test]
fn test_endpoint_status_all_enabled() {
    let status = EndpointStatus {
        prometheus: true,
        json: true,
        websocket: true,
        health: true,
    };

    let json = serde_json::to_string(&status).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["prometheus"], true);
    assert_eq!(parsed["json"], true);
    assert_eq!(parsed["websocket"], true);
    assert_eq!(parsed["health"], true);
}

#[test]
fn test_endpoint_status_websocket_disabled() {
    let status = EndpointStatus {
        prometheus: true,
        json: true,
        websocket: false,
        health: true,
    };

    let json = serde_json::to_string(&status).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["websocket"], false);
}

// =============================================================================
// ProcessStatus Tests
// =============================================================================

#[test]
fn test_process_status_serialize() {
    let status = ProcessStatus {
        pid: 99999,
        memory_mb: 128.75,
        cpu_percent: 25.5,
    };

    let json = serde_json::to_string(&status).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["pid"], 99999);
    assert_eq!(parsed["memory_mb"], 128.75);
    assert_eq!(parsed["cpu_percent"], 25.5);
}
