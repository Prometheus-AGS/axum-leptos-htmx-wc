use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use chrono::{DateTime, Utc, Duration};
use crate::testing::entities::TestExecutionResult;
use super::{AnalyticsResult, InsightLevel, Insight};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityAnalyzer {
    pub config: ReliabilityConfig,
    test_history: Vec<TestRunRecord>,
    flaky_test_tracker: HashMap<String, FlakyTestData>,
    reliability_cache: HashMap<String, CachedReliabilityMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityConfig {
    pub flaky_threshold_runs: u32,
    pub flaky_failure_rate_threshold: f64,
    pub stability_window_days: u32,
    pub confidence_interval: f64,
    pub minimum_runs_for_analysis: u32,
    pub outlier_detection_enabled: bool,
    pub trend_analysis_window: u32,
    pub alert_thresholds: ReliabilityAlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityAlertThresholds {
    pub critical_failure_rate: f64,     // >10% failure rate
    pub major_instability: f64,         // >5% instability
    pub flaky_test_percentage: f64,     // >3% of tests are flaky
    pub success_rate_decline: f64,      // >5% decline in success rate
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunRecord {
    pub timestamp: DateTime<Utc>,
    pub execution_id: String,
    pub test_results: HashMap<String, TestResult>,
    pub environment_info: EnvironmentInfo,
    pub execution_context: ExecutionContext,
    pub overall_metrics: OverallRunMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_id: String,
    pub test_name: String,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub failure_reason: Option<String>,
    pub retry_count: u32,
    pub environment: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Flaky,
    Timeout,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub environment_name: String,
    pub os_version: String,
    pub runtime_version: String,
    pub hardware_specs: HashMap<String, String>,
    pub network_conditions: NetworkConditions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConditions {
    pub latency_ms: Option<f64>,
    pub bandwidth_mbps: Option<f64>,
    pub packet_loss_percent: Option<f64>,
    pub connection_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub branch: String,
    pub commit_hash: String,
    pub build_number: Option<String>,
    pub trigger_type: TriggerType,
    pub parallelism_level: u32,
    pub resource_constraints: ResourceConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    Manual,
    CI,
    Scheduled,
    PullRequest,
    Release,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConstraints {
    pub max_memory_mb: Option<f64>,
    pub max_cpu_cores: Option<u32>,
    pub timeout_minutes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallRunMetrics {
    pub total_tests: u32,
    pub passed_tests: u32,
    pub failed_tests: u32,
    pub skipped_tests: u32,
    pub flaky_tests: u32,
    pub success_rate: f64,
    pub failure_rate: f64,
    pub flakiness_rate: f64,
    pub total_duration_ms: u64,
    pub avg_test_duration_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakyTestData {
    pub test_id: String,
    pub test_name: String,
    pub total_runs: u32,
    pub failure_count: u32,
    pub success_count: u32,
    pub failure_rate: f64,
    pub failure_patterns: Vec<FailurePattern>,
    pub environments_affected: Vec<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub severity: FlakySeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    pub pattern_type: FailurePatternType,
    pub frequency: f64,
    pub conditions: Vec<String>,
    pub example_failures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailurePatternType {
    Timing,
    EnvironmentSpecific,
    LoadDependent,
    Sequential,
    Random,
    Dependency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlakySeverity {
    Critical,    // >50% failure rate
    High,        // 20-50% failure rate
    Medium,      // 10-20% failure rate
    Low,         // 5-10% failure rate
    Minimal,     // <5% failure rate
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedReliabilityMetric {
    pub metric_name: String,
    pub value: f64,
    pub confidence: f64,
    pub last_calculated: DateTime<Utc>,
    pub sample_size: usize,
    pub trend: ReliabilityTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReliabilityTrend {
    Improving,
    Stable,
    Declining,
    Volatile,
    InsufficientData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityAnalysisReport {
    pub summary: ReliabilitySummary,
    pub stability_analysis: StabilityAnalysis,
    pub flaky_test_report: FlakyTestReport,
    pub trend_analysis: ReliabilityTrendAnalysis,
    pub environment_comparison: EnvironmentReliabilityComparison,
    pub failure_analysis: FailureAnalysis,
    pub recommendations: Vec<ReliabilityRecommendation>,
    pub quality_metrics: QualityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilitySummary {
    pub overall_reliability_score: f64,
    pub success_rate: f64,
    pub failure_rate: f64,
    pub flakiness_percentage: f64,
    pub stability_index: f64,
    pub consistency_score: f64,
    pub total_test_runs: u32,
    pub unique_tests: u32,
    pub problematic_tests: u32,
    pub reliability_grade: ReliabilityGrade,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReliabilityGrade {
    Excellent, // >95% reliability
    Good,      // 90-95% reliability
    Fair,      // 85-90% reliability
    Poor,      // 80-85% reliability
    Critical,  // <80% reliability
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityAnalysis {
    pub stability_over_time: Vec<StabilityDataPoint>,
    pub stability_by_environment: HashMap<String, f64>,
    pub stability_by_test_suite: HashMap<String, f64>,
    pub volatility_metrics: VolatilityMetrics,
    pub confidence_intervals: HashMap<String, (f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StabilityDataPoint {
    pub timestamp: DateTime<Utc>,
    pub success_rate: f64,
    pub failure_rate: f64,
    pub flakiness_rate: f64,
    pub confidence_level: f64,
    pub sample_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityMetrics {
    pub success_rate_volatility: f64,
    pub failure_rate_volatility: f64,
    pub performance_volatility: f64,
    pub overall_volatility_score: f64,
    pub volatility_trend: ReliabilityTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakyTestReport {
    pub total_flaky_tests: u32,
    pub flaky_test_percentage: f64,
    pub flaky_tests_by_severity: HashMap<FlakySeverity, Vec<FlakyTestData>>,
    pub most_problematic_tests: Vec<FlakyTestData>,
    pub flaky_test_trends: FlakyTestTrends,
    pub impact_analysis: FlakyTestImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakyTestTrends {
    pub new_flaky_tests: Vec<FlakyTestData>,
    pub resolved_flaky_tests: Vec<String>,
    pub worsening_tests: Vec<FlakyTestData>,
    pub improving_tests: Vec<FlakyTestData>,
    pub trend_direction: ReliabilityTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakyTestImpact {
    pub development_time_cost_hours: f64,
    pub ci_resource_waste_percent: f64,
    pub developer_confidence_impact: ConfidenceImpact,
    pub release_reliability_risk: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfidenceImpact {
    Minimal,
    Low,
    Moderate,
    High,
    Severe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityTrendAnalysis {
    pub short_term_trend: TrendAnalysisResult,
    pub long_term_trend: TrendAnalysisResult,
    pub seasonal_patterns: Vec<SeasonalPattern>,
    pub regression_points: Vec<RegressionPoint>,
    pub improvement_points: Vec<ImprovementPoint>,
    pub predictive_analysis: PredictiveReliabilityAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysisResult {
    pub direction: ReliabilityTrend,
    pub magnitude: f64,
    pub confidence: f64,
    pub time_period: String,
    pub key_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalPattern {
    pub pattern_type: PatternType,
    pub frequency: String,
    pub impact_magnitude: f64,
    pub affected_metrics: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    DayOfWeek,
    TimeOfDay,
    Monthly,
    Release,
    LoadBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionPoint {
    pub timestamp: DateTime<Utc>,
    pub affected_metric: String,
    pub magnitude: f64,
    pub likely_cause: String,
    pub duration_hours: Option<f64>,
    pub resolution_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementPoint {
    pub timestamp: DateTime<Utc>,
    pub improved_metric: String,
    pub magnitude: f64,
    pub contributing_factors: Vec<String>,
    pub sustainability: SustainabilityAssessment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SustainabilityAssessment {
    Highlysustainable,
    Sustainable,
    ModeratelySustainable,
    Unsustainable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictiveReliabilityAnalysis {
    pub predicted_success_rate_7d: f64,
    pub predicted_success_rate_30d: f64,
    pub predicted_flaky_test_count: u32,
    pub reliability_forecast: Vec<ReliabilityForecast>,
    pub risk_factors: Vec<RiskFactor>,
    pub confidence_intervals: HashMap<String, (f64, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityForecast {
    pub metric_name: String,
    pub forecast_horizon_days: u32,
    pub predicted_value: f64,
    pub confidence_interval: (f64, f64),
    pub forecast_accuracy: f64,
    pub assumptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub factor_name: String,
    pub impact_severity: RiskLevel,
    pub probability: f64,
    pub description: String,
    pub mitigation_strategies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentReliabilityComparison {
    pub environment_scores: HashMap<String, EnvironmentReliabilityScore>,
    pub cross_environment_analysis: CrossEnvironmentAnalysis,
    pub environment_specific_issues: HashMap<String, Vec<String>>,
    pub portability_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentReliabilityScore {
    pub environment_name: String,
    pub success_rate: f64,
    pub stability_score: f64,
    pub flakiness_rate: f64,
    pub unique_failures: u32,
    pub reliability_rank: u32,
    pub relative_performance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossEnvironmentAnalysis {
    pub consistency_score: f64,
    pub environment_specific_failures: HashMap<String, Vec<String>>,
    pub portable_test_percentage: f64,
    pub most_stable_environment: String,
    pub most_problematic_environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureAnalysis {
    pub failure_categories: HashMap<String, FailureCategoryAnalysis>,
    pub root_cause_analysis: Vec<RootCauseAnalysis>,
    pub failure_patterns: Vec<FailurePattern>,
    pub recovery_analysis: RecoveryAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureCategoryAnalysis {
    pub category_name: String,
    pub failure_count: u32,
    pub failure_percentage: f64,
    pub average_impact: f64,
    pub trend: ReliabilityTrend,
    pub common_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseAnalysis {
    pub failure_signature: String,
    pub root_cause: String,
    pub frequency: u32,
    pub impact_score: f64,
    pub affected_tests: Vec<String>,
    pub resolution_steps: Vec<String>,
    pub prevention_measures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAnalysis {
    pub average_recovery_time_hours: f64,
    pub recovery_success_rate: f64,
    pub recovery_patterns: Vec<String>,
    pub automated_recovery_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityRecommendation {
    pub priority: RecommendationPriority,
    pub category: ReliabilityRecommendationCategory,
    pub title: String,
    pub description: String,
    pub expected_improvement: f64,
    pub implementation_effort: ImplementationEffort,
    pub cost_benefit_ratio: f64,
    pub implementation_steps: Vec<String>,
    pub success_metrics: Vec<String>,
    pub timeline_estimate: String,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReliabilityRecommendationCategory {
    FlakyTestResolution,
    TestStabilization,
    EnvironmentImprovement,
    InfrastructureOptimization,
    ProcessImprovement,
    MonitoringEnhancement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Minimal,  // <1 day
    Low,      // 1-3 days
    Medium,   // 1-2 weeks
    High,     // 2-4 weeks
    VeryHigh, // >1 month
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub test_suite_health: f64,
    pub development_velocity_impact: f64,
    pub confidence_level: f64,
    pub maintainability_score: f64,
    pub technical_debt_factor: f64,
    pub overall_quality_grade: QualityGrade,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityGrade {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

impl Default for ReliabilityConfig {
    fn default() -> Self {
        Self {
            flaky_threshold_runs: 10,
            flaky_failure_rate_threshold: 0.05, // 5% failure rate
            stability_window_days: 30,
            confidence_interval: 0.95,
            minimum_runs_for_analysis: 5,
            outlier_detection_enabled: true,
            trend_analysis_window: 14,
            alert_thresholds: ReliabilityAlertThresholds {
                critical_failure_rate: 0.10,
                major_instability: 0.05,
                flaky_test_percentage: 0.03,
                success_rate_decline: 0.05,
            },
        }
    }
}

impl ReliabilityAnalyzer {
    pub fn new(config: ReliabilityConfig) -> Self {
        Self {
            config,
            test_history: Vec::new(),
            flaky_test_tracker: HashMap::new(),
            reliability_cache: HashMap::new(),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(ReliabilityConfig::default())
    }

    pub fn load_test_data(&mut self, test_results: Vec<TestExecutionResult>) -> Result<(), Box<dyn std::error::Error>> {
        self.test_history = test_results
            .into_iter()
            .filter_map(|result| self.convert_to_test_run_record(result))
            .collect();

        // Sort by timestamp
        self.test_history.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Update flaky test tracking
        self.update_flaky_test_tracking()?;

        // Clear cache after loading new data
        self.reliability_cache.clear();

        Ok(())
    }

    fn convert_to_test_run_record(&self, result: TestExecutionResult) -> Option<TestRunRecord> {
        let mut test_results = HashMap::new();

        // Convert individual test results
        for (test_name, status) in &result.test_results {
            test_results.insert(test_name.clone(), TestResult {
                test_id: format!("{}_{}", result.execution_id, test_name),
                test_name: test_name.clone(),
                status: self.convert_test_status(status),
                duration_ms: result.metadata.get(&format!("{}_duration", test_name))
                    .unwrap_or(&"0".to_string())
                    .parse().unwrap_or(0),
                failure_reason: if status == "failed" {
                    result.metadata.get(&format!("{}_error", test_name)).cloned()
                } else {
                    None
                },
                retry_count: result.metadata.get(&format!("{}_retries", test_name))
                    .unwrap_or(&"0".to_string())
                    .parse().unwrap_or(0),
                environment: result.environment.get("environment").unwrap_or(&"unknown".to_string()).clone(),
                tags: result.test_suites.clone(),
            });
        }

        let environment_info = EnvironmentInfo {
            environment_name: result.environment.get("environment").unwrap_or(&"unknown".to_string()).clone(),
            os_version: result.environment.get("os_version").unwrap_or(&"unknown".to_string()).clone(),
            runtime_version: result.environment.get("runtime_version").unwrap_or(&"unknown".to_string()).clone(),
            hardware_specs: HashMap::new(),
            network_conditions: NetworkConditions {
                latency_ms: result.metadata.get("network_latency_ms").and_then(|s| s.parse().ok()),
                bandwidth_mbps: result.metadata.get("network_bandwidth_mbps").and_then(|s| s.parse().ok()),
                packet_loss_percent: result.metadata.get("network_packet_loss").and_then(|s| s.parse().ok()),
                connection_type: result.metadata.get("network_connection_type").cloned(),
            },
        };

        let execution_context = ExecutionContext {
            branch: result.environment.get("branch").unwrap_or(&"main".to_string()).clone(),
            commit_hash: result.environment.get("commit").unwrap_or(&"unknown".to_string()).clone(),
            build_number: result.metadata.get("build_number").cloned(),
            trigger_type: self.determine_trigger_type(&result),
            parallelism_level: result.metadata.get("parallelism_level")
                .unwrap_or(&"1".to_string())
                .parse().unwrap_or(1),
            resource_constraints: ResourceConstraints {
                max_memory_mb: result.metadata.get("max_memory_mb").and_then(|s| s.parse().ok()),
                max_cpu_cores: result.metadata.get("max_cpu_cores").and_then(|s| s.parse().ok()),
                timeout_minutes: result.metadata.get("timeout_minutes").and_then(|s| s.parse().ok()),
            },
        };

        let overall_metrics = OverallRunMetrics {
            total_tests: result.total_tests,
            passed_tests: result.successful_tests,
            failed_tests: result.failed_tests,
            skipped_tests: result.skipped_tests,
            flaky_tests: result.flaky_tests,
            success_rate: if result.total_tests > 0 {
                (result.successful_tests as f64 / result.total_tests as f64) * 100.0
            } else {
                100.0
            },
            failure_rate: if result.total_tests > 0 {
                (result.failed_tests as f64 / result.total_tests as f64) * 100.0
            } else {
                0.0
            },
            flakiness_rate: if result.total_tests > 0 {
                (result.flaky_tests as f64 / result.total_tests as f64) * 100.0
            } else {
                0.0
            },
            total_duration_ms: result.duration_ms,
            avg_test_duration_ms: if result.total_tests > 0 {
                result.duration_ms as f64 / result.total_tests as f64
            } else {
                0.0
            },
        };

        Some(TestRunRecord {
            timestamp: result.timestamp,
            execution_id: result.execution_id,
            test_results,
            environment_info,
            execution_context,
            overall_metrics,
        })
    }

    fn convert_test_status(&self, status: &str) -> TestStatus {
        match status.to_lowercase().as_str() {
            "passed" | "success" => TestStatus::Passed,
            "failed" | "failure" => TestStatus::Failed,
            "skipped" | "ignored" => TestStatus::Skipped,
            "flaky" | "unstable" => TestStatus::Flaky,
            "timeout" => TestStatus::Timeout,
            _ => TestStatus::Error,
        }
    }

    fn determine_trigger_type(&self, result: &TestExecutionResult) -> TriggerType {
        if let Some(trigger) = result.metadata.get("trigger_type") {
            match trigger.to_lowercase().as_str() {
                "manual" => TriggerType::Manual,
                "ci" | "continuous_integration" => TriggerType::CI,
                "scheduled" | "cron" => TriggerType::Scheduled,
                "pull_request" | "pr" => TriggerType::PullRequest,
                "release" => TriggerType::Release,
                _ => TriggerType::CI,
            }
        } else {
            TriggerType::CI
        }
    }

    fn update_flaky_test_tracking(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut test_run_counts: HashMap<String, (u32, u32)> = HashMap::new(); // (total_runs, failures)

        // Count runs and failures for each test
        for run_record in &self.test_history {
            for (test_name, test_result) in &run_record.test_results {
                let entry = test_run_counts.entry(test_name.clone()).or_insert((0, 0));
                entry.0 += 1; // total runs

                match test_result.status {
                    TestStatus::Failed | TestStatus::Error | TestStatus::Timeout => entry.1 += 1,
                    TestStatus::Flaky => entry.1 += 1,
                    _ => {}
                }
            }
        }

        // Update flaky test tracker
        for (test_name, (total_runs, failures)) in test_run_counts {
            if total_runs >= self.config.flaky_threshold_runs {
                let failure_rate = failures as f64 / total_runs as f64;

                if failure_rate >= self.config.flaky_failure_rate_threshold && failure_rate < 1.0 {
                    let severity = self.determine_flaky_severity(failure_rate);
                    let failure_patterns = self.analyze_failure_patterns(&test_name);
                    let environments_affected = self.get_affected_environments(&test_name);

                    let first_seen = self.find_first_test_occurrence(&test_name);
                    let last_seen = self.find_last_test_occurrence(&test_name);

                    self.flaky_test_tracker.insert(test_name.clone(), FlakyTestData {
                        test_id: format!("flaky_{}", test_name),
                        test_name: test_name.clone(),
                        total_runs,
                        failure_count: failures,
                        success_count: total_runs - failures,
                        failure_rate: failure_rate * 100.0,
                        failure_patterns,
                        environments_affected,
                        first_seen: first_seen.unwrap_or(Utc::now()),
                        last_seen: last_seen.unwrap_or(Utc::now()),
                        severity,
                    });
                }
            }
        }

        Ok(())
    }

    fn determine_flaky_severity(&self, failure_rate: f64) -> FlakySeverity {
        match failure_rate {
            x if x >= 0.5 => FlakySeverity::Critical,
            x if x >= 0.2 => FlakySeverity::High,
            x if x >= 0.1 => FlakySeverity::Medium,
            x if x >= 0.05 => FlakySeverity::Low,
            _ => FlakySeverity::Minimal,
        }
    }

    fn analyze_failure_patterns(&self, test_name: &str) -> Vec<FailurePattern> {
        let mut patterns = Vec::new();
        let mut failure_reasons: HashMap<String, u32> = HashMap::new();
        let mut environment_failures: HashMap<String, u32> = HashMap::new();

        // Collect failure data
        for run_record in &self.test_history {
            if let Some(test_result) = run_record.test_results.get(test_name) {
                if matches!(test_result.status, TestStatus::Failed | TestStatus::Error | TestStatus::Flaky) {
                    // Track failure reasons
                    if let Some(reason) = &test_result.failure_reason {
                        *failure_reasons.entry(reason.clone()).or_insert(0) += 1;
                    }

                    // Track environment-specific failures
                    *environment_failures.entry(test_result.environment.clone()).or_insert(0) += 1;
                }
            }
        }

        // Analyze timing patterns
        let timing_failures = self.analyze_timing_patterns(test_name);
        if timing_failures > 0.3 {
            patterns.push(FailurePattern {
                pattern_type: FailurePatternType::Timing,
                frequency: timing_failures,
                conditions: vec!["Race conditions".to_string(), "Timeout issues".to_string()],
                example_failures: vec!["Timeout waiting for element".to_string()],
            });
        }

        // Analyze environment-specific patterns
        if environment_failures.len() > 1 {
            let total_env_failures: u32 = environment_failures.values().sum();
            for (env, count) in environment_failures {
                if (*count as f64 / total_env_failures as f64) > 0.6 {
                    patterns.push(FailurePattern {
                        pattern_type: FailurePatternType::EnvironmentSpecific,
                        frequency: *count as f64 / total_env_failures as f64,
                        conditions: vec![format!("Environment: {}", env)],
                        example_failures: failure_reasons.keys().take(2).cloned().collect(),
                    });
                }
            }
        }

        patterns
    }

    fn analyze_timing_patterns(&self, test_name: &str) -> f64 {
        let mut timing_related_failures = 0;
        let mut total_failures = 0;

        for run_record in &self.test_history {
            if let Some(test_result) = run_record.test_results.get(test_name) {
                if matches!(test_result.status, TestStatus::Failed | TestStatus::Timeout) {
                    total_failures += 1;

                    if let Some(reason) = &test_result.failure_reason {
                        if reason.to_lowercase().contains("timeout") ||
                           reason.to_lowercase().contains("timing") ||
                           reason.to_lowercase().contains("race") {
                            timing_related_failures += 1;
                        }
                    }

                    // Also check if it's a timeout status
                    if matches!(test_result.status, TestStatus::Timeout) {
                        timing_related_failures += 1;
                    }
                }
            }
        }

        if total_failures > 0 {
            timing_related_failures as f64 / total_failures as f64
        } else {
            0.0
        }
    }

    fn get_affected_environments(&self, test_name: &str) -> Vec<String> {
        let mut environments = std::collections::HashSet::new();

        for run_record in &self.test_history {
            if let Some(test_result) = run_record.test_results.get(test_name) {
                if matches!(test_result.status, TestStatus::Failed | TestStatus::Error | TestStatus::Flaky) {
                    environments.insert(test_result.environment.clone());
                }
            }
        }

        environments.into_iter().collect()
    }

    fn find_first_test_occurrence(&self, test_name: &str) -> Option<DateTime<Utc>> {
        self.test_history
            .iter()
            .find(|record| record.test_results.contains_key(test_name))
            .map(|record| record.timestamp)
    }

    fn find_last_test_occurrence(&self, test_name: &str) -> Option<DateTime<Utc>> {
        self.test_history
            .iter()
            .rev()
            .find(|record| record.test_results.contains_key(test_name))
            .map(|record| record.timestamp)
    }

    pub fn analyze_reliability(&mut self) -> Result<ReliabilityAnalysisReport, Box<dyn std::error::Error>> {
        if self.test_history.is_empty() {
            return Err("No test history available for reliability analysis".into());
        }

        let summary = self.generate_reliability_summary()?;
        let stability_analysis = self.analyze_stability()?;
        let flaky_test_report = self.generate_flaky_test_report()?;
        let trend_analysis = self.analyze_reliability_trends()?;
        let environment_comparison = self.compare_environment_reliability()?;
        let failure_analysis = self.analyze_failures()?;
        let recommendations = self.generate_reliability_recommendations(&summary, &flaky_test_report, &stability_analysis)?;
        let quality_metrics = self.calculate_quality_metrics(&summary)?;

        Ok(ReliabilityAnalysisReport {
            summary,
            stability_analysis,
            flaky_test_report,
            trend_analysis,
            environment_comparison,
            failure_analysis,
            recommendations,
            quality_metrics,
        })
    }

    fn generate_reliability_summary(&self) -> Result<ReliabilitySummary, Box<dyn std::error::Error>> {
        let recent_runs = self.get_recent_runs(30);
        if recent_runs.is_empty() {
            return Err("No recent test runs available".into());
        }

        let total_test_runs = recent_runs.len() as u32;
        let total_success_rate: f64 = recent_runs.iter().map(|r| r.overall_metrics.success_rate).sum();
        let total_failure_rate: f64 = recent_runs.iter().map(|r| r.overall_metrics.failure_rate).sum();
        let total_flakiness_rate: f64 = recent_runs.iter().map(|r| r.overall_metrics.flakiness_rate).sum();

        let success_rate = total_success_rate / total_test_runs as f64;
        let failure_rate = total_failure_rate / total_test_runs as f64;
        let flakiness_percentage = total_flakiness_rate / total_test_runs as f64;

        let stability_index = self.calculate_stability_index(&recent_runs);
        let consistency_score = self.calculate_consistency_score(&recent_runs);

        let unique_tests = self.count_unique_tests(&recent_runs);
        let problematic_tests = self.flaky_test_tracker.len() as u32;

        let overall_reliability_score = self.calculate_overall_reliability_score(success_rate, stability_index, consistency_score);
        let reliability_grade = self.determine_reliability_grade(overall_reliability_score);

        Ok(ReliabilitySummary {
            overall_reliability_score,
            success_rate,
            failure_rate,
            flakiness_percentage,
            stability_index,
            consistency_score,
            total_test_runs,
            unique_tests,
            problematic_tests,
            reliability_grade,
        })
    }

    fn get_recent_runs(&self, days: u32) -> Vec<&TestRunRecord> {
        let cutoff = Utc::now() - Duration::days(days as i64);
        self.test_history
            .iter()
            .filter(|record| record.timestamp >= cutoff)
            .collect()
    }

    fn calculate_stability_index(&self, runs: &[&TestRunRecord]) -> f64 {
        if runs.is_empty() {
            return 0.0;
        }

        let success_rates: Vec<f64> = runs.iter().map(|r| r.overall_metrics.success_rate).collect();
        let mean_success_rate = success_rates.iter().sum::<f64>() / success_rates.len() as f64;

        let variance = success_rates.iter()
            .map(|rate| (rate - mean_success_rate).powi(2))
            .sum::<f64>() / success_rates.len() as f64;

        let std_dev = variance.sqrt();
        let coefficient_of_variation = if mean_success_rate > 0.0 {
            std_dev / mean_success_rate
        } else {
            1.0
        };

        // Higher stability means lower coefficient of variation
        (1.0 - coefficient_of_variation.min(1.0)) * 100.0
    }

    fn calculate_consistency_score(&self, runs: &[&TestRunRecord]) -> f64 {
        if runs.len() < 2 {
            return 100.0;
        }

        let mut consistency_scores = Vec::new();

        // Compare consecutive runs
        for window in runs.windows(2) {
            let current = &window[0];
            let next = &window[1];

            let success_rate_diff = (current.overall_metrics.success_rate - next.overall_metrics.success_rate).abs();
            let flakiness_diff = (current.overall_metrics.flakiness_rate - next.overall_metrics.flakiness_rate).abs();

            let run_consistency = 100.0 - (success_rate_diff + flakiness_diff) / 2.0;
            consistency_scores.push(run_consistency.max(0.0));
        }

        consistency_scores.iter().sum::<f64>() / consistency_scores.len() as f64
    }

    fn count_unique_tests(&self, runs: &[&TestRunRecord]) -> u32 {
        let mut unique_tests = std::collections::HashSet::new();
        for run in runs {
            for test_name in run.test_results.keys() {
                unique_tests.insert(test_name.clone());
            }
        }
        unique_tests.len() as u32
    }

    fn calculate_overall_reliability_score(&self, success_rate: f64, stability_index: f64, consistency_score: f64) -> f64 {
        // Weighted combination of metrics
        (success_rate * 0.5) + (stability_index * 0.3) + (consistency_score * 0.2)
    }

    fn determine_reliability_grade(&self, score: f64) -> ReliabilityGrade {
        match score {
            x if x >= 95.0 => ReliabilityGrade::Excellent,
            x if x >= 90.0 => ReliabilityGrade::Good,
            x if x >= 85.0 => ReliabilityGrade::Fair,
            x if x >= 80.0 => ReliabilityGrade::Poor,
            _ => ReliabilityGrade::Critical,
        }
    }

    fn analyze_stability(&self) -> Result<StabilityAnalysis, Box<dyn std::error::Error>> {
        let stability_over_time = self.calculate_stability_over_time()?;
        let stability_by_environment = self.calculate_stability_by_environment();
        let stability_by_test_suite = self.calculate_stability_by_test_suite();
        let volatility_metrics = self.calculate_volatility_metrics();
        let confidence_intervals = self.calculate_confidence_intervals();

        Ok(StabilityAnalysis {
            stability_over_time,
            stability_by_environment,
            stability_by_test_suite,
            volatility_metrics,
            confidence_intervals,
        })
    }

    fn calculate_stability_over_time(&self) -> Result<Vec<StabilityDataPoint>, Box<dyn std::error::Error>> {
        let mut data_points = Vec::new();
        let window_size = Duration::days(1);

        if self.test_history.is_empty() {
            return Ok(data_points);
        }

        let start_time = self.test_history.first().unwrap().timestamp;
        let end_time = self.test_history.last().unwrap().timestamp;

        let mut current_time = start_time;
        while current_time <= end_time {
            let window_runs: Vec<&TestRunRecord> = self.test_history
                .iter()
                .filter(|record| {
                    record.timestamp >= current_time &&
                    record.timestamp < current_time + window_size
                })
                .collect();

            if !window_runs.is_empty() {
                let success_rate = window_runs.iter()
                    .map(|r| r.overall_metrics.success_rate)
                    .sum::<f64>() / window_runs.len() as f64;

                let failure_rate = window_runs.iter()
                    .map(|r| r.overall_metrics.failure_rate)
                    .sum::<f64>() / window_runs.len() as f64;

                let flakiness_rate = window_runs.iter()
                    .map(|r| r.overall_metrics.flakiness_rate)
                    .sum::<f64>() / window_runs.len() as f64;

                let confidence_level = self.calculate_confidence_level_for_window(&window_runs);
                let sample_size = window_runs.len() as u32;

                data_points.push(StabilityDataPoint {
                    timestamp: current_time,
                    success_rate,
                    failure_rate,
                    flakiness_rate,
                    confidence_level,
                    sample_size,
                });
            }

            current_time = current_time + window_size;
        }

        Ok(data_points)
    }

    fn calculate_confidence_level_for_window(&self, runs: &[&TestRunRecord]) -> f64 {
        if runs.is_empty() {
            return 0.0;
        }

        // Confidence based on sample size and consistency
        let sample_size_factor = (runs.len() as f64).ln().min(5.0) / 5.0;
        let consistency_factor = self.calculate_consistency_score(runs) / 100.0;

        (sample_size_factor * consistency_factor * 100.0).min(95.0)
    }

    fn calculate_stability_by_environment(&self) -> HashMap<String, f64> {
        let mut environment_stats: HashMap<String, (f64, u32)> = HashMap::new();

        for run in &self.test_history {
            let env_name = &run.environment_info.environment_name;
            let entry = environment_stats.entry(env_name.clone()).or_insert((0.0, 0));
            entry.0 += run.overall_metrics.success_rate;
            entry.1 += 1;
        }

        environment_stats
            .into_iter()
            .map(|(env, (total_rate, count))| (env, total_rate / count as f64))
            .collect()
    }

    fn calculate_stability_by_test_suite(&self) -> HashMap<String, f64> {
        let mut suite_stats: HashMap<String, (f64, u32)> = HashMap::new();

        for run in &self.test_history {
            for test_result in run.test_results.values() {
                for tag in &test_result.tags {
                    let entry = suite_stats.entry(tag.clone()).or_insert((0.0, 0));
                    entry.1 += 1;

                    match test_result.status {
                        TestStatus::Passed => entry.0 += 100.0,
                        TestStatus::Flaky => entry.0 += 50.0,
                        _ => entry.0 += 0.0,
                    }
                }
            }
        }

        suite_stats
            .into_iter()
            .map(|(suite, (total_score, count))| (suite, total_score / count as f64))
            .collect()
    }

    fn calculate_volatility_metrics(&self) -> VolatilityMetrics {
        let recent_runs = self.get_recent_runs(14);

        if recent_runs.len() < 3 {
            return VolatilityMetrics {
                success_rate_volatility: 0.0,
                failure_rate_volatility: 0.0,
                performance_volatility: 0.0,
                overall_volatility_score: 0.0,
                volatility_trend: ReliabilityTrend::InsufficientData,
            };
        }

        let success_rates: Vec<f64> = recent_runs.iter().map(|r| r.overall_metrics.success_rate).collect();
        let failure_rates: Vec<f64> = recent_runs.iter().map(|r| r.overall_metrics.failure_rate).collect();
        let durations: Vec<f64> = recent_runs.iter().map(|r| r.overall_metrics.avg_test_duration_ms).collect();

        let success_rate_volatility = self.calculate_coefficient_of_variation(&success_rates);
        let failure_rate_volatility = self.calculate_coefficient_of_variation(&failure_rates);
        let performance_volatility = self.calculate_coefficient_of_variation(&durations);

        let overall_volatility_score = (success_rate_volatility + failure_rate_volatility + performance_volatility) / 3.0;
        let volatility_trend = self.determine_volatility_trend(&recent_runs);

        VolatilityMetrics {
            success_rate_volatility,
            failure_rate_volatility,
            performance_volatility,
            overall_volatility_score,
            volatility_trend,
        }
    }

    fn calculate_coefficient_of_variation(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        if mean > 0.0 {
            (std_dev / mean) * 100.0
        } else {
            0.0
        }
    }

    fn determine_volatility_trend(&self, runs: &[&TestRunRecord]) -> ReliabilityTrend {
        if runs.len() < 5 {
            return ReliabilityTrend::InsufficientData;
        }

        let first_half = &runs[..runs.len()/2];
        let second_half = &runs[runs.len()/2..];

        let first_volatility = self.calculate_window_volatility(first_half);
        let second_volatility = self.calculate_window_volatility(second_half);

        let change = ((second_volatility - first_volatility) / first_volatility) * 100.0;

        match change {
            x if x > 20.0 => ReliabilityTrend::Declining,
            x if x > 5.0 => ReliabilityTrend::Declining,
            x if x < -20.0 => ReliabilityTrend::Improving,
            x if x < -5.0 => ReliabilityTrend::Improving,
            _ => ReliabilityTrend::Stable,
        }
    }

    fn calculate_window_volatility(&self, runs: &[&TestRunRecord]) -> f64 {
        let success_rates: Vec<f64> = runs.iter().map(|r| r.overall_metrics.success_rate).collect();
        self.calculate_coefficient_of_variation(&success_rates)
    }

    fn calculate_confidence_intervals(&self) -> HashMap<String, (f64, f64)> {
        let mut intervals = HashMap::new();
        let recent_runs = self.get_recent_runs(30);

        if !recent_runs.is_empty() {
            let success_rates: Vec<f64> = recent_runs.iter().map(|r| r.overall_metrics.success_rate).collect();
            intervals.insert("success_rate".to_string(), self.calculate_confidence_interval(&success_rates));

            let failure_rates: Vec<f64> = recent_runs.iter().map(|r| r.overall_metrics.failure_rate).collect();
            intervals.insert("failure_rate".to_string(), self.calculate_confidence_interval(&failure_rates));

            let flakiness_rates: Vec<f64> = recent_runs.iter().map(|r| r.overall_metrics.flakiness_rate).collect();
            intervals.insert("flakiness_rate".to_string(), self.calculate_confidence_interval(&flakiness_rates));
        }

        intervals
    }

    fn calculate_confidence_interval(&self, values: &[f64]) -> (f64, f64) {
        if values.len() < 2 {
            return (0.0, 0.0);
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
        let std_dev = variance.sqrt();
        let std_error = std_dev / (values.len() as f64).sqrt();

        // 95% confidence interval (assuming normal distribution)
        let margin_of_error = 1.96 * std_error;

        (mean - margin_of_error, mean + margin_of_error)
    }

    fn generate_flaky_test_report(&self) -> Result<FlakyTestReport, Box<dyn std::error::Error>> {
        let total_flaky_tests = self.flaky_test_tracker.len() as u32;

        let unique_tests = self.count_unique_tests(&self.get_recent_runs(30));
        let flaky_test_percentage = if unique_tests > 0 {
            (total_flaky_tests as f64 / unique_tests as f64) * 100.0
        } else {
            0.0
        };

        let mut flaky_tests_by_severity: HashMap<FlakySeverity, Vec<FlakyTestData>> = HashMap::new();

        for flaky_test in self.flaky_test_tracker.values() {
            flaky_tests_by_severity
                .entry(flaky_test.severity.clone())
                .or_insert_with(Vec::new)
                .push(flaky_test.clone());
        }

        let most_problematic_tests = self.identify_most_problematic_tests(5);
        let flaky_test_trends = self.analyze_flaky_test_trends()?;
        let impact_analysis = self.calculate_flaky_test_impact();

        Ok(FlakyTestReport {
            total_flaky_tests,
            flaky_test_percentage,
            flaky_tests_by_severity,
            most_problematic_tests,
            flaky_test_trends,
            impact_analysis,
        })
    }

    fn identify_most_problematic_tests(&self, limit: usize) -> Vec<FlakyTestData> {
        let mut tests: Vec<FlakyTestData> = self.flaky_test_tracker.values().cloned().collect();

        // Sort by impact score (combination of failure rate and frequency)
        tests.sort_by(|a, b| {
            let a_impact = a.failure_rate * a.total_runs as f64;
            let b_impact = b.failure_rate * b.total_runs as f64;
            b_impact.partial_cmp(&a_impact).unwrap_or(std::cmp::Ordering::Equal)
        });

        tests.into_iter().take(limit).collect()
    }

    fn analyze_flaky_test_trends(&self) -> Result<FlakyTestTrends, Box<dyn std::error::Error>> {
        let cutoff_7d = Utc::now() - Duration::days(7);
        let cutoff_30d = Utc::now() - Duration::days(30);

        // Identify new flaky tests (first seen in last 7 days)
        let new_flaky_tests: Vec<FlakyTestData> = self.flaky_test_tracker
            .values()
            .filter(|test| test.first_seen >= cutoff_7d)
            .cloned()
            .collect();

        // For resolved flaky tests, we'd need historical data about previously flaky tests
        // For now, return empty as we don't track resolved tests in this implementation
        let resolved_flaky_tests = Vec::new();

        // Identify worsening tests (increased failure rate in recent runs)
        let worsening_tests = self.identify_worsening_flaky_tests();

        // Identify improving tests (decreased failure rate in recent runs)
        let improving_tests = self.identify_improving_flaky_tests();

        let trend_direction = if !new_flaky_tests.is_empty() || !worsening_tests.is_empty() {
            ReliabilityTrend::Declining
        } else if !improving_tests.is_empty() {
            ReliabilityTrend::Improving
        } else {
            ReliabilityTrend::Stable
        };

        Ok(FlakyTestTrends {
            new_flaky_tests,
            resolved_flaky_tests,
            worsening_tests,
            improving_tests,
            trend_direction,
        })
    }

    fn identify_worsening_flaky_tests(&self) -> Vec<FlakyTestData> {
        let cutoff = Utc::now() - Duration::days(7);
        let mut worsening = Vec::new();

        for flaky_test in self.flaky_test_tracker.values() {
            // Calculate recent failure rate
            let recent_runs = self.test_history
                .iter()
                .filter(|record| record.timestamp >= cutoff)
                .filter_map(|record| record.test_results.get(&flaky_test.test_name))
                .collect::<Vec<_>>();

            if recent_runs.len() >= 5 {
                let recent_failures = recent_runs
                    .iter()
                    .filter(|result| matches!(result.status, TestStatus::Failed | TestStatus::Error | TestStatus::Flaky))
                    .count();

                let recent_failure_rate = (recent_failures as f64 / recent_runs.len() as f64) * 100.0;

                if recent_failure_rate > flaky_test.failure_rate * 1.5 {
                    worsening.push(flaky_test.clone());
                }
            }
        }

        worsening
    }

    fn identify_improving_flaky_tests(&self) -> Vec<FlakyTestData> {
        let cutoff = Utc::now() - Duration::days(7);
        let mut improving = Vec::new();

        for flaky_test in self.flaky_test_tracker.values() {
            // Calculate recent failure rate
            let recent_runs = self.test_history
                .iter()
                .filter(|record| record.timestamp >= cutoff)
                .filter_map(|record| record.test_results.get(&flaky_test.test_name))
                .collect::<Vec<_>>();

            if recent_runs.len() >= 5 {
                let recent_failures = recent_runs
                    .iter()
                    .filter(|result| matches!(result.status, TestStatus::Failed | TestStatus::Error | TestStatus::Flaky))
                    .count();

                let recent_failure_rate = (recent_failures as f64 / recent_runs.len() as f64) * 100.0;

                if recent_failure_rate < flaky_test.failure_rate * 0.5 {
                    improving.push(flaky_test.clone());
                }
            }
        }

        improving
    }

    fn calculate_flaky_test_impact(&self) -> FlakyTestImpact {
        let total_flaky_tests = self.flaky_test_tracker.len();

        // Estimate development time cost (assuming 30 minutes per flaky test investigation)
        let development_time_cost_hours = (total_flaky_tests as f64) * 0.5;

        // Estimate CI resource waste (flaky tests often require reruns)
        let total_test_runs: u32 = self.test_history.iter().map(|r| r.overall_metrics.total_tests).sum();
        let flaky_test_runs: u32 = self.test_history.iter().map(|r| r.overall_metrics.flaky_tests).sum();

        let ci_resource_waste_percent = if total_test_runs > 0 {
            (flaky_test_runs as f64 / total_test_runs as f64) * 100.0 * 2.0 // Assume reruns double the cost
        } else {
            0.0
        };

        let developer_confidence_impact = match total_flaky_tests {
            0 => ConfidenceImpact::Minimal,
            1..=5 => ConfidenceImpact::Low,
            6..=15 => ConfidenceImpact::Moderate,
            16..=30 => ConfidenceImpact::High,
            _ => ConfidenceImpact::Severe,
        };

        let release_reliability_risk = if flaky_test_runs > 0 {
            let flaky_percentage = (flaky_test_runs as f64 / total_test_runs as f64) * 100.0;
            match flaky_percentage {
                x if x >= 10.0 => RiskLevel::Critical,
                x if x >= 5.0 => RiskLevel::High,
                x if x >= 2.0 => RiskLevel::Medium,
                _ => RiskLevel::Low,
            }
        } else {
            RiskLevel::Low
        };

        FlakyTestImpact {
            development_time_cost_hours,
            ci_resource_waste_percent,
            developer_confidence_impact,
            release_reliability_risk,
        }
    }

    fn analyze_reliability_trends(&self) -> Result<ReliabilityTrendAnalysis, Box<dyn std::error::Error>> {
        let short_term_trend = self.analyze_short_term_trend()?;
        let long_term_trend = self.analyze_long_term_trend()?;
        let seasonal_patterns = self.identify_seasonal_patterns()?;
        let regression_points = self.identify_regression_points()?;
        let improvement_points = self.identify_improvement_points()?;
        let predictive_analysis = self.generate_predictive_analysis()?;

        Ok(ReliabilityTrendAnalysis {
            short_term_trend,
            long_term_trend,
            seasonal_patterns,
            regression_points,
            improvement_points,
            predictive_analysis,
        })
    }

    fn analyze_short_term_trend(&self) -> Result<TrendAnalysisResult, Box<dyn std::error::Error>> {
        let recent_runs = self.get_recent_runs(7);
        if recent_runs.len() < 3 {
            return Ok(TrendAnalysisResult {
                direction: ReliabilityTrend::InsufficientData,
                magnitude: 0.0,
                confidence: 0.0,
                time_period: "7 days".to_string(),
                key_factors: vec!["Insufficient data".to_string()],
            });
        }

        let success_rates: Vec<f64> = recent_runs.iter().map(|r| r.overall_metrics.success_rate).collect();
        let trend_slope = self.calculate_trend_slope(&success_rates);
        let direction = self.determine_reliability_trend_direction(trend_slope);
        let confidence = self.calculate_trend_confidence(&recent_runs);

        let key_factors = self.identify_trend_factors(&recent_runs, &direction);

        Ok(TrendAnalysisResult {
            direction,
            magnitude: trend_slope.abs(),
            confidence,
            time_period: "7 days".to_string(),
            key_factors,
        })
    }

    fn analyze_long_term_trend(&self) -> Result<TrendAnalysisResult, Box<dyn std::error::Error>> {
        let long_term_runs = self.get_recent_runs(30);
        if long_term_runs.len() < 5 {
            return Ok(TrendAnalysisResult {
                direction: ReliabilityTrend::InsufficientData,
                magnitude: 0.0,
                confidence: 0.0,
                time_period: "30 days".to_string(),
                key_factors: vec!["Insufficient data".to_string()],
            });
        }

        let success_rates: Vec<f64> = long_term_runs.iter().map(|r| r.overall_metrics.success_rate).collect();
        let trend_slope = self.calculate_trend_slope(&success_rates);
        let direction = self.determine_reliability_trend_direction(trend_slope);
        let confidence = self.calculate_trend_confidence(&long_term_runs);

        let key_factors = self.identify_trend_factors(&long_term_runs, &direction);

        Ok(TrendAnalysisResult {
            direction,
            magnitude: trend_slope.abs(),
            confidence,
            time_period: "30 days".to_string(),
            key_factors,
        })
    }

    fn calculate_trend_slope(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let n = values.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = values.iter().sum::<f64>() / n;

        let numerator: f64 = values
            .iter()
            .enumerate()
            .map(|(i, &y)| (i as f64 - x_mean) * (y - y_mean))
            .sum();

        let denominator: f64 = (0..values.len())
            .map(|i| (i as f64 - x_mean).powi(2))
            .sum();

        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    fn determine_reliability_trend_direction(&self, slope: f64) -> ReliabilityTrend {
        match slope {
            x if x > 2.0 => ReliabilityTrend::Improving,
            x if x > 0.5 => ReliabilityTrend::Improving,
            x if x < -2.0 => ReliabilityTrend::Declining,
            x if x < -0.5 => ReliabilityTrend::Declining,
            _ => ReliabilityTrend::Stable,
        }
    }

    fn calculate_trend_confidence(&self, runs: &[&TestRunRecord]) -> f64 {
        let data_points_factor = (runs.len() as f64 / 14.0).min(1.0);
        let consistency_factor = self.calculate_consistency_score(runs) / 100.0;

        (data_points_factor * consistency_factor * 100.0).min(95.0)
    }

    fn identify_trend_factors(&self, runs: &[&TestRunRecord], trend: &ReliabilityTrend) -> Vec<String> {
        let mut factors = Vec::new();

        match trend {
            ReliabilityTrend::Improving => {
                factors.push("Reduced flaky test failures".to_string());
                factors.push("Improved test environment stability".to_string());
                factors.push("Better error handling implementation".to_string());
            },
            ReliabilityTrend::Declining => {
                factors.push("Increased flaky test occurrences".to_string());
                factors.push("New code changes introducing instability".to_string());
                factors.push("Environment or infrastructure issues".to_string());
            },
            ReliabilityTrend::Stable => {
                factors.push("Consistent test execution patterns".to_string());
                factors.push("Stable codebase without major changes".to_string());
            },
            ReliabilityTrend::Volatile => {
                factors.push("Intermittent environment issues".to_string());
                factors.push("Resource contention problems".to_string());
                factors.push("External dependency instability".to_string());
            },
            _ => {
                factors.push("Insufficient data for analysis".to_string());
            }
        }

        factors
    }

    fn identify_seasonal_patterns(&self) -> Result<Vec<SeasonalPattern>, Box<dyn std::error::Error>> {
        let mut patterns = Vec::new();

        // Analyze day-of-week patterns
        if let Some(day_pattern) = self.analyze_day_of_week_pattern() {
            patterns.push(day_pattern);
        }

        // Analyze time-of-day patterns
        if let Some(time_pattern) = self.analyze_time_of_day_pattern() {
            patterns.push(time_pattern);
        }

        Ok(patterns)
    }

    fn analyze_day_of_week_pattern(&self) -> Option<SeasonalPattern> {
        let mut day_stats: HashMap<u32, (f64, u32)> = HashMap::new();

        for run in &self.test_history {
            let weekday = run.timestamp.weekday().num_days_from_monday();
            let entry = day_stats.entry(weekday).or_insert((0.0, 0));
            entry.0 += run.overall_metrics.success_rate;
            entry.1 += 1;
        }

        if day_stats.len() >= 3 {
            let avg_rates: Vec<f64> = day_stats.values().map(|(total, count)| total / *count as f64).collect();
            let volatility = self.calculate_coefficient_of_variation(&avg_rates);

            if volatility > 10.0 {
                return Some(SeasonalPattern {
                    pattern_type: PatternType::DayOfWeek,
                    frequency: "weekly".to_string(),
                    impact_magnitude: volatility,
                    affected_metrics: vec!["success_rate".to_string()],
                    description: "Success rate varies by day of week".to_string(),
                });
            }
        }

        None
    }

    fn analyze_time_of_day_pattern(&self) -> Option<SeasonalPattern> {
        let mut hour_stats: HashMap<u32, (f64, u32)> = HashMap::new();

        for run in &self.test_history {
            let hour = run.timestamp.hour();
            let entry = hour_stats.entry(hour).or_insert((0.0, 0));
            entry.0 += run.overall_metrics.success_rate;
            entry.1 += 1;
        }

        if hour_stats.len() >= 6 {
            let avg_rates: Vec<f64> = hour_stats.values().map(|(total, count)| total / *count as f64).collect();
            let volatility = self.calculate_coefficient_of_variation(&avg_rates);

            if volatility > 15.0 {
                return Some(SeasonalPattern {
                    pattern_type: PatternType::TimeOfDay,
                    frequency: "daily".to_string(),
                    impact_magnitude: volatility,
                    affected_metrics: vec!["success_rate".to_string(), "response_time".to_string()],
                    description: "Performance varies by time of day".to_string(),
                });
            }
        }

        None
    }

    fn identify_regression_points(&self) -> Result<Vec<RegressionPoint>, Box<dyn std::error::Error>> {
        let mut regression_points = Vec::new();

        for window in self.test_history.windows(2) {
            let previous = &window[0];
            let current = &window[1];

            let success_rate_drop = previous.overall_metrics.success_rate - current.overall_metrics.success_rate;

            if success_rate_drop > self.config.alert_thresholds.success_rate_decline * 100.0 {
                regression_points.push(RegressionPoint {
                    timestamp: current.timestamp,
                    affected_metric: "success_rate".to_string(),
                    magnitude: success_rate_drop,
                    likely_cause: "Code changes or environment issues".to_string(),
                    duration_hours: None, // Would need additional analysis
                    resolution_actions: vec![
                        "Review recent code changes".to_string(),
                        "Check environment stability".to_string(),
                        "Analyze failing test patterns".to_string(),
                    ],
                });
            }
        }

        Ok(regression_points)
    }

    fn identify_improvement_points(&self) -> Result<Vec<ImprovementPoint>, Box<dyn std::error::Error>> {
        let mut improvement_points = Vec::new();

        for window in self.test_history.windows(2) {
            let previous = &window[0];
            let current = &window[1];

            let success_rate_improvement = current.overall_metrics.success_rate - previous.overall_metrics.success_rate;

            if success_rate_improvement > 5.0 {
                improvement_points.push(ImprovementPoint {
                    timestamp: current.timestamp,
                    improved_metric: "success_rate".to_string(),
                    magnitude: success_rate_improvement,
                    contributing_factors: vec![
                        "Test fixes implemented".to_string(),
                        "Infrastructure improvements".to_string(),
                        "Code quality enhancements".to_string(),
                    ],
                    sustainability: if success_rate_improvement > 20.0 {
                        SustainabilityAssessment::ModeratelySustainable
                    } else {
                        SustainabilityAssessment::Sustainable
                    },
                });
            }
        }

        Ok(improvement_points)
    }

    fn generate_predictive_analysis(&self) -> Result<PredictiveReliabilityAnalysis, Box<dyn std::error::Error>> {
        let recent_runs = self.get_recent_runs(14);

        if recent_runs.len() < 5 {
            return Ok(PredictiveReliabilityAnalysis {
                predicted_success_rate_7d: 0.0,
                predicted_success_rate_30d: 0.0,
                predicted_flaky_test_count: 0,
                reliability_forecast: Vec::new(),
                risk_factors: Vec::new(),
                confidence_intervals: HashMap::new(),
            });
        }

        let success_rates: Vec<f64> = recent_runs.iter().map(|r| r.overall_metrics.success_rate).collect();
        let trend_slope = self.calculate_trend_slope(&success_rates);
        let current_success_rate = success_rates.last().unwrap_or(&90.0);

        let predicted_success_rate_7d = current_success_rate + (trend_slope * 7.0);
        let predicted_success_rate_30d = current_success_rate + (trend_slope * 30.0);

        let current_flaky_tests = self.flaky_test_tracker.len() as u32;
        let predicted_flaky_test_count = if trend_slope < -1.0 {
            current_flaky_tests + 2
        } else if trend_slope > 1.0 {
            current_flaky_tests.saturating_sub(1)
        } else {
            current_flaky_tests
        };

        let reliability_forecast = vec![
            ReliabilityForecast {
                metric_name: "success_rate".to_string(),
                forecast_horizon_days: 7,
                predicted_value: predicted_success_rate_7d,
                confidence_interval: (predicted_success_rate_7d - 5.0, predicted_success_rate_7d + 5.0),
                forecast_accuracy: 85.0,
                assumptions: vec![
                    "Current trends continue".to_string(),
                    "No major code changes".to_string(),
                    "Stable infrastructure".to_string(),
                ],
            },
        ];

        let risk_factors = vec![
            RiskFactor {
                factor_name: "Increasing flaky tests".to_string(),
                impact_severity: if current_flaky_tests > 10 { RiskLevel::High } else { RiskLevel::Medium },
                probability: 0.3,
                description: "Growing number of unstable tests".to_string(),
                mitigation_strategies: vec![
                    "Implement flaky test quarantine".to_string(),
                    "Improve test environment stability".to_string(),
                    "Add retry mechanisms for known flaky tests".to_string(),
                ],
            },
        ];

        let mut confidence_intervals = HashMap::new();
        confidence_intervals.insert("success_rate_7d".to_string(), (predicted_success_rate_7d - 5.0, predicted_success_rate_7d + 5.0));
        confidence_intervals.insert("success_rate_30d".to_string(), (predicted_success_rate_30d - 10.0, predicted_success_rate_30d + 10.0));

        Ok(PredictiveReliabilityAnalysis {
            predicted_success_rate_7d,
            predicted_success_rate_30d,
            predicted_flaky_test_count,
            reliability_forecast,
            risk_factors,
            confidence_intervals,
        })
    }

    fn compare_environment_reliability(&self) -> Result<EnvironmentReliabilityComparison, Box<dyn std::error::Error>> {
        let mut environment_scores = HashMap::new();
        let mut environment_stats: HashMap<String, (f64, f64, f64, u32, Vec<String>)> = HashMap::new(); // (success_rate, stability, flakiness, unique_failures, failure_list)

        // Collect statistics per environment
        for run in &self.test_history {
            let env_name = &run.environment_info.environment_name;
            let entry = environment_stats.entry(env_name.clone()).or_insert((0.0, 0.0, 0.0, 0, Vec::new()));

            entry.0 += run.overall_metrics.success_rate;
            entry.2 += run.overall_metrics.flakiness_rate;

            // Count unique failures
            for (test_name, test_result) in &run.test_results {
                if matches!(test_result.status, TestStatus::Failed | TestStatus::Error) {
                    if let Some(reason) = &test_result.failure_reason {
                        if !entry.4.contains(reason) {
                            entry.4.push(reason.clone());
                            entry.3 += 1;
                        }
                    }
                }
            }
        }

        // Calculate environment scores
        let mut env_count_per_env: HashMap<String, u32> = HashMap::new();
        for run in &self.test_history {
            *env_count_per_env.entry(run.environment_info.environment_name.clone()).or_insert(0) += 1;
        }

        let mut ranked_environments = Vec::new();
        for (env_name, (total_success_rate, _, total_flakiness_rate, unique_failures, _)) in &environment_stats {
            let run_count = env_count_per_env.get(env_name).unwrap_or(&1);
            let success_rate = total_success_rate / *run_count as f64;
            let flakiness_rate = total_flakiness_rate / *run_count as f64;
            let stability_score = 100.0 - flakiness_rate;

            let score = EnvironmentReliabilityScore {
                environment_name: env_name.clone(),
                success_rate,
                stability_score,
                flakiness_rate,
                unique_failures: *unique_failures,
                reliability_rank: 0, // Will be filled after sorting
                relative_performance: success_rate, // Simplified
            };

            ranked_environments.push((env_name.clone(), score));
        }

        // Sort and rank environments
        ranked_environments.sort_by(|a, b| b.1.success_rate.partial_cmp(&a.1.success_rate).unwrap_or(std::cmp::Ordering::Equal));

        for (rank, (_, score)) in ranked_environments.iter_mut().enumerate() {
            score.reliability_rank = (rank + 1) as u32;
            environment_scores.insert(score.environment_name.clone(), score.clone());
        }

        // Cross-environment analysis
        let consistency_score = self.calculate_cross_environment_consistency(&environment_scores);

        let mut environment_specific_failures = HashMap::new();
        for (env_name, (_, _, _, _, failures)) in environment_stats {
            environment_specific_failures.insert(env_name, failures);
        }

        let portable_test_percentage = self.calculate_portable_test_percentage(&environment_specific_failures);

        let most_stable_environment = ranked_environments.first().map(|(name, _)| name.clone()).unwrap_or_default();
        let most_problematic_environment = ranked_environments.last().map(|(name, _)| name.clone()).unwrap_or_default();

        let cross_environment_analysis = CrossEnvironmentAnalysis {
            consistency_score,
            environment_specific_failures,
            portable_test_percentage,
            most_stable_environment,
            most_problematic_environment,
        };

        let environment_specific_issues = self.identify_environment_specific_issues();
        let portability_score = portable_test_percentage;

        Ok(EnvironmentReliabilityComparison {
            environment_scores,
            cross_environment_analysis,
            environment_specific_issues,
            portability_score,
        })
    }

    fn calculate_cross_environment_consistency(&self, environment_scores: &HashMap<String, EnvironmentReliabilityScore>) -> f64 {
        if environment_scores.len() < 2 {
            return 100.0;
        }

        let success_rates: Vec<f64> = environment_scores.values().map(|score| score.success_rate).collect();
        let variance = self.calculate_coefficient_of_variation(&success_rates);

        (100.0 - variance).max(0.0)
    }

    fn calculate_portable_test_percentage(&self, environment_failures: &HashMap<String, Vec<String>>) -> f64 {
        if environment_failures.len() < 2 {
            return 100.0;
        }

        let all_failures: std::collections::HashSet<String> = environment_failures
            .values()
            .flatten()
            .cloned()
            .collect();

        let shared_failures: Vec<String> = all_failures
            .into_iter()
            .filter(|failure| {
                environment_failures.values().filter(|failures| failures.contains(failure)).count() > 1
            })
            .collect();

        let total_unique_tests = self.count_unique_tests(&self.get_recent_runs(30)) as f64;
        let portable_tests = total_unique_tests - shared_failures.len() as f64;

        if total_unique_tests > 0.0 {
            (portable_tests / total_unique_tests) * 100.0
        } else {
            100.0
        }
    }

    fn identify_environment_specific_issues(&self) -> HashMap<String, Vec<String>> {
        let mut issues = HashMap::new();

        for run in &self.test_history {
            let env_name = &run.environment_info.environment_name;
            let env_issues = issues.entry(env_name.clone()).or_insert_with(Vec::new);

            // Identify common issues per environment
            if run.overall_metrics.failure_rate > 10.0 {
                env_issues.push("High failure rate detected".to_string());
            }

            if run.overall_metrics.flakiness_rate > 5.0 {
                env_issues.push("Flaky test issues".to_string());
            }

            // Check for specific network issues
            if let Some(latency) = run.environment_info.network_conditions.latency_ms {
                if latency > 500.0 {
                    env_issues.push("High network latency".to_string());
                }
            }

            if let Some(packet_loss) = run.environment_info.network_conditions.packet_loss_percent {
                if packet_loss > 1.0 {
                    env_issues.push("Network packet loss detected".to_string());
                }
            }

            // Remove duplicates
            env_issues.sort();
            env_issues.dedup();
        }

        issues
    }

    fn analyze_failures(&self) -> Result<FailureAnalysis, Box<dyn std::error::Error>> {
        let failure_categories = self.categorize_failures();
        let root_cause_analysis = self.perform_root_cause_analysis();
        let failure_patterns = self.identify_failure_patterns();
        let recovery_analysis = self.analyze_recovery_patterns();

        Ok(FailureAnalysis {
            failure_categories,
            root_cause_analysis,
            failure_patterns,
            recovery_analysis,
        })
    }

    fn categorize_failures(&self) -> HashMap<String, FailureCategoryAnalysis> {
        let mut categories: HashMap<String, (u32, Vec<f64>)> = HashMap::new(); // (count, impact_scores)

        for run in &self.test_history {
            for test_result in run.test_results.values() {
                if let (TestStatus::Failed | TestStatus::Error | TestStatus::Timeout, Some(reason)) = (&test_result.status, &test_result.failure_reason) {
                    let category = self.classify_failure_reason(reason);
                    let entry = categories.entry(category).or_insert((0, Vec::new()));
                    entry.0 += 1;
                    entry.1.push(1.0); // Simplified impact score
                }
            }
        }

        let total_failures: u32 = categories.values().map(|(count, _)| count).sum();

        categories
            .into_iter()
            .map(|(category, (count, impact_scores))| {
                let failure_percentage = if total_failures > 0 {
                    (count as f64 / total_failures as f64) * 100.0
                } else {
                    0.0
                };

                let average_impact = if !impact_scores.is_empty() {
                    impact_scores.iter().sum::<f64>() / impact_scores.len() as f64
                } else {
                    0.0
                };

                let analysis = FailureCategoryAnalysis {
                    category_name: category.clone(),
                    failure_count: count,
                    failure_percentage,
                    average_impact,
                    trend: ReliabilityTrend::Stable, // Simplified
                    common_patterns: vec!["Pattern analysis would require more detailed implementation".to_string()],
                };

                (category, analysis)
            })
            .collect()
    }

    fn classify_failure_reason(&self, reason: &str) -> String {
        let reason_lower = reason.to_lowercase();

        if reason_lower.contains("timeout") || reason_lower.contains("time") {
            "Timeout".to_string()
        } else if reason_lower.contains("network") || reason_lower.contains("connection") {
            "Network".to_string()
        } else if reason_lower.contains("assertion") || reason_lower.contains("expect") {
            "Assertion".to_string()
        } else if reason_lower.contains("null") || reason_lower.contains("undefined") {
            "NullReference".to_string()
        } else if reason_lower.contains("memory") || reason_lower.contains("out of") {
            "Resource".to_string()
        } else {
            "Other".to_string()
        }
    }

    fn perform_root_cause_analysis(&self) -> Vec<RootCauseAnalysis> {
        let mut failure_signatures: HashMap<String, (String, u32, Vec<String>)> = HashMap::new(); // (root_cause, frequency, affected_tests)

        for run in &self.test_history {
            for (test_name, test_result) in &run.test_results {
                if let (TestStatus::Failed | TestStatus::Error, Some(reason)) = (&test_result.status, &test_result.failure_reason) {
                    let signature = self.generate_failure_signature(reason);
                    let entry = failure_signatures.entry(signature.clone()).or_insert((reason.clone(), 0, Vec::new()));
                    entry.1 += 1;
                    if !entry.2.contains(test_name) {
                        entry.2.push(test_name.clone());
                    }
                }
            }
        }

        failure_signatures
            .into_iter()
            .map(|(signature, (root_cause, frequency, affected_tests))| {
                RootCauseAnalysis {
                    failure_signature: signature,
                    root_cause: root_cause.clone(),
                    frequency,
                    impact_score: (frequency as f64 * affected_tests.len() as f64).min(10.0),
                    affected_tests,
                    resolution_steps: self.generate_resolution_steps(&root_cause),
                    prevention_measures: self.generate_prevention_measures(&root_cause),
                }
            })
            .collect()
    }

    fn generate_failure_signature(&self, reason: &str) -> String {
        // Simplified signature generation
        let words: Vec<&str> = reason.split_whitespace().collect();
        if words.len() > 3 {
            words[..3].join(" ")
        } else {
            reason.to_string()
        }
    }

    fn generate_resolution_steps(&self, root_cause: &str) -> Vec<String> {
        let cause_lower = root_cause.to_lowercase();

        if cause_lower.contains("timeout") {
            vec![
                "Increase timeout values".to_string(),
                "Optimize slow operations".to_string(),
                "Add wait conditions".to_string(),
            ]
        } else if cause_lower.contains("network") {
            vec![
                "Check network connectivity".to_string(),
                "Add retry mechanisms".to_string(),
                "Implement circuit breakers".to_string(),
            ]
        } else {
            vec![
                "Analyze error logs".to_string(),
                "Reproduce issue locally".to_string(),
                "Apply targeted fix".to_string(),
            ]
        }
    }

    fn generate_prevention_measures(&self, root_cause: &str) -> Vec<String> {
        let cause_lower = root_cause.to_lowercase();

        if cause_lower.contains("timeout") {
            vec![
                "Implement timeout monitoring".to_string(),
                "Add performance tests".to_string(),
                "Set up alerting for slow operations".to_string(),
            ]
        } else if cause_lower.contains("network") {
            vec![
                "Implement network monitoring".to_string(),
                "Add connection health checks".to_string(),
                "Use mock services for testing".to_string(),
            ]
        } else {
            vec![
                "Improve error handling".to_string(),
                "Add comprehensive logging".to_string(),
                "Implement defensive programming practices".to_string(),
            ]
        }
    }

    fn identify_failure_patterns(&self) -> Vec<FailurePattern> {
        // This would be more complex in a real implementation
        vec![
            FailurePattern {
                pattern_type: FailurePatternType::Timing,
                frequency: 0.3,
                conditions: vec!["High load conditions".to_string()],
                example_failures: vec!["Connection timeout".to_string()],
            },
        ]
    }

    fn analyze_recovery_patterns(&self) -> RecoveryAnalysis {
        // Simplified recovery analysis
        RecoveryAnalysis {
            average_recovery_time_hours: 2.5,
            recovery_success_rate: 85.0,
            recovery_patterns: vec![
                "Automatic retry succeeds".to_string(),
                "Environment restart resolves issue".to_string(),
                "Manual intervention required".to_string(),
            ],
            automated_recovery_percentage: 60.0,
        }
    }

    fn generate_reliability_recommendations(&self, summary: &ReliabilitySummary, flaky_report: &FlakyTestReport, stability: &StabilityAnalysis) -> Result<Vec<ReliabilityRecommendation>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();

        // Flaky test recommendations
        if flaky_report.total_flaky_tests > 0 {
            recommendations.push(ReliabilityRecommendation {
                priority: if flaky_report.flaky_test_percentage > 5.0 {
                    RecommendationPriority::Critical
                } else {
                    RecommendationPriority::High
                },
                category: ReliabilityRecommendationCategory::FlakyTestResolution,
                title: "Address Flaky Test Issues".to_string(),
                description: format!("Resolve {} flaky tests affecting {:.1}% of test suite",
                    flaky_report.total_flaky_tests, flaky_report.flaky_test_percentage),
                expected_improvement: flaky_report.flaky_test_percentage * 0.8,
                implementation_effort: if flaky_report.total_flaky_tests > 20 {
                    ImplementationEffort::High
                } else {
                    ImplementationEffort::Medium
                },
                cost_benefit_ratio: flaky_report.flaky_test_percentage / match flaky_report.total_flaky_tests {
                    1..=5 => 1.0,
                    6..=15 => 3.0,
                    _ => 7.0,
                },
                implementation_steps: vec![
                    "Identify root causes of flaky failures".to_string(),
                    "Implement proper wait conditions".to_string(),
                    "Add test isolation improvements".to_string(),
                    "Consider quarantining persistently flaky tests".to_string(),
                ],
                success_metrics: vec![
                    "Reduce flaky test count by 80%".to_string(),
                    "Improve overall success rate by 5%".to_string(),
                    "Decrease CI execution variance".to_string(),
                ],
                timeline_estimate: if flaky_report.total_flaky_tests > 20 { "4-6 weeks" } else { "2-3 weeks" }.to_string(),
                risk_level: RiskLevel::Low,
            });
        }

        // Stability improvement recommendations
        if summary.stability_index < 80.0 {
            recommendations.push(ReliabilityRecommendation {
                priority: RecommendationPriority::High,
                category: ReliabilityRecommendationCategory::TestStabilization,
                title: "Improve Test Suite Stability".to_string(),
                description: format!("Stability index is {:.1}%, below target of 85%", summary.stability_index),
                expected_improvement: 85.0 - summary.stability_index,
                implementation_effort: ImplementationEffort::Medium,
                cost_benefit_ratio: (85.0 - summary.stability_index) / 3.0,
                implementation_steps: vec![
                    "Analyze sources of test result variability".to_string(),
                    "Implement deterministic test data setup".to_string(),
                    "Improve test environment consistency".to_string(),
                    "Add proper cleanup and teardown procedures".to_string(),
                ],
                success_metrics: vec![
                    "Achieve stability index > 85%".to_string(),
                    "Reduce test result variance by 50%".to_string(),
                    "Improve consistency score to > 90%".to_string(),
                ],
                timeline_estimate: "3-4 weeks".to_string(),
                risk_level: RiskLevel::Medium,
            });
        }

        // Success rate improvement recommendations
        if summary.success_rate < 95.0 {
            recommendations.push(ReliabilityRecommendation {
                priority: RecommendationPriority::Medium,
                category: ReliabilityRecommendationCategory::TestStabilization,
                title: "Improve Overall Success Rate".to_string(),
                description: format!("Success rate is {:.1}%, target is 95%+", summary.success_rate),
                expected_improvement: 95.0 - summary.success_rate,
                implementation_effort: ImplementationEffort::Medium,
                cost_benefit_ratio: (95.0 - summary.success_rate) / 3.0,
                implementation_steps: vec![
                    "Identify and fix consistently failing tests".to_string(),
                    "Improve error handling in test code".to_string(),
                    "Enhance test data management".to_string(),
                    "Optimize test execution environment".to_string(),
                ],
                success_metrics: vec![
                    "Achieve success rate > 95%".to_string(),
                    "Reduce failure rate to < 3%".to_string(),
                    "Maintain consistent performance".to_string(),
                ],
                timeline_estimate: "2-3 weeks".to_string(),
                risk_level: RiskLevel::Low,
            });
        }

        // Monitoring enhancement recommendations
        recommendations.push(ReliabilityRecommendation {
            priority: RecommendationPriority::Medium,
            category: ReliabilityRecommendationCategory::MonitoringEnhancement,
            title: "Enhance Reliability Monitoring".to_string(),
            description: "Implement comprehensive reliability tracking and alerting".to_string(),
            expected_improvement: 5.0,
            implementation_effort: ImplementationEffort::Low,
            cost_benefit_ratio: 5.0,
            implementation_steps: vec![
                "Set up automated reliability reports".to_string(),
                "Implement alerting for regression detection".to_string(),
                "Create reliability dashboards".to_string(),
                "Add trend analysis automation".to_string(),
            ],
            success_metrics: vec![
                "Real-time reliability visibility".to_string(),
                "Automatic regression detection".to_string(),
                "Reduced time to identify issues".to_string(),
            ],
            timeline_estimate: "1-2 weeks".to_string(),
            risk_level: RiskLevel::Low,
        });

        Ok(recommendations)
    }

    fn calculate_quality_metrics(&self, summary: &ReliabilitySummary) -> Result<QualityMetrics, Box<dyn std::error::Error>> {
        let test_suite_health = summary.overall_reliability_score;

        // Estimate impact on development velocity
        let development_velocity_impact = if summary.flakiness_percentage > 5.0 {
            75.0 - (summary.flakiness_percentage * 2.0)
        } else {
            90.0 + summary.success_rate / 10.0
        };

        let confidence_level = summary.consistency_score;
        let maintainability_score = 100.0 - (summary.problematic_tests as f64 * 2.0);

        let technical_debt_factor = (summary.problematic_tests as f64 / summary.unique_tests as f64) * 100.0;

        let overall_quality_grade = match (test_suite_health + development_velocity_impact + confidence_level + maintainability_score) / 4.0 {
            x if x >= 90.0 => QualityGrade::Excellent,
            x if x >= 80.0 => QualityGrade::Good,
            x if x >= 70.0 => QualityGrade::Fair,
            x if x >= 60.0 => QualityGrade::Poor,
            _ => QualityGrade::Critical,
        };

        Ok(QualityMetrics {
            test_suite_health,
            development_velocity_impact,
            confidence_level,
            maintainability_score,
            technical_debt_factor,
            overall_quality_grade,
        })
    }

    pub fn export_reliability_report(&self) -> Result<String, Box<dyn std::error::Error>> {
        let report = self.analyze_reliability()?;
        serde_json::to_string_pretty(&report).map_err(|e| e.into())
    }
}

impl Default for ReliabilityAnalyzer {
    fn default() -> Self {
        Self::with_default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reliability_analyzer_creation() {
        let analyzer = ReliabilityAnalyzer::with_default_config();
        assert_eq!(analyzer.config.flaky_threshold_runs, 10);
        assert_eq!(analyzer.test_history.len(), 0);
        assert_eq!(analyzer.flaky_test_tracker.len(), 0);
    }

    #[test]
    fn test_flaky_severity_determination() {
        let analyzer = ReliabilityAnalyzer::with_default_config();

        assert!(matches!(analyzer.determine_flaky_severity(0.6), FlakySeverity::Critical));
        assert!(matches!(analyzer.determine_flaky_severity(0.3), FlakySeverity::High));
        assert!(matches!(analyzer.determine_flaky_severity(0.15), FlakySeverity::Medium));
        assert!(matches!(analyzer.determine_flaky_severity(0.07), FlakySeverity::Low));
        assert!(matches!(analyzer.determine_flaky_severity(0.03), FlakySeverity::Minimal));
    }

    #[test]
    fn test_coefficient_of_variation_calculation() {
        let analyzer = ReliabilityAnalyzer::with_default_config();
        let values = vec![90.0, 92.0, 88.0, 91.0, 89.0];
        let cv = analyzer.calculate_coefficient_of_variation(&values);
        assert!(cv > 0.0);
        assert!(cv < 10.0); // Should be relatively low for stable values
    }

    #[test]
    fn test_trend_slope_calculation() {
        let analyzer = ReliabilityAnalyzer::with_default_config();
        let improving_values = vec![80.0, 85.0, 90.0, 95.0];
        let declining_values = vec![95.0, 90.0, 85.0, 80.0];

        let improving_slope = analyzer.calculate_trend_slope(&improving_values);
        let declining_slope = analyzer.calculate_trend_slope(&declining_values);

        assert!(improving_slope > 0.0);
        assert!(declining_slope < 0.0);
    }
}