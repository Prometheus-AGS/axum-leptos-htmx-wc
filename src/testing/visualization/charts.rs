use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use crate::testing::monitoring::comprehensive::TestExecutionResult;
use super::{ChartData, Dataset, DataPoint, VisualizationConfig, VisualizationError, ChartMetadata, AxisLabels};

/// Specialized chart generators for different visualization types
pub struct ChartGenerators;

impl ChartGenerators {
    /// Generate test success rate over time
    pub fn success_rate_timeline(
        results: &[&TestExecutionResult],
        config: &VisualizationConfig,
    ) -> Result<ChartData, VisualizationError> {
        if results.is_empty() {
            return Err(VisualizationError::InsufficientData("No test results available".to_string()));
        }

        let grouped_data = Self::group_by_time_interval(results, Duration::hours(1))?;

        let mut success_rate_data = Vec::new();
        let mut total_tests_data = Vec::new();
        let mut labels = Vec::new();

        for (timestamp, test_results) in grouped_data {
            let total_tests = test_results.len();
            let successful_tests = test_results.iter().filter(|r| r.success).count();
            let success_rate = if total_tests > 0 {
                (successful_tests as f64 / total_tests as f64) * 100.0
            } else {
                0.0
            };

            labels.push(timestamp.format("%H:%M").to_string());
            success_rate_data.push(DataPoint::TimeSeries {
                x: timestamp,
                y: success_rate,
            });
            total_tests_data.push(DataPoint::TimeSeries {
                x: timestamp,
                y: total_tests as f64,
            });
        }

        Ok(ChartData {
            datasets: vec![
                Dataset {
                    label: "Success Rate %".to_string(),
                    data: success_rate_data,
                    background_color: Some("rgba(46, 204, 113, 0.2)".to_string()),
                    border_color: Some("#2ecc71".to_string()),
                    fill: Some(true),
                    tension: Some(0.4),
                },
                Dataset {
                    label: "Total Tests".to_string(),
                    data: total_tests_data,
                    background_color: Some("rgba(52, 152, 219, 0.2)".to_string()),
                    border_color: Some("#3498db".to_string()),
                    fill: Some(false),
                    tension: Some(0.4),
                },
            ],
            labels,
            metadata: ChartMetadata {
                total_data_points: results.len(),
                time_range: Self::get_time_range_from_results(results),
                aggregation_level: "Hourly".to_string(),
                last_updated: Utc::now(),
                chart_title: "Test Success Rate Timeline".to_string(),
                axis_labels: AxisLabels {
                    x_axis: "Time".to_string(),
                    y_axis: "Success Rate % / Count".to_string(),
                    z_axis: None,
                },
            },
        })
    }

    /// Generate environment comparison bar chart
    pub fn environment_comparison(
        results: &[&TestExecutionResult],
        _config: &VisualizationConfig,
    ) -> Result<ChartData, VisualizationError> {
        if results.is_empty() {
            return Err(VisualizationError::InsufficientData("No test results available".to_string()));
        }

        let mut env_stats: HashMap<String, (usize, usize, f64)> = HashMap::new(); // (total, passed, avg_duration)

        for result in results {
            let entry = env_stats.entry(result.environment.clone())
                .or_insert((0, 0, 0.0));

            entry.0 += 1; // total
            if result.success {
                entry.1 += 1; // passed
            }
            entry.2 += result.duration.as_millis() as f64; // duration sum
        }

        let mut environments = Vec::new();
        let mut success_rates = Vec::new();
        let mut avg_durations = Vec::new();
        let mut total_tests = Vec::new();

        for (env, (total, passed, duration_sum)) in env_stats {
            environments.push(env);
            success_rates.push(DataPoint::Numeric(
                (passed as f64 / total as f64) * 100.0
            ));
            avg_durations.push(DataPoint::Numeric(
                duration_sum / total as f64
            ));
            total_tests.push(DataPoint::Numeric(total as f64));
        }

        Ok(ChartData {
            datasets: vec![
                Dataset {
                    label: "Success Rate %".to_string(),
                    data: success_rates,
                    background_color: Some("#2ecc71".to_string()),
                    border_color: Some("#27ae60".to_string()),
                    fill: None,
                    tension: None,
                },
                Dataset {
                    label: "Avg Duration (ms)".to_string(),
                    data: avg_durations,
                    background_color: Some("#3498db".to_string()),
                    border_color: Some("#2980b9".to_string()),
                    fill: None,
                    tension: None,
                },
                Dataset {
                    label: "Total Tests".to_string(),
                    data: total_tests,
                    background_color: Some("#f39c12".to_string()),
                    border_color: Some("#e67e22".to_string()),
                    fill: None,
                    tension: None,
                },
            ],
            labels: environments,
            metadata: ChartMetadata {
                total_data_points: results.len(),
                time_range: Self::get_time_range_from_results(results),
                aggregation_level: "By Environment".to_string(),
                last_updated: Utc::now(),
                chart_title: "Environment Comparison".to_string(),
                axis_labels: AxisLabels {
                    x_axis: "Environment".to_string(),
                    y_axis: "Metrics".to_string(),
                    z_axis: None,
                },
            },
        })
    }

