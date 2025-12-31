use std::collections::{HashMap, BTreeMap, HashSet};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use crate::testing::entities::TestExecutionResult;

pub mod coverage_trends;
pub mod performance_analysis;
pub mod reliability_metrics;
pub mod api;

/// Advanced analytics engine for test execution data
#[derive(Debug)]
pub struct TestAnalyticsEngine {
    pub coverage_analyzer: coverage_trends::CoverageTrendAnalyzer,
    pub performance_analyzer: performance_analysis::PerformanceAnalyzer,
    pub reliability_analyzer: reliability_metrics::ReliabilityAnalyzer,
    historical_data: Vec<TestExecutionResult>,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub time_window: AnalyticsTimeWindow,
    pub granularity: AnalyticsGranularity,
    pub trend_analysis: TrendAnalysisConfig,
    pub threshold_config: ThresholdConfig,
    pub comparison_config: Option<ComparisonConfig>,
}

/// Time windows for analytics analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsTimeWindow {
    Last24Hours,
    LastWeek,
    LastMonth,
    LastQuarter,
    LastYear,
    Custom { start: DateTime<Utc>, end: DateTime<Utc> },
    Rolling { days: u32 },
}

/// Granularity for data aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsGranularity {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    PerCommit,
    PerBuild,
}

/// Trend analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysisConfig {
    pub detect_seasonality: bool,
    pub smooth_data: bool,
    pub confidence_interval: f64,
    pub minimum_data_points: usize,
    pub regression_type: RegressionType,
}

/// Types of regression analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionType {
    Linear,
    Polynomial { degree: u32 },
    Exponential,
    MovingAverage { window_size: usize },
    ExponentialSmoothing { alpha: f64 },
}

/// Threshold configuration for alerts and notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub coverage_thresholds: CoverageThresholds,
    pub performance_thresholds: PerformanceThresholds,
    pub reliability_thresholds: ReliabilityThresholds,
}

/// Coverage-specific thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageThresholds {
    pub minimum_overall_coverage: f64,
    pub minimum_rust_coverage: f64,
    pub minimum_typescript_coverage: f64,
    pub coverage_drop_alert: f64, // Percentage drop that triggers alert
    pub low_coverage_warning: f64,
    pub critical_coverage_alert: f64,
}

/// Performance-specific thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub max_test_duration_ms: u64,
    pub max_suite_duration_ms: u64,
    pub regression_threshold_percent: f64,
    pub timeout_threshold_ms: u64,
}

/// Reliability-specific thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityThresholds {
    pub minimum_success_rate: f64,
    pub flakiness_threshold: f64, // Max acceptable flakiness percentage
    pub consecutive_failures_alert: usize,
    pub stability_score_threshold: f64,
}

/// Comparison configuration for A/B testing and branch comparisons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonConfig {
    pub comparison_type: ComparisonType,
    pub baseline_criteria: BaselineCriteria,
    pub statistical_tests: Vec<StatisticalTest>,
}

/// Types of comparisons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonType {
    BranchComparison { base_branch: String, feature_branch: String },
    TimeComparison { baseline_start: DateTime<Utc>, baseline_end: DateTime<Utc>, current_start: DateTime<Utc>, current_end: DateTime<Utc> },
    EnvironmentComparison { baseline_env: String, current_env: String },
    TagComparison { baseline_tag: String, current_tag: String },
}

/// Baseline criteria for comparisons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineCriteria {
    pub minimum_samples: usize,
    pub confidence_level: f64,
    pub outlier_detection: bool,
    pub normalization_method: NormalizationMethod,
}

/// Statistical tests to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatisticalTest {
    TTest,
    WelchTest,
    MannWhitneyU,
    KolmogorovSmirnov,
    ChiSquared,
}

/// Data normalization methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NormalizationMethod {
    None,
    ZScore,
    MinMax,
    RobustScaling,
    Quantile,
}

