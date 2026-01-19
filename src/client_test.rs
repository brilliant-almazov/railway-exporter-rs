//! Tests for Railway API client.

use crate::client::{ApiError, Client, EstimatedItem, GraphQLRequest, Project, UsageItem};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use tokio::net::TcpListener;

// =============================================================================
// Mock GraphQL Server Helpers
// =============================================================================

async fn start_mock_server<F, Fut>(handler: F) -> String
where
    F: Fn(Request<hyper::body::Incoming>) -> Fut + Send + 'static + Clone,
    Fut: std::future::Future<Output = Response<Full<Bytes>>> + Send,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}/graphql", addr);

    tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);
            let handler = handler.clone();

            tokio::spawn(async move {
                let service = service_fn(move |req: Request<hyper::body::Incoming>| {
                    let handler = handler.clone();
                    async move { Ok::<_, Infallible>(handler(req).await) }
                });
                let _ = http1::Builder::new().serve_connection(io, service).await;
            });
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    url
}

// =============================================================================
// get_project Tests
// =============================================================================

#[tokio::test]
async fn test_get_project_success() {
    let url = start_mock_server(|_req| async {
        let response = r#"{
            "data": {
                "project": {
                    "name": "my-project",
                    "services": {
                        "edges": [
                            { "node": { "id": "svc-1", "name": "api", "icon": "🚀" } },
                            { "node": { "id": "svc-2", "name": "web", "icon": null } }
                        ]
                    }
                }
            }
        }"#;

        Response::builder()
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(response)))
            .unwrap()
    })
    .await;

    let client = Client::new("test-token", Some(&url));
    let project = client.get_project("project-123").await.unwrap();

    assert_eq!(project.name, "my-project");
    assert_eq!(project.services.edges.len(), 2);
    assert_eq!(project.services.edges[0].node.id, "svc-1");
    assert_eq!(project.services.edges[0].node.name, "api");
    assert_eq!(project.services.edges[0].node.icon, Some("🚀".to_string()));
    assert_eq!(project.services.edges[1].node.icon, None);
}

#[tokio::test]
async fn test_get_project_graphql_error() {
    let url = start_mock_server(|_req| async {
        let response = r#"{
            "data": null,
            "errors": [{ "message": "Project not found" }]
        }"#;

        Response::builder()
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(response)))
            .unwrap()
    })
    .await;

    let client = Client::new("test-token", Some(&url));
    let result = client.get_project("invalid-project").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::GraphQLError(msg) => assert!(msg.contains("Project not found")),
        e => panic!("Expected GraphQLError, got {:?}", e),
    }
}

#[tokio::test]
async fn test_get_project_no_data() {
    let url = start_mock_server(|_req| async {
        let response = r#"{ "data": null }"#;

        Response::builder()
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(response)))
            .unwrap()
    })
    .await;

    let client = Client::new("test-token", Some(&url));
    let result = client.get_project("project-123").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::NoData => {}
        e => panic!("Expected NoData, got {:?}", e),
    }
}

#[tokio::test]
async fn test_get_project_parse_error() {
    let url = start_mock_server(|_req| async {
        Response::builder()
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from("invalid json{{{{")))
            .unwrap()
    })
    .await;

    let client = Client::new("test-token", Some(&url));
    let result = client.get_project("project-123").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::ParseError(_) => {}
        e => panic!("Expected ParseError, got {:?}", e),
    }
}

// =============================================================================
// get_usage Tests
// =============================================================================

#[tokio::test]
async fn test_get_usage_success() {
    let url = start_mock_server(|_req| async {
        let response = r#"{
            "data": {
                "usage": [
                    { "measurement": "CPU_USAGE", "value": 100.5, "tags": { "serviceId": "svc-1" } },
                    { "measurement": "MEMORY_USAGE_GB", "value": 256.0, "tags": { "serviceId": "svc-1" } },
                    { "measurement": "CPU_USAGE", "value": 50.0, "tags": { "serviceId": "svc-2" } }
                ]
            }
        }"#;

        Response::builder()
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(response)))
            .unwrap()
    })
    .await;

    let client = Client::new("test-token", Some(&url));
    let usage = client.get_usage("project-123").await.unwrap();

    // Check svc-1 has both CPU and MEMORY
    let svc1 = usage.get("svc-1").unwrap();
    assert_eq!(*svc1.get("CPU_USAGE").unwrap(), 100.5);
    assert_eq!(*svc1.get("MEMORY_USAGE_GB").unwrap(), 256.0);

    // Check svc-2 has CPU only
    let svc2 = usage.get("svc-2").unwrap();
    assert_eq!(*svc2.get("CPU_USAGE").unwrap(), 50.0);
    assert!(svc2.get("MEMORY_USAGE_GB").is_none());
}

#[tokio::test]
async fn test_get_usage_empty() {
    let url = start_mock_server(|_req| async {
        let response = r#"{ "data": { "usage": [] } }"#;

        Response::builder()
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(response)))
            .unwrap()
    })
    .await;

    let client = Client::new("test-token", Some(&url));
    let usage = client.get_usage("project-123").await.unwrap();

    assert!(usage.is_empty());
}

// =============================================================================
// get_estimated_usage Tests
// =============================================================================

#[tokio::test]
async fn test_get_estimated_usage_success() {
    let url = start_mock_server(|_req| async {
        let response = r#"{
            "data": {
                "estimatedUsage": [
                    { "measurement": "CPU_USAGE", "estimatedValue": 5000.0 },
                    { "measurement": "MEMORY_USAGE_GB", "estimatedValue": 10000.0 },
                    { "measurement": "NETWORK_TX_GB", "estimatedValue": 50.0 }
                ]
            }
        }"#;

        Response::builder()
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(response)))
            .unwrap()
    })
    .await;

    let client = Client::new("test-token", Some(&url));
    let estimated = client.get_estimated_usage("project-123").await.unwrap();

    assert_eq!(*estimated.get("CPU_USAGE").unwrap(), 5000.0);
    assert_eq!(*estimated.get("MEMORY_USAGE_GB").unwrap(), 10000.0);
    assert_eq!(*estimated.get("NETWORK_TX_GB").unwrap(), 50.0);
}