    /// Generate test suite distribution pie chart
    pub fn test_suite_distribution(
        results: &[&TestExecutionResult],
        _config: &VisualizationConfig,
    ) -> Result<ChartData, VisualizationError> {
        if results.is_empty() {
            return Err(VisualizationError::InsufficientData("No test results available".to_string()));
        }

        let mut suite_counts: HashMap<String, usize> = HashMap::new();

        for result in results {
            *suite_counts.entry(result.test_suite.clone()).or_insert(0) += 1;
        }

        let mut labels = Vec::new();
        let mut data = Vec::new();
        let colors = vec![
            "#3498db", "#e74c3c", "#2ecc71", "#f39c12", "#9b59b6",
            "#e67e22", "#1abc9c", "#34495e", "#f1c40f", "#e91e63",
        ];

        for (i, (suite, count)) in suite_counts.iter().enumerate() {
            labels.push(suite.clone());
            data.push(DataPoint::Numeric(*count as f64));
        }

        Ok(ChartData {
            datasets: vec![
                Dataset {
                    label: "Test Count".to_string(),
                    data,
                    background_color: Some(colors.iter().cycle().take(labels.len()).map(|c| c.to_string()).collect::<Vec<_>>().join(",")),
                    border_color: Some("#ffffff".to_string()),
                    fill: None,
                    tension: None,
                },
            ],
            labels,
            metadata: ChartMetadata {
                total_data_points: results.len(),
                time_range: Self::get_time_range_from_results(results),
                aggregation_level: "By Test Suite".to_string(),
                last_updated: Utc::now(),
                chart_title: "Test Suite Distribution".to_string(),
                axis_labels: AxisLabels {
                    x_axis: "Test Suite".to_string(),
                    y_axis: "Count".to_string(),
                    z_axis: None,
                },
            },
        })
    }