/// Comprehensive analytics result
#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsResult {
    pub analysis_timestamp: DateTime<Utc>,
    pub config_used: AnalyticsConfig,
    pub data_summary: DataSummary,
    pub coverage_analysis: coverage_trends::CoverageAnalysisResult,
    pub performance_analysis: performance_analysis::PerformanceAnalysisResult,
    pub reliability_analysis: reliability_metrics::ReliabilityAnalysisResult,
    pub insights: Vec<AnalyticsInsight>,
    pub recommendations: Vec<AnalyticsRecommendation>,
    pub alerts: Vec<AnalyticsAlert>,
}

/// Comprehensive analysis result combining all domains
#[derive(Debug, Clone, Serialize)]
pub struct ComprehensiveAnalysisResult {
    pub coverage_analysis: coverage_trends::CoverageAnalysisResult,
    pub performance_analysis: performance_analysis::PerformanceAnalysisResult,
    pub reliability_analysis: reliability_metrics::ReliabilityAnalysisResult,
    pub cross_domain_insights: Vec<CrossDomainInsight>,
    pub analysis_timestamp: DateTime<Utc>,
    pub data_points_analyzed: usize,
}

/// Cross-domain insights that combine findings from multiple analysis areas
#[derive(Debug, Clone, Serialize)]
pub struct CrossDomainInsight {
    pub insight_type: CrossDomainInsightType,
    pub severity: InsightSeverity,
    pub title: String,
    pub description: String,
    pub recommendation: String,
    pub affected_components: Vec<String>,
    pub confidence: f64,
}

/// Types of cross-domain insights
#[derive(Debug, Clone, Serialize)]
pub enum CrossDomainInsightType {
    CoveragePerformanceCorrelation,
    ReliabilityCoverageCorrelation,
    PerformanceReliabilityCorrelation,
    QualityTrendCorrelation,
    RiskAssessment,
}

/// Insight severity levels
#[derive(Debug, Clone, Serialize)]
pub enum InsightSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Analytics summary for dashboard display
#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsSummary {
    pub total_data_points: usize,
    pub analysis_coverage: AnalysisCoverage,
    pub last_updated: DateTime<Utc>,
    pub health_score: f64,
}

/// Analysis coverage information
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisCoverage {
    pub rust_files_analyzed: usize,
    pub typescript_files_analyzed: usize,
    pub performance_metrics_tracked: usize,
    pub reliability_tests_monitored: usize,
}

/// Summary of analyzed data
#[derive(Debug, Clone, Serialize)]
pub struct DataSummary {
    pub total_test_results: usize,
    pub date_range: (DateTime<Utc>, DateTime<Utc>),
    pub environments_analyzed: Vec<String>,
    pub test_suites_analyzed: Vec<String>,
    pub data_quality_score: f64,
    pub completeness_percentage: f64,
}

/// Generated insights from analytics
#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsInsight {
    pub insight_type: InsightType,
    pub title: String,
    pub description: String,
    pub confidence: f64,
    pub impact: ImpactLevel,
    pub supporting_data: serde_json::Value,
    pub generated_at: DateTime<Utc>,
}

/// Types of insights
#[derive(Debug, Clone, Serialize)]
pub enum InsightType {
    CoverageTrend,
    PerformanceRegression,
    ReliabilityPattern,
    SeasonalVariation,
    Correlation,
    Anomaly,
    PredictiveAlert,
}

/// Impact levels for insights
#[derive(Debug, Clone, Serialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Analytics-generated recommendations
#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsRecommendation {
    pub recommendation_type: RecommendationType,
    pub title: String,
    pub description: String,
    pub action_items: Vec<String>,
    pub priority: Priority,
    pub estimated_effort: EffortEstimate,
    pub expected_impact: ImpactEstimate,
    pub related_insights: Vec<String>,
}

/// Types of recommendations
#[derive(Debug, Clone, Serialize)]
pub enum RecommendationType {
    CoverageImprovement,
    PerformanceOptimization,
    ReliabilityEnhancement,
    TestStrategyChange,
    InfrastructureUpgrade,
    ProcessImprovement,
}

/// Priority levels
#[derive(Debug, Clone, Serialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
    Urgent,
}

