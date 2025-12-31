use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

pub mod regression_dashboard;

/// Performance monitoring and analysis module
pub use regression_dashboard::{
    PerformanceRegressionDashboard, RegressionDashboardOverview,
    PerformanceAlert, PerformanceRecommendation, RegressionHealthScore,
    create_regression_dashboard_router,
};

/// Performance metrics for a single test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub test_identifier: String,
    pub execution_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub io_operations: u64,
    pub network_requests: u64,
    pub database_queries: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub error_count: u64,
    pub warning_count: u64,
    pub measured_at: DateTime<Utc>,
}

/// Aggregated performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStatistics {
    pub total_execution_time_ms: f64,
    pub average_execution_time_ms: f64,
    pub median_execution_time_ms: f64,
    pub p95_execution_time_ms: f64,
    pub p99_execution_time_ms: f64,
    pub min_execution_time_ms: f64,
    pub max_execution_time_ms: f64,
    pub standard_deviation_ms: f64,
    pub total_tests: usize,
    pub successful_tests: usize,
    pub failed_tests: usize,
    pub timeout_tests: usize,
    pub success_rate_percent: f64,
}

/// Performance benchmark for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmark {
    pub test_identifier: String,
    pub benchmark_type: BenchmarkType,
    pub baseline_duration_ms: f64,
    pub target_duration_ms: f64,
    pub acceptable_variance_percent: f64,
    pub established_at: DateTime<Utc>,
    pub last_validated: DateTime<Utc>,
    pub validation_count: u32,
    pub historical_performance: Vec<HistoricalDataPoint>,
}

/// Types of performance benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BenchmarkType {
    UnitTest,
    IntegrationTest,
    EndToEndTest,
    LoadTest,
    StressTest,
    ApiEndpoint,
    DatabaseQuery,
    UIInteraction,
}

/// Historical performance data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDataPoint {
    pub timestamp: DateTime<Utc>,
    pub duration_ms: f64,
    pub environment: String,
    pub git_commit: Option<String>,
    pub build_number: Option<String>,
}

/// Performance alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlertConfig {
    pub regression_threshold_percent: f64,
    pub critical_duration_threshold_ms: f64,
    pub warning_duration_threshold_ms: f64,
    pub consecutive_failures_threshold: u32,
    pub memory_usage_threshold_mb: f64,
    pub cpu_usage_threshold_percent: f64,
    pub alert_cooldown_minutes: u32,
}

/// Performance optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceOptimization {
    pub optimization_id: String,
    pub test_identifier: String,
    pub optimization_type: OptimizationType,
    pub title: String,
    pub description: String,
    pub estimated_improvement_percent: f64,
    pub implementation_effort: ImplementationEffort,
    pub priority_score: f64,
    pub technical_details: Vec<String>,
    pub code_examples: Vec<CodeExample>,
    pub related_issues: Vec<String>,
}

/// Types of performance optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    AlgorithmOptimization,
    DatabaseOptimization,
    MemoryOptimization,
    CacheImplementation,
    ParallelProcessing,
    ResourcePooling,
    QueryOptimization,
    IndexOptimization,
    CompressionImplementation,
    LazyLoading,
}

/// Implementation effort levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Trivial,   // < 1 hour
    Low,       // 1-4 hours
    Medium,    // 4-16 hours
    High,      // 16-40 hours
    VeryHigh,  // > 40 hours
}

/// Code example for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    pub language: String,
    pub title: String,
    pub before_code: String,
    pub after_code: String,
    pub explanation: String,
}

impl PerformanceMetrics {
    /// Calculate performance score (0-100)
    pub fn calculate_performance_score(&self) -> f64 {
        let mut score = 100.0;

        // Deduct based on execution time (assuming 1000ms is baseline)
        if self.execution_time_ms > 1000.0 {
            score -= ((self.execution_time_ms - 1000.0) / 1000.0).min(50.0);
        }

        // Deduct based on memory usage (assuming 100MB is baseline)
        if self.memory_usage_mb > 100.0 {
            score -= ((self.memory_usage_mb - 100.0) / 100.0).min(20.0);
        }

        // Deduct based on CPU usage (assuming 50% is baseline)
        if self.cpu_usage_percent > 50.0 {
            score -= ((self.cpu_usage_percent - 50.0) / 50.0).min(20.0);
        }

        // Deduct based on errors
        score -= (self.error_count as f64 * 5.0).min(10.0);

        score.max(0.0)
    }

    /// Check if performance is acceptable
    pub fn is_acceptable_performance(&self, benchmark: &PerformanceBenchmark) -> bool {
        let variance = ((self.execution_time_ms - benchmark.baseline_duration_ms)
            / benchmark.baseline_duration_ms * 100.0).abs();
        variance <= benchmark.acceptable_variance_percent
    }
}

