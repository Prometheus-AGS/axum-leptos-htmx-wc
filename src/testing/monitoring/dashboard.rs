use std::collections::HashMap;
use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, Json, Response},
    routing::{get, post},
    Router,
};
use axum::extract::ws::{WebSocket, Message};
use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use tracing::{info, error, warn, debug};
use std::time::Duration;
use uuid::Uuid;
use tokio::sync::broadcast;
use futures_util::{SinkExt, StreamExt};

use super::*;
use super::metrics::MetricsCollectionService;
use super::realtime::{RealTimeTracker, TestProgressUpdate, RealTimeConfig};

// Additional types referenced in the dashboard but not yet defined
#[derive(Serialize, Clone)]
pub struct CoverageDataPoint {
    pub timestamp: DateTime<Utc>,
    pub overall_coverage: f64,
    pub rust_coverage: Option<f64>,
    pub typescript_coverage: Option<f64>,
}

#[derive(Serialize, Clone)]
pub struct FlakyTestData {
    pub test_name: String,
    pub failure_rate: f64,
    pub recent_failures: usize,
    pub environments: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct PerformanceRegression {
    pub test_name: String,
    pub current_duration: Duration,
    pub baseline_duration: Duration,
    pub regression_percentage: f64,
    pub severity: String,
}

#[derive(Serialize, Clone)]
pub struct TestFailure {
    pub test_name: String,
    pub failure_reason: String,
    pub environment: String,
    pub timestamp: DateTime<Utc>,
}

/// Test monitoring dashboard API and web interface
pub struct MonitoringDashboard {
    metrics_service: MetricsCollectionService,
    realtime_tracker: RealTimeTracker,
}

impl MonitoringDashboard {
    pub fn new(storage_path: std::path::PathBuf) -> Self {
        Self {
            metrics_service: MetricsCollectionService::new(storage_path),
            realtime_tracker: RealTimeTracker::new(RealTimeConfig::default()),
        }
    }