/// Effort estimation
#[derive(Debug, Clone, Serialize)]
pub struct EffortEstimate {
    pub story_points: Option<u32>,
    pub time_estimate_hours: Option<u32>,
    pub complexity: Complexity,
    pub required_skills: Vec<String>,
}

/// Complexity levels
#[derive(Debug, Clone, Serialize)]
pub enum Complexity {
    Trivial,
    Simple,
    Moderate,
    Complex,
    Expert,
}

/// Impact estimation
#[derive(Debug, Clone, Serialize)]
pub struct ImpactEstimate {
    pub coverage_impact: Option<f64>,
    pub performance_impact: Option<f64>,
    pub reliability_impact: Option<f64>,
    pub overall_impact: ImpactLevel,
    pub roi_estimate: Option<f64>,
}

/// Analytics alerts
#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsAlert {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub triggered_by: String,
    pub threshold_value: Option<f64>,
    pub actual_value: Option<f64>,
    pub first_detected: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub acknowledgment_required: bool,
    pub suppression_rules: Vec<String>,
}

/// Alert types
#[derive(Debug, Clone, Serialize)]
pub enum AlertType {
    CoverageDropped,
    PerformanceRegressed,
    ReliabilityDeclined,
    ThresholdExceeded,
    AnomalyDetected,
    TrendDeviation,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
    Emergency,
}

impl TestAnalyticsEngine {
    pub fn new() -> Self {
        Self {
            coverage_analyzer: coverage_trends::CoverageTrendAnalyzer::new(),
            performance_analyzer: performance_analysis::PerformanceAnalyzer::new(),
            reliability_analyzer: reliability_metrics::ReliabilityAnalyzer::new(),
            historical_data: Vec::new(),
        }
    }

    /// Load test results for analysis
    pub fn load_test_results(&mut self, results: Vec<TestExecutionResult>) {
        self.historical_data = results;

        // Update individual analyzers
        self.coverage_analyzer.load_data(&self.historical_data);
        self.performance_analyzer.load_data(&self.historical_data);
        self.reliability_analyzer.load_data(&self.historical_data);
    }

    /// Load historical test data for analysis
    pub fn load_historical_data(&mut self, results: Vec<TestExecutionResult>) {
        self.historical_data = results;

        // Initialize analyzers with historical data
        self.coverage_analyzer.load_coverage_history(&self.historical_data);
        self.performance_analyzer.load_performance_history(&self.historical_data);
        self.reliability_analyzer.load_reliability_history(&self.historical_data);
    }

    /// Run comprehensive analytics across all domains
    pub async fn run_comprehensive_analysis(&self, config: &AnalyticsConfig) -> Result<ComprehensiveAnalysisResult, AnalyticsError> {
        let coverage_analysis = self.coverage_analyzer.analyze_trends(config).await?;
        let performance_analysis = self.performance_analyzer.analyze_performance(config).await?;
        let reliability_analysis = self.reliability_analyzer.analyze_reliability(config).await?;

        // Generate cross-domain insights
        let cross_domain_insights = self.generate_cross_domain_insights(
            &coverage_analysis,
            &performance_analysis,
            &reliability_analysis,
        );

        Ok(ComprehensiveAnalysisResult {
            coverage_analysis,
            performance_analysis,
            reliability_analysis,
            cross_domain_insights,
            analysis_timestamp: chrono::Utc::now(),
            data_points_analyzed: self.historical_data.len(),
        })
    }