#[tokio::test]
async fn test_get_estimated_usage_empty() {
    let url = start_mock_server(|_req| async {
        let response = r#"{ "data": { "estimatedUsage": [] } }"#;

        Response::builder()
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(response)))
            .unwrap()
    })
    .await;

    let client = Client::new("test-token", Some(&url));
    let estimated = client.get_estimated_usage("project-123").await.unwrap();

    assert!(estimated.is_empty());
}

// =============================================================================
// Network Error Tests
// =============================================================================

#[tokio::test]
async fn test_request_error_connection_refused() {
    // Use a port that's not listening
    let client = Client::new("test-token", Some("http://127.0.0.1:1"));
    let result = client.get_project("project-123").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::RequestError(_) => {}
        e => panic!("Expected RequestError, got {:?}", e),
    }
}

// =============================================================================
// Authorization Header Tests
// =============================================================================

#[tokio::test]
async fn test_sends_auth_header() {
    use std::sync::{Arc, Mutex};

    let received_token = Arc::new(Mutex::new(String::new()));
    let received_token_clone = received_token.clone();

    let url = start_mock_server(move |req| {
        let token = req
            .headers()
            .get("authorization")
            .map(|v| v.to_str().unwrap().to_string())
            .unwrap_or_default();
        *received_token_clone.lock().unwrap() = token;

        async {
            let response = r#"{ "data": { "project": { "name": "test", "services": { "edges": [] } } } }"#;
            Response::builder()
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(response)))
                .unwrap()
        }
    })
    .await;

    let client = Client::new("my-secret-token", Some(&url));
    let _ = client.get_project("project-123").await;

    let token = received_token.lock().unwrap().clone();
    assert_eq!(token, "Bearer my-secret-token");
}

// =============================================================================
// Content-Type Header Tests
// =============================================================================

#[tokio::test]
async fn test_sends_content_type_json() {
    use std::sync::{Arc, Mutex};

    let received_content_type = Arc::new(Mutex::new(String::new()));
    let received_content_type_clone = received_content_type.clone();

    let url = start_mock_server(move |req| {
        let ct = req
            .headers()
            .get("content-type")
            .map(|v| v.to_str().unwrap().to_string())
            .unwrap_or_default();
        *received_content_type_clone.lock().unwrap() = ct;

        async {
            let response = r#"{ "data": { "project": { "name": "test", "services": { "edges": [] } } } }"#;
            Response::builder()
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(response)))
                .unwrap()
        }
    })
    .await;

    let client = Client::new("token", Some(&url));
    let _ = client.get_project("project-123").await;

    let ct = received_content_type.lock().unwrap().clone();
    assert_eq!(ct, "application/json");
}

// =============================================================================
// ApiError Display Tests
// =============================================================================

#[test]
fn test_api_error_display_parse_error() {
    let err = ApiError::ParseError("invalid JSON".to_string());
    assert_eq!(format!("{}", err), "Parse error: invalid JSON");
}

#[test]
fn test_api_error_display_request_error() {
    let err = ApiError::RequestError("connection refused".to_string());
    assert_eq!(format!("{}", err), "Request error: connection refused");
}

#[test]
fn test_api_error_display_graphql_error() {
    let err = ApiError::GraphQLError("unauthorized".to_string());
    assert_eq!(format!("{}", err), "GraphQL error: unauthorized");
}

#[test]
fn test_api_error_display_no_data() {
    let err = ApiError::NoData;
    assert_eq!(format!("{}", err), "No data in response");
}

// =============================================================================
// Serialization/Deserialization Unit Tests
// =============================================================================

#[test]
fn test_graphql_request_serialize() {
    let req = GraphQLRequest {
        query: "{ test }".to_string(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("query"));
    assert!(json.contains("{ test }"));
}

// Note: test_client_new and test_client_custom_url removed
// because they test private fields. Client behavior is tested
// through mock server integration tests above.

#[test]
fn test_usage_item_deserialize() {
    let json = r#"{
        "measurement": "CPU_USAGE",
        "value": 1234.5,
        "tags": { "serviceId": "svc-123" }
    }"#;
    let item: UsageItem = serde_json::from_str(json).unwrap();
    assert_eq!(item.measurement, "CPU_USAGE");
    assert_eq!(item.value, 1234.5);
    assert_eq!(item.tags.service_id, "svc-123");
}

#[test]
fn test_estimated_item_deserialize() {
    let json = r#"{
        "measurement": "MEMORY_USAGE_GB",
        "estimatedValue": 5000.0
    }"#;
    let item: EstimatedItem = serde_json::from_str(json).unwrap();
    assert_eq!(item.measurement, "MEMORY_USAGE_GB");
    assert_eq!(item.estimated_value, 5000.0);
}

#[test]
fn test_project_deserialize() {
    let json = r#"{
        "name": "my-project",
        "services": {
            "edges": [
                { "node": { "id": "svc-1", "name": "api" } },
                { "node": { "id": "svc-2", "name": "web" } }
            ]
        }
    }"#;
    let project: Project = serde_json::from_str(json).unwrap();
    assert_eq!(project.name, "my-project");
    assert_eq!(project.services.edges.len(), 2);
    assert_eq!(project.services.edges[0].node.name, "api");
}
