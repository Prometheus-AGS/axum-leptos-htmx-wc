use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

pub mod metrics;
pub mod dashboard;
pub mod alerts;
pub mod realtime;
pub mod comprehensive;

/// Test execution monitoring and metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecutionMetrics {
    pub run_id: String,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub total_duration: Option<Duration>,
    pub environment: String,
    pub mode: TestExecutionMode,
    pub phase_metrics: HashMap<String, PhaseMetrics>,
    pub coverage_metrics: CoverageMetrics,
    pub performance_metrics: PerformanceMetrics,
    pub reliability_metrics: ReliabilityMetrics,
    pub resource_utilization: ResourceUtilization,
}

/// Test execution modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestExecutionMode {
    Quick,
    Full,
    CI,
    Certification,
}

/// Metrics for individual test phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseMetrics {
    pub phase_name: String,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub duration: Option<Duration>,
    pub tests_total: usize,
    pub tests_passed: usize,
    pub tests_failed: usize,
    pub tests_skipped: usize,
    pub failures: Vec<TestFailure>,
    pub performance_data: Vec<TestPerformanceData>,
}

/// Coverage metrics across different test types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageMetrics {
    pub rust_coverage: Option<CoverageData>,
    pub typescript_coverage: Option<CoverageData>,
    pub integration_coverage: Option<CoverageData>,
    pub e2e_coverage: Option<CoverageData>,
    pub overall_coverage: f64,
    pub coverage_trend: Vec<CoverageDataPoint>,
}

/// Individual coverage data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageData {
    pub lines_covered: usize,
    pub lines_total: usize,
    pub branches_covered: usize,
    pub branches_total: usize,
    pub functions_covered: usize,
    pub functions_total: usize,
    pub percentage: f64,
    pub timestamp: SystemTime,
}

/// Coverage trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageDataPoint {
    pub timestamp: DateTime<Utc>,
    pub run_id: String,
    pub coverage_percentage: f64,
    pub test_count: usize,
}

/// Performance metrics tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub average_test_duration: Duration,
    pub slowest_tests: Vec<TestPerformanceData>,
    pub fastest_tests: Vec<TestPerformanceData>,
    pub database_connection_times: Vec<Duration>,
    pub api_response_times: Vec<Duration>,
    pub memory_usage_peak: Option<u64>,
    pub cpu_usage_peak: Option<f64>,
    pub regression_alerts: Vec<PerformanceRegression>,
}

/// Individual test performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPerformanceData {
    pub test_name: String,
    pub test_type: String,
    pub duration: Duration,
    pub memory_used: Option<u64>,
    pub cpu_time: Option<Duration>,
    pub database_queries: Option<usize>,
    pub api_calls: Option<usize>,
    pub timestamp: SystemTime,
}

/// Performance regression detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRegression {
    pub test_name: String,
    pub baseline_duration: Duration,
    pub current_duration: Duration,
    pub regression_percentage: f64,
    pub severity: RegressionSeverity,
    pub detected_at: SystemTime,
}

/// Severity levels for performance regressions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    Low,     // 10-20% regression
    Medium,  // 20-50% regression
    High,    // 50-100% regression
    Critical, // >100% regression
}

/// Test reliability and stability metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityMetrics {
    pub total_runs: usize,
    pub successful_runs: usize,
    pub failed_runs: usize,
    pub success_rate: f64,
    pub flaky_tests: Vec<FlakyTestData>,
    pub reliability_trend: Vec<ReliabilityDataPoint>,
    pub mean_time_to_failure: Option<Duration>,
    pub mean_time_to_recovery: Option<Duration>,
}

/// Data for tests identified as flaky
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakyTestData {
    pub test_name: String,
    pub test_type: String,
    pub failure_rate: f64,
    pub total_runs: usize,
    pub failures: usize,
    pub recent_failures: Vec<TestFailure>,
    pub stability_score: f64,
}

/// Reliability trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityDataPoint {
    pub timestamp: DateTime<Utc>,
    pub run_id: String,
    pub success_rate: f64,
    pub total_tests: usize,
    pub failed_tests: usize,
}

/// Test failure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFailure {
    pub test_name: String,
    pub test_type: String,
    pub failure_type: FailureType,
    pub error_message: String,
    pub stack_trace: Option<String>,
    pub timestamp: SystemTime,
    pub environment_context: HashMap<String, String>,
}