    /// Generate insights that span multiple analysis domains
    fn generate_cross_domain_insights(
        &self,
        coverage: &coverage_trends::CoverageAnalysisResult,
        performance: &performance_analysis::PerformanceAnalysisResult,
        reliability: &reliability_metrics::ReliabilityAnalysisResult,
    ) -> Vec<CrossDomainInsight> {
        let mut insights = Vec::new();

        // Coverage vs Performance correlation
        if coverage.rust_coverage.current_percentage < 80.0 && performance.regression_risk > 0.7 {
            insights.push(CrossDomainInsight {
                insight_type: CrossDomainInsightType::CoveragePerformanceCorrelation,
                severity: InsightSeverity::High,
                title: "Low Coverage Correlated with Performance Risk".to_string(),
                description: "Areas with insufficient test coverage are showing performance regression risks.".to_string(),
                recommendation: "Prioritize adding tests for performance-critical code paths.".to_string(),
                affected_components: vec!["rust-backend".to_string()],
                confidence: 0.85,
            });
        }

        // Reliability vs Coverage correlation
        if reliability.overall_stability_score < 0.8 && coverage.overall_trend.trend_direction == coverage_trends::TrendDirection::Declining {
            insights.push(CrossDomainInsight {
                insight_type: CrossDomainInsightType::ReliabilityCoverageCorrelation,
                severity: InsightSeverity::Medium,
                title: "Declining Coverage Impacting Test Reliability".to_string(),
                description: "Test reliability is decreasing as code coverage declines.".to_string(),
                recommendation: "Focus on improving test coverage in areas with flaky tests.".to_string(),
                affected_components: reliability.flaky_tests.iter()
                    .map(|t| t.test_suite.clone())
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect(),
                confidence: 0.75,
            });
        }

        // Performance vs Reliability correlation
        if performance.bottlenecks.len() > 3 && reliability.flaky_tests.len() > 5 {
            insights.push(CrossDomainInsight {
                insight_type: CrossDomainInsightType::PerformanceReliabilityCorrelation,
                severity: InsightSeverity::High,
                title: "Performance Bottlenecks Causing Test Instability".to_string(),
                description: "Multiple performance bottlenecks are correlating with increased test flakiness.".to_string(),
                recommendation: "Address performance bottlenecks to improve test reliability.".to_string(),
                affected_components: performance.bottlenecks.iter()
                    .map(|b| b.component.clone())
                    .collect(),
                confidence: 0.90,
            });
        }

        insights
    }

    /// Get analytics summary for dashboard display
    pub fn get_analytics_summary(&self) -> AnalyticsSummary {
        AnalyticsSummary {
            total_data_points: self.historical_data.len(),
            analysis_coverage: AnalysisCoverage {
                rust_files_analyzed: self.coverage_analyzer.get_rust_file_count(),
                typescript_files_analyzed: self.coverage_analyzer.get_typescript_file_count(),
                performance_metrics_tracked: self.performance_analyzer.get_metrics_count(),
                reliability_tests_monitored: self.reliability_analyzer.get_monitored_test_count(),
            },
            last_updated: chrono::Utc::now(),
            health_score: self.calculate_overall_health_score(),
        }
    }

    fn calculate_overall_health_score(&self) -> f64 {
        // Placeholder implementation - would be based on latest analysis results
        0.85 // This would be calculated from actual data
    }

    /// Run comprehensive analytics analysis
    pub async fn run_analysis(&mut self, config: AnalyticsConfig) -> Result<AnalyticsResult, AnalyticsError> {
        let analysis_start = std::time::Instant::now();

        // Filter data based on time window
        let filtered_data = self.filter_by_time_window(&config.time_window)?;

        if filtered_data.is_empty() {
            return Err(AnalyticsError::InsufficientData("No data available for the specified time window".to_string()));
        }

        // Run individual analyses
        let coverage_analysis = self.coverage_analyzer.analyze(&filtered_data, &config).await?;
        let performance_analysis = self.performance_analyzer.analyze(&filtered_data, &config).await?;
        let reliability_analysis = self.reliability_analyzer.analyze(&filtered_data, &config).await?;

        // Generate insights from combined analyses
        let insights = self.generate_insights(&coverage_analysis, &performance_analysis, &reliability_analysis, &config);

        // Generate recommendations
        let recommendations = self.generate_recommendations(&insights, &config);

        // Check for alerts
        let alerts = self.check_thresholds(&coverage_analysis, &performance_analysis, &reliability_analysis, &config);

        // Create data summary
        let data_summary = self.create_data_summary(&filtered_data);

        let analysis_duration = analysis_start.elapsed();
        tracing::info!("Analytics analysis completed in {:?}", analysis_duration);

        Ok(AnalyticsResult {
            analysis_timestamp: Utc::now(),
            config_used: config,
            data_summary,
            coverage_analysis,
            performance_analysis,
            reliability_analysis,
            insights,
            recommendations,
            alerts,
        })
    }

