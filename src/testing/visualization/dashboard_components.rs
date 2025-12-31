use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use axum::{
    extract::{Query, State, Path},
    response::{Html, Json},
    http::StatusCode,
};
use crate::testing::monitoring::comprehensive::TestExecutionResult;
use super::{TestVisualizationEngine, VisualizationConfig, ChartData, VisualizationError};
use super::charts::{ChartGenerators, ChartPresets};

/// Dashboard component state for visualization management
#[derive(Clone)]
pub struct VisualizationDashboardState {
    pub visualization_engine: std::sync::Arc<tokio::sync::Mutex<TestVisualizationEngine>>,
    pub prebuilt_configs: HashMap<String, VisualizationConfig>,
}

impl VisualizationDashboardState {
    pub fn new() -> Self {
        let mut prebuilt_configs = HashMap::new();
        prebuilt_configs.insert("daily_summary".to_string(), ChartPresets::daily_summary());
        prebuilt_configs.insert("performance_monitoring".to_string(), ChartPresets::performance_monitoring());
        prebuilt_configs.insert("environment_comparison".to_string(), ChartPresets::environment_comparison());
        prebuilt_configs.insert("coverage_analysis".to_string(), ChartPresets::coverage_analysis());

        Self {
            visualization_engine: std::sync::Arc::new(tokio::sync::Mutex::new(TestVisualizationEngine::new())),
            prebuilt_configs,
        }
    }

    pub async fn load_test_results(&self, results: Vec<TestExecutionResult>) {
        let mut engine = self.visualization_engine.lock().await;
        engine.load_test_results(results);
    }
}