    /// Generate performance vs reliability scatter plot
    pub fn performance_reliability_scatter(
        results: &[&TestExecutionResult],
        _config: &VisualizationConfig,
    ) -> Result<ChartData, VisualizationError> {
        if results.is_empty() {
            return Err(VisualizationError::InsufficientData("No test results available".to_string()));
        }

        // Group by test suite to calculate metrics
        let mut suite_metrics: HashMap<String, Vec<&TestExecutionResult>> = HashMap::new();

        for result in results {
            suite_metrics.entry(result.test_suite.clone()).or_default().push(*result);
        }

        let mut scatter_data = Vec::new();
        let mut labels = Vec::new();

        for (suite, suite_results) in suite_metrics {
            let avg_duration = suite_results.iter()
                .map(|r| r.duration.as_millis() as f64)
                .sum::<f64>() / suite_results.len() as f64;

            let success_rate = suite_results.iter()
                .filter(|r| r.success)
                .count() as f64 / suite_results.len() as f64 * 100.0;

            scatter_data.push(DataPoint::Complex {
                x: avg_duration,
                y: success_rate,
                size: Some(suite_results.len() as f64),
                label: Some(suite.clone()),
            });

            labels.push(suite);
        }

        Ok(ChartData {
            datasets: vec![
                Dataset {
                    label: "Test Suites".to_string(),
                    data: scatter_data,
                    background_color: Some("rgba(52, 152, 219, 0.6)".to_string()),
                    border_color: Some("#3498db".to_string()),
                    fill: None,
                    tension: None,
                },
            ],
            labels,
            metadata: ChartMetadata {
                total_data_points: results.len(),
                time_range: Self::get_time_range_from_results(results),
                aggregation_level: "By Test Suite".to_string(),
                last_updated: Utc::now(),
                chart_title: "Performance vs Reliability Analysis".to_string(),
                axis_labels: AxisLabels {
                    x_axis: "Average Duration (ms)".to_string(),
                    y_axis: "Success Rate %".to_string(),
                    z_axis: Some("Test Count (bubble size)".to_string()),
                },
            },
        })
    }

    /// Generate coverage comparison radar chart
    pub fn coverage_radar(
        results: &[&TestExecutionResult],
        _config: &VisualizationConfig,
    ) -> Result<ChartData, VisualizationError> {
        if results.is_empty() {
            return Err(VisualizationError::InsufficientData("No test results available".to_string()));
        }

        // Calculate average coverage metrics
        let total_results = results.len() as f64;

        let avg_rust_coverage = results.iter()
            .filter_map(|r| r.rust_coverage)
            .sum::<f64>() / total_results;

        let avg_typescript_coverage = results.iter()
            .filter_map(|r| r.typescript_coverage)
            .sum::<f64>() / total_results;

        let avg_overall_coverage = results.iter()
            .filter_map(|r| r.overall_coverage)
            .sum::<f64>() / total_results;

        let success_rate = results.iter()
            .filter(|r| r.success)
            .count() as f64 / total_results * 100.0;

        let avg_performance_score = results.iter()
            .map(|r| {
                // Convert duration to performance score (lower duration = higher score)
                let max_duration = 10000.0; // 10 seconds max
                let duration_ms = r.duration.as_millis() as f64;
                ((max_duration - duration_ms.min(max_duration)) / max_duration * 100.0).max(0.0)
            })
            .sum::<f64>() / total_results;

        let reliability_score = success_rate;

        let radar_data = vec![
            DataPoint::Numeric(avg_rust_coverage),
            DataPoint::Numeric(avg_typescript_coverage),
            DataPoint::Numeric(avg_overall_coverage),
            DataPoint::Numeric(success_rate),
            DataPoint::Numeric(avg_performance_score),
            DataPoint::Numeric(reliability_score),
        ];

        let labels = vec![
            "Rust Coverage".to_string(),
            "TypeScript Coverage".to_string(),
            "Overall Coverage".to_string(),
            "Success Rate".to_string(),
            "Performance Score".to_string(),
            "Reliability Score".to_string(),
        ];

        Ok(ChartData {
            datasets: vec![
                Dataset {
                    label: "Quality Metrics".to_string(),
                    data: radar_data,
                    background_color: Some("rgba(52, 152, 219, 0.2)".to_string()),
                    border_color: Some("#3498db".to_string()),
                    fill: Some(true),
                    tension: Some(0.1),
                },
            ],
            labels,
            metadata: ChartMetadata {
                total_data_points: results.len(),
                time_range: Self::get_time_range_from_results(results),
                aggregation_level: "Overall Metrics".to_string(),
                last_updated: Utc::now(),
                chart_title: "Test Quality Radar".to_string(),
                axis_labels: AxisLabels {
                    x_axis: "Metrics".to_string(),
                    y_axis: "Score (%)".to_string(),
                    z_axis: None,
                },
            },
        })
    }