    /// Generate actionable insights from analysis results
    fn generate_insights(
        &self,
        coverage: &coverage_trends::CoverageAnalysisResult,
        performance: &performance_analysis::PerformanceAnalysisResult,
        reliability: &reliability_metrics::ReliabilityAnalysisResult,
        config: &AnalyticsConfig,
    ) -> Vec<AnalyticsInsight> {
        let mut insights = Vec::new();

        // Coverage trend insights
        if let Some(trend) = &coverage.overall_trend {
            if trend.direction == coverage_trends::TrendDirection::Decreasing && trend.confidence > 0.8 {
                insights.push(AnalyticsInsight {
                    insight_type: InsightType::CoverageTrend,
                    title: "Declining Coverage Trend Detected".to_string(),
                    description: format!("Overall test coverage has been declining at a rate of {:.2}% per week", trend.rate),
                    confidence: trend.confidence,
                    impact: if trend.rate.abs() > 5.0 { ImpactLevel::High } else { ImpactLevel::Medium },
                    supporting_data: serde_json::json!({
                        "trend_rate": trend.rate,
                        "trend_duration": trend.duration_days,
                        "starting_coverage": trend.start_value,
                        "current_coverage": trend.end_value
                    }),
                    generated_at: Utc::now(),
                });
            }
        }

        // Performance regression insights
        if !performance.regressions.is_empty() {
            let critical_regressions = performance.regressions.iter()
                .filter(|r| r.severity == performance_analysis::RegressionSeverity::Critical)
                .count();

            if critical_regressions > 0 {
                insights.push(AnalyticsInsight {
                    insight_type: InsightType::PerformanceRegression,
                    title: format!("{} Critical Performance Regressions Detected", critical_regressions),
                    description: "Multiple critical performance regressions have been detected that may impact user experience".to_string(),
                    confidence: 0.95,
                    impact: ImpactLevel::Critical,
                    supporting_data: serde_json::json!({
                        "critical_regressions": critical_regressions,
                        "total_regressions": performance.regressions.len(),
                        "worst_regression_percent": performance.regressions.iter()
                            .map(|r| r.regression_percent)
                            .fold(0.0f64, f64::max)
                    }),
                    generated_at: Utc::now(),
                });
            }
        }

        // Reliability pattern insights
        if let Some(stability) = &reliability.stability_analysis {
            if stability.overall_stability_score < 0.8 {
                insights.push(AnalyticsInsight {
                    insight_type: InsightType::ReliabilityPattern,
                    title: "Low Test Reliability Detected".to_string(),
                    description: format!("Overall test reliability score is {:.1}%, indicating potential issues with test stability", stability.overall_stability_score * 100.0),
                    confidence: 0.9,
                    impact: if stability.overall_stability_score < 0.7 { ImpactLevel::High } else { ImpactLevel::Medium },
                    supporting_data: serde_json::json!({
                        "stability_score": stability.overall_stability_score,
                        "flaky_test_count": stability.flaky_tests.len(),
                        "failure_patterns": stability.failure_patterns.len()
                    }),
                    generated_at: Utc::now(),
                });
            }
        }

        // Cross-metric correlations
        if let (Some(cov_trend), Some(perf_trend)) = (&coverage.overall_trend, &performance.performance_trend) {
            let correlation = self.calculate_correlation(
                &coverage.trend_data.iter().map(|d| d.coverage_percentage).collect::<Vec<_>>(),
                &performance.trend_data.iter().map(|d| d.average_duration_ms).collect::<Vec<_>>()
            );

            if correlation.abs() > 0.7 {
                insights.push(AnalyticsInsight {
                    insight_type: InsightType::Correlation,
                    title: "Strong Correlation Between Coverage and Performance".to_string(),
                    description: format!("Test coverage and performance show a {} correlation of {:.2}",
                        if correlation > 0.0 { "positive" } else { "negative" }, correlation.abs()),
                    confidence: 0.85,
                    impact: ImpactLevel::Medium,
                    supporting_data: serde_json::json!({
                        "correlation_coefficient": correlation,
                        "interpretation": if correlation > 0.0 {
                            "Higher coverage may be associated with longer test execution times"
                        } else {
                            "Higher coverage may be associated with shorter test execution times"
                        }
                    }),
                    generated_at: Utc::now(),
                });
            }
        }

        insights
    }

