use std::collections::{HashMap, BTreeMap, VecDeque};
use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

use crate::testing::entities::TestExecutionResult;
use crate::testing::analytics::performance_analysis::{
    PerformanceAnalyzer, PerformanceAnalysisResult, PerformanceRegression,
    RegressionSeverity, PerformanceBottleneck, PerformanceTrend,
};
use crate::testing::analytics::{AnalyticsConfig, AnalyticsTimeWindow, AnalyticsGranularity};

/// Performance regression dashboard state
pub struct PerformanceRegressionDashboard {
    pub analyzer: Arc<RwLock<PerformanceAnalyzer>>,
    pub historical_data: Arc<RwLock<Vec<TestExecutionResult>>>,
    pub regression_cache: Arc<RwLock<RegressionCache>>,
    pub alert_thresholds: PerformanceAlertThresholds,
    pub dashboard_config: DashboardConfig,
}

/// Regression analysis cache
#[derive(Debug, Default)]
pub struct RegressionCache {
    pub cached_results: HashMap<String, (DateTime<Utc>, PerformanceAnalysisResult)>,
    pub regression_history: VecDeque<RegressionSnapshot>,
    pub bottleneck_tracking: HashMap<String, BottleneckHistory>,
    pub performance_baselines: HashMap<String, PerformanceBaseline>,
    pub cache_ttl_minutes: u32,
}

/// Performance alert thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlertThresholds {
    pub critical_regression_percent: f64,
    pub warning_regression_percent: f64,
    pub max_acceptable_duration_ms: u64,
    pub bottleneck_threshold_count: usize,
    pub trend_degradation_days: u32,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub auto_refresh_interval_seconds: u32,
    pub max_displayed_regressions: usize,
    pub performance_history_days: u32,
    pub enable_predictive_analysis: bool,
    pub regression_detection_sensitivity: f64,
}

/// Regression snapshot for historical tracking
#[derive(Debug, Clone, Serialize)]
pub struct RegressionSnapshot {
    pub timestamp: DateTime<Utc>,
    pub total_regressions: usize,
    pub critical_regressions: usize,
    pub average_regression_percent: f64,
    pub worst_regression: Option<PerformanceRegression>,
    pub affected_test_suites: Vec<String>,
}

/// Bottleneck history tracking
#[derive(Debug, Clone, Serialize)]
pub struct BottleneckHistory {
    pub component: String,
    pub first_detected: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub severity_progression: Vec<(DateTime<Utc>, String)>,
    pub resolution_attempts: Vec<ResolutionAttempt>,
    pub impact_metrics: BottleneckImpact,
}

/// Performance baseline for comparison
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceBaseline {
    pub test_identifier: String,
    pub baseline_duration_ms: f64,
    pub established_at: DateTime<Utc>,
    pub sample_size: usize,
    pub confidence_interval: (f64, f64),
    pub variance: f64,
}

/// Resolution attempt tracking
#[derive(Debug, Clone, Serialize)]
pub struct ResolutionAttempt {
    pub attempted_at: DateTime<Utc>,
    pub strategy: String,
    pub description: String,
    pub success: bool,
    pub impact_measured: Option<f64>,
}

/// Bottleneck impact metrics
#[derive(Debug, Clone, Serialize)]
pub struct BottleneckImpact {
    pub affected_tests_count: usize,
    pub total_time_impact_ms: f64,
    pub user_experience_score: f64,
    pub business_impact_level: String,
}

/// Dashboard query parameters
#[derive(Debug, Deserialize)]
pub struct RegressionQuery {
    pub time_window: Option<String>,
    pub severity: Option<String>,
    pub test_suite: Option<String>,
    pub environment: Option<String>,
    pub limit: Option<usize>,
    pub include_resolved: Option<bool>,
}

/// Regression dashboard overview response
#[derive(Debug, Serialize)]
pub struct RegressionDashboardOverview {
    pub summary: RegressionSummary,
    pub active_regressions: Vec<PerformanceRegression>,
    pub critical_bottlenecks: Vec<PerformanceBottleneck>,
    pub performance_trends: Vec<PerformanceTrendData>,
    pub alerts: Vec<PerformanceAlert>,
    pub recommendations: Vec<PerformanceRecommendation>,
    pub health_score: RegressionHealthScore,
    pub last_updated: DateTime<Utc>,
}

/// Regression summary statistics
#[derive(Debug, Serialize)]
pub struct RegressionSummary {
    pub total_regressions: usize,
    pub critical_regressions: usize,
    pub high_severity_regressions: usize,
    pub average_regression_percent: f64,
    pub worst_regression_percent: f64,
    pub affected_test_suites: usize,
    pub resolution_rate_percent: f64,
    pub time_to_resolution_hours: f64,
}