impl PerformanceStatistics {
    /// Create statistics from a collection of metrics
    pub fn from_metrics(metrics: &[PerformanceMetrics]) -> Self {
        if metrics.is_empty() {
            return Self::default();
        }

        let mut execution_times: Vec<f64> = metrics.iter()
            .map(|m| m.execution_time_ms)
            .collect();
        execution_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let total_execution_time = execution_times.iter().sum();
        let average_execution_time = total_execution_time / execution_times.len() as f64;

        let median_execution_time = if execution_times.len() % 2 == 0 {
            (execution_times[execution_times.len() / 2 - 1] +
             execution_times[execution_times.len() / 2]) / 2.0
        } else {
            execution_times[execution_times.len() / 2]
        };

        let p95_index = ((execution_times.len() as f64) * 0.95) as usize;
        let p99_index = ((execution_times.len() as f64) * 0.99) as usize;

        let p95_execution_time = execution_times.get(p95_index.min(execution_times.len() - 1))
            .copied().unwrap_or(0.0);
        let p99_execution_time = execution_times.get(p99_index.min(execution_times.len() - 1))
            .copied().unwrap_or(0.0);

        let min_execution_time = execution_times.first().copied().unwrap_or(0.0);
        let max_execution_time = execution_times.last().copied().unwrap_or(0.0);

        // Calculate standard deviation
        let variance = execution_times.iter()
            .map(|time| (time - average_execution_time).powi(2))
            .sum::<f64>() / execution_times.len() as f64;
        let standard_deviation = variance.sqrt();

        let successful_tests = metrics.iter()
            .filter(|m| m.error_count == 0)
            .count();
        let failed_tests = metrics.len() - successful_tests;
        let success_rate = (successful_tests as f64 / metrics.len() as f64) * 100.0;

        Self {
            total_execution_time_ms: total_execution_time,
            average_execution_time_ms: average_execution_time,
            median_execution_time_ms: median_execution_time,
            p95_execution_time_ms: p95_execution_time,
            p99_execution_time_ms: p99_execution_time,
            min_execution_time_ms: min_execution_time,
            max_execution_time_ms: max_execution_time,
            standard_deviation_ms: standard_deviation,
            total_tests: metrics.len(),
            successful_tests,
            failed_tests,
            timeout_tests: 0, // Would need to be tracked separately
            success_rate_percent: success_rate,
        }
    }

    /// Check if performance has regressed compared to baseline
    pub fn has_regressed(&self, baseline: &Self, threshold_percent: f64) -> bool {
        let regression_percent = ((self.average_execution_time_ms - baseline.average_execution_time_ms)
            / baseline.average_execution_time_ms * 100.0);
        regression_percent > threshold_percent
    }
}

impl PerformanceBenchmark {
    /// Create a new benchmark from recent performance data
    pub fn from_recent_data(
        test_identifier: String,
        benchmark_type: BenchmarkType,
        recent_metrics: &[PerformanceMetrics],
        acceptable_variance_percent: f64,
    ) -> Self {
        let stats = PerformanceStatistics::from_metrics(recent_metrics);

        let historical_performance = recent_metrics.iter().map(|m| HistoricalDataPoint {
            timestamp: m.measured_at,
            duration_ms: m.execution_time_ms,
            environment: "test".to_string(), // Would be extracted from metrics
            git_commit: None,
            build_number: None,
        }).collect();

        Self {
            test_identifier,
            benchmark_type,
            baseline_duration_ms: stats.median_execution_time_ms,
            target_duration_ms: stats.median_execution_time_ms * 0.9, // 10% improvement target
            acceptable_variance_percent,
            established_at: Utc::now(),
            last_validated: Utc::now(),
            validation_count: recent_metrics.len() as u32,
            historical_performance,
        }
    }

    /// Update benchmark with new performance data
    pub fn update_with_new_data(&mut self, metrics: &[PerformanceMetrics]) {
        let stats = PerformanceStatistics::from_metrics(metrics);

        // Update baseline using exponential smoothing
        let alpha = 0.2; // Smoothing factor
        self.baseline_duration_ms = alpha * stats.median_execution_time_ms +
            (1.0 - alpha) * self.baseline_duration_ms;

        self.last_validated = Utc::now();
        self.validation_count += metrics.len() as u32;

        // Add new historical data points
        for metric in metrics {
            self.historical_performance.push(HistoricalDataPoint {
                timestamp: metric.measured_at,
                duration_ms: metric.execution_time_ms,
                environment: "test".to_string(),
                git_commit: None,
                build_number: None,
            });
        }

        // Keep only recent history (last 100 data points)
        if self.historical_performance.len() > 100 {
            self.historical_performance.drain(0..self.historical_performance.len() - 100);
        }
    }

    /// Check if current performance meets the benchmark
    pub fn meets_benchmark(&self, current_duration_ms: f64) -> bool {
        let variance_percent = ((current_duration_ms - self.baseline_duration_ms)
            / self.baseline_duration_ms * 100.0).abs();
        variance_percent <= self.acceptable_variance_percent
    }
}

impl Default for PerformanceStatistics {
    fn default() -> Self {
        Self {
            total_execution_time_ms: 0.0,
            average_execution_time_ms: 0.0,
            median_execution_time_ms: 0.0,
            p95_execution_time_ms: 0.0,
            p99_execution_time_ms: 0.0,
            min_execution_time_ms: 0.0,
            max_execution_time_ms: 0.0,
            standard_deviation_ms: 0.0,
            total_tests: 0,
            successful_tests: 0,
            failed_tests: 0,
            timeout_tests: 0,
            success_rate_percent: 0.0,
        }
    }
}

impl Default for PerformanceAlertConfig {
    fn default() -> Self {
        Self {
            regression_threshold_percent: 20.0,
            critical_duration_threshold_ms: 30000.0,
            warning_duration_threshold_ms: 10000.0,
            consecutive_failures_threshold: 3,
            memory_usage_threshold_mb: 500.0,
            cpu_usage_threshold_percent: 80.0,
            alert_cooldown_minutes: 10,
        }
    }
}