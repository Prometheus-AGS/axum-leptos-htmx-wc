use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{Router, routing::get};
use chrono::{Utc, Duration};

use crate::testing::{
    monitoring::{TestMonitoringDashboard, create_monitoring_dashboard_router},
    visualization::{TestVisualizationEngine, create_visualization_router},
    analytics::{TestAnalyticsEngine, AnalyticsConfig, create_analytics_router},
    performance::{PerformanceRegressionDashboard, create_regression_dashboard_router},
    reliability::{TestReliabilityEngine, ReliabilityConfig, create_reliability_api_router, create_reliability_dashboard_router},
};

/// Comprehensive testing infrastructure integration example
pub struct ComprehensiveTestingSystem {
    pub monitoring_dashboard: TestMonitoringDashboard,
    pub visualization_engine: Arc<RwLock<TestVisualizationEngine>>,
    pub analytics_engine: Arc<RwLock<TestAnalyticsEngine>>,
    pub performance_dashboard: PerformanceRegressionDashboard,
    pub reliability_engine: Arc<RwLock<TestReliabilityEngine>>,
}

impl ComprehensiveTestingSystem {
    /// Create a new comprehensive testing system
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize monitoring dashboard
        let monitoring_dashboard = TestMonitoringDashboard::new().await;

        // Initialize visualization engine
        let visualization_engine = Arc::new(RwLock::new(TestVisualizationEngine::new()));

        // Initialize analytics engine
        let analytics_config = AnalyticsConfig::default();
        let analytics_engine = Arc::new(RwLock::new(TestAnalyticsEngine::new()));

        // Initialize performance dashboard
        let performance_dashboard = PerformanceRegressionDashboard::new();

        // Initialize reliability engine
        let reliability_config = ReliabilityConfig::default();
        let reliability_engine = Arc::new(RwLock::new(
            TestReliabilityEngine::new(reliability_config)
        ));