    /// Generate actionable recommendations
    fn generate_recommendations(
        &self,
        insights: &[AnalyticsInsight],
        _config: &AnalyticsConfig,
    ) -> Vec<AnalyticsRecommendation> {
        let mut recommendations = Vec::new();

        // Analyze insights and generate recommendations
        for insight in insights {
            match insight.insight_type {
                InsightType::CoverageTrend if insight.impact == ImpactLevel::High => {
                    recommendations.push(AnalyticsRecommendation {
                        recommendation_type: RecommendationType::CoverageImprovement,
                        title: "Implement Coverage Improvement Plan".to_string(),
                        description: "Address the declining coverage trend with targeted improvements".to_string(),
                        action_items: vec![
                            "Analyze uncovered code paths and prioritize high-risk areas".to_string(),
                            "Add unit tests for critical business logic".to_string(),
                            "Implement coverage gates in CI/CD pipeline".to_string(),
                            "Review and update coverage targets".to_string(),
                        ],
                        priority: Priority::High,
                        estimated_effort: EffortEstimate {
                            story_points: Some(13),
                            time_estimate_hours: Some(40),
                            complexity: Complexity::Moderate,
                            required_skills: vec!["Testing".to_string(), "Code Analysis".to_string()],
                        },
                        expected_impact: ImpactEstimate {
                            coverage_impact: Some(10.0),
                            performance_impact: Some(-5.0), // May slightly slow tests
                            reliability_impact: Some(15.0),
                            overall_impact: ImpactLevel::High,
                            roi_estimate: Some(2.5),
                        },
                        related_insights: vec![insight.title.clone()],
                    });
                }
                InsightType::PerformanceRegression if insight.impact == ImpactLevel::Critical => {
                    recommendations.push(AnalyticsRecommendation {
                        recommendation_type: RecommendationType::PerformanceOptimization,
                        title: "Urgent Performance Regression Fix".to_string(),
                        description: "Critical performance regressions require immediate attention".to_string(),
                        action_items: vec![
                            "Identify and rollback problematic changes".to_string(),
                            "Profile affected test suites for bottlenecks".to_string(),
                            "Implement performance monitoring alerts".to_string(),
                            "Optimize slow database queries and operations".to_string(),
                        ],
                        priority: Priority::Urgent,
                        estimated_effort: EffortEstimate {
                            story_points: Some(8),
                            time_estimate_hours: Some(24),
                            complexity: Complexity::Complex,
                            required_skills: vec!["Performance Optimization".to_string(), "Profiling".to_string()],
                        },
                        expected_impact: ImpactEstimate {
                            coverage_impact: None,
                            performance_impact: Some(50.0),
                            reliability_impact: Some(10.0),
                            overall_impact: ImpactLevel::Critical,
                            roi_estimate: Some(4.0),
                        },
                        related_insights: vec![insight.title.clone()],
                    });
                }
                _ => {} // Handle other insight types as needed
            }
        }

        recommendations
    }

