use axum::{
    extract::{Query, State, Path},
    response::{Html, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use super::{
    TestReliabilityEngine, ReliabilityOverview, ReliabilityHealthScore,
    FlakyTestSummary, FailurePatternSummary, DateRange, Priority,
};

/// Reliability dashboard
#[derive(Debug, Clone)]
pub struct ReliabilityDashboard {
    pub engine: Arc<RwLock<TestReliabilityEngine>>,
    pub dashboard_config: DashboardConfig,
}

/// Dashboard configuration
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    pub title: String,
    pub refresh_interval_seconds: u32,
    pub max_flaky_tests_display: usize,
    pub max_patterns_display: usize,
    pub chart_colors: ChartColorScheme,
    pub enable_real_time: bool,
}

/// Chart color scheme
#[derive(Debug, Clone)]
pub struct ChartColorScheme {
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
    pub primary: String,
    pub secondary: String,
}

/// Dashboard overview data
#[derive(Debug, Serialize)]
pub struct ReliabilityDashboardOverview {
    pub health_score: ReliabilityHealthScore,
    pub summary_stats: DashboardSummaryStats,
    pub recent_flaky_tests: Vec<FlakyTestSummary>,
    pub critical_patterns: Vec<FailurePatternSummary>,
    pub trend_data: Vec<TrendDataPoint>,
    pub environment_comparison: Vec<EnvironmentComparison>,
    pub alerts_summary: AlertsSummary,
    pub generated_at: DateTime<Utc>,
}

/// Summary statistics for dashboard
#[derive(Debug, Serialize)]
pub struct DashboardSummaryStats {
    pub total_tests: usize,
    pub total_executions: usize,
    pub flaky_test_count: usize,
    pub success_rate: f64,
    pub average_stability_score: f64,
    pub critical_issues: usize,
    pub improvement_opportunities: usize,
    pub time_period: String,
}

/// Trend data point for charts
#[derive(Debug, Serialize)]
pub struct TrendDataPoint {
    pub timestamp: DateTime<Utc>,
    pub success_rate: f64,
    pub flaky_rate: f64,
    pub stability_score: f64,
    pub total_executions: usize,
}

/// Environment comparison data
#[derive(Debug, Serialize)]
pub struct EnvironmentComparison {
    pub environment: String,
    pub success_rate: f64,
    pub stability_score: f64,
    pub flaky_test_count: usize,
    pub total_tests: usize,
    pub issues: Vec<String>,
}

/// Alerts summary
#[derive(Debug, Serialize)]
pub struct AlertsSummary {
    pub critical_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub recent_alerts: Vec<RecentAlert>,
}

/// Recent alert for dashboard
#[derive(Debug, Serialize)]
pub struct RecentAlert {
    pub alert_type: String,
    pub severity: String,
    pub title: String,
    pub timestamp: DateTime<Utc>,
    pub affected_tests: usize,
}

/// Dashboard query parameters
#[derive(Debug, Deserialize)]
pub struct DashboardQuery {
    pub environment: Option<String>,
    pub time_window: Option<String>,
    pub theme: Option<String>,
}

/// Dashboard generator
pub struct DashboardGenerator;

impl ReliabilityDashboard {
    /// Create new reliability dashboard
    pub fn new(engine: Arc<RwLock<TestReliabilityEngine>>) -> Self {
        Self {
            engine,
            dashboard_config: DashboardConfig::default(),
        }
    }

    /// Generate dashboard overview
    pub async fn generate_overview(
        &self,
        time_range: DateRange,
        environment: Option<&str>,
    ) -> Result<ReliabilityDashboardOverview, Box<dyn std::error::Error + Send + Sync>> {
        let engine = self.engine.read().await;

        // Get comprehensive reliability overview
        let overview = engine.generate_reliability_overview(
            time_range.clone(),
            environment,
            None,
            true,
        )?;

        // Calculate summary statistics
        let summary_stats = self.calculate_summary_stats(&engine, &time_range, environment).await?;

        // Get trend data for charts
        let trend_data = self.generate_trend_data(&engine, &time_range, environment).await?;

        // Get environment comparison
        let environment_comparison = self.generate_environment_comparison(&engine, &time_range).await?;

        // Generate alerts summary
        let alerts_summary = self.generate_alerts_summary(&engine).await?;

        // Limit displayed items
        let recent_flaky_tests = overview.flaky_tests
            .into_iter()
            .take(self.dashboard_config.max_flaky_tests_display)
            .collect();

        let critical_patterns = overview.failure_patterns
            .into_iter()
            .filter(|p| matches!(p.severity, super::PatternSeverity::Critical | super::PatternSeverity::High))
            .take(self.dashboard_config.max_patterns_display)
            .collect();

        Ok(ReliabilityDashboardOverview {
            health_score: overview.health_score,
            summary_stats,
            recent_flaky_tests,
            critical_patterns,
            trend_data,
            environment_comparison,
            alerts_summary,
            generated_at: Utc::now(),
        })
    }

    /// Calculate summary statistics
    async fn calculate_summary_stats(
        &self,
        engine: &TestReliabilityEngine,
        time_range: &DateRange,
        environment: Option<&str>,
    ) -> Result<DashboardSummaryStats, Box<dyn std::error::Error + Send + Sync>> {
        let filtered_results: Vec<_> = engine.historical_data
            .iter()
            .filter(|result| {
                time_range.contains(&result.executed_at) &&
                environment.map_or(true, |env| result.environment == env)
            })
            .collect();

        let total_tests = filtered_results.iter()
            .map(|r| r.test_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .len();

        let total_executions = filtered_results.len();

        let successful_executions = filtered_results.iter()
            .filter(|r| r.status.is_success())
            .count();

        let success_rate = if total_executions > 0 {
            (successful_executions as f64 / total_executions as f64) * 100.0
        } else {
            0.0
        };

        let flaky_tests = engine.identify_flaky_tests(time_range.clone(), environment)?;
        let flaky_test_count = flaky_tests.len();

        let average_stability_score = flaky_tests.iter()
            .map(|t| t.consistency_score)
            .sum::<f64>() / flaky_tests.len().max(1) as f64;

        let patterns = engine.detect_failure_patterns(time_range.clone(), environment)?;
        let critical_issues = patterns.iter()
            .filter(|p| matches!(p.severity, super::PatternSeverity::Critical))
            .count();

        let improvement_opportunities = patterns.iter()
            .filter(|p| matches!(p.severity, super::PatternSeverity::High | super::PatternSeverity::Medium))
            .count();

        let time_period = format!("{} to {}",
            time_range.start.format("%Y-%m-%d %H:%M"),
            time_range.end.format("%Y-%m-%d %H:%M")
        );

        Ok(DashboardSummaryStats {
            total_tests,
            total_executions,
            flaky_test_count,
            success_rate,
            average_stability_score,
            critical_issues,
            improvement_opportunities,
            time_period,
        })
    }

    /// Generate trend data for charts
    async fn generate_trend_data(
        &self,
        engine: &TestReliabilityEngine,
        time_range: &DateRange,
        environment: Option<&str>,
    ) -> Result<Vec<TrendDataPoint>, Box<dyn std::error::Error + Send + Sync>> {
        let trends = engine.get_reliability_trends(
            "comprehensive",
            time_range.clone(),
            Some("daily"),
            environment,
        )?;

        let trend_data = trends.into_iter()
            .map(|trend| TrendDataPoint {
                timestamp: trend.timestamp,
                success_rate: trend.success_rate * 100.0,
                flaky_rate: trend.flaky_rate * 100.0,
                stability_score: trend.stability_score,
                total_executions: trend.total_tests,
            })
            .collect();

        Ok(trend_data)
    }

    /// Generate environment comparison
    async fn generate_environment_comparison(
        &self,
        engine: &TestReliabilityEngine,
        time_range: &DateRange,
    ) -> Result<Vec<EnvironmentComparison>, Box<dyn std::error::Error + Send + Sync>> {
        let environments: std::collections::HashSet<String> = engine.historical_data
            .iter()
            .filter(|result| time_range.contains(&result.executed_at))
            .map(|result| result.environment.clone())
            .collect();

        let mut comparisons = Vec::new();

        for env in environments {
            let env_results: Vec<_> = engine.historical_data
                .iter()
                .filter(|result| {
                    time_range.contains(&result.executed_at) && result.environment == env
                })
                .collect();

            let total_tests = env_results.iter()
                .map(|r| r.test_id.clone())
                .collect::<std::collections::HashSet<_>>()
                .len();

            let successful = env_results.iter()
                .filter(|r| r.status.is_success())
                .count();

            let success_rate = if env_results.len() > 0 {
                (successful as f64 / env_results.len() as f64) * 100.0
            } else {
                0.0
            };

            let flaky_tests = engine.identify_flaky_tests(time_range.clone(), Some(&env))?;
            let flaky_test_count = flaky_tests.len();
            let stability_score = flaky_tests.iter()
                .map(|t| t.consistency_score)
                .sum::<f64>() / flaky_tests.len().max(1) as f64;

            // Identify key issues for this environment
            let patterns = engine.detect_failure_patterns(time_range.clone(), Some(&env))?;
            let issues: Vec<String> = patterns.iter()
                .filter(|p| matches!(p.severity, super::PatternSeverity::Critical | super::PatternSeverity::High))
                .take(3)
                .map(|p| p.description.clone())
                .collect();

            comparisons.push(EnvironmentComparison {
                environment: env,
                success_rate,
                stability_score,
                flaky_test_count,
                total_tests,
                issues,
            });
        }

        // Sort by success rate descending
        comparisons.sort_by(|a, b| b.success_rate.partial_cmp(&a.success_rate).unwrap());

        Ok(comparisons)
    }

    /// Generate alerts summary
    async fn generate_alerts_summary(
        &self,
        engine: &TestReliabilityEngine,
    ) -> Result<AlertsSummary, Box<dyn std::error::Error + Send + Sync>> {
        let alerts = engine.get_active_alerts()?;

        let critical_count = alerts.iter()
            .filter(|a| matches!(a.severity, super::api::AlertSeverity::Critical))
            .count();

        let warning_count = alerts.iter()
            .filter(|a| matches!(a.severity, super::api::AlertSeverity::Warning))
            .count();

        let info_count = alerts.iter()
            .filter(|a| matches!(a.severity, super::api::AlertSeverity::Info))
            .count();

        let recent_alerts: Vec<RecentAlert> = alerts.iter()
            .take(5)
            .map(|alert| RecentAlert {
                alert_type: format!("{:?}", alert.alert_type),
                severity: format!("{:?}", alert.severity),
                title: alert.title.clone(),
                timestamp: alert.first_detected,
                affected_tests: alert.affected_tests.len(),
            })
            .collect();

        Ok(AlertsSummary {
            critical_count,
            warning_count,
            info_count,
            recent_alerts,
        })
    }
}

impl DashboardGenerator {
    /// Generate HTML dashboard
    pub fn generate_html_dashboard(
        overview: &ReliabilityDashboardOverview,
        config: &DashboardConfig,
        theme: Option<&str>,
    ) -> String {
        let theme_class = match theme {
            Some("dark") => "dark-theme",
            Some("light") => "light-theme",
            _ => "default-theme",
        };

        format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        .{theme_class} {{
            --bg-primary: #ffffff;
            --bg-secondary: #f8f9fa;
            --text-primary: #212529;
            --text-secondary: #6c757d;
            --border-color: #dee2e6;
            --success-color: {success_color};
            --warning-color: {warning_color};
            --error-color: {error_color};
            --info-color: {info_color};
        }}

        .dark-theme {{
            --bg-primary: #1a1a1a;
            --bg-secondary: #2d2d2d;
            --text-primary: #ffffff;
            --text-secondary: #cccccc;
            --border-color: #404040;
        }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: var(--bg-primary);
            color: var(--text-primary);
        }}

        .dashboard {{
            max-width: 1400px;
            margin: 0 auto;
        }}

        .header {{
            text-align: center;
            margin-bottom: 30px;
        }}

        .health-score {{
            font-size: 2.5em;
            font-weight: bold;
            color: {health_color};
            margin: 10px 0;
        }}

        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}

        .stat-card {{
            background: var(--bg-secondary);
            padding: 20px;
            border-radius: 8px;
            border: 1px solid var(--border-color);
            text-align: center;
        }}

        .stat-value {{
            font-size: 2em;
            font-weight: bold;
            margin-bottom: 5px;
        }}

        .stat-label {{
            color: var(--text-secondary);
            font-size: 0.9em;
        }}

        .charts-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}

        .chart-container {{
            background: var(--bg-secondary);
            padding: 20px;
            border-radius: 8px;
            border: 1px solid var(--border-color);
        }}

        .chart-title {{
            font-size: 1.2em;
            font-weight: bold;
            margin-bottom: 15px;
        }}

        .content-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 20px;
        }}

        .content-card {{
            background: var(--bg-secondary);
            padding: 20px;
            border-radius: 8px;
            border: 1px solid var(--border-color);
        }}

        .content-title {{
            font-size: 1.1em;
            font-weight: bold;
            margin-bottom: 15px;
        }}

        .flaky-test {{
            padding: 10px;
            margin: 5px 0;
            background: var(--bg-primary);
            border-left: 4px solid var(--warning-color);
            border-radius: 4px;
        }}

        .pattern {{
            padding: 10px;
            margin: 5px 0;
            background: var(--bg-primary);
            border-left: 4px solid var(--error-color);
            border-radius: 4px;
        }}

        .environment-item {{
            padding: 10px;
            margin: 5px 0;
            background: var(--bg-primary);
            border-radius: 4px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}

        .alert {{
            padding: 10px;
            margin: 5px 0;
            border-radius: 4px;
        }}

        .alert-critical {{
            background: #fee;
            border-left: 4px solid var(--error-color);
        }}

        .alert-warning {{
            background: #fff8e1;
            border-left: 4px solid var(--warning-color);
        }}

        .alert-info {{
            background: #e3f2fd;
            border-left: 4px solid var(--info-color);
        }}

        .timestamp {{
            text-align: center;
            color: var(--text-secondary);
            margin-top: 30px;
            font-size: 0.9em;
        }}

        .success {{ color: var(--success-color); }}
        .warning {{ color: var(--warning-color); }}
        .error {{ color: var(--error-color); }}
        .info {{ color: var(--info-color); }}

        @media (max-width: 768px) {{
            .stats-grid {{
                grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            }}
            .charts-grid {{
                grid-template-columns: 1fr;
            }}
            .content-grid {{
                grid-template-columns: 1fr;
            }}
        }}
    </style>
</head>
<body class="{theme_class}">
    <div class="dashboard">
        <div class="header">
            <h1>{title}</h1>
            <div class="health-score">{health_score:.1}%</div>
            <div>{health_description} Reliability Health</div>
        </div>

        <div class="stats-grid">
            <div class="stat-card">
                <div class="stat-value">{total_tests}</div>
                <div class="stat-label">Total Tests</div>
            </div>
            <div class="stat-card">
                <div class="stat-value success">{success_rate:.1}%</div>
                <div class="stat-label">Success Rate</div>
            </div>
            <div class="stat-card">
                <div class="stat-value warning">{flaky_test_count}</div>
                <div class="stat-label">Flaky Tests</div>
            </div>
            <div class="stat-card">
                <div class="stat-value">{stability_score:.1}</div>
                <div class="stat-label">Avg Stability</div>
            </div>
            <div class="stat-card">
                <div class="stat-value error">{critical_issues}</div>
                <div class="stat-label">Critical Issues</div>
            </div>
            <div class="stat-card">
                <div class="stat-value info">{improvement_opportunities}</div>
                <div class="stat-label">Improvements</div>
            </div>
        </div>

        <div class="charts-grid">
            <div class="chart-container">
                <div class="chart-title">Reliability Trends</div>
                <canvas id="trendsChart" width="400" height="200"></canvas>
            </div>
            <div class="chart-container">
                <div class="chart-title">Environment Comparison</div>
                <canvas id="environmentChart" width="400" height="200"></canvas>
            </div>
        </div>

        <div class="content-grid">
            <div class="content-card">
                <div class="content-title">Recent Flaky Tests</div>
                {flaky_tests_html}
            </div>
            <div class="content-card">
                <div class="content-title">Critical Patterns</div>
                {critical_patterns_html}
            </div>
            <div class="content-card">
                <div class="content-title">Environment Health</div>
                {environments_html}
            </div>
            <div class="content-card">
                <div class="content-title">Recent Alerts</div>
                {alerts_html}
            </div>
        </div>

        <div class="timestamp">
            Last updated: {timestamp}
        </div>
    </div>

    <script>
        // Trends Chart
        const trendsCtx = document.getElementById('trendsChart').getContext('2d');
        new Chart(trendsCtx, {{
            type: 'line',
            data: {{
                labels: {trend_labels},
                datasets: [
                    {{
                        label: 'Success Rate',
                        data: {success_rate_data},
                        borderColor: '{success_color}',
                        backgroundColor: '{success_color}20',
                        tension: 0.4
                    }},
                    {{
                        label: 'Stability Score',
                        data: {stability_data},
                        borderColor: '{info_color}',
                        backgroundColor: '{info_color}20',
                        tension: 0.4
                    }}
                ]
            }},
            options: {{
                responsive: true,
                scales: {{
                    y: {{
                        beginAtZero: true,
                        max: 100
                    }}
                }}
            }}
        }});

        // Environment Chart
        const envCtx = document.getElementById('environmentChart').getContext('2d');
        new Chart(envCtx, {{
            type: 'bar',
            data: {{
                labels: {env_labels},
                datasets: [
                    {{
                        label: 'Success Rate',
                        data: {env_success_data},
                        backgroundColor: '{success_color}',
                    }},
                    {{
                        label: 'Stability Score',
                        data: {env_stability_data},
                        backgroundColor: '{info_color}',
                    }}
                ]
            }},
            options: {{
                responsive: true,
                scales: {{
                    y: {{
                        beginAtZero: true,
                        max: 100
                    }}
                }}
            }}
        }});

        // Auto-refresh if enabled
        {auto_refresh_script}
    </script>
</body>
</html>
        "#,
            title = config.title,
            theme_class = theme_class,
            health_score = overview.health_score.overall_score,
            health_description = overview.health_score.get_health_description(),
            health_color = overview.health_score.get_health_color(),
            total_tests = overview.summary_stats.total_tests,
            success_rate = overview.summary_stats.success_rate,
            flaky_test_count = overview.summary_stats.flaky_test_count,
            stability_score = overview.summary_stats.average_stability_score,
            critical_issues = overview.summary_stats.critical_issues,
            improvement_opportunities = overview.summary_stats.improvement_opportunities,
            success_color = config.chart_colors.success,
            warning_color = config.chart_colors.warning,
            error_color = config.chart_colors.error,
            info_color = config.chart_colors.info,
            flaky_tests_html = Self::generate_flaky_tests_html(&overview.recent_flaky_tests),
            critical_patterns_html = Self::generate_patterns_html(&overview.critical_patterns),
            environments_html = Self::generate_environments_html(&overview.environment_comparison),
            alerts_html = Self::generate_alerts_html(&overview.alerts_summary.recent_alerts),
            timestamp = overview.generated_at.format("%Y-%m-%d %H:%M:%S UTC"),
            trend_labels = serde_json::to_string(&overview.trend_data.iter()
                .map(|t| t.timestamp.format("%m-%d").to_string())
                .collect::<Vec<_>>()).unwrap_or_default(),
            success_rate_data = serde_json::to_string(&overview.trend_data.iter()
                .map(|t| t.success_rate)
                .collect::<Vec<_>>()).unwrap_or_default(),
            stability_data = serde_json::to_string(&overview.trend_data.iter()
                .map(|t| t.stability_score)
                .collect::<Vec<_>>()).unwrap_or_default(),
            env_labels = serde_json::to_string(&overview.environment_comparison.iter()
                .map(|e| &e.environment)
                .collect::<Vec<_>>()).unwrap_or_default(),
            env_success_data = serde_json::to_string(&overview.environment_comparison.iter()
                .map(|e| e.success_rate)
                .collect::<Vec<_>>()).unwrap_or_default(),
            env_stability_data = serde_json::to_string(&overview.environment_comparison.iter()
                .map(|e| e.stability_score)
                .collect::<Vec<_>>()).unwrap_or_default(),
            auto_refresh_script = if config.enable_real_time {
                format!("setInterval(() => location.reload(), {});", config.refresh_interval_seconds * 1000)
            } else {
                "".to_string()
            },
        )
    }

    /// Generate HTML for flaky tests
    fn generate_flaky_tests_html(flaky_tests: &[FlakyTestSummary]) -> String {
        if flaky_tests.is_empty() {
            return "<p>No flaky tests detected</p>".to_string();
        }

        let mut html = String::new();
        for test in flaky_tests {
            html.push_str(&format!(
                r#"<div class="flaky-test">
                    <strong>{}</strong><br>
                    <small>Flakiness: {:.1}% | Executions: {} | Failures: {}</small><br>
                    <small>{}</small>
                </div>"#,
                test.test_name,
                test.flakiness_probability * 100.0,
                test.total_executions,
                test.failure_count,
                test.recommended_actions.join(", ")
            ));
        }
        html
    }

    /// Generate HTML for failure patterns
    fn generate_patterns_html(patterns: &[FailurePatternSummary]) -> String {
        if patterns.is_empty() {
            return "<p>No critical patterns detected</p>".to_string();
        }

        let mut html = String::new();
        for pattern in patterns {
            html.push_str(&format!(
                r#"<div class="pattern">
                    <strong>{}</strong><br>
                    <small>{}</small><br>
                    <small>Affects {} tests | Occurred {} times</small>
                </div>"#,
                pattern.pattern_type,
                pattern.description,
                pattern.affected_tests,
                pattern.occurrence_count
            ));
        }
        html
    }

    /// Generate HTML for environments
    fn generate_environments_html(environments: &[EnvironmentComparison]) -> String {
        if environments.is_empty() {
            return "<p>No environment data available</p>".to_string();
        }

        let mut html = String::new();
        for env in environments {
            let status_class = if env.success_rate >= 95.0 {
                "success"
            } else if env.success_rate >= 85.0 {
                "warning"
            } else {
                "error"
            };

            html.push_str(&format!(
                r#"<div class="environment-item">
                    <div>
                        <strong>{}</strong><br>
                        <small>Tests: {} | Flaky: {}</small>
                    </div>
                    <div class="{}">{:.1}%</div>
                </div>"#,
                env.environment,
                env.total_tests,
                env.flaky_test_count,
                status_class,
                env.success_rate
            ));
        }
        html
    }

    /// Generate HTML for alerts
    fn generate_alerts_html(alerts: &[RecentAlert]) -> String {
        if alerts.is_empty() {
            return "<p>No recent alerts</p>".to_string();
        }

        let mut html = String::new();
        for alert in alerts {
            let alert_class = match alert.severity.as_str() {
                "Critical" => "alert-critical",
                "Warning" => "alert-warning",
                _ => "alert-info",
            };

            html.push_str(&format!(
                r#"<div class="alert {}">
                    <strong>{}</strong><br>
                    <small>{} | {} affected tests</small><br>
                    <small>{}</small>
                </div>"#,
                alert_class,
                alert.title,
                alert.severity,
                alert.affected_tests,
                alert.timestamp.format("%m-%d %H:%M")
            ));
        }
        html
    }
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            title: "Test Reliability Dashboard".to_string(),
            refresh_interval_seconds: 30,
            max_flaky_tests_display: 10,
            max_patterns_display: 5,
            chart_colors: ChartColorScheme::default(),
            enable_real_time: true,
        }
    }
}