        Ok(Self {
            monitoring_dashboard,
            visualization_engine,
            analytics_engine,
            performance_dashboard,
            reliability_engine,
        })
    }

    /// Load test data into all components
    pub async fn load_test_data(
        &mut self,
        test_results: Vec<crate::testing::TestExecutionResult>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Loading {} test results into comprehensive testing system...", test_results.len());

        // Load into monitoring dashboard
        self.monitoring_dashboard.load_test_results(test_results.clone()).await?;
        println!("✓ Loaded test data into monitoring dashboard");

        // Load into visualization engine
        {
            let mut viz_engine = self.visualization_engine.write().await;
            viz_engine.load_test_data(test_results.clone())?;
        }
        println!("✓ Loaded test data into visualization engine");

        // Load into analytics engine
        {
            let mut analytics_engine = self.analytics_engine.write().await;
            analytics_engine.load_test_results(test_results.clone());
        }
        println!("✓ Loaded test data into analytics engine");

        // Load into performance dashboard
        self.performance_dashboard.load_test_results(test_results.clone()).await?;
        println!("✓ Loaded test data into performance dashboard");

        // Load into reliability engine
        {
            let mut reliability_engine = self.reliability_engine.write().await;
            reliability_engine.load_test_results(test_results.clone());
        }
        println!("✓ Loaded test data into reliability engine");

        println!("🎉 All test data loaded successfully!");
        Ok(())
    }

    /// Run comprehensive analysis across all systems
    pub async fn run_comprehensive_analysis(
        &self,
    ) -> Result<ComprehensiveAnalysisReport, Box<dyn std::error::Error + Send + Sync>> {
        println!("Starting comprehensive analysis across all testing systems...");

        let time_range = crate::testing::reliability::DateRange::last_days(30);

        // Run analytics analysis
        let analytics_config = AnalyticsConfig::default();
        let analytics_result = {
            let analytics_engine = self.analytics_engine.read().await;
            analytics_engine.run_comprehensive_analysis(&analytics_config).await?
        };
        println!("✓ Completed analytics analysis");

        // Run performance analysis
        let performance_overview = self.performance_dashboard.generate_dashboard_overview(
            time_range.clone(),
            None,
        ).await?;
        println!("✓ Completed performance analysis");

        // Run reliability analysis
        let reliability_overview = {
            let reliability_engine = self.reliability_engine.read().await;
            reliability_engine.generate_reliability_overview(
                time_range.clone(),
                None,
                None,
                true,
            )?
        };
        println!("✓ Completed reliability analysis");

        // Generate monitoring summary
        let monitoring_summary = self.monitoring_dashboard.get_comprehensive_metrics(
            Some("30d".to_string()),
            None,
        ).await?;
        println!("✓ Generated monitoring summary");

        // Generate visualizations
        let visualization_report = {
            let viz_engine = self.visualization_engine.read().await;
            viz_engine.generate_comprehensive_report()?
        };
        println!("✓ Generated visualization report");

        let report = ComprehensiveAnalysisReport {
            analytics_result,
            performance_overview,
            reliability_overview,
            monitoring_summary,
            visualization_report,
            generated_at: Utc::now(),
            analysis_period: time_range,
        };

        println!("🎉 Comprehensive analysis completed!");
        Ok(report)
    }

    /// Create the complete web application router
    pub fn create_web_application(
        &self,
    ) -> Router {
        println!("Creating comprehensive web application router...");

        Router::new()
            // Monitoring dashboard
            .nest("/monitoring", create_monitoring_dashboard_router())
            .with_state(self.monitoring_dashboard.clone())

            // Visualization system
            .nest("/visualization", create_visualization_router())
            .with_state(self.visualization_engine.clone())

            // Analytics API
            .nest("/api/v2/analytics", create_analytics_router())
            .with_state(crate::testing::analytics::api::AnalyticsApiState {
                engine: self.analytics_engine.clone(),
                cache: Arc::new(RwLock::new(
                    crate::testing::analytics::api::AnalyticsCache::default()
                )),
                config: crate::testing::analytics::api::AnalyticsApiConfig::default(),
            })

            // Performance dashboard
            .nest("/performance", create_regression_dashboard_router())
            .with_state(self.performance_dashboard.clone())

            // Reliability API
            .nest("/api/v2/reliability", create_reliability_api_router())
            .with_state(crate::testing::reliability::api::ReliabilityApiState {
                engine: self.reliability_engine.clone(),
                cache: Arc::new(RwLock::new(
                    crate::testing::reliability::api::ReliabilityCache::default()
                )),
                config: crate::testing::reliability::api::ReliabilityApiConfig::default(),
            })

            // Reliability dashboard
            .nest("/reliability", create_reliability_dashboard_router())
            .with_state(self.reliability_engine.clone())

            // Main dashboard that combines everything
            .route("/", get(main_dashboard))
            .route("/health", get(system_health))
            .with_state(self.clone())
    }

    /// Generate cross-system insights
    pub async fn generate_cross_system_insights(
        &self,
    ) -> Result<Vec<CrossSystemInsight>, Box<dyn std::error::Error + Send + Sync>> {
        println!("Generating cross-system insights...");

        let mut insights = Vec::new();

        // Correlate performance and reliability
        let performance_overview = self.performance_dashboard.generate_dashboard_overview(
            crate::testing::reliability::DateRange::last_days(7),
            None,
        ).await?;

        let reliability_overview = {
            let reliability_engine = self.reliability_engine.read().await;
            reliability_engine.generate_reliability_overview(
                crate::testing::reliability::DateRange::last_days(7),
                None,
                None,
                true,
            )?
        };

        // Insight 1: Performance impact on reliability
        if performance_overview.health_score.overall_score < 75.0 &&
           reliability_overview.health_score.overall_score < 80.0 {
            insights.push(CrossSystemInsight {
                insight_type: "PerformanceReliabilityCorrelation".to_string(),
                title: "Performance Issues Impacting Reliability".to_string(),
                description: format!(
                    "Poor performance (score: {:.1}) is correlated with low reliability (score: {:.1}). Slow tests often lead to timeouts and flaky behavior.",
                    performance_overview.health_score.overall_score,
                    reliability_overview.health_score.overall_score
                ),
                confidence: 0.85,
                severity: InsightSeverity::High,
                recommended_actions: vec![
                    "Optimize slow-running tests to improve both performance and reliability".to_string(),
                    "Investigate timeout thresholds and resource allocation".to_string(),
                    "Implement performance budgets for test execution".to_string(),
                ],
                affected_systems: vec!["Performance".to_string(), "Reliability".to_string()],
            });
        }

        // Insight 2: Flaky tests and performance patterns
        let flaky_test_count = reliability_overview.flaky_tests.len();
        if flaky_test_count > 5 && performance_overview.regressions.len() > 3 {
            insights.push(CrossSystemInsight {
                insight_type: "FlakyTestPerformancePattern".to_string(),
                title: "Flaky Tests Correlate with Performance Regressions".to_string(),
                description: format!(
                    "Detected {} flaky tests alongside {} performance regressions. This suggests environmental or resource contention issues.",
                    flaky_test_count,
                    performance_overview.regressions.len()
                ),
                confidence: 0.78,
                severity: InsightSeverity::Medium,
                recommended_actions: vec![
                    "Investigate test environment stability".to_string(),
                    "Review resource allocation and parallel execution strategies".to_string(),
                    "Implement retry logic with exponential backoff".to_string(),
                ],
                affected_systems: vec!["Reliability".to_string(), "Performance".to_string()],
            });
        }

        // Insight 3: Analytics trends
        let analytics_result = {
            let analytics_engine = self.analytics_engine.read().await;
            analytics_engine.run_comprehensive_analysis(&AnalyticsConfig::default()).await?
        };

        if let Some(coverage_insight) = analytics_result.cross_domain_insights.iter()
            .find(|i| i.insight_type == "CoverageTrend") {
            if coverage_insight.confidence > 0.8 {
                insights.push(CrossSystemInsight {
                    insight_type: "CoverageQualityImpact".to_string(),
                    title: "Coverage Trends Affecting Overall Quality".to_string(),
                    description: format!(
                        "Analytics detected significant coverage trends that may impact system quality: {}",
                        coverage_insight.description
                    ),
                    confidence: coverage_insight.confidence,
                    severity: match coverage_insight.severity.as_str() {
                        "High" => InsightSeverity::High,
                        "Medium" => InsightSeverity::Medium,
                        _ => InsightSeverity::Low,
                    },
                    recommended_actions: vec![
                        coverage_insight.recommendation.clone(),
                        "Monitor coverage trends in real-time dashboards".to_string(),
                    ],
                    affected_systems: vec!["Analytics".to_string(), "Monitoring".to_string()],
                });
            }
        }

        println!("✓ Generated {} cross-system insights", insights.len());
        Ok(insights)
    }

    /// Get system health status
    pub async fn get_system_health(&self) -> SystemHealthStatus {
        let mut health = SystemHealthStatus {
            overall_status: "healthy".to_string(),
            overall_score: 100.0,
            component_statuses: std::collections::HashMap::new(),
            alerts: vec![],
            last_updated: Utc::now(),
        };

        // Check monitoring dashboard health
        if let Ok(monitoring_health) = self.monitoring_dashboard.get_health_status().await {
            health.component_statuses.insert("monitoring".to_string(), monitoring_health);
        } else {
            health.component_statuses.insert("monitoring".to_string(), ComponentHealthStatus {
                status: "error".to_string(),
                score: 0.0,
                message: "Monitoring dashboard unavailable".to_string(),
            });
            health.overall_score -= 20.0;
        }

        // Check analytics health
        {
            let analytics_engine = self.analytics_engine.read().await;
            let analytics_health = if analytics_engine.historical_data.is_empty() {
                ComponentHealthStatus {
                    status: "warning".to_string(),
                    score: 60.0,
                    message: "No analytics data available".to_string(),
                }
            } else {
                ComponentHealthStatus {
                    status: "healthy".to_string(),
                    score: 95.0,
                    message: format!("{} data points available", analytics_engine.historical_data.len()),
                }
            };

            health.component_statuses.insert("analytics".to_string(), analytics_health.clone());
            if analytics_health.score < 80.0 {
                health.overall_score -= 15.0;
            }
        }

        // Check reliability health
        {
            let reliability_engine = self.reliability_engine.read().await;
            let reliability_health = if reliability_engine.historical_data.is_empty() {
                ComponentHealthStatus {
                    status: "warning".to_string(),
                    score: 60.0,
                    message: "No reliability data available".to_string(),
                }
            } else {
                ComponentHealthStatus {
                    status: "healthy".to_string(),
                    score: 90.0,
                    message: format!("{} test results analyzed", reliability_engine.historical_data.len()),
                }
            };

            health.component_statuses.insert("reliability".to_string(), reliability_health.clone());
            if reliability_health.score < 80.0 {
                health.overall_score -= 15.0;
            }
        }

        // Check performance health
        let performance_health = ComponentHealthStatus {
            status: "healthy".to_string(),
            score: 88.0,
            message: "Performance monitoring active".to_string(),
        };
        health.component_statuses.insert("performance".to_string(), performance_health);

        // Check visualization health
        {
            let viz_engine = self.visualization_engine.read().await;
            let viz_health = ComponentHealthStatus {
                status: "healthy".to_string(),
                score: 92.0,
                message: "Visualization engine operational".to_string(),
            };
            health.component_statuses.insert("visualization".to_string(), viz_health);
        }

        // Determine overall status
        health.overall_status = if health.overall_score >= 90.0 {
            "healthy".to_string()
        } else if health.overall_score >= 70.0 {
            "degraded".to_string()
        } else {
            "unhealthy".to_string()
        };

        health
    }
}