    /// Check thresholds and generate alerts
    fn check_thresholds(
        &self,
        coverage: &coverage_trends::CoverageAnalysisResult,
        performance: &performance_analysis::PerformanceAnalysisResult,
        reliability: &reliability_metrics::ReliabilityAnalysisResult,
        config: &AnalyticsConfig,
    ) -> Vec<AnalyticsAlert> {
        let mut alerts = Vec::new();

        // Coverage alerts
        if let Some(latest_coverage) = coverage.trend_data.last() {
            if latest_coverage.coverage_percentage < config.threshold_config.coverage_thresholds.minimum_overall_coverage {
                alerts.push(AnalyticsAlert {
                    alert_type: AlertType::ThresholdExceeded,
                    severity: AlertSeverity::Warning,
                    title: "Coverage Below Minimum Threshold".to_string(),
                    message: format!("Current coverage {:.1}% is below minimum threshold of {:.1}%",
                        latest_coverage.coverage_percentage,
                        config.threshold_config.coverage_thresholds.minimum_overall_coverage),
                    triggered_by: "Coverage Threshold Monitor".to_string(),
                    threshold_value: Some(config.threshold_config.coverage_thresholds.minimum_overall_coverage),
                    actual_value: Some(latest_coverage.coverage_percentage),
                    first_detected: Utc::now(),
                    last_seen: Utc::now(),
                    acknowledgment_required: true,
                    suppression_rules: vec!["coverage_alerts".to_string()],
                });
            }
        }

        // Performance alerts
        for regression in &performance.regressions {
            if regression.regression_percent > config.threshold_config.performance_thresholds.regression_threshold_percent {
                let severity = match regression.severity {
                    performance_analysis::RegressionSeverity::Critical => AlertSeverity::Critical,
                    performance_analysis::RegressionSeverity::High => AlertSeverity::Error,
                    performance_analysis::RegressionSeverity::Medium => AlertSeverity::Warning,
                    performance_analysis::RegressionSeverity::Low => AlertSeverity::Info,
                };

                alerts.push(AnalyticsAlert {
                    alert_type: AlertType::PerformanceRegressed,
                    severity,
                    title: "Performance Regression Detected".to_string(),
                    message: format!("Test '{}' has regressed by {:.1}% ({:.0}ms -> {:.0}ms)",
                        regression.test_identifier, regression.regression_percent,
                        regression.baseline_duration_ms, regression.current_duration_ms),
                    triggered_by: "Performance Monitor".to_string(),
                    threshold_value: Some(config.threshold_config.performance_thresholds.regression_threshold_percent),
                    actual_value: Some(regression.regression_percent),
                    first_detected: Utc::now(),
                    last_seen: Utc::now(),
                    acknowledgment_required: severity == AlertSeverity::Critical,
                    suppression_rules: vec!["performance_alerts".to_string()],
                });
            }
        }

        alerts
    }

    // Helper methods
    fn filter_by_time_window(&self, time_window: &AnalyticsTimeWindow) -> Result<Vec<&TestExecutionResult>, AnalyticsError> {
        let now = Utc::now();
        let filter_start = match time_window {
            AnalyticsTimeWindow::Last24Hours => now - Duration::days(1),
            AnalyticsTimeWindow::LastWeek => now - Duration::weeks(1),
            AnalyticsTimeWindow::LastMonth => now - Duration::days(30),
            AnalyticsTimeWindow::LastQuarter => now - Duration::days(90),
            AnalyticsTimeWindow::LastYear => now - Duration::days(365),
            AnalyticsTimeWindow::Custom { start, end: _ } => *start,
            AnalyticsTimeWindow::Rolling { days } => now - Duration::days(*days as i64),
        };

        let filter_end = match time_window {
            AnalyticsTimeWindow::Custom { start: _, end } => *end,
            _ => now,
        };

        let filtered: Vec<&TestExecutionResult> = self.historical_data
            .iter()
            .filter(|result| result.executed_at >= filter_start && result.executed_at <= filter_end)
            .collect();

        Ok(filtered)
    }

    fn create_data_summary(&self, data: &[&TestExecutionResult]) -> DataSummary {
        let environments: std::collections::HashSet<_> = data.iter().map(|r| r.environment.clone()).collect();
        let test_suites: std::collections::HashSet<_> = data.iter().map(|r| r.test_suite.clone()).collect();

        let (min_date, max_date) = data.iter().fold(
            (Utc::now(), DateTime::UNIX_EPOCH),
            |(min, max), result| {
                (min.min(result.executed_at), max.max(result.executed_at))
            }
        );

        DataSummary {
            total_test_results: data.len(),
            date_range: (min_date, max_date),
            environments_analyzed: environments.into_iter().collect(),
            test_suites_analyzed: test_suites.into_iter().collect(),
            data_quality_score: self.calculate_data_quality_score(data),
            completeness_percentage: self.calculate_data_completeness(data),
        }
    }