impl Default for ChartColorScheme {
    fn default() -> Self {
        Self {
            success: "#22c55e".to_string(),
            warning: "#f59e0b".to_string(),
            error: "#ef4444".to_string(),
            info: "#3b82f6".to_string(),
            primary: "#8b5cf6".to_string(),
            secondary: "#6b7280".to_string(),
        }
    }
}

/// Create reliability dashboard router
pub fn create_reliability_dashboard_router() -> Router<Arc<RwLock<TestReliabilityEngine>>> {
    Router::new()
        .route("/", get(dashboard_home))
        .route("/overview", get(dashboard_overview_json))
        .route("/test/:test_id", get(test_reliability_dashboard))
        .route("/environment/:env", get(environment_dashboard))
}

/// GET / - Main dashboard HTML
async fn dashboard_home(
    State(engine): State<Arc<RwLock<TestReliabilityEngine>>>,
    Query(query): Query<DashboardQuery>,
) -> Html<String> {
    let dashboard = ReliabilityDashboard::new(engine);
    let time_range = parse_time_window(&query.time_window);

    match dashboard.generate_overview(time_range, query.environment.as_deref()).await {
        Ok(overview) => {
            let html = DashboardGenerator::generate_html_dashboard(
                &overview,
                &dashboard.dashboard_config,
                query.theme.as_deref(),
            );
            Html(html)
        }
        Err(_) => Html("<h1>Error generating dashboard</h1>".to_string()),
    }
}