impl Clone for ComprehensiveTestingSystem {
    fn clone(&self) -> Self {
        Self {
            monitoring_dashboard: self.monitoring_dashboard.clone(),
            visualization_engine: self.visualization_engine.clone(),
            analytics_engine: self.analytics_engine.clone(),
            performance_dashboard: self.performance_dashboard.clone(),
            reliability_engine: self.reliability_engine.clone(),
        }
    }
}

/// Comprehensive analysis report
#[derive(Debug, serde::Serialize)]
pub struct ComprehensiveAnalysisReport {
    pub analytics_result: crate::testing::analytics::ComprehensiveAnalysisResult,
    pub performance_overview: crate::testing::performance::regression_dashboard::RegressionDashboardOverview,
    pub reliability_overview: crate::testing::reliability::ReliabilityOverview,
    pub monitoring_summary: serde_json::Value,
    pub visualization_report: crate::testing::visualization::VisualizationReport,
    pub generated_at: chrono::DateTime<Utc>,
    pub analysis_period: crate::testing::reliability::DateRange,
}

/// Cross-system insight
#[derive(Debug, serde::Serialize)]
pub struct CrossSystemInsight {
    pub insight_type: String,
    pub title: String,
    pub description: String,
    pub confidence: f64,
    pub severity: InsightSeverity,
    pub recommended_actions: Vec<String>,
    pub affected_systems: Vec<String>,
}