/// Performance trend data for visualization
#[derive(Debug, Serialize)]
pub struct PerformanceTrendData {
    pub timestamp: DateTime<Utc>,
    pub average_duration_ms: f64,
    pub p95_duration_ms: f64,
    pub p99_duration_ms: f64,
    pub regression_count: usize,
    pub test_count: usize,
}

/// Performance alerts
#[derive(Debug, Serialize)]
pub struct PerformanceAlert {
    pub alert_type: PerformanceAlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub affected_components: Vec<String>,
    pub threshold_exceeded: Option<f64>,
    pub current_value: Option<f64>,
    pub first_detected: DateTime<Utc>,
    pub requires_immediate_action: bool,
}

/// Types of performance alerts
#[derive(Debug, Serialize)]
pub enum PerformanceAlertType {
    RegressionDetected,
    BottleneckIdentified,
    ThresholdExceeded,
    TrendDegradation,
    BaselineDeviation,
    ResourceExhaustion,
}

/// Alert severity levels
#[derive(Debug, Serialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
    Emergency,
}

/// Performance improvement recommendations
#[derive(Debug, Serialize)]
pub struct PerformanceRecommendation {
    pub recommendation_type: RecommendationType,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub estimated_impact: f64,
    pub implementation_effort: String,
    pub affected_components: Vec<String>,
    pub technical_details: Vec<String>,
}

/// Types of performance recommendations
#[derive(Debug, Serialize)]
pub enum RecommendationType {
    OptimizeQuery,
    ReduceMemoryUsage,
    ImproveAlgorithm,
    CacheImplementation,
    ResourceScaling,
    CodeRefactoring,
    TestOptimization,
}

/// Recommendation priority levels
#[derive(Debug, Serialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
    Urgent,
}

/// Regression health score
#[derive(Debug, Serialize)]
pub struct RegressionHealthScore {
    pub overall_score: f64,
    pub grade: String,
    pub performance_stability: f64,
    pub regression_frequency: f64,
    pub resolution_efficiency: f64,
    pub trend_direction: String,
}

impl PerformanceRegressionDashboard {
    pub fn new() -> Self {
        Self {
            analyzer: Arc::new(RwLock::new(PerformanceAnalyzer::new())),
            historical_data: Arc::new(RwLock::new(Vec::new())),
            regression_cache: Arc::new(RwLock::new(RegressionCache::new())),
            alert_thresholds: PerformanceAlertThresholds::default(),
            dashboard_config: DashboardConfig::default(),
        }
    }

    /// Load test data for analysis
    pub async fn load_test_data(&self, results: Vec<TestExecutionResult>) {
        let mut data = self.historical_data.write().await;
        *data = results.clone();

        let mut analyzer = self.analyzer.write().await;
        analyzer.load_performance_history(&results);

        // Clear cache when new data is loaded
        let mut cache = self.regression_cache.write().await;
        cache.clear_cache();
    }

    /// Generate comprehensive dashboard overview
    pub async fn generate_dashboard_overview(&self, query: &RegressionQuery) -> Result<RegressionDashboardOverview, String> {
        let config = self.build_analysis_config_from_query(query);

        let analyzer = self.analyzer.read().await;
        let analysis_result = analyzer.analyze_performance(&config).await
            .map_err(|e| format!("Performance analysis failed: {}", e))?;

        let summary = self.calculate_regression_summary(&analysis_result);
        let health_score = self.calculate_health_score(&analysis_result);
        let alerts = self.generate_performance_alerts(&analysis_result);
        let recommendations = self.generate_recommendations(&analysis_result);
        let trends = self.generate_trend_data(&analysis_result);

        // Update cache with latest results
        let mut cache = self.regression_cache.write().await;
        cache.update_cache(analysis_result.clone());

        Ok(RegressionDashboardOverview {
            summary,
            active_regressions: analysis_result.regressions,
            critical_bottlenecks: analysis_result.bottlenecks,
            performance_trends: trends,
            alerts,
            recommendations,
            health_score,
            last_updated: Utc::now(),
        })
    }