/// GET /overview - Dashboard data as JSON
async fn dashboard_overview_json(
    State(engine): State<Arc<RwLock<TestReliabilityEngine>>>,
    Query(query): Query<DashboardQuery>,
) -> Json<ReliabilityDashboardOverview> {
    let dashboard = ReliabilityDashboard::new(engine);
    let time_range = parse_time_window(&query.time_window);

    let overview = dashboard.generate_overview(time_range, query.environment.as_deref()).await
        .unwrap_or_else(|_| {
            // Return empty overview on error
            ReliabilityDashboardOverview {
                health_score: ReliabilityHealthScore::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
                summary_stats: DashboardSummaryStats {
                    total_tests: 0,
                    total_executions: 0,
                    flaky_test_count: 0,
                    success_rate: 0.0,
                    average_stability_score: 0.0,
                    critical_issues: 0,
                    improvement_opportunities: 0,
                    time_period: "Error".to_string(),
                },
                recent_flaky_tests: vec![],
                critical_patterns: vec![],
                trend_data: vec![],
                environment_comparison: vec![],
                alerts_summary: AlertsSummary {
                    critical_count: 0,
                    warning_count: 0,
                    info_count: 0,
                    recent_alerts: vec![],
                },
                generated_at: Utc::now(),
            }
        });

    Json(overview)
}

