use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Performance metrics collector
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    metrics: Arc<Mutex<PerformanceMetrics>>,
}

/// Performance metrics storage
#[derive(Debug, Default)]
struct PerformanceMetrics {
    command_metrics: HashMap<String, CommandMetrics>,
    database_metrics: DatabaseMetrics,
    system_metrics: SystemMetrics,
}

/// Metrics for individual commands
#[derive(Debug, Default)]
struct CommandMetrics {
    total_calls: u64,
    total_duration: Duration,
    min_duration: Option<Duration>,
    max_duration: Option<Duration>,
    error_count: u64,
    slow_calls: u64, // Calls exceeding threshold
}

/// Database performance metrics
#[derive(Debug, Default)]
struct DatabaseMetrics {
    total_queries: u64,
    total_query_time: Duration,
    slow_queries: u64,
    failed_queries: u64,
    transaction_count: u64,
    transaction_rollbacks: u64,
}

/// System-level metrics
#[derive(Debug, Default)]
struct SystemMetrics {
    startup_time: Option<Instant>,
    total_requests: u64,
    active_requests: u64,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
        }
    }

    /// Record command execution metrics
    pub fn record_command(&self, command_name: &str, duration: Duration, success: bool) {
        if let Ok(mut metrics) = self.metrics.lock() {
            let command_metrics = metrics
                .command_metrics
                .entry(command_name.to_string())
                .or_default();

            command_metrics.total_calls += 1;
            command_metrics.total_duration += duration;

            // Update min/max durations
            match command_metrics.min_duration {
                Some(min) if duration < min => command_metrics.min_duration = Some(duration),
                None => command_metrics.min_duration = Some(duration),
                _ => {}
            }

            match command_metrics.max_duration {
                Some(max) if duration > max => command_metrics.max_duration = Some(duration),
                None => command_metrics.max_duration = Some(duration),
                _ => {}
            }

            if !success {
                command_metrics.error_count += 1;
            }

            // Check for slow commands (> 1 second)
            if duration > Duration::from_secs(1) {
                command_metrics.slow_calls += 1;
                warn!(
                    command = command_name,
                    duration_ms = duration.as_millis(),
                    "Slow command execution detected"
                );
            }
        }
    }

    /// Record database query metrics
    pub fn record_database_query(&self, duration: Duration, success: bool, is_slow: bool) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.database_metrics.total_queries += 1;
            metrics.database_metrics.total_query_time += duration;

            if !success {
                metrics.database_metrics.failed_queries += 1;
            }

            if is_slow {
                metrics.database_metrics.slow_queries += 1;
            }
        }
    }

    /// Record database transaction metrics
    pub fn record_transaction(&self, success: bool) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.database_metrics.transaction_count += 1;
            if !success {
                metrics.database_metrics.transaction_rollbacks += 1;
            }
        }
    }

    /// Record system-level metrics
    pub fn record_request_start(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.system_metrics.total_requests += 1;
            metrics.system_metrics.active_requests += 1;
        }
    }

    /// Record request completion
    pub fn record_request_end(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            if metrics.system_metrics.active_requests > 0 {
                metrics.system_metrics.active_requests -= 1;
            }
        }
    }

    /// Set application startup time
    pub fn set_startup_time(&self, startup_time: Instant) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.system_metrics.startup_time = Some(startup_time);
        }
    }

    /// Get performance summary
    pub fn get_summary(&self) -> PerformanceSummary {
        if let Ok(metrics) = self.metrics.lock() {
            let mut command_summaries = Vec::new();

            for (name, cmd_metrics) in &metrics.command_metrics {
                let avg_duration = if cmd_metrics.total_calls > 0 {
                    Duration::from_nanos(
                        cmd_metrics.total_duration.as_nanos() as u64 / cmd_metrics.total_calls,
                    )
                } else {
                    Duration::ZERO
                };

                command_summaries.push(CommandSummary {
                    name: name.clone(),
                    total_calls: cmd_metrics.total_calls,
                    average_duration: avg_duration,
                    min_duration: cmd_metrics.min_duration.unwrap_or(Duration::ZERO),
                    max_duration: cmd_metrics.max_duration.unwrap_or(Duration::ZERO),
                    error_rate: if cmd_metrics.total_calls > 0 {
                        (cmd_metrics.error_count as f64 / cmd_metrics.total_calls as f64) * 100.0
                    } else {
                        0.0
                    },
                    slow_call_rate: if cmd_metrics.total_calls > 0 {
                        (cmd_metrics.slow_calls as f64 / cmd_metrics.total_calls as f64) * 100.0
                    } else {
                        0.0
                    },
                });
            }

            let db_avg_query_time = if metrics.database_metrics.total_queries > 0 {
                Duration::from_nanos(
                    metrics.database_metrics.total_query_time.as_nanos() as u64
                        / metrics.database_metrics.total_queries,
                )
            } else {
                Duration::ZERO
            };

            PerformanceSummary {
                commands: command_summaries,
                database: DatabaseSummary {
                    total_queries: metrics.database_metrics.total_queries,
                    average_query_time: db_avg_query_time,
                    slow_query_rate: if metrics.database_metrics.total_queries > 0 {
                        (metrics.database_metrics.slow_queries as f64
                            / metrics.database_metrics.total_queries as f64)
                            * 100.0
                    } else {
                        0.0
                    },
                    error_rate: if metrics.database_metrics.total_queries > 0 {
                        (metrics.database_metrics.failed_queries as f64
                            / metrics.database_metrics.total_queries as f64)
                            * 100.0
                    } else {
                        0.0
                    },
                    transaction_count: metrics.database_metrics.transaction_count,
                    rollback_rate: if metrics.database_metrics.transaction_count > 0 {
                        (metrics.database_metrics.transaction_rollbacks as f64
                            / metrics.database_metrics.transaction_count as f64)
                            * 100.0
                    } else {
                        0.0
                    },
                },
                system: SystemSummary {
                    uptime: metrics
                        .system_metrics
                        .startup_time
                        .map(|start| start.elapsed()),
                    total_requests: metrics.system_metrics.total_requests,
                    active_requests: metrics.system_metrics.active_requests,
                },
            }
        } else {
            PerformanceSummary::default()
        }
    }

    /// Log performance summary
    pub fn log_summary(&self) {
        let summary = self.get_summary();

        info!(
            total_requests = summary.system.total_requests,
            active_requests = summary.system.active_requests,
            uptime_seconds = summary.system.uptime.map(|u| u.as_secs()).unwrap_or(0),
            "System performance summary"
        );

        info!(
            total_queries = summary.database.total_queries,
            avg_query_time_ms = summary.database.average_query_time.as_millis(),
            slow_query_rate = summary.database.slow_query_rate,
            error_rate = summary.database.error_rate,
            "Database performance summary"
        );

        for cmd in &summary.commands {
            info!(
                command = cmd.name,
                total_calls = cmd.total_calls,
                avg_duration_ms = cmd.average_duration.as_millis(),
                error_rate = cmd.error_rate,
                slow_call_rate = cmd.slow_call_rate,
                "Command performance summary"
            );
        }
    }
}