    /// Calculate regression summary statistics
    fn calculate_regression_summary(&self, analysis: &PerformanceAnalysisResult) -> RegressionSummary {
        let critical_count = analysis.regressions.iter()
            .filter(|r| matches!(r.severity, RegressionSeverity::Critical))
            .count();

        let high_count = analysis.regressions.iter()
            .filter(|r| matches!(r.severity, RegressionSeverity::High))
            .count();

        let avg_regression = if !analysis.regressions.is_empty() {
            analysis.regressions.iter()
                .map(|r| r.regression_percent)
                .sum::<f64>() / analysis.regressions.len() as f64
        } else {
            0.0
        };

        let worst_regression = analysis.regressions.iter()
            .map(|r| r.regression_percent)
            .fold(0.0f64, f64::max);

        let affected_suites = analysis.regressions.iter()
            .map(|r| r.test_identifier.split("::").next().unwrap_or("unknown"))
            .collect::<std::collections::HashSet<_>>()
            .len();

        RegressionSummary {
            total_regressions: analysis.regressions.len(),
            critical_regressions: critical_count,
            high_severity_regressions: high_count,
            average_regression_percent: avg_regression,
            worst_regression_percent: worst_regression,
            affected_test_suites: affected_suites,
            resolution_rate_percent: 85.0, // Placeholder - would be calculated from historical data
            time_to_resolution_hours: 4.5, // Placeholder - would be calculated from historical data
        }
    }

    /// Calculate overall health score
    fn calculate_health_score(&self, analysis: &PerformanceAnalysisResult) -> RegressionHealthScore {
        let regression_count = analysis.regressions.len();
        let critical_count = analysis.regressions.iter()
            .filter(|r| matches!(r.severity, RegressionSeverity::Critical))
            .count();

        // Calculate health score components
        let performance_stability = if regression_count == 0 { 100.0 } else {
            100.0 - (regression_count as f64 * 5.0).min(90.0)
        };

        let regression_frequency = if critical_count == 0 { 100.0 } else {
            100.0 - (critical_count as f64 * 20.0).min(90.0)
        };

        let resolution_efficiency = 85.0; // Placeholder - would be calculated from historical data

        let overall_score = (performance_stability + regression_frequency + resolution_efficiency) / 3.0;

        let grade = match overall_score {
            90.0..=100.0 => "A",
            80.0..=89.9 => "B",
            70.0..=79.9 => "C",
            60.0..=69.9 => "D",
            _ => "F",
        }.to_string();

        let trend_direction = if analysis.regression_risk > 0.7 {
            "Degrading"
        } else if analysis.regression_risk < 0.3 {
            "Improving"
        } else {
            "Stable"
        }.to_string();

        RegressionHealthScore {
            overall_score,
            grade,
            performance_stability,
            regression_frequency,
            resolution_efficiency,
            trend_direction,
        }
    }

    /// Generate performance alerts
    fn generate_performance_alerts(&self, analysis: &PerformanceAnalysisResult) -> Vec<PerformanceAlert> {
        let mut alerts = Vec::new();

        // Critical regression alerts
        for regression in &analysis.regressions {
            if matches!(regression.severity, RegressionSeverity::Critical) {
                alerts.push(PerformanceAlert {
                    alert_type: PerformanceAlertType::RegressionDetected,
                    severity: AlertSeverity::Critical,
                    title: "Critical Performance Regression Detected".to_string(),
                    message: format!("Test '{}' has regressed by {:.1}% ({:.0}ms -> {:.0}ms)",
                        regression.test_identifier, regression.regression_percent,
                        regression.baseline_duration_ms, regression.current_duration_ms),
                    affected_components: vec![regression.test_identifier.clone()],
                    threshold_exceeded: Some(self.alert_thresholds.critical_regression_percent),
                    current_value: Some(regression.regression_percent),
                    first_detected: regression.detected_at,
                    requires_immediate_action: true,
                });
            }
        }

        // Bottleneck alerts
        if analysis.bottlenecks.len() >= self.alert_thresholds.bottleneck_threshold_count {
            alerts.push(PerformanceAlert {
                alert_type: PerformanceAlertType::BottleneckIdentified,
                severity: AlertSeverity::Warning,
                title: "Multiple Performance Bottlenecks Detected".to_string(),
                message: format!("{} performance bottlenecks identified across different components",
                    analysis.bottlenecks.len()),
                affected_components: analysis.bottlenecks.iter()
                    .map(|b| b.component.clone())
                    .collect(),
                threshold_exceeded: Some(self.alert_thresholds.bottleneck_threshold_count as f64),
                current_value: Some(analysis.bottlenecks.len() as f64),
                first_detected: Utc::now(),
                requires_immediate_action: false,
            });
        }

        // Trend degradation alerts
        if analysis.regression_risk > 0.8 {
            alerts.push(PerformanceAlert {
                alert_type: PerformanceAlertType::TrendDegradation,
                severity: AlertSeverity::Error,
                title: "Performance Trend Degradation".to_string(),
                message: "Overall performance trend is showing significant degradation".to_string(),
                affected_components: vec!["System-wide".to_string()],
                threshold_exceeded: Some(0.8),
                current_value: Some(analysis.regression_risk),
                first_detected: Utc::now(),
                requires_immediate_action: true,
            });
        }

        alerts
    }