/// GET /test/:test_id - Test-specific reliability dashboard
async fn test_reliability_dashboard(
    State(engine): State<Arc<RwLock<TestReliabilityEngine>>>,
    Path(test_id): Path<String>,
    Query(query): Query<DashboardQuery>,
) -> Html<String> {
    let time_range = parse_time_window(&query.time_window);

    let engine_guard = engine.read().await;
    match engine_guard.analyze_test_reliability(&test_id, time_range, query.environment.as_deref()) {
        Ok(test_reliability) => {
            let html = format!(
                r#"<html>
                <head><title>Test Reliability: {}</title></head>
                <body>
                    <h1>Reliability Analysis: {}</h1>
                    <p>Flakiness Probability: {:.2}%</p>
                    <p>Total Executions: {}</p>
                    <p>Failure Count: {}</p>
                    <p>Consistency Score: {:.1}</p>
                    <h2>Recommended Actions:</h2>
                    <ul>{}</ul>
                </body>
                </html>"#,
                test_reliability.test_name,
                test_reliability.test_name,
                test_reliability.flakiness_probability * 100.0,
                test_reliability.total_executions,
                test_reliability.failure_count,
                test_reliability.consistency_score,
                test_reliability.recommended_actions
                    .iter()
                    .map(|action| format!("<li>{}</li>", action))
                    .collect::<String>()
            );
            Html(html)
        }
        Err(_) => Html("<h1>Error: Test not found or analysis failed</h1>".to_string()),
    }
}