/// Performance summary for reporting
#[derive(Debug, Default)]
pub struct PerformanceSummary {
    pub commands: Vec<CommandSummary>,
    pub database: DatabaseSummary,
    pub system: SystemSummary,
}

/// Command performance summary
#[derive(Debug)]
pub struct CommandSummary {
    pub name: String,
    pub total_calls: u64,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub error_rate: f64,
    pub slow_call_rate: f64,
}

/// Database performance summary
#[derive(Debug, Default)]
pub struct DatabaseSummary {
    pub total_queries: u64,
    pub average_query_time: Duration,
    pub slow_query_rate: f64,
    pub error_rate: f64,
    pub transaction_count: u64,
    pub rollback_rate: f64,
}

/// System performance summary
#[derive(Debug, Default)]
pub struct SystemSummary {
    pub uptime: Option<Duration>,
    pub total_requests: u64,
    pub active_requests: u64,
}

/// Global performance monitor instance
static PERFORMANCE_MONITOR: std::sync::OnceLock<PerformanceMonitor> = std::sync::OnceLock::new();

/// Get the global performance monitor instance
pub fn get_performance_monitor() -> &'static PerformanceMonitor {
    PERFORMANCE_MONITOR.get_or_init(PerformanceMonitor::new)
}

/// Initialize performance monitoring
pub fn init_performance_monitoring() {
    let monitor = get_performance_monitor();
    monitor.set_startup_time(Instant::now());
    info!("Performance monitoring initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    #[test]
    fn test_command_metrics() {
        let monitor = PerformanceMonitor::new();

        // Record some test metrics
        monitor.record_command("test_command", Duration::from_millis(100), true);
        monitor.record_command("test_command", Duration::from_millis(200), true);
        monitor.record_command("test_command", Duration::from_millis(50), false);

        let summary = monitor.get_summary();
        assert_eq!(summary.commands.len(), 1);

        let cmd_summary = &summary.commands[0];
        assert_eq!(cmd_summary.name, "test_command");
        assert_eq!(cmd_summary.total_calls, 3);
        assert_eq!(cmd_summary.min_duration, Duration::from_millis(50));
        assert_eq!(cmd_summary.max_duration, Duration::from_millis(200));
    }

    #[test]
    fn test_database_metrics() {
        let monitor = PerformanceMonitor::new();

        monitor.record_database_query(Duration::from_millis(10), true, false);
        monitor.record_database_query(Duration::from_millis(2000), true, true); // slow query
        monitor.record_database_query(Duration::from_millis(5), false, false); // failed query

        let summary = monitor.get_summary();
        assert_eq!(summary.database.total_queries, 3);
        assert!((summary.database.slow_query_rate - 33.333333333333336).abs() < 0.001); // 1/3
        assert!((summary.database.error_rate - 33.333333333333336).abs() < 0.001);
        // 1/3
    }
}