    fn calculate_data_quality_score(&self, data: &[&TestExecutionResult]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mut quality_factors = Vec::new();

        // Check for missing essential fields
        let complete_records = data.iter().filter(|r| {
            !r.test_id.is_empty() &&
            !r.test_suite.is_empty() &&
            !r.environment.is_empty()
        }).count();
        quality_factors.push(complete_records as f64 / data.len() as f64);

        // Check for reasonable duration values
        let reasonable_durations = data.iter().filter(|r| {
            let duration_ms = r.duration.as_millis();
            duration_ms > 0 && duration_ms < 3600000 // Between 0 and 1 hour
        }).count();
        quality_factors.push(reasonable_durations as f64 / data.len() as f64);

        // Check for coverage data availability
        let with_coverage = data.iter().filter(|r| {
            r.rust_coverage.is_some() || r.typescript_coverage.is_some()
        }).count();
        quality_factors.push(with_coverage as f64 / data.len() as f64);

        // Calculate average quality score
        quality_factors.iter().sum::<f64>() / quality_factors.len() as f64 * 100.0
    }

    fn calculate_data_completeness(&self, data: &[&TestExecutionResult]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let fields_to_check = 6; // test_id, test_suite, environment, success, duration, executed_at
        let mut total_fields = 0;
        let mut complete_fields = 0;

        for result in data {
            total_fields += fields_to_check;

            if !result.test_id.is_empty() { complete_fields += 1; }
            if !result.test_suite.is_empty() { complete_fields += 1; }
            if !result.environment.is_empty() { complete_fields += 1; }
            complete_fields += 1; // success is always present
            complete_fields += 1; // duration is always present
            complete_fields += 1; // executed_at is always present
        }

        complete_fields as f64 / total_fields as f64 * 100.0
    }

    fn calculate_correlation(&self, x_values: &[f64], y_values: &[f64]) -> f64 {
        if x_values.len() != y_values.len() || x_values.len() < 2 {
            return 0.0;
        }

        let n = x_values.len() as f64;
        let sum_x: f64 = x_values.iter().sum();
        let sum_y: f64 = y_values.iter().sum();
        let sum_xy: f64 = x_values.iter().zip(y_values).map(|(x, y)| x * y).sum();
        let sum_x2: f64 = x_values.iter().map(|x| x * x).sum();
        let sum_y2: f64 = y_values.iter().map(|y| y * y).sum();

        let numerator = n * sum_xy - sum_x * sum_y;
        let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();

        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }
}

/// Analytics errors
#[derive(Debug, thiserror::Error)]
pub enum AnalyticsError {
    #[error("Insufficient data for analysis: {0}")]
    InsufficientData(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Analysis computation error: {0}")]
    ComputationError(String),

    #[error("Data quality issues: {0}")]
    DataQuality(String),

    #[error("Statistical analysis error: {0}")]
    StatisticalError(String),
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            time_window: AnalyticsTimeWindow::LastWeek,
            granularity: AnalyticsGranularity::Daily,
            trend_analysis: TrendAnalysisConfig {
                detect_seasonality: true,
                smooth_data: true,
                confidence_interval: 0.95,
                minimum_data_points: 7,
                regression_type: RegressionType::Linear,
            },
            threshold_config: ThresholdConfig {
                coverage_thresholds: CoverageThresholds {
                    minimum_overall_coverage: 80.0,
                    minimum_rust_coverage: 85.0,
                    minimum_typescript_coverage: 75.0,
                    coverage_drop_alert: 5.0,
                    low_coverage_warning: 70.0,
                    critical_coverage_alert: 60.0,
                },
                performance_thresholds: PerformanceThresholds {
                    max_test_duration_ms: 30000,
                    max_suite_duration_ms: 300000,
                    regression_threshold_percent: 20.0,
                    timeout_threshold_ms: 60000,
                },
                reliability_thresholds: ReliabilityThresholds {
                    minimum_success_rate: 95.0,
                    flakiness_threshold: 5.0,
                    consecutive_failures_alert: 3,
                    stability_score_threshold: 90.0,
                },
            },
            comparison_config: None,
        }
    }
}

impl Default for TestAnalyticsEngine {
    fn default() -> Self {
        Self::new()
    }
}