    /// Generate flaky test identification
    pub fn flaky_test_analysis(
        results: &[&TestExecutionResult],
        _config: &VisualizationConfig,
    ) -> Result<ChartData, VisualizationError> {
        if results.is_empty() {
            return Err(VisualizationError::InsufficientData("No test results available".to_string()));
        }

        // Group by test identifier and analyze success patterns
        let mut test_patterns: HashMap<String, Vec<bool>> = HashMap::new();

        for result in results {
            let test_id = format!("{}::{}", result.test_suite, result.test_type);
            test_patterns.entry(test_id).or_default().push(result.success);
        }

        let mut flaky_tests = Vec::new();
        let mut stable_tests = Vec::new();
        let mut test_names = Vec::new();

        for (test_id, success_pattern) in test_patterns {
            if success_pattern.len() < 3 {
                continue; // Need at least 3 runs to identify flakiness
            }

            let success_count = success_pattern.iter().filter(|&&s| s).count();
            let total_runs = success_pattern.len();
            let success_rate = success_count as f64 / total_runs as f64;

            // Consider a test flaky if it has mixed results (neither always passing nor always failing)
            let is_flaky = success_rate > 0.1 && success_rate < 0.9 && total_runs >= 3;

            if is_flaky {
                flaky_tests.push(DataPoint::Numeric(success_rate * 100.0));
                stable_tests.push(DataPoint::Numeric(0.0));
            } else {
                flaky_tests.push(DataPoint::Numeric(0.0));
                stable_tests.push(DataPoint::Numeric(success_rate * 100.0));
            }

            test_names.push(test_id);
        }

        Ok(ChartData {
            datasets: vec![
                Dataset {
                    label: "Flaky Tests".to_string(),
                    data: flaky_tests,
                    background_color: Some("#e74c3c".to_string()),
                    border_color: Some("#c0392b".to_string()),
                    fill: None,
                    tension: None,
                },
                Dataset {
                    label: "Stable Tests".to_string(),
                    data: stable_tests,
                    background_color: Some("#2ecc71".to_string()),
                    border_color: Some("#27ae60".to_string()),
                    fill: None,
                    tension: None,
                },
            ],
            labels: test_names,
            metadata: ChartMetadata {
                total_data_points: results.len(),
                time_range: Self::get_time_range_from_results(results),
                aggregation_level: "By Test Identifier".to_string(),
                last_updated: Utc::now(),
                chart_title: "Flaky Test Analysis".to_string(),
                axis_labels: AxisLabels {
                    x_axis: "Test".to_string(),
                    y_axis: "Success Rate %".to_string(),
                    z_axis: None,
                },
            },
        })
    }

    // Helper methods
    fn group_by_time_interval(
        results: &[&TestExecutionResult],
        interval: Duration,
    ) -> Result<Vec<(DateTime<Utc>, Vec<&TestExecutionResult>)>, VisualizationError> {
        if results.is_empty() {
            return Ok(Vec::new());
        }

        let mut groups: HashMap<i64, Vec<&TestExecutionResult>> = HashMap::new();

        for result in results {
            let interval_key = result.executed_at.timestamp() / interval.num_seconds();
            groups.entry(interval_key).or_default().push(*result);
        }

        let mut sorted_groups: Vec<_> = groups.into_iter()
            .map(|(key, results)| {
                let timestamp = DateTime::from_timestamp(key * interval.num_seconds(), 0)
                    .unwrap_or_else(Utc::now);
                (timestamp, results)
            })
            .collect();

        sorted_groups.sort_by_key(|(timestamp, _)| *timestamp);

        Ok(sorted_groups)
    }

    fn get_time_range_from_results(results: &[&TestExecutionResult]) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
        if results.is_empty() {
            return None;
        }

        let min_time = results.iter().map(|r| r.executed_at).min()?;
        let max_time = results.iter().map(|r| r.executed_at).max()?;

