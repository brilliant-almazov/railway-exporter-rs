//! Unit tests for Railway pricing calculations.

use crate::pricing::{calculate_cost, get_price, PricingConfig};
use std::collections::HashMap;

// =============================================================================
// get_price Tests
// =============================================================================

#[test]
fn test_get_price_pro_cpu() {
    assert_eq!(get_price("pro", "CPU_USAGE"), 0.000231);
}

#[test]
fn test_get_price_hobby_cpu() {
    assert_eq!(get_price("hobby", "CPU_USAGE"), 0.000463);
}

#[test]
fn test_get_price_pro_memory() {
    assert_eq!(get_price("pro", "MEMORY_USAGE_GB"), 0.000116);
}

#[test]
fn test_get_price_hobby_memory() {
    assert_eq!(get_price("hobby", "MEMORY_USAGE_GB"), 0.000231);
}

#[test]
fn test_get_price_disk_same_for_all_plans() {
    assert_eq!(get_price("pro", "DISK_USAGE_GB"), 0.000021);
    assert_eq!(get_price("hobby", "DISK_USAGE_GB"), 0.000021);
}

#[test]
fn test_get_price_network_tx() {
    assert_eq!(get_price("pro", "NETWORK_TX_GB"), 0.10);
    assert_eq!(get_price("hobby", "NETWORK_TX_GB"), 0.10);
}

// NETWORK_RX removed - ingress is free, no need to track

#[test]
fn test_get_price_unknown_measurement() {
    assert_eq!(get_price("pro", "UNKNOWN"), 0.0);
}

#[test]
fn test_get_price_case_insensitive_plan() {
    assert_eq!(get_price("PRO", "CPU_USAGE"), 0.000231);
    assert_eq!(get_price("Pro", "CPU_USAGE"), 0.000231);
}

// =============================================================================
// PricingConfig Tests
// =============================================================================

#[test]
fn test_pricing_config_default_prices() {
    let config = PricingConfig::new("pro");
    assert_eq!(config.get_price("CPU_USAGE"), 0.000231);
    assert_eq!(config.get_price("MEMORY_USAGE_GB"), 0.000116);
}

#[test]
fn test_pricing_config_custom_override() {
    let mut config = PricingConfig::new("pro");
    config.set_price("CPU_USAGE", 0.0005);
    assert_eq!(config.get_price("CPU_USAGE"), 0.0005);
    // Other prices remain default
    assert_eq!(config.get_price("MEMORY_USAGE_GB"), 0.000116);
}

#[test]
fn test_pricing_config_plan() {
    let config = PricingConfig::new("Pro");
    // Plan should be lowercased
    assert_eq!(config.plan(), "pro");

    let config_hobby = PricingConfig::new("HOBBY");
    assert_eq!(config_hobby.plan(), "hobby");
}

// =============================================================================
// calculate_cost Tests
// =============================================================================

#[test]
fn test_calculate_cost_empty() {
    let usage = HashMap::new();
    assert_eq!(calculate_cost("pro", &usage), 0.0);
}

#[test]
fn test_calculate_cost_single_measurement() {
    let mut usage = HashMap::new();
    usage.insert("CPU_USAGE".to_string(), 1000.0);
    let cost = calculate_cost("pro", &usage);
    assert!((cost - 0.231).abs() < 0.0001);
}

#[test]
fn test_calculate_cost_multiple_measurements() {
    let mut usage = HashMap::new();
    usage.insert("CPU_USAGE".to_string(), 1000.0);
    usage.insert("MEMORY_USAGE_GB".to_string(), 1000.0);
    usage.insert("DISK_USAGE_GB".to_string(), 1000.0);

    let cost = calculate_cost("pro", &usage);
    let expected = 1000.0 * 0.000231 + 1000.0 * 0.000116 + 1000.0 * 0.000021;
    assert!((cost - expected).abs() < 0.0001);
}

#[test]
fn test_hobby_is_more_expensive_than_pro() {
    let mut usage = HashMap::new();
    usage.insert("CPU_USAGE".to_string(), 1000.0);
    usage.insert("MEMORY_USAGE_GB".to_string(), 1000.0);

    let hobby_cost = calculate_cost("hobby", &usage);
    let pro_cost = calculate_cost("pro", &usage);

    assert!(hobby_cost > pro_cost);
}
