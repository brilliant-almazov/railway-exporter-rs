//! Process information utility.
//!
//! Provides unified access to current process metrics (CPU, memory, PID).
//! Used by both /status handler and Prometheus metrics.

use crate::types::ProcessStatus;
use sysinfo::System;

/// Provides process information (CPU, memory, PID).
///
/// # Example
///
/// ```rust,no_run
/// use railway_exporter::utils::ProcessInfoProvider;
///
/// let provider = ProcessInfoProvider::new();
/// let status = provider.status();
/// println!("Memory: {} MB, CPU: {}%", status.memory_mb, status.cpu_percent);
/// ```
pub struct ProcessInfoProvider {
    pub pid: sysinfo::Pid,
}

impl ProcessInfoProvider {
    /// Creates a new ProcessInfoProvider.
    pub fn new() -> Self {
        Self {
            pid: sysinfo::get_current_pid().expect("Failed to get current PID"),
        }
    }

    /// Gets current process status (CPU, memory).
    pub fn status(&self) -> ProcessStatus {
        let mut sys = System::new();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[self.pid]), true);

        sys.process(self.pid)
            .map(|p| ProcessStatus {
                pid: self.pid.as_u32(),
                memory_mb: p.memory() as f64 / 1024.0 / 1024.0,
                cpu_percent: p.cpu_usage(),
            })
            .unwrap_or(ProcessStatus {
                pid: self.pid.as_u32(),
                memory_mb: 0.0,
                cpu_percent: 0.0,
            })
    }

    /// Gets the process ID.
    pub fn pid(&self) -> u32 {
        self.pid.as_u32()
    }
}

impl Default for ProcessInfoProvider {
    fn default() -> Self {
        Self::new()
    }
}