        Some((min_time, max_time))
    }
}

/// Chart configuration presets for common visualizations
pub struct ChartPresets;

impl ChartPresets {
    /// Daily summary dashboard configuration
    pub fn daily_summary() -> VisualizationConfig {
        VisualizationConfig {
            chart_type: super::ChartType::Line,
            time_window: super::TimeWindow::LastDay,
            grouping: super::GroupingStrategy::ByTimeInterval(Duration::hours(1)),
            filters: super::VisualizationFilters {
                environments: None,
                test_types: None,
                execution_modes: None,
                branches: None,
                min_duration: None,
                max_duration: None,
                success_only: None,
                failure_only: None,
            },
            styling: super::ChartStyling {
                theme: super::VisualizationTheme::Auto,
                colors: vec![
                    "#2ecc71".to_string(), // Success - Green
                    "#e74c3c".to_string(), // Failure - Red
                    "#3498db".to_string(), // Info - Blue
                    "#f39c12".to_string(), // Warning - Orange
                ],
                font_size: 14,
                width: 1200,
                height: 400,
                responsive: true,
                animation: true,
            },
        }
    }

    /// Performance monitoring configuration
    pub fn performance_monitoring() -> VisualizationConfig {
        VisualizationConfig {
            chart_type: super::ChartType::Scatter,
            time_window: super::TimeWindow::LastWeek,
            grouping: super::GroupingStrategy::ByTestSuite,
            filters: super::VisualizationFilters {
                environments: None,
                test_types: None,
                execution_modes: None,
                branches: None,
                min_duration: None,
                max_duration: Some(60000), // 1 minute max
                success_only: None,
                failure_only: None,
            },
            styling: super::ChartStyling {
                theme: super::VisualizationTheme::Dark,
                colors: vec![
                    "#3498db".to_string(),
                    "#e74c3c".to_string(),
                    "#f39c12".to_string(),
                ],
                font_size: 12,
                width: 800,
                height: 600,
                responsive: true,
                animation: true,
            },
        }
    }

    /// Environment comparison configuration
    pub fn environment_comparison() -> VisualizationConfig {
        VisualizationConfig {
            chart_type: super::ChartType::Bar,
            time_window: super::TimeWindow::LastWeek,
            grouping: super::GroupingStrategy::ByEnvironment,
            filters: super::VisualizationFilters {
                environments: None,
                test_types: None,
                execution_modes: None,
                branches: None,
                min_duration: None,
                max_duration: None,
                success_only: None,
                failure_only: None,
            },
            styling: super::ChartStyling {
                theme: super::VisualizationTheme::Light,
                colors: vec![
                    "#2ecc71".to_string(), // Development - Green
                    "#f39c12".to_string(), // Staging - Orange
                    "#e74c3c".to_string(), // Production - Red
                ],
                font_size: 14,
                width: 1000,
                height: 500,
                responsive: true,
                animation: true,
            },
        }
    }

    /// Coverage analysis configuration
    pub fn coverage_analysis() -> VisualizationConfig {
        VisualizationConfig {
            chart_type: super::ChartType::Area,
            time_window: super::TimeWindow::LastMonth,
            grouping: super::GroupingStrategy::ByTimeInterval(Duration::days(1)),
            filters: super::VisualizationFilters {
                environments: None,
                test_types: None,
                execution_modes: None,
                branches: None,
                min_duration: None,
                max_duration: None,
                success_only: Some(true), // Only successful tests for coverage
                failure_only: None,
            },
            styling: super::ChartStyling {
                theme: super::VisualizationTheme::Auto,
                colors: vec![
                    "#e74c3c".to_string(), // Rust - Red/Orange
                    "#3498db".to_string(), // TypeScript - Blue
                    "#2ecc71".to_string(), // Overall - Green
                ],
                font_size: 13,
                width: 1200,
                height: 450,
                responsive: true,
                animation: true,
            },
        }
    }
}