    /// Generate performance improvement recommendations
    fn generate_recommendations(&self, analysis: &PerformanceAnalysisResult) -> Vec<PerformanceRecommendation> {
        let mut recommendations = Vec::new();

        // Critical regression recommendations
        let critical_regressions = analysis.regressions.iter()
            .filter(|r| matches!(r.severity, RegressionSeverity::Critical))
            .count();

        if critical_regressions > 0 {
            recommendations.push(PerformanceRecommendation {
                recommendation_type: RecommendationType::CodeRefactoring,
                title: "Address Critical Performance Regressions".to_string(),
                description: "Multiple critical performance regressions require immediate attention".to_string(),
                priority: Priority::Urgent,
                estimated_impact: critical_regressions as f64 * 20.0,
                implementation_effort: "High".to_string(),
                affected_components: analysis.regressions.iter()
                    .filter(|r| matches!(r.severity, RegressionSeverity::Critical))
                    .map(|r| r.test_identifier.clone())
                    .collect(),
                technical_details: vec![
                    "Profile affected test suites for performance bottlenecks".to_string(),
                    "Implement targeted optimizations for slow operations".to_string(),
                    "Consider caching strategies for frequently accessed data".to_string(),
                ],
            });
        }

        // Bottleneck optimization recommendations
        if !analysis.bottlenecks.is_empty() {
            let database_bottlenecks = analysis.bottlenecks.iter()
                .filter(|b| b.component.contains("database"))
                .count();

            if database_bottlenecks > 0 {
                recommendations.push(PerformanceRecommendation {
                    recommendation_type: RecommendationType::OptimizeQuery,
                    title: "Optimize Database Performance".to_string(),
                    description: "Database operations are causing performance bottlenecks".to_string(),
                    priority: Priority::High,
                    estimated_impact: 30.0,
                    implementation_effort: "Medium".to_string(),
                    affected_components: analysis.bottlenecks.iter()
                        .filter(|b| b.component.contains("database"))
                        .map(|b| b.component.clone())
                        .collect(),
                    technical_details: vec![
                        "Analyze and optimize slow database queries".to_string(),
                        "Add appropriate database indexes".to_string(),
                        "Consider connection pooling optimization".to_string(),
                    ],
                });
            }
        }

        // Memory optimization recommendations
        let high_memory_tests = analysis.bottlenecks.iter()
            .filter(|b| b.bottleneck_type.contains("memory"))
            .count();

        if high_memory_tests > 2 {
            recommendations.push(PerformanceRecommendation {
                recommendation_type: RecommendationType::ReduceMemoryUsage,
                title: "Reduce Memory Usage in Tests".to_string(),
                description: "Multiple tests are showing high memory usage patterns".to_string(),
                priority: Priority::Medium,
                estimated_impact: 15.0,
                implementation_effort: "Medium".to_string(),
                affected_components: analysis.bottlenecks.iter()
                    .filter(|b| b.bottleneck_type.contains("memory"))
                    .map(|b| b.component.clone())
                    .collect(),
                technical_details: vec![
                    "Profile memory usage patterns in affected tests".to_string(),
                    "Implement memory cleanup in test teardown".to_string(),
                    "Consider using mock objects to reduce memory footprint".to_string(),
                ],
            });
        }

        recommendations
    }

    /// Generate trend data for visualization
    fn generate_trend_data(&self, analysis: &PerformanceAnalysisResult) -> Vec<PerformanceTrendData> {
        // This would normally process historical data to generate trend points
        // For now, we'll generate sample trend data based on the analysis
        let mut trends = Vec::new();
        let now = Utc::now();

        // Generate trend data points for the last 30 days
        for i in (0..30).rev() {
            let date = now - Duration::days(i as i64);

            // This is sample data - in reality, this would be calculated from historical results
            let base_duration = 2000.0 + (i as f64 * 10.0); // Simulated increasing duration
            let regression_count = if i < 5 { 2 } else { 0 }; // Recent regressions

            trends.push(PerformanceTrendData {
                timestamp: date,
                average_duration_ms: base_duration,
                p95_duration_ms: base_duration * 1.5,
                p99_duration_ms: base_duration * 2.0,
                regression_count,
                test_count: 100 + (i % 10), // Simulated test count variation
            });
        }

        trends
    }

