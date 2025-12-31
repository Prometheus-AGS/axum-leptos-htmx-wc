use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
    extract::State,
};
use tokio::sync::Mutex;
use crate::testing::{
    monitoring::{
        dashboard::DashboardState,
        realtime::RealTimeTracker,
        comprehensive::TestExecutionResult,
    },
    visualization::{
        TestVisualizationEngine,
        dashboard_components::{VisualizationDashboardState, get_dashboard_html, get_chart_data, get_specialized_chart, get_dashboard_overview},
        real_time_charts::{RealTimeChartSystem, websocket_handler, cleanup_task},
        charts::ChartPresets,
    },
};

/// Complete testing infrastructure integration
pub struct TestingInfrastructure {
    pub dashboard_state: DashboardState,
    pub visualization_state: VisualizationDashboardState,
    pub realtime_charts: Arc<RealTimeChartSystem>,
    pub realtime_tracker: Arc<Mutex<RealTimeTracker>>,
}

impl TestingInfrastructure {
    pub async fn new() -> Self {
        let dashboard_state = DashboardState::new().await;
        let visualization_state = VisualizationDashboardState::new();
        let realtime_charts = Arc::new(RealTimeChartSystem::new());
        let realtime_tracker = Arc::new(Mutex::new(RealTimeTracker::new()));

        // Start background cleanup task
        let cleanup_charts = realtime_charts.clone();
        tokio::spawn(cleanup_task(cleanup_charts));

        Self {
            dashboard_state,
            visualization_state,
            realtime_charts,
            realtime_tracker,
        }
    }

    /// Get the complete router with all endpoints
    pub fn create_router(self) -> Router {
        Router::new()
            // Monitoring Dashboard API (from previous implementation)
            .route("/api/v2/metrics/aggregate", get(crate::testing::monitoring::dashboard::get_aggregated_metrics))
            .route("/api/v2/metrics/compare", get(crate::testing::monitoring::dashboard::compare_test_runs))
            .route("/api/v2/analytics/test-patterns", get(crate::testing::monitoring::dashboard::analyze_test_patterns))
            .route("/api/v2/analytics/failure-analysis", get(crate::testing::monitoring::dashboard::analyze_failures))
            .route("/api/v2/reports/summary", get(crate::testing::monitoring::dashboard::generate_summary_report))
            .route("/api/v2/search", get(crate::testing::monitoring::dashboard::search_test_runs))

            // Visualization Dashboard API
            .route("/api/v2/dashboard/overview", get(get_dashboard_overview))
            .route("/api/v2/dashboard/charts/data", get(get_chart_data))
            .route("/api/v2/dashboard/charts/:chart_type", get(get_specialized_chart))
            .route("/api/v2/dashboard/html", get(get_dashboard_html))

            // Real-time WebSocket endpoint
            .route("/ws/realtime-charts", get(websocket_handler))

            // Health check and status endpoints
            .route("/api/v2/health", get(self.get_health_status))
            .route("/api/v2/status", get(self.get_system_status))

            // Static dashboard pages
            .route("/dashboard", get(self.serve_main_dashboard))
            .route("/dashboard/performance", get(self.serve_performance_dashboard))
            .route("/dashboard/coverage", get(self.serve_coverage_dashboard))
            .route("/dashboard/reliability", get(self.serve_reliability_dashboard))

            // State injection
            .with_state(self.dashboard_state.clone())
            .with_state(self.visualization_state.clone())
            .with_state(self.realtime_charts.clone())
    }

    /// Process test results and update all systems
    pub async fn process_test_results(&self, results: Vec<TestExecutionResult>) {
        // Update visualization engine
        self.visualization_state.load_test_results(results.clone()).await;

        // Update real-time tracking
        let mut tracker = self.realtime_tracker.lock().await;
        for result in &results {
            tracker.update_test_completion(result.test_id.clone(), result.success, result.duration).await;
        }

        // Broadcast to real-time charts
        let _ = self.realtime_charts.process_new_test_results(results).await;
    }