/// Types of test failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureType {
    Assertion,
    Timeout,
    DatabaseConnection,
    NetworkError,
    EnvironmentSetup,
    ResourceExhaustion,
    Unknown,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub docker_containers: Vec<ContainerMetrics>,
    pub system_memory: Option<MemoryMetrics>,
    pub system_cpu: Option<CpuMetrics>,
    pub disk_io: Option<DiskIOMetrics>,
    pub network_io: Option<NetworkIOMetrics>,
}

/// Docker container resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerMetrics {
    pub container_name: String,
    pub cpu_usage_percent: f64,
    pub memory_usage_bytes: u64,
    pub memory_limit_bytes: u64,
    pub network_io_bytes: u64,
    pub block_io_bytes: u64,
    pub timestamp: SystemTime,
}

/// System memory metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
    pub swap_used_bytes: u64,
    pub swap_total_bytes: u64,
}

/// CPU utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage_percent: f64,
    pub load_average_1m: f64,
    pub load_average_5m: f64,
    pub load_average_15m: f64,
    pub core_count: usize,
}

/// Disk I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskIOMetrics {
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub read_operations: u64,
    pub write_operations: u64,
    pub io_wait_percent: f64,
}

/// Network I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIOMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub errors: u64,
}

impl TestExecutionMetrics {
    /// Create new test execution metrics
    pub fn new(run_id: String, environment: String, mode: TestExecutionMode) -> Self {
        Self {
            run_id,
            start_time: SystemTime::now(),
            end_time: None,
            total_duration: None,
            environment,
            mode,
            phase_metrics: HashMap::new(),
            coverage_metrics: CoverageMetrics::default(),
            performance_metrics: PerformanceMetrics::default(),
            reliability_metrics: ReliabilityMetrics::default(),
            resource_utilization: ResourceUtilization::default(),
        }
    }

    /// Mark test execution as completed
    pub fn complete(&mut self) {
        self.end_time = Some(SystemTime::now());
        self.total_duration = self.start_time.elapsed().ok();
    }

    /// Add phase metrics
    pub fn add_phase(&mut self, phase_name: String, metrics: PhaseMetrics) {
        self.phase_metrics.insert(phase_name, metrics);
    }