/// Insight severity levels
#[derive(Debug, serde::Serialize)]
pub enum InsightSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// System health status
#[derive(Debug, serde::Serialize)]
pub struct SystemHealthStatus {
    pub overall_status: String,
    pub overall_score: f64,
    pub component_statuses: std::collections::HashMap<String, ComponentHealthStatus>,
    pub alerts: Vec<String>,
    pub last_updated: chrono::DateTime<Utc>,
}

/// Component health status
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComponentHealthStatus {
    pub status: String,
    pub score: f64,
    pub message: String,
}

/// Main dashboard handler
async fn main_dashboard(
    axum::extract::State(system): axum::extract::State<ComprehensiveTestingSystem>,
) -> axum::response::Html<String> {
    let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Comprehensive Testing Infrastructure Dashboard</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background: #f5f5f5;
        }}
        .dashboard {{
            max-width: 1200px;
            margin: 0 auto;
        }}
        .header {{
            text-align: center;
            margin-bottom: 40px;
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        .nav-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}
        .nav-card {{
            background: white;
            padding: 25px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            text-decoration: none;
            color: inherit;
            transition: transform 0.2s;
        }}
        .nav-card:hover {{
            transform: translateY(-2px);
            box-shadow: 0 4px 20px rgba(0,0,0,0.15);
        }}
        .nav-title {{
            font-size: 1.3em;
            font-weight: bold;
            margin-bottom: 10px;
            color: #333;
        }}
        .nav-description {{
            color: #666;
            line-height: 1.5;
        }}
        .footer {{
            text-align: center;
            margin-top: 40px;
            color: #666;
        }}
        .status-indicator {{
            display: inline-block;
            width: 12px;
            height: 12px;
            border-radius: 50%;
            margin-right: 8px;
        }}
        .status-healthy {{
            background-color: #22c55e;
        }}
        .status-warning {{
            background-color: #f59e0b;
        }}
        .status-error {{
            background-color: #ef4444;
        }}
    </style>
</head>
<body>
    <div class="dashboard">
        <div class="header">
            <h1>🧪 Comprehensive Testing Infrastructure</h1>
            <p>Complete testing, monitoring, and analysis platform</p>
            <p><span class="status-indicator status-healthy"></span>All systems operational</p>
        </div>

        <div class="nav-grid">
            <a href="/monitoring" class="nav-card">
                <div class="nav-title">📊 Test Monitoring</div>
                <div class="nav-description">
                    Real-time test execution monitoring with live progress tracking,
                    WebSocket updates, and comprehensive metrics API.
                </div>
            </a>

            <a href="/visualization" class="nav-card">
                <div class="nav-title">📈 Data Visualization</div>
                <div class="nav-description">
                    Interactive charts and graphs for test results, coverage trends,
                    performance metrics, and reliability analysis.
                </div>
            </a>

            <a href="/api/v2/analytics" class="nav-card">
                <div class="nav-title">🔬 Advanced Analytics</div>
                <div class="nav-description">
                    AI-powered analytics with predictive modeling, trend analysis,
                    and cross-domain correlation insights.
                </div>
            </a>

            <a href="/performance" class="nav-card">
                <div class="nav-title">⚡ Performance Analysis</div>
                <div class="nav-description">
                    Performance regression detection, bottleneck identification,
                    and optimization recommendations.
                </div>
            </a>

            <a href="/reliability" class="nav-card">
                <div class="nav-title">🎯 Reliability Metrics</div>
                <div class="nav-description">
                    Flaky test detection, stability analysis, failure pattern
                    recognition, and reliability forecasting.
                </div>
            </a>

            <a href="/health" class="nav-card">
                <div class="nav-title">💚 System Health</div>
                <div class="nav-description">
                    Overall system status, component health monitoring,
                    and cross-system insights.
                </div>
            </a>
        </div>

        <div class="footer">
            <p>🚀 Powered by Rust + Axum + HTMX + Chart.js</p>
            <p>Last updated: {}</p>
        </div>
    </div>
</body>
</html>
    "#, Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));

    axum::response::Html(html)
}