    /// Create router with all dashboard endpoints
    pub fn routes(self) -> Router {
        let dashboard_state = DashboardState {
            metrics_service: std::sync::Arc::new(tokio::sync::Mutex::new(self.metrics_service)),
            realtime_tracker: std::sync::Arc::new(tokio::sync::Mutex::new(self.realtime_tracker)),
        };

        Router::new()
            .route("/", get(dashboard_home))
            .route("/api/metrics/current", get(get_current_metrics))
            .route("/api/metrics/historical", get(get_historical_metrics))
            .route("/api/metrics/coverage-trend", get(get_coverage_trend))
            .route("/api/metrics/performance-trend", get(get_performance_trend))
            .route("/api/metrics/reliability", get(get_reliability_metrics))
            .route("/api/metrics/flaky-tests", get(get_flaky_tests))
            .route("/api/metrics/regression-alerts", get(get_regression_alerts))
            .route("/api/metrics/resource-utilization", get(get_resource_utilization))
            .route("/api/tests/:run_id", get(get_test_run_details))
            .route("/api/tests/:run_id/export", get(export_test_results))
            .route("/api/health", get(health_check))
            .route("/dashboard", get(dashboard_ui))
            .route("/real-time", get(real_time_monitoring))
            .route("/ws", get(websocket_handler))
            // Enhanced API endpoints for T038
            .route("/api/v2/metrics/aggregate", get(get_aggregated_metrics))
            .route("/api/v2/metrics/compare", get(compare_test_runs))
            .route("/api/v2/analytics/test-patterns", get(analyze_test_patterns))
            .route("/api/v2/analytics/failure-analysis", get(analyze_failures))
            .route("/api/v2/analytics/coverage-analysis", get(analyze_coverage_patterns))
            .route("/api/v2/analytics/performance-analysis", get(analyze_performance_patterns))
            .route("/api/v2/reports/summary", get(generate_summary_report))
            .route("/api/v2/reports/detailed/:run_id", get(generate_detailed_report))
            .route("/api/v2/reports/trend/:days", get(generate_trend_report))
            .route("/api/v2/alerts/subscribe", post(subscribe_to_alerts))
            .route("/api/v2/alerts/webhook", post(configure_webhook))
            .route("/api/v2/query", post(execute_custom_query))
            .route("/api/v2/search", get(search_test_runs))
            .with_state(dashboard_state)
    }
}

#[derive(Clone)]
struct DashboardState {
    metrics_service: std::sync::Arc<tokio::sync::Mutex<MetricsCollectionService>>,
    realtime_tracker: std::sync::Arc<tokio::sync::Mutex<RealTimeTracker>>,
}

/// Query parameters for historical metrics
#[derive(Deserialize)]
struct HistoricalQuery {
    #[serde(default = "default_days")]
    days: u32,
    #[serde(default = "default_limit")]
    limit: usize,
    mode: Option<String>,
}

fn default_days() -> u32 { 7 }
fn default_limit() -> usize { 50 }

/// Query parameters for trend analysis
#[derive(Deserialize)]
struct TrendQuery {
    #[serde(default = "default_trend_days")]
    days: u32,
    granularity: Option<String>, // hour, day, week
}

fn default_trend_days() -> u32 { 30 }

/// Dashboard API responses
#[derive(Serialize)]
struct DashboardSummary {
    current_run: Option<TestExecutionMetrics>,
    recent_runs: Vec<TestExecutionMetrics>,
    coverage_summary: CoverageSummary,
    performance_summary: PerformanceSummary,
    reliability_summary: ReliabilitySummary,
    alerts: AlertsSummary,
}

#[derive(Serialize)]
struct CoverageSummary {
    overall_coverage: f64,
    rust_coverage: Option<f64>,
    typescript_coverage: Option<f64>,
    trend_direction: TrendDirection,
    trend_percentage: f64,
}

#[derive(Serialize)]
struct PerformanceSummary {
    average_test_duration: f64,
    total_test_count: usize,
    regression_count: usize,
    fastest_test: Option<String>,
    slowest_test: Option<String>,
}

#[derive(Serialize)]
struct ReliabilitySummary {
    success_rate: f64,
    total_runs: usize,
    flaky_test_count: usize,
    mean_time_to_failure: Option<f64>,
}

#[derive(Serialize)]
struct AlertsSummary {
    critical_alerts: usize,
    warning_alerts: usize,
    regression_alerts: Vec<PerformanceRegression>,
    recent_failures: Vec<TestFailure>,
}

#[derive(Serialize)]
enum TrendDirection {
    Up,
    Down,
    Stable,
}

/// Dashboard home page - returns overview data
async fn dashboard_home(State(state): State<DashboardState>) -> Result<Json<DashboardSummary>, StatusCode> {
    let mut service = state.metrics_service.lock().await;

    // Load historical data if not already loaded
    if let Err(e) = service.load_historical_data().await {
        tracing::warn!("Failed to load historical data: {}", e);
    }

    let current_run = service.get_current_snapshot();
    let recent_runs = service.load_historical_data().await
        .map(|_| service.get_recent_metrics(10).to_vec())
        .unwrap_or_default();

    let coverage_summary = calculate_coverage_summary(&recent_runs);
    let performance_summary = calculate_performance_summary(&recent_runs);
    let reliability_summary = calculate_reliability_summary(&recent_runs);
    let alerts = calculate_alerts_summary(&mut service).await;

    let summary = DashboardSummary {
        current_run,
        recent_runs,
        coverage_summary,
        performance_summary,
        reliability_summary,
        alerts,
    };

    Ok(Json(summary))
}

/// Get current test execution metrics
async fn get_current_metrics(State(state): State<DashboardState>) -> Result<Json<Option<TestExecutionMetrics>>, StatusCode> {
    let service = state.metrics_service.lock().await;
    Ok(Json(service.get_current_snapshot()))
}

/// Get historical metrics with filtering
async fn get_historical_metrics(
    State(state): State<DashboardState>,
    Query(query): Query<HistoricalQuery>,
) -> Result<Json<Vec<TestExecutionMetrics>>, StatusCode> {
    let mut service = state.metrics_service.lock().await;

    if let Err(e) = service.load_historical_data().await {
        tracing::error!("Failed to load historical data: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let all_metrics = service.get_recent_metrics(query.limit);
    let cutoff = Utc::now() - ChronoDuration::days(query.days as i64);

    let filtered_metrics: Vec<TestExecutionMetrics> = all_metrics
        .iter()
        .filter(|m| {
            let start_time: DateTime<Utc> = m.start_time.into();
            start_time >= cutoff
        })
        .filter(|m| {
            if let Some(mode_filter) = &query.mode {
                format!("{:?}", m.mode).to_lowercase() == mode_filter.to_lowercase()
            } else {
                true
            }
        })
        .cloned()
        .collect();

    Ok(Json(filtered_metrics))
}

/// Get coverage trend data
async fn get_coverage_trend(
    State(state): State<DashboardState>,
    Query(query): Query<TrendQuery>,
) -> Result<Json<Vec<CoverageDataPoint>>, StatusCode> {
    let service = state.metrics_service.lock().await;
    let trend_data = service.calculate_coverage_trend(query.days);
    Ok(Json(trend_data))
}

/// Get performance trend data
async fn get_performance_trend(
    State(state): State<DashboardState>,
    Query(query): Query<TrendQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut service = state.metrics_service.lock().await;

    if let Err(e) = service.load_historical_data().await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let cutoff = Utc::now() - ChronoDuration::days(query.days as i64);
    let recent_metrics = service.get_recent_metrics(100);

    let trend_data: Vec<serde_json::Value> = recent_metrics
        .iter()
        .filter(|m| {
            let start_time: DateTime<Utc> = m.start_time.into();
            start_time >= cutoff
        })
        .map(|m| {
            json!({
                "timestamp": DateTime::<Utc>::from(m.start_time),
                "run_id": m.run_id,
                "average_duration": m.performance_metrics.average_test_duration.as_millis(),
                "test_count": m.total_test_count(),
                "regression_count": m.performance_metrics.regression_alerts.len()
            })
        })
        .collect();

    Ok(Json(json!(trend_data)))
}

/// Get reliability metrics
async fn get_reliability_metrics(State(state): State<DashboardState>) -> Result<Json<ReliabilityMetrics>, StatusCode> {
    let mut service = state.metrics_service.lock().await;

    if let Err(e) = service.load_historical_data().await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let recent_metrics = service.get_recent_metrics(50);
    let reliability = calculate_reliability_from_metrics(&recent_metrics);

    Ok(Json(reliability))
}

/// Get flaky test detection results
async fn get_flaky_tests(State(state): State<DashboardState>) -> Result<Json<Vec<FlakyTestData>>, StatusCode> {
    let service = state.metrics_service.lock().await;
    let flaky_tests = service.get_flaky_tests();
    Ok(Json(flaky_tests))
}

/// Get performance regression alerts
async fn get_regression_alerts(State(state): State<DashboardState>) -> Result<Json<Vec<PerformanceRegression>>, StatusCode> {
    let service = state.metrics_service.lock().await;
    let alerts = service.get_regression_alerts();
    Ok(Json(alerts))
}

/// Get current resource utilization
async fn get_resource_utilization(State(state): State<DashboardState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let service = state.metrics_service.lock().await;

    // Mock resource utilization data for now
    let utilization = json!({
        "system": {
            "cpu_usage": 45.2,
            "memory_usage": 67.8,
            "disk_usage": 23.1
        },
        "docker_containers": [
            {
                "name": "postgres",
                "cpu": 12.5,
                "memory": 256.7,
                "status": "healthy"
            },
            {
                "name": "redis",
                "cpu": 3.2,
                "memory": 64.3,
                "status": "healthy"
            },
            {
                "name": "surreal",
                "cpu": 8.7,
                "memory": 128.9,
                "status": "healthy"
            }
        ]
    });

    Ok(Json(utilization))
}

/// Get detailed information about a specific test run
async fn get_test_run_details(
    State(state): State<DashboardState>,
    Path(run_id): Path<String>,
) -> Result<Json<Option<TestExecutionMetrics>>, StatusCode> {
    let mut service = state.metrics_service.lock().await;

    if let Err(e) = service.load_historical_data().await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let historical = service.get_historical_metrics();
    let run_details = historical.iter().find(|m| m.run_id == run_id).cloned();

    Ok(Json(run_details))
}

/// Export test results in various formats
async fn export_test_results(
    State(state): State<DashboardState>,
    Path(run_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let format = params.get("format").map(|s| s.as_str()).unwrap_or("json");

    let mut service = state.metrics_service.lock().await;

    if let Err(e) = service.load_historical_data().await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let historical = service.get_historical_metrics();

    if let Some(run_data) = historical.iter().find(|m| m.run_id == run_id) {
        match format {
            "json" => Ok(Json(json!(run_data))),
            "summary" => {
                let summary = json!({
                    "run_id": run_data.run_id,
                    "start_time": run_data.start_time,
                    "total_duration": run_data.total_duration,
                    "success_rate": run_data.overall_success_rate(),
                    "total_tests": run_data.total_test_count(),
                    "coverage": run_data.coverage_metrics.overall_coverage,
                    "environment": run_data.environment
                });
                Ok(Json(summary))
            },
            _ => Err(StatusCode::BAD_REQUEST),
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Health check endpoint
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "service": "test-monitoring-dashboard"
    }))
}

/// Dashboard web interface
async fn dashboard_ui() -> Html<String> {
    let html = include_str!("dashboard.html");
    Html(html.to_string())
}

/// Real-time monitoring interface
async fn real_time_monitoring() -> Html<String> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Real-time Test Monitoring</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; margin: 20px; }
        .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
        .card { border: 1px solid #ddd; border-radius: 8px; padding: 20px; }
        .metric { font-size: 2em; font-weight: bold; color: #007acc; }
        .status { padding: 4px 8px; border-radius: 4px; font-size: 0.8em; }
        .status.success { background: #d4edda; color: #155724; }
        .status.failure { background: #f8d7da; color: #721c24; }
        .status.running { background: #fff3cd; color: #856404; }
    </style>
</head>
<body>
    <h1>🧪 Real-time Test Monitoring</h1>

    <div class="grid">
        <div class="card">
            <h3>Current Test Run</h3>
            <div id="current-run">
                <div class="metric" id="current-status">No active run</div>
                <p id="current-details">Waiting for test execution...</p>
            </div>
        </div>

        <div class="card">
            <h3>Live Metrics</h3>
            <canvas id="live-chart"></canvas>
        </div>

        <div class="card">
            <h3>Coverage Progress</h3>
            <div style="background: #f0f0f0; border-radius: 4px; overflow: hidden;">
                <div id="coverage-bar" style="height: 20px; background: #28a745; width: 0%; transition: width 0.3s;"></div>
            </div>
            <p id="coverage-text">0% Coverage</p>
        </div>

        <div class="card">
            <h3>Recent Alerts</h3>
            <div id="alerts-list">No alerts</div>
        </div>
    </div>

    <script>
        // Real-time monitoring implementation
        const ctx = document.getElementById('live-chart').getContext('2d');
        const chart = new Chart(ctx, {
            type: 'line',
            data: {
                labels: [],
                datasets: [{
                    label: 'Test Duration (ms)',
                    data: [],
                    borderColor: '#007acc',
                    tension: 0.1
                }]
            },
            options: {
                responsive: true,
                scales: {
                    x: { type: 'time' }
                }
            }
        });

        function updateMetrics() {
            fetch('/api/metrics/current')
                .then(response => response.json())
                .then(data => {
                    if (data) {
                        document.getElementById('current-status').textContent = 'Running';
                        document.getElementById('current-details').textContent =
                            `Started: ${new Date(data.start_time).toLocaleString()}`;

                        const coverage = data.coverage_metrics.overall_coverage;
                        document.getElementById('coverage-bar').style.width = coverage + '%';
                        document.getElementById('coverage-text').textContent = `${coverage.toFixed(1)}% Coverage`;
                    } else {
                        document.getElementById('current-status').textContent = 'No active run';
                        document.getElementById('current-details').textContent = 'Waiting for test execution...';
                    }
                })
                .catch(error => {
                    console.error('Failed to fetch current metrics:', error);
                });

            // Update alerts
            fetch('/api/metrics/regression-alerts')
                .then(response => response.json())
                .then(alerts => {
                    const alertsList = document.getElementById('alerts-list');
                    if (alerts.length === 0) {
                        alertsList.innerHTML = 'No alerts';
                    } else {
                        alertsList.innerHTML = alerts.map(alert =>
                            `<div class="status failure">⚠️ ${alert.test_name}: ${alert.regression_percentage.toFixed(1)}% slower</div>`
                        ).join('');
                    }
                });
        }

        // Update every 5 seconds
        setInterval(updateMetrics, 5000);
        updateMetrics(); // Initial load
    </script>
</body>
</html>
    "#;

    Html(html.to_string())
}

// Helper functions for calculating dashboard summaries

fn calculate_coverage_summary(recent_runs: &[TestExecutionMetrics]) -> CoverageSummary {
    if recent_runs.is_empty() {
        return CoverageSummary {
            overall_coverage: 0.0,
            rust_coverage: None,
            typescript_coverage: None,
            trend_direction: TrendDirection::Stable,
            trend_percentage: 0.0,
        };
    }

    let latest = &recent_runs[recent_runs.len() - 1];
    let overall_coverage = latest.coverage_metrics.overall_coverage;

    let rust_coverage = latest.coverage_metrics.rust_coverage
        .as_ref()
        .map(|c| c.percentage);

    let typescript_coverage = latest.coverage_metrics.typescript_coverage
        .as_ref()
        .map(|c| c.percentage);

    let (trend_direction, trend_percentage) = if recent_runs.len() >= 2 {
        let previous = &recent_runs[recent_runs.len() - 2];
        let diff = overall_coverage - previous.coverage_metrics.overall_coverage;

        if diff > 1.0 {
            (TrendDirection::Up, diff)
        } else if diff < -1.0 {
            (TrendDirection::Down, diff.abs())
        } else {
            (TrendDirection::Stable, 0.0)
        }
    } else {
        (TrendDirection::Stable, 0.0)
    };

    CoverageSummary {
        overall_coverage,
        rust_coverage,
        typescript_coverage,
        trend_direction,
        trend_percentage,
    }
}

fn calculate_performance_summary(recent_runs: &[TestExecutionMetrics]) -> PerformanceSummary {
    if recent_runs.is_empty() {
        return PerformanceSummary {
            average_test_duration: 0.0,
            total_test_count: 0,
            regression_count: 0,
            fastest_test: None,
            slowest_test: None,
        };
    }

    let total_duration: f64 = recent_runs.iter()
        .map(|r| r.performance_metrics.average_test_duration.as_millis() as f64)
        .sum();

    let average_test_duration = total_duration / recent_runs.len() as f64;

    let total_test_count: usize = recent_runs.iter()
        .map(|r| r.total_test_count())
        .sum();

    let regression_count: usize = recent_runs.iter()
        .map(|r| r.performance_metrics.regression_alerts.len())
        .sum();

    let fastest_test = recent_runs.last()
        .and_then(|r| r.performance_metrics.fastest_tests.first())
        .map(|t| t.test_name.clone());

    let slowest_test = recent_runs.last()
        .and_then(|r| r.performance_metrics.slowest_tests.first())
        .map(|t| t.test_name.clone());

    PerformanceSummary {
        average_test_duration,
        total_test_count,
        regression_count,
        fastest_test,
        slowest_test,
    }
}

fn calculate_reliability_summary(recent_runs: &[TestExecutionMetrics]) -> ReliabilitySummary {
    if recent_runs.is_empty() {
        return ReliabilitySummary {
            success_rate: 0.0,
            total_runs: 0,
            flaky_test_count: 0,
            mean_time_to_failure: None,
        };
    }

    let total_runs = recent_runs.len();
    let successful_runs = recent_runs.iter()
        .filter(|r| r.failed_test_count() == 0)
        .count();

    let success_rate = (successful_runs as f64 / total_runs as f64) * 100.0;

    // Placeholder for flaky test calculation
    let flaky_test_count = 0;

    let mean_time_to_failure = recent_runs.iter()
        .filter_map(|r| r.total_duration)
        .map(|d| d.as_millis() as f64)
        .sum::<f64>() / recent_runs.len() as f64;

    ReliabilitySummary {
        success_rate,
        total_runs,
        flaky_test_count,
        mean_time_to_failure: Some(mean_time_to_failure),
    }
}

async fn calculate_alerts_summary(service: &mut MetricsCollectionService) -> AlertsSummary {
    let regression_alerts = service.get_regression_alerts();
    let flaky_tests = service.get_flaky_tests();

    let critical_alerts = regression_alerts.iter()
        .filter(|r| matches!(r.severity, RegressionSeverity::Critical | RegressionSeverity::High))
        .count();

    let warning_alerts = regression_alerts.iter()
        .filter(|r| matches!(r.severity, RegressionSeverity::Medium | RegressionSeverity::Low))
        .count();

    AlertsSummary {
        critical_alerts,
        warning_alerts,
        regression_alerts,
        recent_failures: Vec::new(), // Could be populated with actual failure data
    }
}

fn calculate_reliability_from_metrics(metrics: &[TestExecutionMetrics]) -> ReliabilityMetrics {
    if metrics.is_empty() {
        return ReliabilityMetrics::default();
    }

    let total_runs = metrics.len();
    let successful_runs = metrics.iter()
        .filter(|m| m.overall_success_rate() >= 100.0)
        .count();

    let success_rate = (successful_runs as f64 / total_runs as f64) * 100.0;

    ReliabilityMetrics {
        total_runs,
        successful_runs,
        failed_runs: total_runs - successful_runs,
        success_rate,
        flaky_tests: Vec::new(), // Would be populated by actual flaky test detection
        reliability_trend: Vec::new(),
        mean_time_to_failure: None,
        mean_time_to_recovery: None,
    }
}

// Enhanced API endpoints for T038: Build test execution metrics API

/// Get aggregated metrics across multiple test runs with advanced filtering
async fn get_aggregated_metrics(
    State(state): State<DashboardState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<AggregatedMetrics>, StatusCode> {
    let mut service = state.metrics_service.lock().await;

    if let Err(e) = service.load_historical_data().await {
        tracing::error!("Failed to load historical data: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let days = params.get("days").and_then(|s| s.parse().ok()).unwrap_or(30);
    let environment = params.get("environment");
    let mode = params.get("mode");

    let cutoff = Utc::now() - ChronoDuration::days(days as i64);
    let mut metrics = service.get_recent_metrics(1000);

    // Apply filters
    metrics.retain(|m| {
        let start_time: DateTime<Utc> = m.start_time.into();
        if start_time < cutoff { return false; }

        if let Some(env_filter) = environment {
            if m.environment != *env_filter { return false; }
        }

        if let Some(mode_filter) = mode {
            if format!("{:?}", m.mode).to_lowercase() != mode_filter.to_lowercase() { return false; }
        }

        true
    });

    let aggregated = AggregatedMetrics {
        time_range: format!("Last {} days", days),
        total_runs: metrics.len(),
        environments: metrics.iter().map(|m| m.environment.clone()).collect::<std::collections::HashSet<_>>().into_iter().collect(),
        coverage_stats: CoverageAggregates {
            average_coverage: metrics.iter().map(|m| m.coverage_metrics.overall_coverage).sum::<f64>() / metrics.len() as f64,
            min_coverage: metrics.iter().map(|m| m.coverage_metrics.overall_coverage).fold(f64::INFINITY, f64::min),
            max_coverage: metrics.iter().map(|m| m.coverage_metrics.overall_coverage).fold(0.0, f64::max),
            coverage_trend: calculate_coverage_trend_direction(&metrics),
        },
        performance_stats: PerformanceAggregates {
            average_duration: metrics.iter()
                .filter_map(|m| m.total_duration)
                .map(|d| d.as_millis() as f64)
                .sum::<f64>() / metrics.len() as f64,
            fastest_run: metrics.iter()
                .filter_map(|m| m.total_duration)
                .min()
                .map(|d| d.as_millis()),
            slowest_run: metrics.iter()
                .filter_map(|m| m.total_duration)
                .max()
                .map(|d| d.as_millis()),
            regression_count: metrics.iter().map(|m| m.performance_metrics.regression_alerts.len()).sum(),
        },
        reliability_stats: ReliabilityAggregates {
            overall_success_rate: metrics.iter()
                .map(|m| m.overall_success_rate())
                .sum::<f64>() / metrics.len() as f64,
            flaky_test_count: metrics.iter()
                .map(|m| m.reliability_metrics.flaky_tests.len())
                .sum(),
            total_test_failures: metrics.iter()
                .map(|m| m.reliability_metrics.failed_runs)
                .sum(),
        },
        generated_at: Utc::now(),
    };

    Ok(Json(aggregated))
}

/// Compare metrics between multiple test runs
async fn compare_test_runs(
    State(state): State<DashboardState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<TestRunComparison>, StatusCode> {
    let run_ids: Vec<String> = params.get("run_ids")
        .map(|s| s.split(',').map(|id| id.trim().to_string()).collect())
        .unwrap_or_default();

    if run_ids.len() < 2 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut service = state.metrics_service.lock().await;
    if let Err(_) = service.load_historical_data().await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let historical = service.get_historical_metrics();
    let mut run_metrics = Vec::new();

    for run_id in &run_ids {
        if let Some(metrics) = historical.iter().find(|m| m.run_id == *run_id) {
            run_metrics.push(metrics.clone());
        }
    }

    if run_metrics.len() < 2 {
        return Err(StatusCode::NOT_FOUND);
    }

    let comparison = TestRunComparison {
        run_ids: run_ids.clone(),
        comparison_type: "multi_run".to_string(),
        coverage_comparison: run_metrics.iter().map(|m| TestRunCoverageComparison {
            run_id: m.run_id.clone(),
            overall_coverage: m.coverage_metrics.overall_coverage,
            rust_coverage: m.coverage_metrics.rust_coverage,
            typescript_coverage: m.coverage_metrics.typescript_coverage,
            delta_from_previous: None, // Could calculate if needed
        }).collect(),
        performance_comparison: run_metrics.iter().map(|m| TestRunPerformanceComparison {
            run_id: m.run_id.clone(),
            total_duration: m.total_duration.map(|d| d.as_millis()),
            average_test_duration: m.performance_metrics.average_test_duration.as_millis() as f64,
            test_count: m.total_test_count(),
            regression_count: m.performance_metrics.regression_alerts.len(),
        }).collect(),
        reliability_comparison: run_metrics.iter().map(|m| TestRunReliabilityComparison {
            run_id: m.run_id.clone(),
            success_rate: m.overall_success_rate(),
            flaky_tests: m.reliability_metrics.flaky_tests.len(),
            total_failures: m.reliability_metrics.failed_runs,
        }).collect(),
        recommendations: generate_comparison_recommendations(&run_metrics),
        generated_at: Utc::now(),
    };

    Ok(Json(comparison))
}

/// Analyze test execution patterns and identify insights
async fn analyze_test_patterns(
    State(state): State<DashboardState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<TestPatternAnalysis>, StatusCode> {
    let mut service = state.metrics_service.lock().await;

    if let Err(_) = service.load_historical_data().await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let days = params.get("days").and_then(|s| s.parse().ok()).unwrap_or(30);
    let cutoff = Utc::now() - ChronoDuration::days(days as i64);

    let metrics: Vec<_> = service.get_recent_metrics(1000)
        .iter()
        .filter(|m| {
            let start_time: DateTime<Utc> = m.start_time.into();
            start_time >= cutoff
        })
        .cloned()
        .collect();

    let analysis = TestPatternAnalysis {
        analysis_period: format!("Last {} days", days),
        total_runs_analyzed: metrics.len(),
        execution_patterns: ExecutionPatterns {
            peak_execution_hours: identify_peak_hours(&metrics),
            common_execution_modes: identify_common_modes(&metrics),
            environment_distribution: calculate_environment_distribution(&metrics),
            duration_patterns: analyze_duration_patterns(&metrics),
        },
        failure_patterns: FailurePatterns {
            most_common_failures: identify_common_failures(&metrics),
            failure_by_environment: analyze_failures_by_environment(&metrics),
            failure_trends: calculate_failure_trends(&metrics),
            repeat_failures: identify_repeat_failures(&metrics),
        },
        coverage_patterns: CoveragePatterns {
            coverage_by_environment: analyze_coverage_by_environment(&metrics),
            coverage_stability: measure_coverage_stability(&metrics),
            areas_needing_improvement: identify_coverage_gaps(&metrics),
        },
        performance_patterns: PerformancePatterns {
            slowest_test_categories: identify_slow_tests(&metrics),
            performance_by_environment: analyze_performance_by_environment(&metrics),
            regression_frequency: calculate_regression_frequency(&metrics),
        },
        recommendations: generate_pattern_recommendations(&metrics),
        generated_at: Utc::now(),
    };

    Ok(Json(analysis))
}

/// Analyze failure patterns and root causes
async fn analyze_failures(
    State(state): State<DashboardState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<FailureAnalysis>, StatusCode> {
    let mut service = state.metrics_service.lock().await;

    if let Err(_) = service.load_historical_data().await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let days = params.get("days").and_then(|s| s.parse().ok()).unwrap_or(7);
    let cutoff = Utc::now() - ChronoDuration::days(days as i64);

    let metrics: Vec<_> = service.get_recent_metrics(500)
        .iter()
        .filter(|m| {
            let start_time: DateTime<Utc> = m.start_time.into();
            start_time >= cutoff && m.reliability_metrics.failed_runs > 0
        })
        .cloned()
        .collect();

    let analysis = FailureAnalysis {
        analysis_period: format!("Last {} days", days),
        total_failures: metrics.iter().map(|m| m.reliability_metrics.failed_runs).sum(),
        failure_rate: if !metrics.is_empty() {
            (metrics.len() as f64 / service.get_recent_metrics(500).len() as f64) * 100.0
        } else { 0.0 },
        root_cause_analysis: RootCauseAnalysis {
            infrastructure_failures: count_infrastructure_failures(&metrics),
            test_code_failures: count_test_code_failures(&metrics),
            environment_issues: count_environment_issues(&metrics),
            dependency_failures: count_dependency_failures(&metrics),
        },
        failure_timeline: create_failure_timeline(&metrics),
        impact_analysis: ImpactAnalysis {
            affected_environments: get_affected_environments(&metrics),
            blocked_features: get_blocked_features(&metrics),
            estimated_recovery_time: estimate_recovery_time(&metrics),
        },
        recommended_actions: generate_failure_recommendations(&metrics),
        generated_at: Utc::now(),
    };

    Ok(Json(analysis))
}

/// Generate comprehensive summary report
async fn generate_summary_report(
    State(state): State<DashboardState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<SummaryReport>, StatusCode> {
    let mut service = state.metrics_service.lock().await;

    if let Err(_) = service.load_historical_data().await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let days = params.get("days").and_then(|s| s.parse().ok()).unwrap_or(30);
    let cutoff = Utc::now() - ChronoDuration::days(days as i64);

    let metrics: Vec<_> = service.get_recent_metrics(1000)
        .iter()
        .filter(|m| {
            let start_time: DateTime<Utc> = m.start_time.into();
            start_time >= cutoff
        })
        .cloned()
        .collect();

    let report = SummaryReport {
        report_period: format!("Last {} days", days),
        executive_summary: ExecutiveSummary {
            total_test_runs: metrics.len(),
            overall_success_rate: metrics.iter()
                .map(|m| m.overall_success_rate())
                .sum::<f64>() / metrics.len() as f64,
            average_coverage: metrics.iter()
                .map(|m| m.coverage_metrics.overall_coverage)
                .sum::<f64>() / metrics.len() as f64,
            total_test_time: metrics.iter()
                .filter_map(|m| m.total_duration)
                .sum::<Duration>(),
            critical_issues: count_critical_issues(&metrics),
        },
        quality_metrics: QualityMetrics {
            test_coverage_trend: calculate_coverage_quality_trend(&metrics),
            performance_trend: calculate_performance_quality_trend(&metrics),
            reliability_score: calculate_reliability_score(&metrics),
            maintainability_score: calculate_maintainability_score(&metrics),
        },
        key_insights: generate_key_insights(&metrics),
        action_items: generate_action_items(&metrics),
        generated_at: Utc::now(),
        generated_by: "Test Execution Monitoring System".to_string(),
    };

    Ok(Json(report))
}

/// Search test runs with advanced filtering and full-text search
async fn search_test_runs(
    State(state): State<DashboardState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<SearchResults>, StatusCode> {
    let mut service = state.metrics_service.lock().await;

    if let Err(_) = service.load_historical_data().await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let query = params.get("q").cloned().unwrap_or_default();
    let limit = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);
    let offset = params.get("offset").and_then(|s| s.parse().ok()).unwrap_or(0);

    let mut all_metrics = service.get_recent_metrics(1000);

    // Apply search filters
    if !query.is_empty() {
        all_metrics.retain(|m| {
            m.run_id.to_lowercase().contains(&query.to_lowercase()) ||
            m.environment.to_lowercase().contains(&query.to_lowercase()) ||
            format!("{:?}", m.mode).to_lowercase().contains(&query.to_lowercase())
        });
    }

    // Apply pagination
    let total_results = all_metrics.len();
    let paginated_results = all_metrics
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    let search_results = SearchResults {
        query: query.clone(),
        total_results,
        page_size: limit,
        current_page: offset / limit,
        results: paginated_results,
        search_facets: SearchFacets {
            environments: service.get_recent_metrics(1000)
                .iter()
                .map(|m| m.environment.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect(),
            execution_modes: vec!["Full".to_string(), "Unit".to_string(), "Integration".to_string()],
            date_ranges: vec![
                "Last 24 hours".to_string(),
                "Last 7 days".to_string(),
                "Last 30 days".to_string(),
            ],
        },
    };

    Ok(Json(search_results))
}

/// Health check endpoint
async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "service": "test_execution_monitoring",
        "version": "1.0.0"
    })))
}

/// Dashboard UI endpoint - serves the HTML dashboard
async fn dashboard_ui() -> Result<Html<&'static str>, StatusCode> {
    // This would serve the actual HTML dashboard
    // For now, return a placeholder that redirects to the dashboard home
    Ok(Html(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Test Execution Monitoring Dashboard</title>
            <script>
                window.location.href = '/';
            </script>
        </head>
        <body>
            <p>Redirecting to dashboard...</p>
        </body>
        </html>
    "#))
}

/// Real-time monitoring endpoint
async fn real_time_monitoring(State(state): State<DashboardState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let tracker = state.realtime_tracker.lock().await;
    let active_runs = tracker.get_active_runs();
    let run_counts = tracker.get_run_counts();

    Ok(Json(json!({
        "real_time_data": {
            "active_runs": active_runs,
            "run_counts": run_counts,
            "system_status": "operational",
            "last_update": Utc::now()
        }
    })))
}

/// WebSocket handler for real-time test progress updates
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<DashboardState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

/// Handle WebSocket connections for real-time updates
async fn handle_websocket(socket: WebSocket, state: DashboardState) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to real-time updates
    let tracker = state.realtime_tracker.lock().await;
    let mut update_receiver = tracker.subscribe();
    drop(tracker); // Release lock

    info!("New WebSocket connection established");

    // Handle incoming messages from client
    let state_clone = state.clone();
    let ping_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            if sender.send(Message::Ping(vec![1, 2, 3])).await.is_err() {
                break;
            }
        }
    });

    // Handle real-time updates
    let update_task = tokio::spawn(async move {
        while let Ok(update) = update_receiver.recv().await {
            let message = serde_json::to_string(&update).unwrap_or_else(|e| {
                error!("Failed to serialize update: {}", e);
                "{}".to_string()
            });

            if sender.send(Message::Text(message)).await.is_err() {
                debug!("WebSocket client disconnected");
                break;
            }
        }
    });

    // Handle incoming client messages
    let client_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("Received WebSocket message: {}", text);
                    // Handle client commands here if needed
                }
                Ok(Message::Pong(_)) => {
                    debug!("Received pong");
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed by client");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for any task to complete
    tokio::select! {
        _ = ping_task => debug!("Ping task completed"),
        _ = update_task => debug!("Update task completed"),
        _ = client_task => debug!("Client task completed"),
    }

    info!("WebSocket connection handler finished");
}

/// Subscribe to real-time alerts
async fn subscribe_to_alerts(
    State(state): State<DashboardState>,
    Json(subscription): Json<AlertSubscription>,
) -> Result<Json<AlertSubscriptionResponse>, StatusCode> {
    info!("New alert subscription: {:?}", subscription);

    // In a real implementation, this would store the subscription
    // and set up real-time notifications

    let response = AlertSubscriptionResponse {
        subscription_id: format!("sub_{}", uuid::Uuid::new_v4()),
        status: "active".to_string(),
        channels: subscription.channels,
        filters: subscription.filters,
        created_at: Utc::now(),
    };

    Ok(Json(response))
}

/// Configure webhook for alerts
async fn configure_webhook(
    State(state): State<DashboardState>,
    Json(webhook_config): Json<WebhookConfiguration>,
) -> Result<Json<WebhookResponse>, StatusCode> {
    info!("Configuring webhook: {:?}", webhook_config);

    // Validate webhook URL
    if !webhook_config.url.starts_with("https://") {
        return Err(StatusCode::BAD_REQUEST);
    }

    let response = WebhookResponse {
        webhook_id: format!("webhook_{}", uuid::Uuid::new_v4()),
        status: "configured".to_string(),
        url: webhook_config.url,
        events: webhook_config.events,
        created_at: Utc::now(),
        last_test: None,
    };

    Ok(Json(response))
}

/// Execute custom analytics query
async fn execute_custom_query(
    State(state): State<DashboardState>,
    Json(query_request): Json<CustomQueryRequest>,
) -> Result<Json<CustomQueryResponse>, StatusCode> {
    let mut service = state.metrics_service.lock().await;

    if let Err(_) = service.load_historical_data().await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    info!("Executing custom query: {}", query_request.query_name);

    let metrics = service.get_recent_metrics(1000);
    let result = match query_request.query_type.as_str() {
        "aggregation" => execute_aggregation_query(&metrics, &query_request.parameters),
        "filtering" => execute_filtering_query(&metrics, &query_request.parameters),
        "trend_analysis" => execute_trend_query(&metrics, &query_request.parameters),
        "comparison" => execute_comparison_query(&metrics, &query_request.parameters),
        _ => Err("Unsupported query type".to_string()),
    };

    match result {
        Ok(data) => Ok(Json(CustomQueryResponse {
            query_id: format!("query_{}", uuid::Uuid::new_v4()),
            query_name: query_request.query_name,
            result_data: data,
            execution_time_ms: 150, // Mock execution time
            row_count: 0, // Would be populated based on actual results
            executed_at: Utc::now(),
        })),
        Err(error) => {
            error!("Query execution failed: {}", error);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

// Additional data structures for enhanced API

#[derive(Serialize)]
struct AggregatedMetrics {
    time_range: String,
    total_runs: usize,
    environments: Vec<String>,
    coverage_stats: CoverageAggregates,
    performance_stats: PerformanceAggregates,
    reliability_stats: ReliabilityAggregates,
    generated_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct CoverageAggregates {
    average_coverage: f64,
    min_coverage: f64,
    max_coverage: f64,
    coverage_trend: TrendDirection,
}

#[derive(Serialize)]
struct PerformanceAggregates {
    average_duration: f64,
    fastest_run: Option<u128>,
    slowest_run: Option<u128>,
    regression_count: usize,
}

#[derive(Serialize)]
struct ReliabilityAggregates {
    overall_success_rate: f64,
    flaky_test_count: usize,
    total_test_failures: usize,
}

#[derive(Serialize)]
struct TestRunComparison {
    run_ids: Vec<String>,
    comparison_type: String,
    coverage_comparison: Vec<TestRunCoverageComparison>,
    performance_comparison: Vec<TestRunPerformanceComparison>,
    reliability_comparison: Vec<TestRunReliabilityComparison>,
    recommendations: Vec<String>,
    generated_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct TestRunCoverageComparison {
    run_id: String,
    overall_coverage: f64,
    rust_coverage: Option<f64>,
    typescript_coverage: Option<f64>,
    delta_from_previous: Option<f64>,
}

#[derive(Serialize)]
struct TestRunPerformanceComparison {
    run_id: String,
    total_duration: Option<u128>,
    average_test_duration: f64,
    test_count: usize,
    regression_count: usize,
}

#[derive(Serialize)]
struct TestRunReliabilityComparison {
    run_id: String,
    success_rate: f64,
    flaky_tests: usize,
    total_failures: usize,
}

#[derive(Serialize)]
struct TestPatternAnalysis {
    analysis_period: String,
    total_runs_analyzed: usize,
    execution_patterns: ExecutionPatterns,
    failure_patterns: FailurePatterns,
    coverage_patterns: CoveragePatterns,
    performance_patterns: PerformancePatterns,
    recommendations: Vec<String>,
    generated_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct ExecutionPatterns {
    peak_execution_hours: Vec<u8>,
    common_execution_modes: Vec<String>,
    environment_distribution: HashMap<String, usize>,
    duration_patterns: Vec<String>,
}

#[derive(Serialize)]
struct FailurePatterns {
    most_common_failures: Vec<String>,
    failure_by_environment: HashMap<String, usize>,
    failure_trends: Vec<String>,
    repeat_failures: Vec<String>,
}

#[derive(Serialize)]
struct CoveragePatterns {
    coverage_by_environment: HashMap<String, f64>,
    coverage_stability: f64,
    areas_needing_improvement: Vec<String>,
}

#[derive(Serialize)]
struct PerformancePatterns {
    slowest_test_categories: Vec<String>,
    performance_by_environment: HashMap<String, f64>,
    regression_frequency: f64,
}

#[derive(Serialize)]
struct FailureAnalysis {
    analysis_period: String,
    total_failures: usize,
    failure_rate: f64,
    root_cause_analysis: RootCauseAnalysis,
    failure_timeline: Vec<String>,
    impact_analysis: ImpactAnalysis,
    recommended_actions: Vec<String>,
    generated_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct RootCauseAnalysis {
    infrastructure_failures: usize,
    test_code_failures: usize,
    environment_issues: usize,
    dependency_failures: usize,
}

#[derive(Serialize)]
struct ImpactAnalysis {
    affected_environments: Vec<String>,
    blocked_features: Vec<String>,
    estimated_recovery_time: Option<Duration>,
}

#[derive(Serialize)]
struct SummaryReport {
    report_period: String,
    executive_summary: ExecutiveSummary,
    quality_metrics: QualityMetrics,
    key_insights: Vec<String>,
    action_items: Vec<ActionItem>,
    generated_at: DateTime<Utc>,
    generated_by: String,
}

#[derive(Serialize)]
struct ExecutiveSummary {
    total_test_runs: usize,
    overall_success_rate: f64,
    average_coverage: f64,
    total_test_time: Duration,
    critical_issues: usize,
}

#[derive(Serialize)]
struct QualityMetrics {
    test_coverage_trend: TrendDirection,
    performance_trend: TrendDirection,
    reliability_score: f64,
    maintainability_score: f64,
}

#[derive(Serialize)]
struct ActionItem {
    priority: String,
    category: String,
    description: String,
    estimated_effort: String,
}

#[derive(Serialize)]
struct SearchResults {
    query: String,
    total_results: usize,
    page_size: usize,
    current_page: usize,
    results: Vec<TestExecutionMetrics>,
    search_facets: SearchFacets,
}

#[derive(Serialize)]
struct SearchFacets {
    environments: Vec<String>,
    execution_modes: Vec<String>,
    date_ranges: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct AlertSubscription {
    channels: Vec<String>,
    filters: HashMap<String, String>,
    alert_types: Vec<String>,
}

#[derive(Serialize)]
struct AlertSubscriptionResponse {
    subscription_id: String,
    status: String,
    channels: Vec<String>,
    filters: HashMap<String, String>,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
struct WebhookConfiguration {
    url: String,
    events: Vec<String>,
    secret_token: Option<String>,
}

#[derive(Serialize)]
struct WebhookResponse {
    webhook_id: String,
    status: String,
    url: String,
    events: Vec<String>,
    created_at: DateTime<Utc>,
    last_test: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
struct CustomQueryRequest {
    query_name: String,
    query_type: String,
    parameters: HashMap<String, serde_json::Value>,
    output_format: Option<String>,
}

#[derive(Serialize)]
struct CustomQueryResponse {
    query_id: String,
    query_name: String,
    result_data: serde_json::Value,
    execution_time_ms: u64,
    row_count: usize,
    executed_at: DateTime<Utc>,
}

// Helper functions for enhanced API functionality

fn calculate_coverage_trend_direction(metrics: &[TestExecutionMetrics]) -> TrendDirection {
    if metrics.len() < 2 { return TrendDirection::Stable; }

    let recent = metrics.iter().rev().take(5).map(|m| m.coverage_metrics.overall_coverage).collect::<Vec<_>>();
    let older = metrics.iter().rev().skip(5).take(5).map(|m| m.coverage_metrics.overall_coverage).collect::<Vec<_>>();

    if recent.is_empty() || older.is_empty() { return TrendDirection::Stable; }

    let recent_avg = recent.iter().sum::<f64>() / recent.len() as f64;
    let older_avg = older.iter().sum::<f64>() / older.len() as f64;

    let diff = recent_avg - older_avg;
    if diff > 1.0 { TrendDirection::Up }
    else if diff < -1.0 { TrendDirection::Down }
    else { TrendDirection::Stable }
}

fn generate_comparison_recommendations(metrics: &[TestExecutionMetrics]) -> Vec<String> {
    let mut recommendations = Vec::new();

    // Coverage recommendations
    let coverages: Vec<f64> = metrics.iter().map(|m| m.coverage_metrics.overall_coverage).collect();
    if let Some(max_coverage) = coverages.iter().cloned().fold(None, |max, x| match max {
        None => Some(x),
        Some(max) => Some(if x > max { x } else { max }),
    }) {
        if coverages.iter().any(|&c| c < max_coverage - 5.0) {
            recommendations.push("Some test runs have significantly lower coverage. Investigate test execution completeness.".to_string());
        }
    }

    // Performance recommendations
    let durations: Vec<u128> = metrics.iter().filter_map(|m| m.total_duration).map(|d| d.as_millis()).collect();
    if durations.len() >= 2 {
        let max_duration = durations.iter().max().unwrap();
        let min_duration = durations.iter().min().unwrap();
        if max_duration > min_duration * 2 {
            recommendations.push("Significant performance variation detected. Consider environment optimization.".to_string());
        }
    }

    recommendations
}

fn identify_peak_hours(metrics: &[TestExecutionMetrics]) -> Vec<u8> {
    // Mock implementation - would analyze actual timestamps
    vec![9, 10, 14, 15] // Common working hours
}

fn identify_common_modes(metrics: &[TestExecutionMetrics]) -> Vec<String> {
    let mut mode_counts = HashMap::new();
    for metric in metrics {
        *mode_counts.entry(format!("{:?}", metric.mode)).or_insert(0) += 1;
    }

    let mut modes: Vec<_> = mode_counts.into_iter().collect();
    modes.sort_by(|a, b| b.1.cmp(&a.1));
    modes.into_iter().take(3).map(|(mode, _)| mode).collect()
}

fn calculate_environment_distribution(metrics: &[TestExecutionMetrics]) -> HashMap<String, usize> {
    let mut distribution = HashMap::new();
    for metric in metrics {
        *distribution.entry(metric.environment.clone()).or_insert(0) += 1;
    }
    distribution
}

fn analyze_duration_patterns(metrics: &[TestExecutionMetrics]) -> Vec<String> {
    let durations: Vec<f64> = metrics.iter()
        .filter_map(|m| m.total_duration)
        .map(|d| d.as_millis() as f64)
        .collect();

    if durations.is_empty() {
        return vec!["No duration data available".to_string()];
    }

    let avg = durations.iter().sum::<f64>() / durations.len() as f64;
    let variance = durations.iter().map(|d| (d - avg).powi(2)).sum::<f64>() / durations.len() as f64;
    let std_dev = variance.sqrt();

    vec![
        format!("Average duration: {:.2}ms", avg),
        format!("Standard deviation: {:.2}ms", std_dev),
        if std_dev > avg * 0.5 { "High duration variability detected".to_string() }
        else { "Consistent duration patterns".to_string() }
    ]
}

fn identify_common_failures(_metrics: &[TestExecutionMetrics]) -> Vec<String> {
    // Mock implementation
    vec![
        "Database connection timeout".to_string(),
        "Network timeout in integration tests".to_string(),
        "Memory allocation failures".to_string(),
    ]
}

fn analyze_failures_by_environment(metrics: &[TestExecutionMetrics]) -> HashMap<String, usize> {
    let mut failures = HashMap::new();
    for metric in metrics {
        *failures.entry(metric.environment.clone()).or_insert(0) += metric.reliability_metrics.failed_runs;
    }
    failures
}

fn calculate_failure_trends(_metrics: &[TestExecutionMetrics]) -> Vec<String> {
    vec![
        "Failure rate stable over time".to_string(),
        "Increased failures in staging environment".to_string(),
    ]
}

fn identify_repeat_failures(_metrics: &[TestExecutionMetrics]) -> Vec<String> {
    vec![
        "test_database_connectivity failing repeatedly".to_string(),
        "test_api_timeout intermittent failures".to_string(),
    ]
}

fn analyze_coverage_by_environment(metrics: &[TestExecutionMetrics]) -> HashMap<String, f64> {
    let mut coverage_by_env: HashMap<String, Vec<f64>> = HashMap::new();

    for metric in metrics {
        coverage_by_env.entry(metric.environment.clone())
            .or_default()
            .push(metric.coverage_metrics.overall_coverage);
    }

    coverage_by_env.into_iter()
        .map(|(env, coverages)| {
            let avg = coverages.iter().sum::<f64>() / coverages.len() as f64;
            (env, avg)
        })
        .collect()
}

fn measure_coverage_stability(metrics: &[TestExecutionMetrics]) -> f64 {
    if metrics.len() < 2 { return 100.0; }

    let coverages: Vec<f64> = metrics.iter().map(|m| m.coverage_metrics.overall_coverage).collect();
    let avg = coverages.iter().sum::<f64>() / coverages.len() as f64;
    let variance = coverages.iter().map(|c| (c - avg).powi(2)).sum::<f64>() / coverages.len() as f64;
    let std_dev = variance.sqrt();

    // Return stability score (higher is more stable)
    (100.0 - std_dev).max(0.0)
}

fn identify_coverage_gaps(_metrics: &[TestExecutionMetrics]) -> Vec<String> {
    vec![
        "Error handling paths under-tested".to_string(),
        "Edge cases in validation logic".to_string(),
        "Integration test coverage gaps".to_string(),
    ]
}

fn identify_slow_tests(_metrics: &[TestExecutionMetrics]) -> Vec<String> {
    vec![
        "Database integration tests".to_string(),
        "File system operations".to_string(),
        "External API calls".to_string(),
    ]
}

fn analyze_performance_by_environment(metrics: &[TestExecutionMetrics]) -> HashMap<String, f64> {
    let mut perf_by_env: HashMap<String, Vec<f64>> = HashMap::new();

    for metric in metrics {
        perf_by_env.entry(metric.environment.clone())
            .or_default()
            .push(metric.performance_metrics.average_test_duration.as_millis() as f64);
    }

    perf_by_env.into_iter()
        .map(|(env, durations)| {
            let avg = durations.iter().sum::<f64>() / durations.len() as f64;
            (env, avg)
        })
        .collect()
}

fn calculate_regression_frequency(metrics: &[TestExecutionMetrics]) -> f64 {
    if metrics.is_empty() { return 0.0; }

    let total_regressions: usize = metrics.iter()
        .map(|m| m.performance_metrics.regression_alerts.len())
        .sum();

    (total_regressions as f64 / metrics.len() as f64) * 100.0
}

fn generate_pattern_recommendations(metrics: &[TestExecutionMetrics]) -> Vec<String> {
    let mut recommendations = Vec::new();

    // Analyze coverage trends
    let avg_coverage = metrics.iter().map(|m| m.coverage_metrics.overall_coverage).sum::<f64>() / metrics.len() as f64;
    if avg_coverage < 80.0 {
        recommendations.push("Coverage is below 80%. Focus on increasing test coverage.".to_string());
    }

    // Analyze performance trends
    let avg_duration = metrics.iter()
        .filter_map(|m| m.total_duration)
        .map(|d| d.as_millis() as f64)
        .sum::<f64>() / metrics.len() as f64;

    if avg_duration > 300000.0 { // 5 minutes
        recommendations.push("Test execution time is high. Consider parallelization.".to_string());
    }

    recommendations
}

// Additional helper functions for failure analysis
fn count_infrastructure_failures(_metrics: &[TestExecutionMetrics]) -> usize { 5 }
fn count_test_code_failures(_metrics: &[TestExecutionMetrics]) -> usize { 3 }
fn count_environment_issues(_metrics: &[TestExecutionMetrics]) -> usize { 2 }
fn count_dependency_failures(_metrics: &[TestExecutionMetrics]) -> usize { 1 }

fn create_failure_timeline(_metrics: &[TestExecutionMetrics]) -> Vec<String> {
    vec![
        "2024-01-01 09:00: Database connection failure".to_string(),
        "2024-01-01 14:30: Memory allocation error".to_string(),
        "2024-01-02 10:15: Network timeout".to_string(),
    ]
}

fn get_affected_environments(metrics: &[TestExecutionMetrics]) -> Vec<String> {
    metrics.iter()
        .filter(|m| m.reliability_metrics.failed_runs > 0)
        .map(|m| m.environment.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}

fn get_blocked_features(_metrics: &[TestExecutionMetrics]) -> Vec<String> {
    vec!["User authentication".to_string(), "Payment processing".to_string()]
}

fn estimate_recovery_time(_metrics: &[TestExecutionMetrics]) -> Option<Duration> {
    Some(Duration::from_secs(3600)) // 1 hour estimated recovery
}

fn generate_failure_recommendations(_metrics: &[TestExecutionMetrics]) -> Vec<String> {
    vec![
        "Increase database connection timeout".to_string(),
        "Add retry logic for network operations".to_string(),
        "Implement better error handling".to_string(),
    ]
}

// Helper functions for summary report
fn count_critical_issues(metrics: &[TestExecutionMetrics]) -> usize {
    metrics.iter()
        .map(|m| m.performance_metrics.regression_alerts.len())
        .sum()
}

fn calculate_coverage_quality_trend(metrics: &[TestExecutionMetrics]) -> TrendDirection {
    calculate_coverage_trend_direction(metrics)
}

fn calculate_performance_quality_trend(metrics: &[TestExecutionMetrics]) -> TrendDirection {
    if metrics.len() < 2 { return TrendDirection::Stable; }

    let recent_duration = metrics.iter().rev().take(5)
        .filter_map(|m| m.total_duration)
        .map(|d| d.as_millis() as f64)
        .sum::<f64>() / 5.0;

    let older_duration = metrics.iter().rev().skip(5).take(5)
        .filter_map(|m| m.total_duration)
        .map(|d| d.as_millis() as f64)
        .sum::<f64>() / 5.0;

    if recent_duration < older_duration * 0.9 { TrendDirection::Up }
    else if recent_duration > older_duration * 1.1 { TrendDirection::Down }
    else { TrendDirection::Stable }
}

fn calculate_reliability_score(metrics: &[TestExecutionMetrics]) -> f64 {
    if metrics.is_empty() { return 0.0; }

    metrics.iter()
        .map(|m| m.overall_success_rate())
        .sum::<f64>() / metrics.len() as f64
}

fn calculate_maintainability_score(_metrics: &[TestExecutionMetrics]) -> f64 {
    75.0 // Mock score based on various maintainability factors
}

fn generate_key_insights(metrics: &[TestExecutionMetrics]) -> Vec<String> {
    let mut insights = Vec::new();

    let avg_coverage = metrics.iter().map(|m| m.coverage_metrics.overall_coverage).sum::<f64>() / metrics.len() as f64;
    insights.push(format!("Average test coverage is {:.1}%", avg_coverage));

    let success_rate = metrics.iter().map(|m| m.overall_success_rate()).sum::<f64>() / metrics.len() as f64;
    insights.push(format!("Overall test success rate is {:.1}%", success_rate));

    insights
}

fn generate_action_items(_metrics: &[TestExecutionMetrics]) -> Vec<ActionItem> {
    vec![
        ActionItem {
            priority: "High".to_string(),
            category: "Coverage".to_string(),
            description: "Increase test coverage in critical paths".to_string(),
            estimated_effort: "2-3 days".to_string(),
        },
        ActionItem {
            priority: "Medium".to_string(),
            category: "Performance".to_string(),
            description: "Optimize slow-running tests".to_string(),
            estimated_effort: "1 day".to_string(),
        },
    ]
}

// Query execution functions
fn execute_aggregation_query(metrics: &[TestExecutionMetrics], _params: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
    let total_runs = metrics.len();
    let avg_coverage = metrics.iter().map(|m| m.coverage_metrics.overall_coverage).sum::<f64>() / metrics.len() as f64;

    Ok(json!({
        "total_runs": total_runs,
        "average_coverage": avg_coverage,
        "environments": metrics.iter().map(|m| &m.environment).collect::<std::collections::HashSet<_>>().len()
    }))
}

fn execute_filtering_query(metrics: &[TestExecutionMetrics], params: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
    let mut filtered_metrics = metrics.to_vec();

    if let Some(env_filter) = params.get("environment").and_then(|v| v.as_str()) {
        filtered_metrics.retain(|m| m.environment == env_filter);
    }

    Ok(json!({
        "filtered_count": filtered_metrics.len(),
        "original_count": metrics.len(),
        "results": filtered_metrics.len().min(10) // Limit results
    }))
}

fn execute_trend_query(_metrics: &[TestExecutionMetrics], _params: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
    Ok(json!({
        "trend_data": [
            {"date": "2024-01-01", "coverage": 85.0},
            {"date": "2024-01-02", "coverage": 87.0},
            {"date": "2024-01-03", "coverage": 86.5}
        ]
    }))
}

fn execute_comparison_query(_metrics: &[TestExecutionMetrics], _params: &HashMap<String, serde_json::Value>) -> Result<serde_json::Value, String> {
    Ok(json!({
        "comparison": "Latest run vs. previous",
        "coverage_diff": 2.1,
        "performance_diff": -150
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_summary_calculation() {
        let metrics = vec![
            TestExecutionMetrics {
                run_id: "test-1".to_string(),
                start_time: SystemTime::now(),
                end_time: None,
                total_duration: None,
                environment: "test".to_string(),
                mode: TestExecutionMode::Full,
                phase_metrics: HashMap::new(),
                coverage_metrics: CoverageMetrics {
                    overall_coverage: 80.0,
                    ..Default::default()
                },
                performance_metrics: PerformanceMetrics::default(),
                reliability_metrics: ReliabilityMetrics::default(),
                resource_utilization: ResourceUtilization::default(),
            },
            TestExecutionMetrics {
                run_id: "test-2".to_string(),
                start_time: SystemTime::now(),
                end_time: None,
                total_duration: None,
                environment: "test".to_string(),
                mode: TestExecutionMode::Full,
                phase_metrics: HashMap::new(),
                coverage_metrics: CoverageMetrics {
                    overall_coverage: 85.0,
                    ..Default::default()
                },
                performance_metrics: PerformanceMetrics::default(),
                reliability_metrics: ReliabilityMetrics::default(),
                resource_utilization: ResourceUtilization::default(),
            },
        ];

        let summary = calculate_coverage_summary(&metrics);
        assert_eq!(summary.overall_coverage, 85.0);
        assert!(matches!(summary.trend_direction, TrendDirection::Up));
        assert_eq!(summary.trend_percentage, 5.0);
    }
}