    /// Calculate overall success rate
    pub fn overall_success_rate(&self) -> f64 {
        let total_tests: usize = self.phase_metrics.values().map(|p| p.tests_total).sum();
        let passed_tests: usize = self.phase_metrics.values().map(|p| p.tests_passed).sum();

        if total_tests > 0 {
            (passed_tests as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Get total test count across all phases
    pub fn total_test_count(&self) -> usize {
        self.phase_metrics.values().map(|p| p.tests_total).sum()
    }

    /// Get failed test count across all phases
    pub fn failed_test_count(&self) -> usize {
        self.phase_metrics.values().map(|p| p.tests_failed).sum()
    }

    /// Identify slowest phases
    pub fn slowest_phases(&self, limit: usize) -> Vec<(&String, &PhaseMetrics)> {
        let mut phases: Vec<_> = self.phase_metrics.iter().collect();
        phases.sort_by(|a, b| {
            let duration_a = a.1.duration.unwrap_or(Duration::from_secs(0));
            let duration_b = b.1.duration.unwrap_or(Duration::from_secs(0));
            duration_b.cmp(&duration_a)
        });
        phases.into_iter().take(limit).collect()
    }
}

impl Default for CoverageMetrics {
    fn default() -> Self {
        Self {
            rust_coverage: None,
            typescript_coverage: None,
            integration_coverage: None,
            e2e_coverage: None,
            overall_coverage: 0.0,
            coverage_trend: Vec::new(),
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            average_test_duration: Duration::from_secs(0),
            slowest_tests: Vec::new(),
            fastest_tests: Vec::new(),
            database_connection_times: Vec::new(),
            api_response_times: Vec::new(),
            memory_usage_peak: None,
            cpu_usage_peak: None,
            regression_alerts: Vec::new(),
        }
    }
}

impl Default for ReliabilityMetrics {
    fn default() -> Self {
        Self {
            total_runs: 0,
            successful_runs: 0,
            failed_runs: 0,
            success_rate: 0.0,
            flaky_tests: Vec::new(),
            reliability_trend: Vec::new(),
            mean_time_to_failure: None,
            mean_time_to_recovery: None,
        }
    }
}

impl Default for ResourceUtilization {
    fn default() -> Self {
        Self {
            docker_containers: Vec::new(),
            system_memory: None,
            system_cpu: None,
            disk_io: None,
            network_io: None,
        }
    }
}

impl PhaseMetrics {
    /// Create new phase metrics
    pub fn new(phase_name: String) -> Self {
        Self {
            phase_name,
            start_time: SystemTime::now(),
            end_time: None,
            duration: None,
            tests_total: 0,
            tests_passed: 0,
            tests_failed: 0,
            tests_skipped: 0,
            failures: Vec::new(),
            performance_data: Vec::new(),
        }
    }

    /// Mark phase as completed
    pub fn complete(&mut self) {
        self.end_time = Some(SystemTime::now());
        self.duration = self.start_time.elapsed().ok();
    }

    /// Calculate success rate for this phase
    pub fn success_rate(&self) -> f64 {
        if self.tests_total > 0 {
            (self.tests_passed as f64 / self.tests_total as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Add test failure
    pub fn add_failure(&mut self, failure: TestFailure) {
        self.failures.push(failure);
        self.tests_failed += 1;
    }

    /// Add performance data point
    pub fn add_performance_data(&mut self, performance: TestPerformanceData) {
        self.performance_data.push(performance);
    }
}

impl RegressionSeverity {
    /// Determine severity based on regression percentage
    pub fn from_percentage(percentage: f64) -> Self {
        if percentage >= 100.0 {
            Self::Critical
        } else if percentage >= 50.0 {
            Self::High
        } else if percentage >= 20.0 {
            Self::Medium
        } else {
            Self::Low
        }
    }

    /// Get color code for UI display
    pub fn color_code(&self) -> &'static str {
        match self {
            Self::Low => "#FFA500",      // Orange
            Self::Medium => "#FF6B6B",   // Light Red
            Self::High => "#DC3545",     // Red
            Self::Critical => "#721C24", // Dark Red
        }
    }
}

impl FailureType {
    /// Categorize failure type from error message
    pub fn from_error_message(error: &str) -> Self {
        let error_lower = error.to_lowercase();

        if error_lower.contains("timeout") {
            Self::Timeout
        } else if error_lower.contains("connection") || error_lower.contains("database") {
            Self::DatabaseConnection
        } else if error_lower.contains("network") || error_lower.contains("http") {
            Self::NetworkError
        } else if error_lower.contains("docker") || error_lower.contains("environment") {
            Self::EnvironmentSetup
        } else if error_lower.contains("memory") || error_lower.contains("resource") {
            Self::ResourceExhaustion
        } else if error_lower.contains("assert") || error_lower.contains("expected") {
            Self::Assertion
        } else {
            Self::Unknown
        }
    }
}

/// Test monitoring collector - collects metrics during test execution
#[derive(Debug)]
pub struct TestMonitoringCollector {
    current_metrics: Option<TestExecutionMetrics>,
    historical_data: Vec<TestExecutionMetrics>,
    storage_path: std::path::PathBuf,
}

impl TestMonitoringCollector {
    /// Create new monitoring collector
    pub fn new(storage_path: std::path::PathBuf) -> Self {
        Self {
            current_metrics: None,
            historical_data: Vec::new(),
            storage_path,
        }
    }

    /// Start monitoring a test run
    pub fn start_run(&mut self, run_id: String, environment: String, mode: TestExecutionMode) {
        self.current_metrics = Some(TestExecutionMetrics::new(run_id, environment, mode));
    }

    /// Complete current test run
    pub fn complete_run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(mut metrics) = self.current_metrics.take() {
            metrics.complete();
            self.historical_data.push(metrics.clone());
            self.save_metrics(&metrics)?;
        }
        Ok(())
    }

    /// Add phase to current run
    pub fn add_phase(&mut self, phase_name: String, metrics: PhaseMetrics) {
        if let Some(current) = &mut self.current_metrics {
            current.add_phase(phase_name, metrics);
        }
    }

    /// Get current metrics
    pub fn current_metrics(&self) -> Option<&TestExecutionMetrics> {
        self.current_metrics.as_ref()
    }

    /// Save metrics to storage
    fn save_metrics(&self, metrics: &TestExecutionMetrics) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        std::fs::create_dir_all(&self.storage_path)?;

        let filename = format!("test-metrics-{}.json", metrics.run_id);
        let filepath = self.storage_path.join(filename);

        let json = serde_json::to_string_pretty(metrics)?;
        std::fs::write(filepath, json)?;

        Ok(())
    }

    /// Load historical metrics
    pub fn load_historical_metrics(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.storage_path.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.storage_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    if let Ok(metrics) = serde_json::from_str::<TestExecutionMetrics>(&contents) {
                        self.historical_data.push(metrics);
                    }
                }
            }
        }

        // Sort by start time
        self.historical_data.sort_by(|a, b| a.start_time.cmp(&b.start_time));

        Ok(())
    }

