use std::collections::HashMap;
use std::process::Command;
use std::time::{Duration, SystemTime};
use serde_json::Value;
use tracing::{info, warn, error};

use super::*;

/// Metrics collection service for test execution monitoring
#[derive(Debug)]
pub struct MetricsCollectionService {
    collector: TestMonitoringCollector,
    coverage_parser: CoverageParser,
    performance_tracker: PerformanceTracker,
    resource_monitor: ResourceMonitor,
}

impl MetricsCollectionService {
    /// Create new metrics collection service
    pub fn new(storage_path: std::path::PathBuf) -> Self {
        Self {
            collector: TestMonitoringCollector::new(storage_path),
            coverage_parser: CoverageParser::new(),
            performance_tracker: PerformanceTracker::new(),
            resource_monitor: ResourceMonitor::new(),
        }
    }

    /// Start collecting metrics for a test run
    pub async fn start_collection(&mut self, run_id: String, environment: String, mode: TestExecutionMode) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting metrics collection for run: {}", run_id);

        self.collector.start_run(run_id.clone(), environment, mode);
        self.resource_monitor.start_monitoring().await?;

        Ok(())
    }

    /// Stop collecting metrics and finalize the run
    pub async fn stop_collection(&mut self) -> Result<TestExecutionMetrics, Box<dyn std::error::Error + Send + Sync>> {
        info!("Stopping metrics collection");

        self.resource_monitor.stop_monitoring().await?;

        // Update current metrics with collected data
        if let Some(current) = self.collector.current_metrics() {
            let mut final_metrics = current.clone();

            // Add coverage metrics
            final_metrics.coverage_metrics = self.collect_coverage_metrics().await?;

            // Add performance metrics
            final_metrics.performance_metrics = self.performance_tracker.get_metrics();

            // Add resource utilization
            final_metrics.resource_utilization = self.resource_monitor.get_utilization();

            // Complete the run
            self.collector.complete_run()?;

            Ok(final_metrics)
        } else {
            Err("No active metrics collection session".into())
        }
    }

    /// Record phase completion
    pub fn record_phase_completion(&mut self, phase_name: String, metrics: PhaseMetrics) {
        self.collector.add_phase(phase_name, metrics);
    }

    /// Record test performance data
    pub fn record_test_performance(&mut self, performance: TestPerformanceData) {
        self.performance_tracker.add_test_data(performance);
    }

    /// Record test failure
    pub fn record_test_failure(&mut self, failure: TestFailure) {
        self.performance_tracker.add_failure(failure);
    }

    /// Get current metrics snapshot
    pub fn get_current_snapshot(&self) -> Option<TestExecutionMetrics> {
        self.collector.current_metrics().cloned()
    }

    /// Get historical metrics
    pub async fn load_historical_data(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.collector.load_historical_metrics()?;
        Ok(())
    }

    /// Collect coverage metrics from various sources
    async fn collect_coverage_metrics(&self) -> Result<CoverageMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let mut coverage_metrics = CoverageMetrics::default();

        // Parse Rust coverage
        if let Ok(rust_coverage) = self.coverage_parser.parse_rust_coverage().await {
            coverage_metrics.rust_coverage = Some(rust_coverage);
        }

        // Parse TypeScript coverage
        if let Ok(ts_coverage) = self.coverage_parser.parse_typescript_coverage().await {
            coverage_metrics.typescript_coverage = Some(ts_coverage);
        }

        // Parse integration test coverage
        if let Ok(integration_coverage) = self.coverage_parser.parse_integration_coverage().await {
            coverage_metrics.integration_coverage = Some(integration_coverage);
        }

        // Parse E2E coverage
        if let Ok(e2e_coverage) = self.coverage_parser.parse_e2e_coverage().await {
            coverage_metrics.e2e_coverage = Some(e2e_coverage);
        }

        // Calculate overall coverage
        coverage_metrics.overall_coverage = self.calculate_overall_coverage(&coverage_metrics);

        Ok(coverage_metrics)
    }

    /// Calculate overall coverage percentage
    fn calculate_overall_coverage(&self, metrics: &CoverageMetrics) -> f64 {
        let mut total_lines = 0;
        let mut covered_lines = 0;

        if let Some(rust) = &metrics.rust_coverage {
            total_lines += rust.lines_total;
            covered_lines += rust.lines_covered;
        }

        if let Some(ts) = &metrics.typescript_coverage {
            total_lines += ts.lines_total;
            covered_lines += ts.lines_covered;
        }

        if total_lines > 0 {
            (covered_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Get performance regression alerts
    pub fn get_regression_alerts(&self) -> Vec<PerformanceRegression> {
        self.performance_tracker.get_regression_alerts()
    }

    /// Get flaky test detection results
    pub fn get_flaky_tests(&self) -> Vec<FlakyTestData> {
        // Analyze historical data to identify flaky tests
        let historical = self.collector.historical_metrics();
        let mut test_results: HashMap<String, Vec<bool>> = HashMap::new();

        // Collect test results across multiple runs
        for metrics in historical {
            for phase in metrics.phase_metrics.values() {
                for failure in &phase.failures {
                    test_results.entry(failure.test_name.clone())
                        .or_default()
                        .push(false);
                }

                for perf_data in &phase.performance_data {
                    let results = test_results.entry(perf_data.test_name.clone())
                        .or_default();

                    // If we haven't recorded a failure for this test, assume success
                    if results.is_empty() || results.last() != Some(&false) {
                        results.push(true);
                    }
                }
            }
        }

        // Identify flaky tests (inconsistent results)
        test_results.into_iter()
            .filter_map(|(test_name, results)| {
                if results.len() < 3 { // Need at least 3 runs to detect flakiness
                    return None;
                }

                let failures = results.iter().filter(|&&r| !r).count();
                let failure_rate = failures as f64 / results.len() as f64;

                // Consider a test flaky if it fails between 10% and 90% of the time
                if failure_rate > 0.1 && failure_rate < 0.9 {
                    let stability_score = 1.0 - (failure_rate - 0.5).abs() * 2.0;

                    Some(FlakyTestData {
                        test_name: test_name.clone(),
                        test_type: "unknown".to_string(), // Could be enhanced with actual test type
                        failure_rate: failure_rate * 100.0,
                        total_runs: results.len(),
                        failures,
                        recent_failures: Vec::new(), // Could be populated with actual failure data
                        stability_score,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Coverage parser for different coverage report formats
#[derive(Debug)]
pub struct CoverageParser {
    project_root: std::path::PathBuf,
}

impl CoverageParser {
    pub fn new() -> Self {
        Self {
            project_root: std::env::current_dir().unwrap_or_default(),
        }
    }

    /// Parse Rust coverage from grcov output
    pub async fn parse_rust_coverage(&self) -> Result<CoverageData, Box<dyn std::error::Error + Send + Sync>> {
        let coverage_path = self.project_root.join("tests/coverage/rust/coverage.json");

        if !coverage_path.exists() {
            return Err("Rust coverage file not found".into());
        }

        let content = tokio::fs::read_to_string(&coverage_path).await?;
        let coverage_json: Value = serde_json::from_str(&content)?;

        // Parse grcov JSON format
        let mut lines_covered = 0;
        let mut lines_total = 0;
        let mut functions_covered = 0;
        let mut functions_total = 0;

        if let Some(files) = coverage_json.get("data").and_then(|d| d.as_array()) {
            for file_data in files {
                if let Some(lines) = file_data.get("lines").and_then(|l| l.as_array()) {
                    for line in lines {
                        lines_total += 1;
                        if let Some(count) = line.get("count").and_then(|c| c.as_u64()) {
                            if count > 0 {
                                lines_covered += 1;
                            }
                        }
                    }
                }

                if let Some(functions) = file_data.get("functions").and_then(|f| f.as_array()) {
                    for function in functions {
                        functions_total += 1;
                        if let Some(count) = function.get("count").and_then(|c| c.as_u64()) {
                            if count > 0 {
                                functions_covered += 1;
                            }
                        }
                    }
                }
            }
        }

        let percentage = if lines_total > 0 {
            (lines_covered as f64 / lines_total as f64) * 100.0
        } else {
            0.0
        };

        Ok(CoverageData {
            lines_covered,
            lines_total,
            branches_covered: 0, // Not easily available from grcov JSON
            branches_total: 0,
            functions_covered,
            functions_total,
            percentage,
            timestamp: SystemTime::now(),
        })
    }

    /// Parse TypeScript coverage from c8/nyc output
    pub async fn parse_typescript_coverage(&self) -> Result<CoverageData, Box<dyn std::error::Error + Send + Sync>> {
        let coverage_path = self.project_root.join("tests/coverage/typescript/coverage-final.json");

        if !coverage_path.exists() {
            return Err("TypeScript coverage file not found".into());
        }

        let content = tokio::fs::read_to_string(&coverage_path).await?;
        let coverage_json: Value = serde_json::from_str(&content)?;

        let mut lines_covered = 0;
        let mut lines_total = 0;
        let mut functions_covered = 0;
        let mut functions_total = 0;
        let mut branches_covered = 0;
        let mut branches_total = 0;

        // Parse nyc/c8 JSON format
        for (_, file_data) in coverage_json.as_object().unwrap_or(&serde_json::Map::new()) {
            if let Some(s) = file_data.get("s").and_then(|s| s.as_object()) {
                for (_, count) in s {
                    lines_total += 1;
                    if let Some(count_val) = count.as_u64() {
                        if count_val > 0 {
                            lines_covered += 1;
                        }
                    }
                }
            }

            if let Some(f) = file_data.get("f").and_then(|f| f.as_object()) {
                for (_, count) in f {
                    functions_total += 1;
                    if let Some(count_val) = count.as_u64() {
                        if count_val > 0 {
                            functions_covered += 1;
                        }
                    }
                }
            }

            if let Some(b) = file_data.get("b").and_then(|b| b.as_object()) {
                for (_, branch_data) in b {
                    if let Some(branch_array) = branch_data.as_array() {
                        for branch_count in branch_array {
                            branches_total += 1;
                            if let Some(count_val) = branch_count.as_u64() {
                                if count_val > 0 {
                                    branches_covered += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        let percentage = if lines_total > 0 {
            (lines_covered as f64 / lines_total as f64) * 100.0
        } else {
            0.0
        };

        Ok(CoverageData {
            lines_covered,
            lines_total,
            branches_covered,
            branches_total,
            functions_covered,
            functions_total,
            percentage,
            timestamp: SystemTime::now(),
        })
    }

    /// Parse integration test coverage
    pub async fn parse_integration_coverage(&self) -> Result<CoverageData, Box<dyn std::error::Error + Send + Sync>> {
        // For now, assume integration coverage is included in Rust coverage
        // This could be enhanced to parse separate integration coverage data
        self.parse_rust_coverage().await
    }

    /// Parse E2E test coverage
    pub async fn parse_e2e_coverage(&self) -> Result<CoverageData, Box<dyn std::error::Error + Send + Sync>> {
        let coverage_path = self.project_root.join("tests/coverage/e2e/coverage-final.json");

        if !coverage_path.exists() {
            return Err("E2E coverage file not found".into());
        }

        // Similar to TypeScript coverage parsing
        self.parse_typescript_coverage().await
    }
}

/// Performance tracking and regression detection
#[derive(Debug)]
pub struct PerformanceTracker {
    test_data: Vec<TestPerformanceData>,
    failures: Vec<TestFailure>,
    regression_threshold: f64, // Percentage threshold for regression alerts
    baseline_data: HashMap<String, Duration>, // Baseline performance data
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            test_data: Vec::new(),
            failures: Vec::new(),
            regression_threshold: 20.0, // 20% regression threshold
            baseline_data: HashMap::new(),
        }
    }

    /// Add test performance data
    pub fn add_test_data(&mut self, data: TestPerformanceData) {
        // Check for performance regression
        if let Some(&baseline_duration) = self.baseline_data.get(&data.test_name) {
            let regression_ratio = data.duration.as_millis() as f64 / baseline_duration.as_millis() as f64;
            let regression_percentage = (regression_ratio - 1.0) * 100.0;

            if regression_percentage > self.regression_threshold {
                let regression = PerformanceRegression {
                    test_name: data.test_name.clone(),
                    baseline_duration,
                    current_duration: data.duration,
                    regression_percentage,
                    severity: RegressionSeverity::from_percentage(regression_percentage),
                    detected_at: SystemTime::now(),
                };

                warn!("Performance regression detected for {}: {:.1}% slower",
                      data.test_name, regression_percentage);
            }
        } else {
            // Set as baseline if no previous data
            self.baseline_data.insert(data.test_name.clone(), data.duration);
        }

        self.test_data.push(data);
    }

    /// Add test failure
    pub fn add_failure(&mut self, failure: TestFailure) {
        self.failures.push(failure);
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        let total_duration: Duration = self.test_data.iter().map(|d| d.duration).sum();
        let average_duration = if !self.test_data.is_empty() {
            total_duration / self.test_data.len() as u32
        } else {
            Duration::from_secs(0)
        };

        let mut sorted_tests = self.test_data.clone();
        sorted_tests.sort_by(|a, b| b.duration.cmp(&a.duration));

        let slowest_tests = sorted_tests.into_iter().take(10).collect();

        let mut sorted_fast = self.test_data.clone();
        sorted_fast.sort_by(|a, b| a.duration.cmp(&b.duration));
        let fastest_tests = sorted_fast.into_iter().take(10).collect();

        PerformanceMetrics {
            average_test_duration: average_duration,
            slowest_tests,
            fastest_tests,
            database_connection_times: Vec::new(), // Could be populated from actual DB metrics
            api_response_times: Vec::new(), // Could be populated from actual API metrics
            memory_usage_peak: None,
            cpu_usage_peak: None,
            regression_alerts: self.get_regression_alerts(),
        }
    }

    /// Get performance regression alerts
    pub fn get_regression_alerts(&self) -> Vec<PerformanceRegression> {
        let mut regressions = Vec::new();

        for data in &self.test_data {
            if let Some(&baseline_duration) = self.baseline_data.get(&data.test_name) {
                let regression_ratio = data.duration.as_millis() as f64 / baseline_duration.as_millis() as f64;
                let regression_percentage = (regression_ratio - 1.0) * 100.0;

                if regression_percentage > self.regression_threshold {
                    regressions.push(PerformanceRegression {
                        test_name: data.test_name.clone(),
                        baseline_duration,
                        current_duration: data.duration,
                        regression_percentage,
                        severity: RegressionSeverity::from_percentage(regression_percentage),
                        detected_at: data.timestamp,
                    });
                }
            }
        }

        regressions
    }

    /// Load baseline performance data
    pub fn load_baseline_data(&mut self, baseline: HashMap<String, Duration>) {
        self.baseline_data = baseline;
    }
}

/// System resource monitoring
#[derive(Debug)]
pub struct ResourceMonitor {
    monitoring_active: bool,
    container_metrics: Vec<ContainerMetrics>,
    system_metrics_history: Vec<(SystemTime, MemoryMetrics, CpuMetrics)>,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            monitoring_active: false,
            container_metrics: Vec::new(),
            system_metrics_history: Vec::new(),
        }
    }

    /// Start resource monitoring
    pub async fn start_monitoring(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.monitoring_active = true;
        info!("Started resource monitoring");

        // Start background monitoring task
        tokio::spawn(async move {
            // This would contain actual monitoring implementation
            // For now, just a placeholder
        });

        Ok(())
    }

    /// Stop resource monitoring
    pub async fn stop_monitoring(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.monitoring_active = false;
        info!("Stopped resource monitoring");
        Ok(())
    }

    /// Get current resource utilization
    pub fn get_utilization(&self) -> ResourceUtilization {
        ResourceUtilization {
            docker_containers: self.container_metrics.clone(),
            system_memory: self.get_current_memory_metrics(),
            system_cpu: self.get_current_cpu_metrics(),
            disk_io: None,
            network_io: None,
        }
    }

    /// Get current memory metrics
    fn get_current_memory_metrics(&self) -> Option<MemoryMetrics> {
        // This would implement actual memory metrics collection
        // For now, return None as placeholder
        None
    }

    /// Get current CPU metrics
    fn get_current_cpu_metrics(&self) -> Option<CpuMetrics> {
        // This would implement actual CPU metrics collection
        // For now, return None as placeholder
        None
    }

    /// Collect Docker container metrics
    pub async fn collect_container_metrics(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Use docker stats command to collect container metrics
        let output = Command::new("docker")
            .args(&["stats", "--no-stream", "--format", "json"])
            .output()?;

        if output.status.success() {
            let stdout = String::from_utf8(output.stdout)?;
            for line in stdout.lines() {
                if let Ok(stats) = serde_json::from_str::<Value>(line) {
                    if let Some(container_name) = stats.get("Name").and_then(|n| n.as_str()) {
                        let cpu_usage = stats.get("CPUPerc")
                            .and_then(|c| c.as_str())
                            .and_then(|s| s.trim_end_matches('%').parse::<f64>().ok())
                            .unwrap_or(0.0);

                        let memory_usage = stats.get("MemUsage")
                            .and_then(|m| m.as_str())
                            .and_then(|s| s.split('/').next())
                            .and_then(|s| self.parse_memory_value(s))
                            .unwrap_or(0);

                        let memory_limit = stats.get("MemUsage")
                            .and_then(|m| m.as_str())
                            .and_then(|s| s.split('/').nth(1))
                            .and_then(|s| self.parse_memory_value(s))
                            .unwrap_or(0);

                        let container_metrics = ContainerMetrics {
                            container_name: container_name.to_string(),
                            cpu_usage_percent: cpu_usage,
                            memory_usage_bytes: memory_usage,
                            memory_limit_bytes: memory_limit,
                            network_io_bytes: 0, // Could be parsed from NetIO field
                            block_io_bytes: 0,   // Could be parsed from BlockIO field
                            timestamp: SystemTime::now(),
                        };

                        self.container_metrics.push(container_metrics);
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse memory value from Docker stats (e.g., "123.4MiB" -> bytes)
    fn parse_memory_value(&self, value: &str) -> Option<u64> {
        let value = value.trim();

        if value.ends_with("GiB") {
            value.trim_end_matches("GiB").parse::<f64>().ok()
                .map(|v| (v * 1024.0 * 1024.0 * 1024.0) as u64)
        } else if value.ends_with("MiB") {
            value.trim_end_matches("MiB").parse::<f64>().ok()
                .map(|v| (v * 1024.0 * 1024.0) as u64)
        } else if value.ends_with("KiB") {
            value.trim_end_matches("KiB").parse::<f64>().ok()
                .map(|v| (v * 1024.0) as u64)
        } else if value.ends_with("B") {
            value.trim_end_matches("B").parse::<u64>().ok()
        } else {
            value.parse::<u64>().ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_metrics_collection_service() {
        let temp_dir = TempDir::new().unwrap();
        let mut service = MetricsCollectionService::new(temp_dir.path().to_path_buf());

        service.start_collection(
            "test-run-1".to_string(),
            "test".to_string(),
            TestExecutionMode::Quick
        ).await.unwrap();

        let snapshot = service.get_current_snapshot();
        assert!(snapshot.is_some());

        let phase_metrics = PhaseMetrics::new("test-phase".to_string());
        service.record_phase_completion("test-phase".to_string(), phase_metrics);

        let final_metrics = service.stop_collection().await.unwrap();
        assert_eq!(final_metrics.run_id, "test-run-1");
    }

    #[test]
    fn test_performance_tracker() {
        let mut tracker = PerformanceTracker::new();

        let test_data = TestPerformanceData {
            test_name: "test_function".to_string(),
            test_type: "unit".to_string(),
            duration: Duration::from_millis(100),
            memory_used: Some(1024),
            cpu_time: Some(Duration::from_millis(50)),
            database_queries: None,
            api_calls: None,
            timestamp: SystemTime::now(),
        };

        tracker.add_test_data(test_data);

        let metrics = tracker.get_metrics();
        assert_eq!(metrics.average_test_duration, Duration::from_millis(100));
        assert_eq!(metrics.slowest_tests.len(), 1);
    }

    #[test]
    fn test_regression_detection() {
        let mut tracker = PerformanceTracker::new();

        // Add baseline performance
        let baseline = TestPerformanceData {
            test_name: "slow_test".to_string(),
            test_type: "integration".to_string(),
            duration: Duration::from_millis(100),
            memory_used: None,
            cpu_time: None,
            database_queries: None,
            api_calls: None,
            timestamp: SystemTime::now(),
        };

        tracker.add_test_data(baseline);

        // Add regressed performance (50% slower)
        let regressed = TestPerformanceData {
            test_name: "slow_test".to_string(),
            test_type: "integration".to_string(),
            duration: Duration::from_millis(150),
            memory_used: None,
            cpu_time: None,
            database_queries: None,
            api_calls: None,
            timestamp: SystemTime::now(),
        };

        tracker.add_test_data(regressed);

        let alerts = tracker.get_regression_alerts();
        assert_eq!(alerts.len(), 1);
        assert!(alerts[0].regression_percentage > 20.0);
    }
}