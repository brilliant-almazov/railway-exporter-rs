//! Tests for process info provider.

use super::process_info::ProcessInfoProvider;

#[test]
fn test_process_info_provider_new() {
    let provider = ProcessInfoProvider::new();
    // Should create without panic
    assert!(provider.pid() > 0);
}

#[test]
fn test_process_info_provider_default() {
    let provider = ProcessInfoProvider::default();
    // Default should work the same as new
    assert!(provider.pid() > 0);
}

#[test]
fn test_process_info_provider_pid() {
    let provider = ProcessInfoProvider::new();
    let pid = provider.pid();

    // PID should be positive
    assert!(pid > 0);

    // PID should be consistent
    assert_eq!(provider.pid(), pid);
}

#[test]
fn test_process_info_provider_status() {
    let provider = ProcessInfoProvider::new();
    let status = provider.status();

    // PID should match
    assert_eq!(status.pid, provider.pid());

    // Memory should be positive (process is using some memory)
    assert!(status.memory_mb >= 0.0);

    // CPU can be 0 on first call (sysinfo needs time to collect)
    assert!(status.cpu_percent >= 0.0);
}

#[test]
fn test_process_info_provider_status_multiple_calls() {
    let provider = ProcessInfoProvider::new();

    // Call status multiple times
    let status1 = provider.status();
    let status2 = provider.status();
    let status3 = provider.status();

    // PID should remain constant
    assert_eq!(status1.pid, status2.pid);
    assert_eq!(status2.pid, status3.pid);

    // Memory should be positive
    assert!(status1.memory_mb >= 0.0);
    assert!(status2.memory_mb >= 0.0);
    assert!(status3.memory_mb >= 0.0);
}

#[test]
fn test_process_info_provider_status_memory_in_mb() {
    let provider = ProcessInfoProvider::new();
    let status = provider.status();

    // Memory should be reasonable (less than 10GB for a test process)
    assert!(status.memory_mb < 10_000.0);

    // Memory should be at least a few MB for any Rust process
    assert!(status.memory_mb > 0.1);
}
