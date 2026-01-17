use chrono::{Datelike, Utc};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use prometheus::{Encoder, GaugeVec, Opts, Registry, TextEncoder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::time::{interval, Duration};
use tracing::{error, info};

const RAILWAY_API_URL: &str = "https://backboard.railway.app/graphql/v2";

#[derive(Clone)]
struct Config {
    api_token: String,
    project_id: String,
    plan: String,
    scrape_interval: u64,
    port: u16,
}

fn get_price(plan: &str, measurement: &str) -> f64 {
    match (plan, measurement) {
        ("pro", "CPU_USAGE") => 0.000231,
        ("pro", "MEMORY_USAGE_GB") => 0.000116,
        (_, "CPU_USAGE") => 0.000463,
        (_, "MEMORY_USAGE_GB") => 0.000231,
        (_, "DISK_USAGE_GB") => 0.000021,
        (_, "NETWORK_TX_GB") => 0.10,
        _ => 0.0,
    }
}

#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
}

#[derive(Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Deserialize)]
struct ProjectData {
    project: Project,
}

#[derive(Deserialize)]
struct Project {
    name: String,
    services: ServiceEdges,
}

#[derive(Deserialize)]
struct ServiceEdges {
    edges: Vec<ServiceEdge>,
}

#[derive(Deserialize)]
struct ServiceEdge {
    node: ServiceNode,
}

#[derive(Deserialize)]
struct ServiceNode {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct UsageData {
    usage: Vec<UsageItem>,
}

#[derive(Deserialize)]
struct UsageItem {
    measurement: String,
    value: f64,
    tags: UsageTags,
}

#[derive(Deserialize)]
struct UsageTags {
    #[serde(rename = "serviceId")]
    service_id: String,
}

#[derive(Deserialize)]
struct EstimatedData {
    #[serde(rename = "estimatedUsage")]
    estimated_usage: Vec<EstimatedItem>,
}

#[derive(Deserialize)]
struct EstimatedItem {
    measurement: String,
    #[serde(rename = "estimatedValue")]
    estimated_value: f64,
}

struct Metrics {
    cpu_usage: GaugeVec,
    memory_usage: GaugeVec,
    disk_usage: GaugeVec,
    network_tx: GaugeVec,
    network_rx: GaugeVec,
    service_cost: GaugeVec,
    service_estimated_monthly: GaugeVec,
    current_usage: GaugeVec,
    estimated_monthly: GaugeVec,
    daily_average: GaugeVec,
    registry: Registry,
}

impl Metrics {
    fn new() -> Self {
        let registry = Registry::new();

        let cpu_usage = GaugeVec::new(
            Opts::new("railway_cpu_usage_vcpu_minutes", "CPU usage"),
            &["service", "project"],
        )
        .unwrap();
        let memory_usage = GaugeVec::new(
            Opts::new("railway_memory_usage_gb_minutes", "Memory usage"),
            &["service", "project"],
        )
        .unwrap();
        let disk_usage = GaugeVec::new(
            Opts::new("railway_disk_usage_gb_minutes", "Disk usage"),
            &["service", "project"],
        )
        .unwrap();
        let network_tx = GaugeVec::new(
            Opts::new("railway_network_tx_gb", "Network TX"),
            &["service", "project"],
        )
        .unwrap();
        let network_rx = GaugeVec::new(
            Opts::new("railway_network_rx_gb", "Network RX"),
            &["service", "project"],
        )
        .unwrap();
        let service_cost = GaugeVec::new(
            Opts::new("railway_service_cost_usd", "Service cost"),
            &["service", "project"],
        )
        .unwrap();
        let service_estimated_monthly = GaugeVec::new(
            Opts::new(
                "railway_service_estimated_monthly_usd",
                "Service estimated monthly",
            ),
            &["service", "project"],
        )
        .unwrap();
        let current_usage = GaugeVec::new(
            Opts::new("railway_current_usage_usd", "Current usage"),
            &["project"],
        )
        .unwrap();
        let estimated_monthly = GaugeVec::new(
            Opts::new("railway_estimated_monthly_usd", "Estimated monthly"),
            &["project"],
        )
        .unwrap();
        let daily_average = GaugeVec::new(
            Opts::new("railway_daily_average_usd", "Daily average"),
            &["project"],
        )
        .unwrap();

        registry.register(Box::new(cpu_usage.clone())).unwrap();
        registry.register(Box::new(memory_usage.clone())).unwrap();
        registry.register(Box::new(disk_usage.clone())).unwrap();
        registry.register(Box::new(network_tx.clone())).unwrap();
        registry.register(Box::new(network_rx.clone())).unwrap();
        registry.register(Box::new(service_cost.clone())).unwrap();
        registry
            .register(Box::new(service_estimated_monthly.clone()))
            .unwrap();
        registry.register(Box::new(current_usage.clone())).unwrap();
        registry
            .register(Box::new(estimated_monthly.clone()))
            .unwrap();
        registry.register(Box::new(daily_average.clone())).unwrap();

        Self {
            cpu_usage,
            memory_usage,
            disk_usage,
            network_tx,
            network_rx,
            service_cost,
            service_estimated_monthly,
            current_usage,
            estimated_monthly,
            daily_average,
            registry,
        }
    }