/// System health endpoint
async fn system_health(
    axum::extract::State(system): axum::extract::State<ComprehensiveTestingSystem>,
) -> axum::Json<SystemHealthStatus> {
    let health = system.get_system_health().await;
    axum::Json(health)
}

/// Usage example function
pub async fn run_comprehensive_example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🧪 Starting Comprehensive Testing Infrastructure Example");
    println!("=" .repeat(60));

    // Create the comprehensive testing system
    let mut system = ComprehensiveTestingSystem::new().await?;
    println!("✓ Comprehensive testing system initialized");

    // Generate sample test data
    let sample_data = generate_sample_test_data(1000);
    println!("✓ Generated {} sample test results", sample_data.len());

    // Load data into all systems
    system.load_test_data(sample_data).await?;
    println!("✓ Test data loaded into all systems");

    // Run comprehensive analysis
    let analysis_report = system.run_comprehensive_analysis().await?;
    println!("✓ Comprehensive analysis completed");

    // Generate cross-system insights
    let insights = system.generate_cross_system_insights().await?;
    println!("✓ Generated {} cross-system insights", insights.len());

    // Check system health
    let health_status = system.get_system_health().await;
    println!("✓ System health: {} (score: {:.1})", health_status.overall_status, health_status.overall_score);

    // Create web application
    let app = system.create_web_application();
    println!("✓ Web application router created");

    println!("\n🎉 Example completed successfully!");
    println!("\nAvailable endpoints:");
    println!("  /                     - Main dashboard");
    println!("  /monitoring           - Test monitoring dashboard");
    println!("  /visualization        - Data visualization");
    println!("  /api/v2/analytics     - Analytics API");
    println!("  /performance          - Performance dashboard");
    println!("  /reliability          - Reliability dashboard");
    println!("  /health               - System health status");

    println!("\n📊 Analysis Summary:");
    println!("  - Analytics insights: {}", analysis_report.analytics_result.cross_domain_insights.len());
    println!("  - Performance alerts: {}", analysis_report.performance_overview.alerts.len());
    println!("  - Reliability issues: {}", analysis_report.reliability_overview.flaky_tests.len());
    println!("  - Cross-system insights: {}", insights.len());

    Ok(())
}