/// GET /environment/:env - Environment-specific dashboard
async fn environment_dashboard(
    State(engine): State<Arc<RwLock<TestReliabilityEngine>>>,
    Path(env): Path<String>,
    Query(query): Query<DashboardQuery>,
) -> Html<String> {
    let time_range = parse_time_window(&query.time_window);

    let engine_guard = engine.read().await;
    match engine_guard.analyze_environment_reliability(&env, time_range) {
        Ok(env_analysis) => {
            let html = format!(
                r#"<html>
                <head><title>Environment Reliability: {}</title></head>
                <body>
                    <h1>Reliability Analysis: {} Environment</h1>
                    <p>Cross-Environment Consistency: {:.1}%</p>
                    <h2>Environment Scores:</h2>
                    <ul>{}</ul>
                    <h2>Issues:</h2>
                    <ul>{}</ul>
                </body>
                </html>"#,
                env,
                env,
                env_analysis.cross_environment_consistency * 100.0,
                env_analysis.environment_scores
                    .iter()
                    .map(|(env, score)| format!("<li>{}: {:.1}%</li>", env, score))
                    .collect::<String>(),
                env_analysis.environment_specific_issues
                    .get(&env)
                    .map(|issues| issues
                        .iter()
                        .map(|issue| format!("<li>{}</li>", issue))
                        .collect::<String>())
                    .unwrap_or_default()
            );
            Html(html)
        }
        Err(_) => Html("<h1>Error: Environment analysis failed</h1>".to_string()),
    }
}

/// Parse time window string into DateRange
fn parse_time_window(time_window: &Option<String>) -> DateRange {
    match time_window.as_deref() {
        Some("1h") => DateRange::last_hours(1),
        Some("6h") => DateRange::last_hours(6),
        Some("1d") => DateRange::last_days(1),
        Some("7d") | Some("1w") => DateRange::last_days(7),
        Some("30d") | Some("1m") => DateRange::last_days(30),
        Some("90d") | Some("3m") => DateRange::last_days(90),
        Some("365d") | Some("1y") => DateRange::last_days(365),
        _ => DateRange::last_days(7), // Default to last week
    }
}