    fn encode(&self) -> String {
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder
            .encode(&self.registry.gather(), &mut buffer)
            .unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

async fn graphql_query<T: for<'de> Deserialize<'de>>(
    client: &Client,
    token: &str,
    query: &str,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
    let resp = client
        .post(RAILWAY_API_URL)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&GraphQLRequest {
            query: query.to_string(),
        })
        .send()
        .await?;
    let gql_resp: GraphQLResponse<T> = resp.json().await?;
    if let Some(errors) = gql_resp.errors {
        if !errors.is_empty() {
            return Err(format!("GraphQL error: {}", errors[0].message).into());
        }
    }
    gql_resp.data.ok_or("No data".into())
}

async fn collect_metrics(
    client: &Client,
    config: &Config,
    metrics: &Metrics,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let start = std::time::Instant::now();

    let query = format!(
        r#"{{ project(id: "{}") {{ name services {{ edges {{ node {{ id name }} }} }} }} }}"#,
        config.project_id
    );
    let project_data: ProjectData = graphql_query(client, &config.api_token, &query).await?;
    let project_name = &project_data.project.name;
    let services: HashMap<String, String> = project_data
        .project
        .services
        .edges
        .iter()
        .map(|e| (e.node.id.clone(), e.node.name.clone()))
        .collect();

    let query = format!(
        r#"{{ usage(projectId: "{}", measurements: [CPU_USAGE, MEMORY_USAGE_GB, DISK_USAGE_GB, NETWORK_TX_GB, NETWORK_RX_GB], groupBy: [SERVICE_ID]) {{ measurement value tags {{ serviceId }} }} }}"#,
        config.project_id
    );
    let usage_data: UsageData = graphql_query(client, &config.api_token, &query).await?;

    let mut service_usage: HashMap<String, HashMap<String, f64>> = HashMap::new();
    for item in &usage_data.usage {
        service_usage
            .entry(item.tags.service_id.clone())
            .or_default()
            .insert(item.measurement.clone(), item.value);
    }

    let mut total_cost = 0.0;
    for (sid, measurements) in &service_usage {
        let name = services.get(sid).unwrap_or(sid);
        let cpu = *measurements.get("CPU_USAGE").unwrap_or(&0.0);
        let mem = *measurements.get("MEMORY_USAGE_GB").unwrap_or(&0.0);
        let disk = *measurements.get("DISK_USAGE_GB").unwrap_or(&0.0);
        let tx = *measurements.get("NETWORK_TX_GB").unwrap_or(&0.0);
        let rx = *measurements.get("NETWORK_RX_GB").unwrap_or(&0.0);

        metrics
            .cpu_usage
            .with_label_values(&[name, project_name])
            .set(cpu);
        metrics
            .memory_usage
            .with_label_values(&[name, project_name])
            .set(mem);
        metrics
            .disk_usage
            .with_label_values(&[name, project_name])
            .set(disk);
        metrics
            .network_tx
            .with_label_values(&[name, project_name])
            .set(tx);
        metrics
            .network_rx
            .with_label_values(&[name, project_name])
            .set(rx);

        let cost = cpu * get_price(&config.plan, "CPU_USAGE")
            + mem * get_price(&config.plan, "MEMORY_USAGE_GB")
            + disk * get_price(&config.plan, "DISK_USAGE_GB")
            + tx * get_price(&config.plan, "NETWORK_TX_GB");
        metrics
            .service_cost
            .with_label_values(&[name, project_name])
            .set(cost);
        total_cost += cost;
    }

    let query = format!(
        r#"{{ estimatedUsage(projectId: "{}", measurements: [CPU_USAGE, MEMORY_USAGE_GB, DISK_USAGE_GB, NETWORK_TX_GB, NETWORK_RX_GB]) {{ measurement estimatedValue }} }}"#,
        config.project_id
    );
    let est_data: EstimatedData = graphql_query(client, &config.api_token, &query).await?;
    let est_monthly: f64 = est_data
        .estimated_usage
        .iter()
        .map(|i| i.estimated_value * get_price(&config.plan, &i.measurement))
        .sum();

    if total_cost > 0.0 {
        for (sid, measurements) in &service_usage {
            let name = services.get(sid).unwrap_or(sid);
            let cost = measurements.get("CPU_USAGE").unwrap_or(&0.0)
                * get_price(&config.plan, "CPU_USAGE")
                + measurements.get("MEMORY_USAGE_GB").unwrap_or(&0.0)
                    * get_price(&config.plan, "MEMORY_USAGE_GB")
                + measurements.get("DISK_USAGE_GB").unwrap_or(&0.0)
                    * get_price(&config.plan, "DISK_USAGE_GB")
                + measurements.get("NETWORK_TX_GB").unwrap_or(&0.0)
                    * get_price(&config.plan, "NETWORK_TX_GB");
            metrics
                .service_estimated_monthly
                .with_label_values(&[name, project_name])
                .set(est_monthly * cost / total_cost);
        }
    }

    metrics
        .current_usage
        .with_label_values(&[project_name])
        .set(total_cost);
    metrics
        .estimated_monthly
        .with_label_values(&[project_name])
        .set(est_monthly);
    metrics
        .daily_average
        .with_label_values(&[project_name])
        .set(total_cost / Utc::now().day() as f64);

    info!(
        "Collected in {:.2}s. Current: ${:.2}, Estimated: ${:.2}/month",
        start.elapsed().as_secs_f64(),
        total_cost,
        est_monthly
    );
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config {
        api_token: env::var("RAILWAY_API_TOKEN").expect("RAILWAY_API_TOKEN required"),
        project_id: env::var("RAILWAY_PROJECT_ID").expect("RAILWAY_PROJECT_ID required"),
        plan: env::var("RAILWAY_PLAN").unwrap_or_else(|_| "hobby".to_string()),
        scrape_interval: env::var("SCRAPE_INTERVAL")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .unwrap_or(300),
        port: env::var("PORT")
            .unwrap_or_else(|_| "9333".to_string())
            .parse()
            .unwrap_or(9333),
    };

    info!("Using {} plan pricing", config.plan);

    let client = Client::new();
    let metrics = Arc::new(Metrics::new());

    if let Err(e) = collect_metrics(&client, &config, &metrics).await {
        error!("Initial collection failed: {}", e);
    }

    let metrics_bg = metrics.clone();
    let config_bg = config.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(config_bg.scrape_interval));
        let client = Client::new();
        loop {
            interval.tick().await;
            let _ = collect_metrics(&client, &config_bg, &metrics_bg).await;
        }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("Listening on {}", addr);

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);
        let metrics = metrics.clone();
        tokio::spawn(async move {
            let svc = service_fn(move |req: Request<hyper::body::Incoming>| {
                let metrics = metrics.clone();
                async move {
                    let resp = match req.uri().path() {
                        "/metrics" => Response::builder()
                            .header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
                            .body(Full::new(Bytes::from(metrics.encode())))
                            .unwrap(),
                        "/health" => Response::new(Full::new(Bytes::from("ok"))),
                        _ => Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Full::new(Bytes::from("Not Found")))
                            .unwrap(),
                    };
                    Ok::<_, Infallible>(resp)
                }
            });
            let _ = http1::Builder::new().serve_connection(io, svc).await;
        });
    }
}