    /// Build analytics configuration from query parameters
    fn build_analysis_config_from_query(&self, query: &RegressionQuery) -> AnalyticsConfig {
        let mut config = AnalyticsConfig::default();

        // Parse time window
        if let Some(ref time_window) = query.time_window {
            config.time_window = match time_window.as_str() {
                "1h" | "hour" => AnalyticsTimeWindow::Last24Hours,
                "1d" | "day" => AnalyticsTimeWindow::Last24Hours,
                "1w" | "week" => AnalyticsTimeWindow::LastWeek,
                "1m" | "month" => AnalyticsTimeWindow::LastMonth,
                _ => AnalyticsTimeWindow::LastWeek,
            };
        }

        // Set granularity based on time window
        config.granularity = match config.time_window {
            AnalyticsTimeWindow::Last24Hours => AnalyticsGranularity::Hourly,
            AnalyticsTimeWindow::LastWeek => AnalyticsGranularity::Daily,
            _ => AnalyticsGranularity::Daily,
        };

        config
    }

    /// Generate HTML dashboard
    pub async fn generate_html_dashboard(&self, query: &RegressionQuery) -> Result<String, String> {
        let overview = self.generate_dashboard_overview(query).await?;

        Ok(format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Performance Regression Dashboard</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
        .dashboard {{ max-width: 1400px; margin: 0 auto; }}
        .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 2rem; border-radius: 12px; margin-bottom: 2rem; }}
        .health-score {{ display: inline-block; background: rgba(255,255,255,0.2); padding: 1rem; border-radius: 8px; }}
        .cards {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1.5rem; margin-bottom: 2rem; }}
        .card {{ background: white; border-radius: 12px; padding: 1.5rem; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }}
        .card h3 {{ margin: 0 0 1rem 0; color: #333; }}
        .metric {{ font-size: 2rem; font-weight: bold; margin: 0.5rem 0; }}
        .critical {{ color: #e74c3c; }}
        .warning {{ color: #f39c12; }}
        .success {{ color: #27ae60; }}
        .chart-container {{ background: white; border-radius: 12px; padding: 1.5rem; margin-bottom: 2rem; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }}
        .regression-list {{ background: white; border-radius: 12px; padding: 1.5rem; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }}
        .regression-item {{ border-left: 4px solid #e74c3c; padding: 1rem; margin: 1rem 0; background: #fff5f5; border-radius: 0 8px 8px 0; }}
        .regression-item.high {{ border-left-color: #f39c12; background: #fffbf0; }}
        .regression-item.medium {{ border-left-color: #f1c40f; background: #fffef0; }}
        .alerts {{ background: white; border-radius: 12px; padding: 1.5rem; margin-bottom: 2rem; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }}
        .alert {{ padding: 1rem; margin: 1rem 0; border-radius: 8px; border-left: 4px solid; }}
        .alert.critical {{ background: #fff5f5; border-left-color: #e74c3c; }}
        .alert.warning {{ background: #fffbf0; border-left-color: #f39c12; }}
        .alert.error {{ background: #fff0f0; border-left-color: #c0392b; }}
        .recommendations {{ background: white; border-radius: 12px; padding: 1.5rem; box-shadow: 0 4px 6px rgba(0,0,0,0.1); }}
        .recommendation {{ background: #f8f9ff; border: 1px solid #e3e8ff; border-radius: 8px; padding: 1.5rem; margin: 1rem 0; }}
        .recommendation h4 {{ color: #4c51bf; margin: 0 0 1rem 0; }}
        .tag {{ display: inline-block; background: #e2e8f0; color: #2d3748; padding: 0.25rem 0.75rem; border-radius: 9999px; font-size: 0.75rem; margin: 0.25rem; }}
        .urgent {{ background: #fed7d7; color: #c53030; }}
        .high {{ background: #feebc8; color: #c05621; }}
        .last-updated {{ color: #666; font-size: 0.9rem; text-align: right; margin-top: 2rem; }}
    </style>
</head>
<body>
    <div class="dashboard">
        <div class="header">
            <h1>Performance Regression Dashboard</h1>
            <div class="health-score">
                <h3>System Health Score</h3>
                <div class="metric">{:.1}% ({})</div>
                <p>Trend: {}</p>
            </div>
        </div>

        <div class="cards">
            <div class="card">
                <h3>Total Regressions</h3>
                <div class="metric critical">{}</div>
                <p>{} critical, {} high severity</p>
            </div>
            <div class="card">
                <h3>Average Regression</h3>
                <div class="metric warning">{:.1}%</div>
                <p>Worst: {:.1}%</p>
            </div>
            <div class="card">
                <h3>Affected Test Suites</h3>
                <div class="metric">{}</div>
                <p>{:.1}% resolution rate</p>
            </div>
            <div class="card">
                <h3>Time to Resolution</h3>
                <div class="metric success">{:.1}h</div>
                <p>Average resolution time</p>
            </div>
        </div>

        <div class="chart-container">
            <h3>Performance Trend (30 Days)</h3>
            <canvas id="trendChart" width="400" height="200"></canvas>
        </div>

        {}

        {}

        <div class="regression-list">
            <h3>Active Regressions ({})</h3>
            {}
        </div>

        <div class="recommendations">
            <h3>Performance Recommendations</h3>
            {}
        </div>

        <div class="last-updated">
            Last updated: {}
        </div>
    </div>

    <script>
        // Trend Chart
        const ctx = document.getElementById('trendChart').getContext('2d');
        const trendChart = new Chart(ctx, {{
            type: 'line',
            data: {{
                labels: {:#},
                datasets: [{{
                    label: 'Average Duration (ms)',
                    data: {},
                    borderColor: '#667eea',
                    backgroundColor: 'rgba(102, 126, 234, 0.1)',
                    tension: 0.4
                }}, {{
                    label: 'P95 Duration (ms)',
                    data: {},
                    borderColor: '#f39c12',
                    backgroundColor: 'rgba(243, 156, 18, 0.1)',
                    tension: 0.4
                }}]
            }},
            options: {{
                responsive: true,
                interaction: {{
                    mode: 'index',
                    intersect: false,
                }},
                scales: {{
                    x: {{
                        display: true,
                        title: {{
                            display: true,
                            text: 'Date'
                        }}
                    }},
                    y: {{
                        display: true,
                        title: {{
                            display: true,
                            text: 'Duration (ms)'
                        }}
                    }}
                }}
            }}
        }});

        // Auto-refresh every {} seconds
        setTimeout(() => {{ location.reload(); }}, {} * 1000);
    </script>
</body>
</html>"#,
            overview.health_score.overall_score,
            overview.health_score.grade,
            overview.health_score.trend_direction,
            overview.summary.total_regressions,
            overview.summary.critical_regressions,
            overview.summary.high_severity_regressions,
            overview.summary.average_regression_percent,
            overview.summary.worst_regression_percent,
            overview.summary.affected_test_suites,
            overview.summary.resolution_rate_percent,
            overview.summary.time_to_resolution_hours,
            self.generate_alerts_html(&overview.alerts),
            self.generate_bottlenecks_html(&overview.critical_bottlenecks),
            overview.active_regressions.len(),
            self.generate_regressions_html(&overview.active_regressions),
            self.generate_recommendations_html(&overview.recommendations),
            overview.last_updated.format("%Y-%m-%d %H:%M:%S UTC"),
            serde_json::to_string(&overview.performance_trends.iter().map(|t| t.timestamp.format("%m-%d").to_string()).collect::<Vec<_>>()).unwrap(),
            serde_json::to_string(&overview.performance_trends.iter().map(|t| t.average_duration_ms).collect::<Vec<_>>()).unwrap(),
            serde_json::to_string(&overview.performance_trends.iter().map(|t| t.p95_duration_ms).collect::<Vec<_>>()).unwrap(),
            self.dashboard_config.auto_refresh_interval_seconds,
            self.dashboard_config.auto_refresh_interval_seconds
        ))
    }

    fn generate_alerts_html(&self, alerts: &[PerformanceAlert]) -> String {
        if alerts.is_empty() {
            return String::new();
        }

        let alerts_html = alerts.iter().map(|alert| {
            let class = match alert.severity {
                AlertSeverity::Critical => "critical",
                AlertSeverity::Error => "error",
                AlertSeverity::Warning => "warning",
                _ => "info",
            };

            format!(r#"
                <div class="alert {}">
                    <h4>{}</h4>
                    <p>{}</p>
                    {}
                </div>
            "#, class, alert.title, alert.message,
            if alert.requires_immediate_action {
                "<strong>⚠️ Requires immediate action</strong>"
            } else {
                ""
            })
        }).collect::<String>();

        format!(r#"
        <div class="alerts">
            <h3>Active Alerts ({})</h3>
            {}
        </div>
        "#, alerts.len(), alerts_html)
    }

    fn generate_bottlenecks_html(&self, bottlenecks: &[PerformanceBottleneck]) -> String {
        if bottlenecks.is_empty() {
            return String::new();
        }

        let bottlenecks_html = bottlenecks.iter().map(|bottleneck| {
            format!(r#"
                <div class="card">
                    <h4>🔧 {}</h4>
                    <p><strong>Type:</strong> {}</p>
                    <p><strong>Impact:</strong> {:.1}ms</p>
                    <p><strong>Description:</strong> {}</p>
                </div>
            "#, bottleneck.component, bottleneck.bottleneck_type,
            bottleneck.impact_ms, bottleneck.description)
        }).collect::<String>();

        format!(r#"
        <div class="cards">
            <div class="card">
                <h3>Critical Bottlenecks ({})</h3>
            </div>
            {}
        </div>
        "#, bottlenecks.len(), bottlenecks_html)
    }

    fn generate_regressions_html(&self, regressions: &[PerformanceRegression]) -> String {
        regressions.iter().map(|regression| {
            let severity_class = match regression.severity {
                RegressionSeverity::Critical => "critical",
                RegressionSeverity::High => "high",
                RegressionSeverity::Medium => "medium",
                _ => "low",
            };

            format!(r#"
                <div class="regression-item {}">
                    <h4>{}</h4>
                    <p><strong>Regression:</strong> {:.1}% ({:.0}ms → {:.0}ms)</p>
                    <p><strong>Detected:</strong> {}</p>
                    <p><strong>Severity:</strong> {:?}</p>
                </div>
            "#, severity_class, regression.test_identifier,
            regression.regression_percent,
            regression.baseline_duration_ms,
            regression.current_duration_ms,
            regression.detected_at.format("%Y-%m-%d %H:%M"),
            regression.severity)
        }).collect::<String>()
    }

    fn generate_recommendations_html(&self, recommendations: &[PerformanceRecommendation]) -> String {
        recommendations.iter().map(|rec| {
            let priority_class = match rec.priority {
                Priority::Urgent => "urgent",
                Priority::High => "high",
                _ => "",
            };

            let technical_details = rec.technical_details.iter()
                .map(|detail| format!("<li>{}</li>", detail))
                .collect::<String>();

            format!(r#"
                <div class="recommendation">
                    <h4>{} <span class="tag {}">Priority: {:?}</span></h4>
                    <p>{}</p>
                    <p><strong>Estimated Impact:</strong> {:.1}%</p>
                    <p><strong>Implementation Effort:</strong> {}</p>
                    <ul>{}</ul>
                </div>
            "#, rec.title, priority_class, rec.priority,
            rec.description, rec.estimated_impact, rec.implementation_effort,
            technical_details)
        }).collect::<String>()
    }
}

impl RegressionCache {
    pub fn new() -> Self {
        Self {
            cached_results: HashMap::new(),
            regression_history: VecDeque::new(),
            bottleneck_tracking: HashMap::new(),
            performance_baselines: HashMap::new(),
            cache_ttl_minutes: 10, // 10-minute cache TTL for performance data
        }
    }

    pub fn update_cache(&mut self, result: PerformanceAnalysisResult) {
        let cache_key = format!("analysis_{}", Utc::now().timestamp());
        self.cached_results.insert(cache_key, (Utc::now(), result.clone()));

        // Update regression history
        let snapshot = RegressionSnapshot {
            timestamp: Utc::now(),
            total_regressions: result.regressions.len(),
            critical_regressions: result.regressions.iter()
                .filter(|r| matches!(r.severity, RegressionSeverity::Critical))
                .count(),
            average_regression_percent: if result.regressions.is_empty() { 0.0 } else {
                result.regressions.iter().map(|r| r.regression_percent).sum::<f64>() / result.regressions.len() as f64
            },
            worst_regression: result.regressions.iter()
                .max_by(|a, b| a.regression_percent.partial_cmp(&b.regression_percent).unwrap())
                .cloned(),
            affected_test_suites: result.regressions.iter()
                .map(|r| r.test_identifier.split("::").next().unwrap_or("unknown").to_string())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect(),
        };

        self.regression_history.push_back(snapshot);
        if self.regression_history.len() > 100 {
            self.regression_history.pop_front();
        }

        // Clean up old cache entries
        let cutoff = Utc::now() - Duration::minutes(self.cache_ttl_minutes as i64);
        self.cached_results.retain(|_, (timestamp, _)| *timestamp > cutoff);
    }

    pub fn clear_cache(&mut self) {
        self.cached_results.clear();
        self.regression_history.clear();
        self.bottleneck_tracking.clear();
    }
}

impl Default for PerformanceAlertThresholds {
    fn default() -> Self {
        Self {
            critical_regression_percent: 50.0,
            warning_regression_percent: 20.0,
            max_acceptable_duration_ms: 30000,
            bottleneck_threshold_count: 3,
            trend_degradation_days: 7,
        }
    }
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            auto_refresh_interval_seconds: 30,
            max_displayed_regressions: 20,
            performance_history_days: 30,
            enable_predictive_analysis: true,
            regression_detection_sensitivity: 0.8,
        }
    }
}

/// Create the performance regression dashboard router
pub fn create_regression_dashboard_router() -> Router<Arc<PerformanceRegressionDashboard>> {
    Router::new()
        .route("/performance/dashboard", get(get_dashboard_html))
        .route("/performance/dashboard/data", get(get_dashboard_data))
        .route("/performance/regressions", get(get_regressions))
        .route("/performance/bottlenecks", get(get_bottlenecks))
        .route("/performance/alerts", get(get_performance_alerts))
        .route("/performance/health", get(get_performance_health))
        .route("/performance/trends", get(get_performance_trends))
        .route("/performance/recommendations", get(get_performance_recommendations))
}

/// Get dashboard HTML
async fn get_dashboard_html(
    State(dashboard): State<Arc<PerformanceRegressionDashboard>>,
    Query(query): Query<RegressionQuery>,
) -> Result<Html<String>, StatusCode> {
    match dashboard.generate_html_dashboard(&query).await {
        Ok(html) => Ok(Html(html)),
        Err(err) => {
            tracing::error!("Failed to generate dashboard HTML: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get dashboard data
async fn get_dashboard_data(
    State(dashboard): State<Arc<PerformanceRegressionDashboard>>,
    Query(query): Query<RegressionQuery>,
) -> Result<Json<RegressionDashboardOverview>, StatusCode> {
    match dashboard.generate_dashboard_overview(&query).await {
        Ok(overview) => Ok(Json(overview)),
        Err(err) => {
            tracing::error!("Failed to generate dashboard overview: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get regressions
async fn get_regressions(
    State(dashboard): State<Arc<PerformanceRegressionDashboard>>,
    Query(query): Query<RegressionQuery>,
) -> Result<Json<Vec<PerformanceRegression>>, StatusCode> {
    match dashboard.generate_dashboard_overview(&query).await {
        Ok(overview) => Ok(Json(overview.active_regressions)),
        Err(err) => {
            tracing::error!("Failed to get regressions: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get bottlenecks
async fn get_bottlenecks(
    State(dashboard): State<Arc<PerformanceRegressionDashboard>>,
    Query(query): Query<RegressionQuery>,
) -> Result<Json<Vec<PerformanceBottleneck>>, StatusCode> {
    match dashboard.generate_dashboard_overview(&query).await {
        Ok(overview) => Ok(Json(overview.critical_bottlenecks)),
        Err(err) => {
            tracing::error!("Failed to get bottlenecks: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get performance alerts
async fn get_performance_alerts(
    State(dashboard): State<Arc<PerformanceRegressionDashboard>>,
    Query(query): Query<RegressionQuery>,
) -> Result<Json<Vec<PerformanceAlert>>, StatusCode> {
    match dashboard.generate_dashboard_overview(&query).await {
        Ok(overview) => Ok(Json(overview.alerts)),
        Err(err) => {
            tracing::error!("Failed to get performance alerts: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get performance health score
async fn get_performance_health(
    State(dashboard): State<Arc<PerformanceRegressionDashboard>>,
    Query(query): Query<RegressionQuery>,
) -> Result<Json<RegressionHealthScore>, StatusCode> {
    match dashboard.generate_dashboard_overview(&query).await {
        Ok(overview) => Ok(Json(overview.health_score)),
        Err(err) => {
            tracing::error!("Failed to get performance health: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get performance trends
async fn get_performance_trends(
    State(dashboard): State<Arc<PerformanceRegressionDashboard>>,
    Query(query): Query<RegressionQuery>,
) -> Result<Json<Vec<PerformanceTrendData>>, StatusCode> {
    match dashboard.generate_dashboard_overview(&query).await {
        Ok(overview) => Ok(Json(overview.performance_trends)),
        Err(err) => {
            tracing::error!("Failed to get performance trends: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get performance recommendations
async fn get_performance_recommendations(
    State(dashboard): State<Arc<PerformanceRegressionDashboard>>,
    Query(query): Query<RegressionQuery>,
) -> Result<Json<Vec<PerformanceRecommendation>>, StatusCode> {
    match dashboard.generate_dashboard_overview(&query).await {
        Ok(overview) => Ok(Json(overview.recommendations)),
        Err(err) => {
            tracing::error!("Failed to get performance recommendations: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}