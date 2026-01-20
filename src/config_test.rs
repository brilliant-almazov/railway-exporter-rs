//! Configuration tests for Railway Exporter.

use crate::config::{Config, ConfigError, GzipConfig, Plan, YamlConfig};
use std::str::FromStr;

// =============================================================================
// Plan Tests
// =============================================================================

#[test]
fn test_plan_from_str() {
    assert_eq!(Plan::from_str("hobby").unwrap(), Plan::Hobby);
    assert_eq!(Plan::from_str("HOBBY").unwrap(), Plan::Hobby);
    assert_eq!(Plan::from_str("pro").unwrap(), Plan::Pro);
    assert_eq!(Plan::from_str("PRO").unwrap(), Plan::Pro);
    assert_eq!(Plan::from_str("Pro").unwrap(), Plan::Pro);
}

#[test]
fn test_plan_from_str_invalid() {
    let result = Plan::from_str("enterprise");
    assert!(result.is_err());
    match result.unwrap_err() {
        ConfigError::InvalidPlan(v) => assert_eq!(v, "enterprise"),
        _ => panic!("Expected InvalidPlan error"),
    }
}

#[test]
fn test_plan_display() {
    assert_eq!(format!("{}", Plan::Hobby), "hobby");
    assert_eq!(format!("{}", Plan::Pro), "pro");
}

#[test]
fn test_plan_as_str() {
    assert_eq!(Plan::Hobby.as_str(), "hobby");
    assert_eq!(Plan::Pro.as_str(), "pro");
}

#[test]
fn test_plan_default() {
    assert_eq!(Plan::default(), Plan::Hobby);
}

// =============================================================================
// Config Tests
// =============================================================================

#[test]
fn test_config_new() {
    let config = Config::new("token", "project", Plan::Pro, 60, 8080);
    assert_eq!(config.api_token, "token");
    assert_eq!(config.project_id, "project");
    assert_eq!(config.plan, Plan::Pro);
    assert_eq!(config.scrape_interval, 60);
    assert_eq!(config.port, 8080);
    assert!(config.service_groups.is_empty());
}

#[test]
fn test_config_api_url() {
    let config = Config::new("t", "p", Plan::Hobby, 60, 8080);
    assert_eq!(config.api_url, "https://backboard.railway.app/graphql/v2");
}

#[test]
fn test_config_pricing_uses_plan() {
    let hobby = Config::new("t", "p", Plan::Hobby, 60, 8080);
    let pro = Config::new("t", "p", Plan::Pro, 60, 8080);

    // Pro should have lower CPU price
    assert!(pro.pricing.get_price("CPU_USAGE") < hobby.pricing.get_price("CPU_USAGE"));
}

#[test]
fn test_config_default_gzip() {
    let config = Config::new("t", "p", Plan::Hobby, 60, 8080);
    assert!(config.gzip.enabled);
    assert_eq!(config.gzip.min_size, 256);
    assert_eq!(config.gzip.level, 1);
}

// =============================================================================
// ConfigError Tests
// =============================================================================

#[test]
fn test_config_error_display() {
    let err = ConfigError::MissingValue("TEST".to_string());
    assert_eq!(format!("{}", err), "Missing required config: TEST");

    let err = ConfigError::YamlError("invalid".to_string());
    assert_eq!(format!("{}", err), "YAML parse error: invalid");

    let err = ConfigError::ParseError("PORT".to_string(), "not a number".to_string());
    assert_eq!(format!("{}", err), "Failed to parse PORT: not a number");

    let err = ConfigError::FileError("not found".to_string());
    assert_eq!(format!("{}", err), "Config file error: not found");

    let err = ConfigError::Base64Error("invalid".to_string());
    assert_eq!(format!("{}", err), "Base64 decode error: invalid");

    let err = ConfigError::InvalidPlan("enterprise".to_string());
    assert_eq!(
        format!("{}", err),
        "Invalid plan 'enterprise': must be 'hobby' or 'pro'"
    );
}

// =============================================================================
// GzipConfig Tests
// =============================================================================

#[test]
fn test_gzip_config_default() {
    let gzip = GzipConfig::default();
    assert!(gzip.enabled);
    assert_eq!(gzip.min_size, 256);
    assert_eq!(gzip.level, 1);
}

#[test]
fn test_gzip_config_deserialize_defaults() {
    let yaml = "{}";
    let gzip: GzipConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(gzip.enabled);
    assert_eq!(gzip.min_size, 256);
    assert_eq!(gzip.level, 1);
}

#[test]
fn test_gzip_config_deserialize_custom() {
    let yaml = r#"
enabled: false
min_size: 1024
level: 6
"#;
    let gzip: GzipConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(!gzip.enabled);
    assert_eq!(gzip.min_size, 1024);
    assert_eq!(gzip.level, 6);
}

#[test]
fn test_gzip_config_deserialize_partial() {
    let yaml = r#"
enabled: true
min_size: 512
"#;
    let gzip: GzipConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(gzip.enabled);
    assert_eq!(gzip.min_size, 512);
    assert_eq!(gzip.level, 1); // default
}

// =============================================================================
// YamlConfig Tests
// =============================================================================

#[test]
fn test_yaml_config_deserialize() {
    let yaml = r#"
railway_api_token: test-token
railway_project_id: test-project
railway_plan: pro
port: 9090
scrape_interval: 120
gzip:
  enabled: true
  min_size: 512
  level: 3
service_groups:
  monitoring:
    - prometheus
    - grafana
  database:
    - postgres
"#;
    let config: YamlConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.railway_api_token.unwrap(), "test-token");
    assert_eq!(config.railway_project_id.unwrap(), "test-project");
    assert_eq!(config.railway_plan.unwrap(), Plan::Pro);
    assert_eq!(config.port.unwrap(), 9090);
    assert_eq!(config.scrape_interval.unwrap(), 120);

    let gzip = config.gzip.unwrap();
    assert!(gzip.enabled);
    assert_eq!(gzip.min_size, 512);
    assert_eq!(gzip.level, 3);

    let groups = config.service_groups.unwrap();
    assert_eq!(
        groups.get("monitoring").unwrap(),
        &vec!["prometheus", "grafana"]
    );
    assert_eq!(groups.get("database").unwrap(), &vec!["postgres"]);
}