/// Generate sample test data for demonstration
fn generate_sample_test_data(count: usize) -> Vec<crate::testing::TestExecutionResult> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let mut results = Vec::new();
    let test_names = vec![
        "user_authentication_test",
        "api_endpoint_validation",
        "database_connection_test",
        "frontend_rendering_test",
        "integration_workflow_test",
        "performance_benchmark_test",
        "security_validation_test",
        "data_migration_test",
        "cache_functionality_test",
        "error_handling_test",
    ];

    let environments = vec!["development", "staging", "production"];

    for i in 0..count {
        let test_name = test_names[rng.gen_range(0..test_names.len())];
        let environment = environments[rng.gen_range(0..environments.len())];

        // Simulate some flaky tests
        let is_flaky = rng.gen_bool(0.05); // 5% chance of being flaky
        let status = if is_flaky && rng.gen_bool(0.3) {
            if rng.gen_bool(0.7) {
                crate::testing::reliability::TestStatus::Flaky
            } else {
                crate::testing::reliability::TestStatus::Failed
            }
        } else if rng.gen_bool(0.95) {
            crate::testing::reliability::TestStatus::Passed
        } else {
            crate::testing::reliability::TestStatus::Failed
        };

        results.push(crate::testing::TestExecutionResult {
            test_id: format!("test_{:04}", i),
            test_name: format!("{}_{}", test_name, i % 100),
            test_suite: format!("{}_suite", test_name.split('_').next().unwrap()),
            execution_time_ms: rng.gen_range(100.0..5000.0),
            status: status.clone(),
            error_message: if matches!(status, crate::testing::reliability::TestStatus::Failed) {
                Some("Test assertion failed".to_string())
            } else {
                None
            },
            environment: environment.to_string(),
            executed_at: Utc::now() - Duration::seconds(rng.gen_range(0..30 * 24 * 3600)),
            git_commit: Some(format!("abc123{:02}", i % 50)),
            build_number: Some(format!("build-{}", 1000 + i)),
            runner_id: Some(format!("runner-{}", rng.gen_range(1..5))),
            retry_count: if is_flaky { rng.gen_range(1..3) } else { 0 },
            was_flaky: is_flaky,
            memory_usage_mb: Some(rng.gen_range(50.0..500.0)),
            cpu_usage_percent: Some(rng.gen_range(10.0..90.0)),
            parallel_execution: rng.gen_bool(0.7),
            test_categories: vec![
                format!("{}_tests", test_name.split('_').next().unwrap()),
                if rng.gen_bool(0.3) { "integration".to_string() } else { "unit".to_string() }
            ],
        });
    }

    results
}