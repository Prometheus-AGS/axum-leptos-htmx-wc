use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

pub mod metrics_engine;
pub mod api;
pub mod dashboard;

/// Core reliability metrics module
pub use metrics_engine::{
    TestReliabilityEngine, ReliabilityConfig, FlakyTestTracker, StabilityAnalyzer,
    FailurePatternDetector, TestStabilityMetrics, FlakinessProbability,
    FailureAnalysisResult, EnvironmentalReliabilityMetrics,
    TemporalReliabilityMetrics, ReliabilityPrediction, TestFailurePattern,
};

pub use api::{
    create_reliability_api_router, ReliabilityApiState, ReliabilityApiConfig,
    ReliabilityAlert, ReliabilityRecommendation,
};

pub use dashboard::{
    ReliabilityDashboard, DashboardGenerator, ReliabilityDashboardOverview,
    create_reliability_dashboard_router,
};

/// Test execution result for reliability analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecutionResult {
    pub test_id: String,
    pub test_name: String,
    pub test_suite: String,
    pub execution_time_ms: f64,
    pub status: TestStatus,
    pub error_message: Option<String>,
    pub environment: String,
    pub executed_at: DateTime<Utc>,
    pub git_commit: Option<String>,
    pub build_number: Option<String>,
    pub runner_id: Option<String>,
    pub retry_count: u32,
    pub was_flaky: bool,
    pub memory_usage_mb: Option<f64>,
    pub cpu_usage_percent: Option<f64>,
    pub parallel_execution: bool,
    pub test_categories: Vec<String>,
}

/// Test execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    TimedOut,
    Cancelled,
    Flaky,  // Passed after retry
}

/// Reliability health score (0-100)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityHealthScore {
    pub overall_score: f64,
    pub stability_score: f64,
    pub consistency_score: f64,
    pub predictability_score: f64,
    pub environmental_score: f64,
    pub temporal_score: f64,
    pub calculated_at: DateTime<Utc>,
    pub score_breakdown: HashMap<String, f64>,
    pub improvement_areas: Vec<String>,
}

/// Reliability trend data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityTrendPoint {
    pub timestamp: DateTime<Utc>,
    pub success_rate: f64,
    pub flaky_rate: f64,
    pub stability_score: f64,
    pub total_tests: usize,
    pub environment: String,
    pub test_category: Option<String>,
}

/// Comprehensive reliability overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityOverview {
    pub health_score: ReliabilityHealthScore,
    pub trend_analysis: Vec<ReliabilityTrendPoint>,
    pub flaky_tests: Vec<FlakyTestSummary>,
    pub failure_patterns: Vec<FailurePatternSummary>,
    pub environmental_analysis: EnvironmentalReliabilityAnalysis,
    pub recommendations: Vec<ReliabilityImprovement>,
    pub analysis_period: DateRange,
    pub total_executions: usize,
    pub generated_at: DateTime<Utc>,
}

/// Summary of a flaky test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakyTestSummary {
    pub test_id: String,
    pub test_name: String,
    pub flakiness_probability: f64,
    pub total_executions: usize,
    pub failure_count: usize,
    pub retry_success_count: usize,
    pub last_failure: Option<DateTime<Utc>>,
    pub consistency_score: f64,
    pub primary_failure_reasons: Vec<String>,
    pub environmental_factors: Vec<String>,
    pub recommended_actions: Vec<String>,
}

/// Summary of a failure pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePatternSummary {
    pub pattern_id: String,
    pub pattern_type: String,
    pub description: String,
    pub affected_tests: usize,
    pub occurrence_count: usize,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub severity: PatternSeverity,
    pub root_cause_hypothesis: Option<String>,
    pub mitigation_suggestions: Vec<String>,
}

/// Pattern severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternSeverity {
    Critical,   // Affects > 50% of tests or causes complete failures
    High,       // Affects 20-50% of tests
    Medium,     // Affects 5-20% of tests
    Low,        // Affects < 5% of tests
}

/// Environmental reliability analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalReliabilityAnalysis {
    pub environment_scores: HashMap<String, f64>,
    pub cross_environment_consistency: f64,
    pub problematic_environments: Vec<String>,
    pub environment_specific_issues: HashMap<String, Vec<String>>,
    pub resource_correlation: HashMap<String, f64>,
}

/// Reliability improvement recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityImprovement {
    pub improvement_type: ImprovementType,
    pub title: String,
    pub description: String,
    pub affected_tests: Vec<String>,
    pub priority: Priority,
    pub estimated_impact: f64,
    pub implementation_effort: EffortLevel,
    pub technical_details: Vec<String>,
    pub success_metrics: Vec<String>,
}

/// Types of reliability improvements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImprovementType {
    FlakeReduction,
    EnvironmentalStabilization,
    TestOptimization,
    InfrastructureImprovement,
    MonitoringEnhancement,
    ProcessImprovement,
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// Implementation effort levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Minimal,   // < 4 hours
    Low,       // 4-16 hours
    Medium,    // 16-40 hours
    High,      // 40-80 hours
    VeryHigh,  // > 80 hours
}