#[derive(Debug, Deserialize)]
pub struct ChartRequest {
    pub chart_type: Option<String>,
    pub time_window: Option<String>,
    pub environment: Option<String>,
    pub test_suite: Option<String>,
    pub preset: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChartResponse {
    pub success: bool,
    pub chart_data: Option<ChartData>,
    pub error: Option<String>,
    pub config_used: Option<VisualizationConfig>,
}

/// API endpoints for visualization dashboard
pub async fn get_chart_data(
    Query(params): Query<ChartRequest>,
    State(state): State<VisualizationDashboardState>,
) -> Result<Json<ChartResponse>, StatusCode> {
    let config = if let Some(preset_name) = &params.preset {
        // Use predefined configuration
        state.prebuilt_configs.get(preset_name)
            .cloned()
            .unwrap_or_default()
    } else {
        // Build configuration from parameters
        build_config_from_params(&params)
    };

    let mut engine = state.visualization_engine.lock().await;

    match engine.generate_visualization(config.clone()) {
        Ok(chart_data) => {
            Ok(Json(ChartResponse {
                success: true,
                chart_data: Some(chart_data),
                error: None,
                config_used: Some(config),
            }))
        }
        Err(e) => {
            Ok(Json(ChartResponse {
                success: false,
                chart_data: None,
                error: Some(e.to_string()),
                config_used: Some(config),
            }))
        }
    }
}

/// Get specialized chart data for specific analysis types
pub async fn get_specialized_chart(
    Path(chart_type): Path<String>,
    Query(params): Query<ChartRequest>,
    State(state): State<VisualizationDashboardState>,
) -> Result<Json<ChartResponse>, StatusCode> {
    let engine = state.visualization_engine.lock().await;

    // Get all test results for specialized charts
    let all_results: Vec<&TestExecutionResult> = engine.test_results.iter().collect();

    let config = if let Some(preset_name) = &params.preset {
        state.prebuilt_configs.get(preset_name)
            .cloned()
            .unwrap_or_default()
    } else {
        build_config_from_params(&params)
    };

    let chart_result = match chart_type.as_str() {
        "success-rate-timeline" => ChartGenerators::success_rate_timeline(&all_results, &config),
        "environment-comparison" => ChartGenerators::environment_comparison(&all_results, &config),
        "test-suite-distribution" => ChartGenerators::test_suite_distribution(&all_results, &config),
        "performance-reliability" => ChartGenerators::performance_reliability_scatter(&all_results, &config),
        "coverage-radar" => ChartGenerators::coverage_radar(&all_results, &config),
        "flaky-test-analysis" => ChartGenerators::flaky_test_analysis(&all_results, &config),
        _ => Err(VisualizationError::InvalidConfiguration(format!("Unknown chart type: {}", chart_type))),
    };

    match chart_result {
        Ok(chart_data) => {
            Ok(Json(ChartResponse {
                success: true,
                chart_data: Some(chart_data),
                error: None,
                config_used: Some(config),
            }))
        }
        Err(e) => {
            Ok(Json(ChartResponse {
                success: false,
                chart_data: None,
                error: Some(e.to_string()),
                config_used: Some(config),
            }))
        }
    }
}

/// Get dashboard HTML with embedded charts
pub async fn get_dashboard_html(
    Query(params): Query<ChartRequest>,
    State(state): State<VisualizationDashboardState>,
) -> Result<Html<String>, StatusCode> {
    let dashboard_html = generate_dashboard_html(&params, &state).await;
    Ok(Html(dashboard_html))
}

/// Generate comprehensive dashboard overview
pub async fn get_dashboard_overview(
    State(state): State<VisualizationDashboardState>,
) -> Result<Json<DashboardOverview>, StatusCode> {
    let engine = state.visualization_engine.lock().await;
    let all_results: Vec<&TestExecutionResult> = engine.test_results.iter().collect();

    if all_results.is_empty() {
        return Ok(Json(DashboardOverview::empty()));
    }

    let total_tests = all_results.len();
    let successful_tests = all_results.iter().filter(|r| r.success).count();
    let success_rate = (successful_tests as f64 / total_tests as f64) * 100.0;

    let avg_duration = all_results.iter()
        .map(|r| r.duration.as_millis() as f64)
        .sum::<f64>() / total_tests as f64;

    let avg_rust_coverage = all_results.iter()
        .filter_map(|r| r.rust_coverage)
        .sum::<f64>() / total_tests as f64;

    let avg_typescript_coverage = all_results.iter()
        .filter_map(|r| r.typescript_coverage)
        .sum::<f64>() / total_tests as f64;

    let avg_overall_coverage = all_results.iter()
        .filter_map(|r| r.overall_coverage)
        .sum::<f64>() / total_tests as f64;

    // Environment breakdown
    let mut env_stats = HashMap::new();
    for result in &all_results {
        let entry = env_stats.entry(result.environment.clone()).or_insert((0, 0));
        entry.0 += 1;
        if result.success {
            entry.1 += 1;
        }
    }

    let environment_breakdown: HashMap<String, EnvironmentStats> = env_stats
        .into_iter()
        .map(|(env, (total, passed))| {
            (env.clone(), EnvironmentStats {
                environment: env,
                total_tests: total,
                successful_tests: passed,
                success_rate: (passed as f64 / total as f64) * 100.0,
            })
        })
        .collect();

    // Test suite breakdown
    let mut suite_stats = HashMap::new();
    for result in &all_results {
        let entry = suite_stats.entry(result.test_suite.clone()).or_insert((0, 0));
        entry.0 += 1;
        if result.success {
            entry.1 += 1;
        }
    }

    let test_suite_breakdown: HashMap<String, TestSuiteStats> = suite_stats
        .into_iter()
        .map(|(suite, (total, passed))| {
            (suite.clone(), TestSuiteStats {
                test_suite: suite,
                total_tests: total,
                successful_tests: passed,
                success_rate: (passed as f64 / total as f64) * 100.0,
                avg_duration: all_results.iter()
                    .filter(|r| r.test_suite == suite)
                    .map(|r| r.duration.as_millis() as f64)
                    .sum::<f64>() / total as f64,
            })
        })
        .collect();

    Ok(Json(DashboardOverview {
        total_tests,
        successful_tests,
        failed_tests: total_tests - successful_tests,
        success_rate,
        average_duration_ms: avg_duration,
        coverage_stats: CoverageStats {
            rust_coverage: avg_rust_coverage,
            typescript_coverage: avg_typescript_coverage,
            overall_coverage: avg_overall_coverage,
        },
        environment_breakdown,
        test_suite_breakdown,
        recent_trends: generate_recent_trends(&all_results),
        performance_insights: generate_performance_insights(&all_results),
        quality_score: calculate_quality_score(success_rate, avg_overall_coverage, avg_duration),
    }))
}

#[derive(Debug, Serialize)]
pub struct DashboardOverview {
    pub total_tests: usize,
    pub successful_tests: usize,
    pub failed_tests: usize,
    pub success_rate: f64,
    pub average_duration_ms: f64,
    pub coverage_stats: CoverageStats,
    pub environment_breakdown: HashMap<String, EnvironmentStats>,
    pub test_suite_breakdown: HashMap<String, TestSuiteStats>,
    pub recent_trends: RecentTrends,
    pub performance_insights: PerformanceInsights,
    pub quality_score: QualityScore,
}

#[derive(Debug, Serialize)]
pub struct CoverageStats {
    pub rust_coverage: f64,
    pub typescript_coverage: f64,
    pub overall_coverage: f64,
}

#[derive(Debug, Serialize)]
pub struct EnvironmentStats {
    pub environment: String,
    pub total_tests: usize,
    pub successful_tests: usize,
    pub success_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct TestSuiteStats {
    pub test_suite: String,
    pub total_tests: usize,
    pub successful_tests: usize,
    pub success_rate: f64,
    pub avg_duration: f64,
}

#[derive(Debug, Serialize)]
pub struct RecentTrends {
    pub success_rate_trend: String, // "up", "down", "stable"
    pub coverage_trend: String,
    pub performance_trend: String,
    pub trend_period: String,
}

#[derive(Debug, Serialize)]
pub struct PerformanceInsights {
    pub slowest_test_suite: String,
    pub fastest_test_suite: String,
    pub performance_variance: f64,
    pub regression_alerts: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct QualityScore {
    pub overall_score: f64,
    pub reliability_score: f64,
    pub coverage_score: f64,
    pub performance_score: f64,
    pub grade: String, // A, B, C, D, F
}

impl DashboardOverview {
    fn empty() -> Self {
        Self {
            total_tests: 0,
            successful_tests: 0,
            failed_tests: 0,
            success_rate: 0.0,
            average_duration_ms: 0.0,
            coverage_stats: CoverageStats {
                rust_coverage: 0.0,
                typescript_coverage: 0.0,
                overall_coverage: 0.0,
            },
            environment_breakdown: HashMap::new(),
            test_suite_breakdown: HashMap::new(),
            recent_trends: RecentTrends {
                success_rate_trend: "stable".to_string(),
                coverage_trend: "stable".to_string(),
                performance_trend: "stable".to_string(),
                trend_period: "No data".to_string(),
            },
            performance_insights: PerformanceInsights {
                slowest_test_suite: "N/A".to_string(),
                fastest_test_suite: "N/A".to_string(),
                performance_variance: 0.0,
                regression_alerts: Vec::new(),
            },
            quality_score: QualityScore {
                overall_score: 0.0,
                reliability_score: 0.0,
                coverage_score: 0.0,
                performance_score: 0.0,
                grade: "N/A".to_string(),
            },
        }
    }
}

// Helper functions
fn build_config_from_params(params: &ChartRequest) -> VisualizationConfig {
    let mut config = VisualizationConfig::default();

    if let Some(chart_type) = &params.chart_type {
        config.chart_type = match chart_type.as_str() {
            "line" => super::ChartType::Line,
            "bar" => super::ChartType::Bar,
            "pie" => super::ChartType::Pie,
            "scatter" => super::ChartType::Scatter,
            "heatmap" => super::ChartType::Heatmap,
            "area" => super::ChartType::Area,
            "radar" => super::ChartType::Radar,
            _ => super::ChartType::Line,
        };
    }

    if let Some(time_window) = &params.time_window {
        config.time_window = match time_window.as_str() {
            "1h" => super::TimeWindow::LastHour,
            "6h" => super::TimeWindow::Last6Hours,
            "1d" => super::TimeWindow::LastDay,
            "1w" => super::TimeWindow::LastWeek,
            "1m" => super::TimeWindow::LastMonth,
            _ => super::TimeWindow::LastDay,
        };
    }

    if let Some(environment) = &params.environment {
        config.filters.environments = Some(vec![environment.clone()]);
    }

    config
}

async fn generate_dashboard_html(
    _params: &ChartRequest,
    _state: &VisualizationDashboardState,
) -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Test Execution Dashboard</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/chartjs-adapter-date-fns/dist/chartjs-adapter-date-fns.bundle.min.js"></script>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }
        .dashboard-container {
            max-width: 1400px;
            margin: 0 auto;
        }
        .dashboard-header {
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            margin-bottom: 20px;
        }
        .charts-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(600px, 1fr));
            gap: 20px;
        }
        .chart-container {
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .chart-title {
            font-size: 18px;
            font-weight: 600;
            margin-bottom: 15px;
            color: #333;
        }
        .stats-bar {
            display: flex;
            justify-content: space-between;
            margin-bottom: 20px;
        }
        .stat-item {
            text-align: center;
        }
        .stat-value {
            font-size: 24px;
            font-weight: bold;
            color: #2c3e50;
        }
        .stat-label {
            font-size: 14px;
            color: #7f8c8d;
        }
        .success { color: #27ae60; }
        .warning { color: #f39c12; }
        .error { color: #e74c3c; }
    </style>
</head>
<body>
    <div class="dashboard-container">
        <div class="dashboard-header">
            <h1>Test Execution Monitoring Dashboard</h1>
            <div class="stats-bar">
                <div class="stat-item">
                    <div class="stat-value success" id="total-tests">-</div>
                    <div class="stat-label">Total Tests</div>
                </div>
                <div class="stat-item">
                    <div class="stat-value success" id="success-rate">-%</div>
                    <div class="stat-label">Success Rate</div>
                </div>
                <div class="stat-item">
                    <div class="stat-value" id="avg-duration">-ms</div>
                    <div class="stat-label">Avg Duration</div>
                </div>
                <div class="stat-item">
                    <div class="stat-value" id="coverage-score">-%</div>
                    <div class="stat-label">Coverage</div>
                </div>
            </div>
        </div>

        <div class="charts-grid">
            <div class="chart-container">
                <div class="chart-title">Success Rate Timeline</div>
                <canvas id="successRateChart"></canvas>
            </div>

            <div class="chart-container">
                <div class="chart-title">Environment Comparison</div>
                <canvas id="environmentChart"></canvas>
            </div>

            <div class="chart-container">
                <div class="chart-title">Test Suite Distribution</div>
                <canvas id="testSuiteChart"></canvas>
            </div>

            <div class="chart-container">
                <div class="chart-title">Performance vs Reliability</div>
                <canvas id="performanceChart"></canvas>
            </div>

            <div class="chart-container">
                <div class="chart-title">Coverage Trends</div>
                <canvas id="coverageChart"></canvas>
            </div>

            <div class="chart-container">
                <div class="chart-title">Quality Metrics Radar</div>
                <canvas id="radarChart"></canvas>
            </div>
        </div>
    </div>

    <script>
        // Dashboard initialization and chart rendering
        document.addEventListener('DOMContentLoaded', async function() {
            await loadDashboardData();
            await initializeCharts();
        });

        async function loadDashboardData() {
            try {
                const response = await fetch('/api/v2/dashboard/overview');
                const data = await response.json();

                document.getElementById('total-tests').textContent = data.total_tests;
                document.getElementById('success-rate').textContent = data.success_rate.toFixed(1) + '%';
                document.getElementById('avg-duration').textContent = data.average_duration_ms.toFixed(0) + 'ms';
                document.getElementById('coverage-score').textContent = data.coverage_stats.overall_coverage.toFixed(1) + '%';
            } catch (error) {
                console.error('Failed to load dashboard data:', error);
            }
        }

        async function initializeCharts() {
            // Initialize each chart
            await initializeChart('successRateChart', 'success-rate-timeline');
            await initializeChart('environmentChart', 'environment-comparison');
            await initializeChart('testSuiteChart', 'test-suite-distribution');
            await initializeChart('performanceChart', 'performance-reliability');
            await initializeChart('coverageChart', 'coverage-analysis', 'coverage_analysis');
            await initializeChart('radarChart', 'coverage-radar');
        }

        async function initializeChart(canvasId, chartType, preset = null) {
            try {
                const url = preset ?
                    `/api/v2/dashboard/charts/${chartType}?preset=${preset}` :
                    `/api/v2/dashboard/charts/${chartType}`;

                const response = await fetch(url);
                const result = await response.json();

                if (result.success && result.chart_data) {
                    renderChart(canvasId, result.chart_data, chartType);
                } else {
                    console.error(`Failed to load chart ${chartType}:`, result.error);
                }
            } catch (error) {
                console.error(`Error initializing chart ${chartType}:`, error);
            }
        }

        function renderChart(canvasId, chartData, chartType) {
            const ctx = document.getElementById(canvasId).getContext('2d');

            const chartConfig = {
                type: getChartJSType(chartType),
                data: {
                    labels: chartData.labels,
                    datasets: chartData.datasets.map(dataset => ({
                        ...dataset,
                        backgroundColor: dataset.background_color,
                        borderColor: dataset.border_color,
                    }))
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {
                        title: {
                            display: true,
                            text: chartData.metadata.chart_title
                        },
                        legend: {
                            position: 'top',
                        }
                    },
                    scales: getScaleConfig(chartType, chartData)
                }
            };

            new Chart(ctx, chartConfig);
        }

        function getChartJSType(chartType) {
            const typeMap = {
                'success-rate-timeline': 'line',
                'environment-comparison': 'bar',
                'test-suite-distribution': 'doughnut',
                'performance-reliability': 'scatter',
                'coverage-analysis': 'line',
                'coverage-radar': 'radar',
            };
            return typeMap[chartType] || 'line';
        }

        function getScaleConfig(chartType, chartData) {
            if (chartType === 'coverage-radar') {
                return {
                    r: {
                        angleLines: { display: false },
                        suggestedMin: 0,
                        suggestedMax: 100
                    }
                };
            }

            if (chartType === 'test-suite-distribution') {
                return {};
            }

            return {
                x: {
                    display: true,
                    title: {
                        display: true,
                        text: chartData.metadata.axis_labels.x_axis
                    }
                },
                y: {
                    display: true,
                    title: {
                        display: true,
                        text: chartData.metadata.axis_labels.y_axis
                    }
                }
            };
        }

        // Auto-refresh dashboard every 30 seconds
        setInterval(async function() {
            await loadDashboardData();
        }, 30000);
    </script>
</body>
</html>"#.to_string()
}

fn generate_recent_trends(results: &[&TestExecutionResult]) -> RecentTrends {
    // Simple trend analysis based on recent vs older results
    let mid_point = results.len() / 2;
    if mid_point == 0 {
        return RecentTrends {
            success_rate_trend: "stable".to_string(),
            coverage_trend: "stable".to_string(),
            performance_trend: "stable".to_string(),
            trend_period: "Insufficient data".to_string(),
        };
    }

    let recent_results = &results[mid_point..];
    let older_results = &results[..mid_point];

    let recent_success_rate = recent_results.iter().filter(|r| r.success).count() as f64 / recent_results.len() as f64;
    let older_success_rate = older_results.iter().filter(|r| r.success).count() as f64 / older_results.len() as f64;

    let success_rate_trend = if recent_success_rate > older_success_rate + 0.05 {
        "up"
    } else if recent_success_rate < older_success_rate - 0.05 {
        "down"
    } else {
        "stable"
    };

    RecentTrends {
        success_rate_trend: success_rate_trend.to_string(),
        coverage_trend: "stable".to_string(), // Simplified for now
        performance_trend: "stable".to_string(), // Simplified for now
        trend_period: "Last 50% of results".to_string(),
    }
}

fn generate_performance_insights(results: &[&TestExecutionResult]) -> PerformanceInsights {
    let mut suite_durations: HashMap<String, Vec<f64>> = HashMap::new();

    for result in results {
        suite_durations.entry(result.test_suite.clone())
            .or_default()
            .push(result.duration.as_millis() as f64);
    }

    let mut suite_averages: Vec<(String, f64)> = suite_durations
        .iter()
        .map(|(suite, durations)| {
            let avg = durations.iter().sum::<f64>() / durations.len() as f64;
            (suite.clone(), avg)
        })
        .collect();

    suite_averages.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let slowest_suite = suite_averages.last().map(|(s, _)| s.clone()).unwrap_or("N/A".to_string());
    let fastest_suite = suite_averages.first().map(|(s, _)| s.clone()).unwrap_or("N/A".to_string());

    PerformanceInsights {
        slowest_test_suite: slowest_suite,
        fastest_test_suite: fastest_suite,
        performance_variance: 0.0, // Simplified calculation
        regression_alerts: Vec::new(), // Would be populated with actual regression detection
    }
}

fn calculate_quality_score(success_rate: f64, coverage: f64, avg_duration: f64) -> QualityScore {
    let reliability_score = success_rate;
    let coverage_score = coverage;

    // Performance score: lower duration is better (capped at 10 seconds)
    let performance_score = ((10000.0 - avg_duration.min(10000.0)) / 10000.0 * 100.0).max(0.0);

    let overall_score = (reliability_score + coverage_score + performance_score) / 3.0;

    let grade = if overall_score >= 90.0 {
        "A"
    } else if overall_score >= 80.0 {
        "B"
    } else if overall_score >= 70.0 {
        "C"
    } else if overall_score >= 60.0 {
        "D"
    } else {
        "F"
    };

    QualityScore {
        overall_score,
        reliability_score,
        coverage_score,
        performance_score,
        grade: grade.to_string(),
    }
}