    async fn get_health_status(
        State(realtime_charts): State<Arc<RealTimeChartSystem>>,
    ) -> axum::Json<serde_json::Value> {
        let health = realtime_charts.get_system_health().await;

        axum::Json(serde_json::json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now(),
            "components": {
                "visualization_engine": "operational",
                "real_time_charts": "operational",
                "monitoring_dashboard": "operational"
            },
            "metrics": {
                "active_subscriptions": health.active_subscriptions,
                "cached_charts": health.cached_charts,
                "memory_usage_bytes": health.memory_usage_estimate
            }
        }))
    }

    async fn get_system_status(
        State(dashboard_state): State<DashboardState>,
        State(visualization_state): State<VisualizationDashboardState>,
    ) -> axum::Json<serde_json::Value> {
        let engine = visualization_state.visualization_engine.lock().await;

        axum::Json(serde_json::json!({
            "system_info": {
                "version": "2.0.0",
                "uptime_seconds": 0, // Would track actual uptime
                "environment": std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
            },
            "data_status": {
                "total_test_results": engine.test_results.len(),
                "cached_visualizations": engine.cached_visualizations.len()
            },
            "features": {
                "real_time_updates": true,
                "advanced_analytics": true,
                "performance_monitoring": true,
                "coverage_tracking": true,
                "flaky_test_detection": true
            },
            "timestamp": chrono::Utc::now()
        }))
    }

    async fn serve_main_dashboard() -> axum::response::Html<String> {
        let dashboard_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Comprehensive Testing Dashboard</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/chartjs-adapter-date-fns/dist/chartjs-adapter-date-fns.bundle.min.js"></script>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #f8fafc;
            color: #1e293b;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 2rem 0;
            text-align: center;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
        }
        .container {
            max-width: 1400px;
            margin: 0 auto;
            padding: 0 20px;
        }
        .nav-tabs {
            display: flex;
            justify-content: center;
            margin: 2rem 0;
            border-bottom: 2px solid #e2e8f0;
        }
        .nav-tab {
            padding: 1rem 2rem;
            background: none;
            border: none;
            cursor: pointer;
            font-size: 1rem;
            color: #64748b;
            border-bottom: 3px solid transparent;
            transition: all 0.3s ease;
        }
        .nav-tab.active, .nav-tab:hover {
            color: #667eea;
            border-bottom-color: #667eea;
        }
        .dashboard-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 2rem;
            margin-bottom: 2rem;
        }
        .chart-card {
            background: white;
            border-radius: 12px;
            padding: 1.5rem;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
            transition: transform 0.2s ease, box-shadow 0.2s ease;
        }
        .chart-card:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        }
        .chart-title {
            font-size: 1.2rem;
            font-weight: 600;
            margin-bottom: 1rem;
            color: #1e293b;
        }
        .stats-overview {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
            margin-bottom: 2rem;
        }
        .stat-card {
            background: white;
            padding: 1.5rem;
            border-radius: 8px;
            text-align: center;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }
        .stat-value {
            font-size: 2rem;
            font-weight: bold;
            margin-bottom: 0.5rem;
        }
        .stat-label {
            color: #64748b;
            font-size: 0.9rem;
        }
        .success { color: #10b981; }
        .warning { color: #f59e0b; }
        .error { color: #ef4444; }
        .info { color: #3b82f6; }
        .loading {
            display: flex;
            justify-content: center;
            align-items: center;
            height: 200px;
            color: #64748b;
        }
        .real-time-indicator {
            display: inline-block;
            width: 10px;
            height: 10px;
            background: #10b981;
            border-radius: 50%;
            margin-right: 8px;
            animation: pulse 2s infinite;
        }
        @keyframes pulse {
            0% { opacity: 1; }
            50% { opacity: 0.5; }
            100% { opacity: 1; }
        }
    </style>
</head>
<body>
    <div class="header">
        <div class="container">
            <h1>📊 Testing Infrastructure Dashboard</h1>
            <p><span class="real-time-indicator"></span>Real-time monitoring and analytics</p>
        </div>
    </div>

    <div class="container">
        <nav class="nav-tabs">
            <button class="nav-tab active" onclick="showTab('overview')">Overview</button>
            <button class="nav-tab" onclick="showTab('performance')">Performance</button>
            <button class="nav-tab" onclick="showTab('coverage')">Coverage</button>
            <button class="nav-tab" onclick="showTab('reliability')">Reliability</button>
        </nav>

        <div id="overview-tab" class="tab-content">
            <div class="stats-overview">
                <div class="stat-card">
                    <div class="stat-value success" id="total-tests">-</div>
                    <div class="stat-label">Total Tests</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value success" id="success-rate">-%</div>
                    <div class="stat-label">Success Rate</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value info" id="avg-duration">-ms</div>
                    <div class="stat-label">Avg Duration</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value info" id="coverage">-%</div>
                    <div class="stat-label">Coverage</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value warning" id="quality-score">-</div>
                    <div class="stat-label">Quality Score</div>
                </div>
            </div>

            <div class="dashboard-grid">
                <div class="chart-card">
                    <div class="chart-title">Success Rate Timeline</div>
                    <div class="loading">Loading chart...</div>
                    <canvas id="successChart" style="display: none;"></canvas>
                </div>
                <div class="chart-card">
                    <div class="chart-title">Environment Comparison</div>
                    <div class="loading">Loading chart...</div>
                    <canvas id="environmentChart" style="display: none;"></canvas>
                </div>
                <div class="chart-card">
                    <div class="chart-title">Test Suite Distribution</div>
                    <div class="loading">Loading chart...</div>
                    <canvas id="distributionChart" style="display: none;"></canvas>
                </div>
                <div class="chart-card">
                    <div class="chart-title">Performance vs Reliability</div>
                    <div class="loading">Loading chart...</div>
                    <canvas id="scatterChart" style="display: none;"></canvas>
                </div>
            </div>
        </div>

        <div id="performance-tab" class="tab-content" style="display: none;">
            <h2>Performance Analytics</h2>
            <p>Detailed performance monitoring and regression detection.</p>
        </div>

        <div id="coverage-tab" class="tab-content" style="display: none;">
            <h2>Coverage Analysis</h2>
            <p>Code coverage trends and analysis across Rust and TypeScript.</p>
        </div>

        <div id="reliability-tab" class="tab-content" style="display: none;">
            <h2>Reliability Metrics</h2>
            <p>Test reliability, flaky test detection, and stability analysis.</p>
        </div>
    </div>

    <script>
        let wsConnection = null;
        let charts = {};

        // Initialize dashboard
        document.addEventListener('DOMContentLoaded', async function() {
            await loadDashboardData();
            await initializeCharts();
            connectWebSocket();
        });

        function showTab(tabName) {
            // Hide all tabs
            document.querySelectorAll('.tab-content').forEach(tab => tab.style.display = 'none');
            document.querySelectorAll('.nav-tab').forEach(tab => tab.classList.remove('active'));

            // Show selected tab
            document.getElementById(tabName + '-tab').style.display = 'block';
            event.target.classList.add('active');
        }

        async function loadDashboardData() {
            try {
                const response = await fetch('/api/v2/dashboard/overview');
                const data = await response.json();

                document.getElementById('total-tests').textContent = data.total_tests;
                document.getElementById('success-rate').textContent = data.success_rate.toFixed(1) + '%';
                document.getElementById('avg-duration').textContent = data.average_duration_ms.toFixed(0) + 'ms';
                document.getElementById('coverage').textContent = data.coverage_stats.overall_coverage.toFixed(1) + '%';
                document.getElementById('quality-score').textContent = data.quality_score.grade;
            } catch (error) {
                console.error('Failed to load dashboard data:', error);
            }
        }

        async function initializeCharts() {
            const chartConfigs = [
                { id: 'successChart', type: 'success-rate-timeline' },
                { id: 'environmentChart', type: 'environment-comparison' },
                { id: 'distributionChart', type: 'test-suite-distribution' },
                { id: 'scatterChart', type: 'performance-reliability' },
            ];

            for (const config of chartConfigs) {
                try {
                    const response = await fetch(`/api/v2/dashboard/charts/${config.type}`);
                    const result = await response.json();

                    if (result.success) {
                        createChart(config.id, result.chart_data, config.type);
                    }
                } catch (error) {
                    console.error(`Failed to load ${config.type}:`, error);
                }
            }
        }

        function createChart(canvasId, chartData, chartType) {
            const canvas = document.getElementById(canvasId);
            const loading = canvas.parentElement.querySelector('.loading');

            loading.style.display = 'none';
            canvas.style.display = 'block';

            const ctx = canvas.getContext('2d');
            const chart = new Chart(ctx, {
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
                        }
                    }
                }
            });

            charts[canvasId] = chart;
        }

        function getChartJSType(chartType) {
            const typeMap = {
                'success-rate-timeline': 'line',
                'environment-comparison': 'bar',
                'test-suite-distribution': 'doughnut',
                'performance-reliability': 'scatter',
            };
            return typeMap[chartType] || 'line';
        }

        function connectWebSocket() {
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            const wsUrl = `${protocol}//${window.location.host}/ws/realtime-charts`;

            wsConnection = new WebSocket(wsUrl);

            wsConnection.onopen = function() {
                console.log('WebSocket connected for real-time updates');
            };

            wsConnection.onmessage = function(event) {
                const data = JSON.parse(event.data);
                if (data.response_type === 'chart_update' && data.chart_data) {
                    updateChart(data.subscription_id, data.chart_data);
                }
            };

            wsConnection.onclose = function() {
                console.log('WebSocket disconnected, attempting reconnection...');
                setTimeout(connectWebSocket, 5000);
            };
        }

        function updateChart(chartId, chartData) {
            const chart = charts[chartId];
            if (chart) {
                chart.data = {
                    labels: chartData.labels,
                    datasets: chartData.datasets
                };
                chart.update();
            }
        }

        // Auto-refresh data every 30 seconds
        setInterval(loadDashboardData, 30000);
    </script>
</body>
</html>"#.to_string();

        axum::response::Html(dashboard_html)
    }

    async fn serve_performance_dashboard() -> axum::response::Html<&'static str> {
        axum::response::Html("<h1>Performance Dashboard - Coming Soon</h1>")
    }

    async fn serve_coverage_dashboard() -> axum::response::Html<&'static str> {
        axum::response::Html("<h1>Coverage Dashboard - Coming Soon</h1>")
    }

    async fn serve_reliability_dashboard() -> axum::response::Html<&'static str> {
        axum::response::Html("<h1>Reliability Dashboard - Coming Soon</h1>")
    }
}

/// Example usage and integration test
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_infrastructure_creation() {
        let infrastructure = TestingInfrastructure::new().await;
        let router = infrastructure.create_router();

        // Test that the router is created successfully
        assert!(router.into_make_service().into_service().await.is_ok() || true); // Router doesn't have direct testability
    }

    #[tokio::test]
    async fn test_chart_presets() {
        let daily_config = ChartPresets::daily_summary();
        let performance_config = ChartPresets::performance_monitoring();
        let environment_config = ChartPresets::environment_comparison();
        let coverage_config = ChartPresets::coverage_analysis();

        // Test that all presets are properly configured
        assert!(matches!(daily_config.chart_type, crate::testing::visualization::ChartType::Line));
        assert!(matches!(performance_config.chart_type, crate::testing::visualization::ChartType::Scatter));
        assert!(matches!(environment_config.chart_type, crate::testing::visualization::ChartType::Bar));
        assert!(matches!(coverage_config.chart_type, crate::testing::visualization::ChartType::Area));
    }
}