/// Date range for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Reliability alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityAlertConfig {
    pub flakiness_threshold: f64,
    pub stability_threshold: f64,
    pub success_rate_threshold: f64,
    pub consecutive_failures_threshold: u32,
    pub pattern_frequency_threshold: u32,
    pub alert_cooldown_minutes: u32,
    pub notification_channels: Vec<NotificationChannel>,
}

/// Notification channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email { recipients: Vec<String> },
    Slack { webhook_url: String, channel: String },
    Webhook { url: String, headers: HashMap<String, String> },
    PagerDuty { integration_key: String },
}

impl TestStatus {
    /// Check if the test status indicates success
    pub fn is_success(&self) -> bool {
        matches!(self, TestStatus::Passed | TestStatus::Flaky)
    }

    /// Check if the test status indicates failure
    pub fn is_failure(&self) -> bool {
        matches!(self, TestStatus::Failed | TestStatus::TimedOut | TestStatus::Cancelled)
    }

    /// Check if the test was skipped
    pub fn is_skipped(&self) -> bool {
        matches!(self, TestStatus::Skipped)
    }

    /// Check if the test was flaky (passed after retry)
    pub fn is_flaky(&self) -> bool {
        matches!(self, TestStatus::Flaky)
    }
}

impl ReliabilityHealthScore {
    /// Create a new health score with calculated breakdown
    pub fn new(
        overall: f64,
        stability: f64,
        consistency: f64,
        predictability: f64,
        environmental: f64,
        temporal: f64,
    ) -> Self {
        let mut breakdown = HashMap::new();
        breakdown.insert("stability".to_string(), stability);
        breakdown.insert("consistency".to_string(), consistency);
        breakdown.insert("predictability".to_string(), predictability);
        breakdown.insert("environmental".to_string(), environmental);
        breakdown.insert("temporal".to_string(), temporal);

        let improvement_areas = Self::identify_improvement_areas(&breakdown);

        Self {
            overall_score: overall,
            stability_score: stability,
            consistency_score: consistency,
            predictability_score: predictability,
            environmental_score: environmental,
            temporal_score: temporal,
            calculated_at: Utc::now(),
            score_breakdown: breakdown,
            improvement_areas,
        }
    }

    /// Identify areas that need improvement based on scores
    fn identify_improvement_areas(breakdown: &HashMap<String, f64>) -> Vec<String> {
        let mut areas = Vec::new();
        let threshold = 75.0; // Scores below this need improvement

        for (area, score) in breakdown {
            if *score < threshold {
                areas.push(format!("{} (score: {:.1})", area, score));
            }
        }

        areas.sort_by(|a, b| {
            let score_a = a.split("(score: ").nth(1)
                .and_then(|s| s.trim_end_matches(')').parse::<f64>().ok())
                .unwrap_or(0.0);
            let score_b = b.split("(score: ").nth(1)
                .and_then(|s| s.trim_end_matches(')').parse::<f64>().ok())
                .unwrap_or(0.0);
            score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        areas
    }

    /// Get color coding for the overall health score
    pub fn get_health_color(&self) -> &'static str {
        match self.overall_score {
            90.0..=100.0 => "green",
            75.0..90.0 => "yellow",
            50.0..75.0 => "orange",
            _ => "red",
        }
    }

    /// Get descriptive text for the health score
    pub fn get_health_description(&self) -> &'static str {
        match self.overall_score {
            95.0..=100.0 => "Excellent",
            85.0..95.0 => "Very Good",
            75.0..85.0 => "Good",
            60.0..75.0 => "Fair",
            40.0..60.0 => "Poor",
            _ => "Critical",
        }
    }
}

impl DateRange {
    /// Create a new date range
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self { start, end }
    }

    /// Create a date range for the last N days
    pub fn last_days(days: i64) -> Self {
        let end = Utc::now();
        let start = end - Duration::days(days);
        Self { start, end }
    }

    /// Create a date range for the last N hours
    pub fn last_hours(hours: i64) -> Self {
        let end = Utc::now();
        let start = end - Duration::hours(hours);
        Self { start, end }
    }

    /// Check if a timestamp falls within this range
    pub fn contains(&self, timestamp: &DateTime<Utc>) -> bool {
        timestamp >= &self.start && timestamp <= &self.end
    }

    /// Get the duration of this date range
    pub fn duration(&self) -> Duration {
        self.end - self.start
    }
}

impl Default for ReliabilityAlertConfig {
    fn default() -> Self {
        Self {
            flakiness_threshold: 0.3,  // 30% flakiness triggers alert
            stability_threshold: 80.0,  // Stability score below 80 triggers alert
            success_rate_threshold: 90.0,  // Success rate below 90% triggers alert
            consecutive_failures_threshold: 3,
            pattern_frequency_threshold: 5,  // Pattern occurs 5+ times
            alert_cooldown_minutes: 30,
            notification_channels: vec![],
        }
    }
}