    /// Get historical metrics
    pub fn historical_metrics(&self) -> &[TestExecutionMetrics] {
        &self.historical_data
    }

    /// Get recent metrics (last N runs)
    pub fn recent_metrics(&self, count: usize) -> &[TestExecutionMetrics] {
        let start = self.historical_data.len().saturating_sub(count);
        &self.historical_data[start..]
    }

    /// Calculate coverage trends
    pub fn coverage_trend(&self, days: u32) -> Vec<CoverageDataPoint> {
        let cutoff = SystemTime::now() - Duration::from_secs(days as u64 * 24 * 3600);

        self.historical_data
            .iter()
            .filter(|m| m.start_time >= cutoff)
            .map(|m| CoverageDataPoint {
                timestamp: DateTime::from(m.start_time),
                run_id: m.run_id.clone(),
                coverage_percentage: m.coverage_metrics.overall_coverage,
                test_count: m.total_test_count(),
            })
            .collect()
    }

    /// Calculate reliability trends
    pub fn reliability_trend(&self, days: u32) -> Vec<ReliabilityDataPoint> {
        let cutoff = SystemTime::now() - Duration::from_secs(days as u64 * 24 * 3600);

        self.historical_data
            .iter()
            .filter(|m| m.start_time >= cutoff)
            .map(|m| ReliabilityDataPoint {
                timestamp: DateTime::from(m.start_time),
                run_id: m.run_id.clone(),
                success_rate: m.overall_success_rate(),
                total_tests: m.total_test_count(),
                failed_tests: m.failed_test_count(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_metrics_creation() {
        let metrics = TestExecutionMetrics::new(
            "test-run-1".to_string(),
            "test".to_string(),
            TestExecutionMode::Full
        );

        assert_eq!(metrics.run_id, "test-run-1");
        assert_eq!(metrics.environment, "test");
        assert!(matches!(metrics.mode, TestExecutionMode::Full));
        assert!(metrics.end_time.is_none());
    }

    #[test]
    fn test_phase_metrics() {
        let mut phase = PhaseMetrics::new("unit-tests".to_string());

        phase.tests_total = 100;
        phase.tests_passed = 85;
        phase.tests_failed = 15;

        assert_eq!(phase.success_rate(), 85.0);
    }

    #[test]
    fn test_regression_severity() {
        assert!(matches!(RegressionSeverity::from_percentage(150.0), RegressionSeverity::Critical));
        assert!(matches!(RegressionSeverity::from_percentage(75.0), RegressionSeverity::High));
        assert!(matches!(RegressionSeverity::from_percentage(35.0), RegressionSeverity::Medium));
        assert!(matches!(RegressionSeverity::from_percentage(15.0), RegressionSeverity::Low));
    }

    #[test]
    fn test_failure_type_classification() {
        assert!(matches!(FailureType::from_error_message("Connection timeout"), FailureType::Timeout));
        assert!(matches!(FailureType::from_error_message("Database connection failed"), FailureType::DatabaseConnection));
        assert!(matches!(FailureType::from_error_message("Network error"), FailureType::NetworkError));
        assert!(matches!(FailureType::from_error_message("Expected 5, got 3"), FailureType::Assertion));
    }

    #[tokio::test]
    async fn test_monitoring_collector() {
        let temp_dir = TempDir::new().unwrap();
        let mut collector = TestMonitoringCollector::new(temp_dir.path().to_path_buf());

        collector.start_run(
            "test-123".to_string(),
            "test".to_string(),
            TestExecutionMode::Quick
        );

        assert!(collector.current_metrics().is_some());

        collector.complete_run().unwrap();
        assert!(collector.current_metrics().is_none());
        assert_eq!(collector.historical_metrics().len(), 1);
    }
}