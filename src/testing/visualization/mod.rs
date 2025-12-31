use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use crate::testing::monitoring::comprehensive::TestExecutionResult;

pub mod charts;
pub mod dashboard_components;
pub mod real_time_charts;

/// Configuration for different visualization types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub chart_type: ChartType,
    pub time_window: TimeWindow,
    pub grouping: GroupingStrategy,
    pub filters: VisualizationFilters,
    pub styling: ChartStyling,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartType {
    Line,
    Bar,
    Pie,
    Scatter,
    Heatmap,
    Treemap,
    Sunburst,
    Radar,
    Area,
    Timeline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeWindow {
    LastHour,
    Last6Hours,
    LastDay,
    LastWeek,
    LastMonth,
    LastQuarter,
    LastYear,
    Custom { start: DateTime<Utc>, end: DateTime<Utc> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupingStrategy {
    ByEnvironment,
    ByTestSuite,
    ByTestType,
    ByExecutionMode,
    ByTimeInterval(Duration),
    ByBranch,
    ByCommit,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationFilters {
    pub environments: Option<Vec<String>>,
    pub test_types: Option<Vec<String>>,
    pub execution_modes: Option<Vec<String>>,
    pub branches: Option<Vec<String>>,
    pub min_duration: Option<u64>,
    pub max_duration: Option<u64>,
    pub success_only: Option<bool>,
    pub failure_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartStyling {
    pub theme: VisualizationTheme,
    pub colors: Vec<String>,
    pub font_size: u32,
    pub width: u32,
    pub height: u32,
    pub responsive: bool,
    pub animation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationTheme {
    Light,
    Dark,
    Auto,
    Custom { primary: String, secondary: String, background: String },
}

/// Main visualization engine for test results
#[derive(Debug)]
pub struct TestVisualizationEngine {
    test_results: Vec<TestExecutionResult>,
    cached_visualizations: HashMap<String, CachedVisualization>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CachedVisualization {
    pub chart_data: ChartData,
    pub generated_at: DateTime<Utc>,
    pub config: VisualizationConfig,
    pub cache_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub datasets: Vec<Dataset>,
    pub labels: Vec<String>,
    pub metadata: ChartMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub label: String,
    pub data: Vec<DataPoint>,
    pub background_color: Option<String>,
    pub border_color: Option<String>,
    pub fill: Option<bool>,
    pub tension: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataPoint {
    Numeric(f64),
    TimeSeries { x: DateTime<Utc>, y: f64 },
    Categorical { x: String, y: f64 },
    Complex { x: f64, y: f64, size: Option<f64>, label: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartMetadata {
    pub total_data_points: usize,
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub aggregation_level: String,
    pub last_updated: DateTime<Utc>,
    pub chart_title: String,
    pub axis_labels: AxisLabels,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxisLabels {
    pub x_axis: String,
    pub y_axis: String,
    pub z_axis: Option<String>,
}

impl TestVisualizationEngine {
    pub fn new() -> Self {
        Self {
            test_results: Vec::new(),
            cached_visualizations: HashMap::new(),
        }
    }

    /// Load test results for visualization
    pub fn load_test_results(&mut self, results: Vec<TestExecutionResult>) {
        self.test_results = results;
        // Clear cache when new data is loaded
        self.cached_visualizations.clear();
    }

    /// Generate visualization based on configuration
    pub fn generate_visualization(&mut self, config: VisualizationConfig) -> Result<ChartData, VisualizationError> {
        let cache_key = self.generate_cache_key(&config);

        // Check cache first
        if let Some(cached) = self.cached_visualizations.get(&cache_key) {
            if Utc::now().signed_duration_since(cached.generated_at) < Duration::minutes(5) {
                return Ok(cached.chart_data.clone());
            }
        }

        let chart_data = match config.chart_type {
            ChartType::Line => self.generate_line_chart(&config)?,
            ChartType::Bar => self.generate_bar_chart(&config)?,
            ChartType::Pie => self.generate_pie_chart(&config)?,
            ChartType::Scatter => self.generate_scatter_chart(&config)?,
            ChartType::Heatmap => self.generate_heatmap(&config)?,
            ChartType::Treemap => self.generate_treemap(&config)?,
            ChartType::Sunburst => self.generate_sunburst(&config)?,
            ChartType::Radar => self.generate_radar_chart(&config)?,
            ChartType::Area => self.generate_area_chart(&config)?,
            ChartType::Timeline => self.generate_timeline(&config)?,
        };

        // Cache the result
        self.cached_visualizations.insert(cache_key.clone(), CachedVisualization {
            chart_data: chart_data.clone(),
            generated_at: Utc::now(),
            config: config.clone(),
            cache_key,
        });

        Ok(chart_data)
    }

    /// Generate coverage trend visualization
    pub fn generate_coverage_trends(&self, config: &VisualizationConfig) -> Result<ChartData, VisualizationError> {
        let filtered_results = self.filter_results(config);
        let grouped_data = self.group_by_time(&filtered_results, &config.grouping)?;

        let mut rust_coverage_data = Vec::new();
        let mut typescript_coverage_data = Vec::new();
        let mut labels = Vec::new();

        for (time_key, results) in grouped_data {
            labels.push(time_key.clone());

            let rust_avg = results.iter()
                .filter_map(|r| r.rust_coverage)
                .fold(0.0, |acc, x| acc + x) / results.len() as f64;

            let ts_avg = results.iter()
                .filter_map(|r| r.typescript_coverage)
                .fold(0.0, |acc, x| acc + x) / results.len() as f64;

            rust_coverage_data.push(DataPoint::TimeSeries {
                x: results[0].executed_at,
                y: rust_avg,
            });

            typescript_coverage_data.push(DataPoint::TimeSeries {
                x: results[0].executed_at,
                y: ts_avg,
            });
        }

        Ok(ChartData {
            datasets: vec![
                Dataset {
                    label: "Rust Coverage".to_string(),
                    data: rust_coverage_data,
                    background_color: Some("#e74c3c".to_string()),
                    border_color: Some("#c0392b".to_string()),
                    fill: Some(false),
                    tension: Some(0.4),
                },
                Dataset {
                    label: "TypeScript Coverage".to_string(),
                    data: typescript_coverage_data,
                    background_color: Some("#3498db".to_string()),
                    border_color: Some("#2980b9".to_string()),
                    fill: Some(false),
                    tension: Some(0.4),
                },
            ],
            labels,
            metadata: ChartMetadata {
                total_data_points: self.test_results.len(),
                time_range: self.get_time_range(&config.time_window),
                aggregation_level: format!("{:?}", config.grouping),
                last_updated: Utc::now(),
                chart_title: "Code Coverage Trends".to_string(),
                axis_labels: AxisLabels {
                    x_axis: "Time".to_string(),
                    y_axis: "Coverage %".to_string(),
                    z_axis: None,
                },
            },
        })
    }

    /// Generate performance regression analysis
    pub fn generate_performance_analysis(&self, config: &VisualizationConfig) -> Result<ChartData, VisualizationError> {
        let filtered_results = self.filter_results(config);
        let grouped_data = self.group_by_time(&filtered_results, &config.grouping)?;

        let mut duration_data = Vec::new();
        let mut regression_markers = Vec::new();
        let mut labels = Vec::new();

        let mut baseline_duration: Option<f64> = None;

        for (time_key, results) in grouped_data {
            labels.push(time_key.clone());

            let avg_duration = results.iter()
                .map(|r| r.duration.as_millis() as f64)
                .fold(0.0, |acc, x| acc + x) / results.len() as f64;

            duration_data.push(DataPoint::TimeSeries {
                x: results[0].executed_at,
                y: avg_duration,
            });

            // Detect regressions (>20% increase from baseline)
            if let Some(baseline) = baseline_duration {
                let increase_percent = (avg_duration - baseline) / baseline * 100.0;
                if increase_percent > 20.0 {
                    regression_markers.push(DataPoint::TimeSeries {
                        x: results[0].executed_at,
                        y: avg_duration,
                    });
                }
            } else {
                baseline_duration = Some(avg_duration);
            }
        }

        let mut datasets = vec![
            Dataset {
                label: "Average Duration".to_string(),
                data: duration_data,
                background_color: Some("#27ae60".to_string()),
                border_color: Some("#2ecc71".to_string()),
                fill: Some(false),
                tension: Some(0.4),
            },
        ];

        if !regression_markers.is_empty() {
            datasets.push(Dataset {
                label: "Performance Regressions".to_string(),
                data: regression_markers,
                background_color: Some("#e74c3c".to_string()),
                border_color: Some("#c0392b".to_string()),
                fill: Some(false),
                tension: Some(0.0),
            });
        }

        Ok(ChartData {
            datasets,
            labels,
            metadata: ChartMetadata {
                total_data_points: self.test_results.len(),
                time_range: self.get_time_range(&config.time_window),
                aggregation_level: format!("{:?}", config.grouping),
                last_updated: Utc::now(),
                chart_title: "Performance Analysis".to_string(),
                axis_labels: AxisLabels {
                    x_axis: "Time".to_string(),
                    y_axis: "Duration (ms)".to_string(),
                    z_axis: None,
                },
            },
        })
    }

    /// Generate reliability heatmap
    pub fn generate_reliability_heatmap(&self, config: &VisualizationConfig) -> Result<ChartData, VisualizationError> {
        let filtered_results = self.filter_results(config);

        // Group by test suite and environment
        let mut heatmap_data = HashMap::new();
        let mut test_suites = std::collections::HashSet::new();
        let mut environments = std::collections::HashSet::new();

        for result in filtered_results {
            test_suites.insert(result.test_suite.clone());
            environments.insert(result.environment.clone());

            let key = (result.test_suite.clone(), result.environment.clone());
            let entry = heatmap_data.entry(key).or_insert_with(|| (0, 0));

            entry.0 += 1; // Total tests
            if result.success {
                entry.1 += 1; // Successful tests
            }
        }

        let mut data_points = Vec::new();
        let test_suite_list: Vec<_> = test_suites.into_iter().collect();
        let environment_list: Vec<_> = environments.into_iter().collect();

        for (i, test_suite) in test_suite_list.iter().enumerate() {
            for (j, environment) in environment_list.iter().enumerate() {
                let key = (test_suite.clone(), environment.clone());
                if let Some((total, passed)) = heatmap_data.get(&key) {
                    let success_rate = (*passed as f64 / *total as f64) * 100.0;
                    data_points.push(DataPoint::Complex {
                        x: i as f64,
                        y: j as f64,
                        size: Some(*total as f64),
                        label: Some(format!("{:.1}%", success_rate)),
                    });
                }
            }
        }

        Ok(ChartData {
            datasets: vec![
                Dataset {
                    label: "Test Reliability".to_string(),
                    data: data_points,
                    background_color: Some("#3498db".to_string()),
                    border_color: Some("#2980b9".to_string()),
                    fill: None,
                    tension: None,
                },
            ],
            labels: test_suite_list,
            metadata: ChartMetadata {
                total_data_points: self.test_results.len(),
                time_range: self.get_time_range(&config.time_window),
                aggregation_level: "Test Suite x Environment".to_string(),
                last_updated: Utc::now(),
                chart_title: "Test Reliability Heatmap".to_string(),
                axis_labels: AxisLabels {
                    x_axis: "Test Suite".to_string(),
                    y_axis: "Environment".to_string(),
                    z_axis: Some("Success Rate %".to_string()),
                },
            },
        })
    }

    // Private helper methods
    fn generate_cache_key(&self, config: &VisualizationConfig) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        format!("{:?}", config).hash(&mut hasher);
        self.test_results.len().hash(&mut hasher);

        format!("viz_{}", hasher.finish())
    }

    fn filter_results(&self, config: &VisualizationConfig) -> Vec<&TestExecutionResult> {
        self.test_results.iter()
            .filter(|result| {
                // Apply time window filter
                if !self.is_in_time_window(result.executed_at, &config.time_window) {
                    return false;
                }

                // Apply other filters
                if let Some(envs) = &config.filters.environments {
                    if !envs.contains(&result.environment) {
                        return false;
                    }
                }

                if let Some(success_only) = config.filters.success_only {
                    if success_only && !result.success {
                        return false;
                    }
                }

                if let Some(failure_only) = config.filters.failure_only {
                    if failure_only && result.success {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    fn group_by_time(&self, results: &[&TestExecutionResult], grouping: &GroupingStrategy) -> Result<Vec<(String, Vec<&TestExecutionResult>)>, VisualizationError> {
        let mut groups: HashMap<String, Vec<&TestExecutionResult>> = HashMap::new();

        for result in results {
            let key = match grouping {
                GroupingStrategy::ByEnvironment => result.environment.clone(),
                GroupingStrategy::ByTestSuite => result.test_suite.clone(),
                GroupingStrategy::ByTimeInterval(duration) => {
                    let interval = result.executed_at.timestamp() / duration.num_seconds();
                    format!("interval_{}", interval)
                },
                _ => "default".to_string(),
            };

            groups.entry(key).or_default().push(*result);
        }

        let mut sorted_groups: Vec<_> = groups.into_iter().collect();
        sorted_groups.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(sorted_groups)
    }

    fn is_in_time_window(&self, timestamp: DateTime<Utc>, window: &TimeWindow) -> bool {
        let now = Utc::now();
        match window {
            TimeWindow::LastHour => now.signed_duration_since(timestamp) <= Duration::hours(1),
            TimeWindow::Last6Hours => now.signed_duration_since(timestamp) <= Duration::hours(6),
            TimeWindow::LastDay => now.signed_duration_since(timestamp) <= Duration::days(1),
            TimeWindow::LastWeek => now.signed_duration_since(timestamp) <= Duration::weeks(1),
            TimeWindow::LastMonth => now.signed_duration_since(timestamp) <= Duration::days(30),
            TimeWindow::LastQuarter => now.signed_duration_since(timestamp) <= Duration::days(90),
            TimeWindow::LastYear => now.signed_duration_since(timestamp) <= Duration::days(365),
            TimeWindow::Custom { start, end } => timestamp >= *start && timestamp <= *end,
        }
    }

    fn get_time_range(&self, window: &TimeWindow) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
        let now = Utc::now();
        match window {
            TimeWindow::LastHour => Some((now - Duration::hours(1), now)),
            TimeWindow::Last6Hours => Some((now - Duration::hours(6), now)),
            TimeWindow::LastDay => Some((now - Duration::days(1), now)),
            TimeWindow::LastWeek => Some((now - Duration::weeks(1), now)),
            TimeWindow::LastMonth => Some((now - Duration::days(30), now)),
            TimeWindow::LastQuarter => Some((now - Duration::days(90), now)),
            TimeWindow::LastYear => Some((now - Duration::days(365), now)),
            TimeWindow::Custom { start, end } => Some((*start, *end)),
        }
    }

    // Placeholder methods for different chart types
    fn generate_line_chart(&self, config: &VisualizationConfig) -> Result<ChartData, VisualizationError> {
        self.generate_coverage_trends(config)
    }

    fn generate_bar_chart(&self, config: &VisualizationConfig) -> Result<ChartData, VisualizationError> {
        // Implement bar chart logic
        self.generate_coverage_trends(config)
    }

    fn generate_pie_chart(&self, config: &VisualizationConfig) -> Result<ChartData, VisualizationError> {
        // Implement pie chart logic
        self.generate_coverage_trends(config)
    }

    fn generate_scatter_chart(&self, config: &VisualizationConfig) -> Result<ChartData, VisualizationError> {
        self.generate_performance_analysis(config)
    }

    fn generate_heatmap(&self, config: &VisualizationConfig) -> Result<ChartData, VisualizationError> {
        self.generate_reliability_heatmap(config)
    }

    fn generate_treemap(&self, _config: &VisualizationConfig) -> Result<ChartData, VisualizationError> {
        Err(VisualizationError::NotImplemented("Treemap visualization".to_string()))
    }

    fn generate_sunburst(&self, _config: &VisualizationConfig) -> Result<ChartData, VisualizationError> {
        Err(VisualizationError::NotImplemented("Sunburst visualization".to_string()))
    }

    fn generate_radar_chart(&self, _config: &VisualizationConfig) -> Result<ChartData, VisualizationError> {
        Err(VisualizationError::NotImplemented("Radar chart visualization".to_string()))
    }

    fn generate_area_chart(&self, config: &VisualizationConfig) -> Result<ChartData, VisualizationError> {
        self.generate_coverage_trends(config)
    }

    fn generate_timeline(&self, config: &VisualizationConfig) -> Result<ChartData, VisualizationError> {
        self.generate_performance_analysis(config)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VisualizationError {
    #[error("Insufficient data for visualization: {0}")]
    InsufficientData(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Visualization type not implemented: {0}")]
    NotImplemented(String),

    #[error("Data processing error: {0}")]
    DataProcessing(String),

    #[error("Cache error: {0}")]
    Cache(String),
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            chart_type: ChartType::Line,
            time_window: TimeWindow::LastDay,
            grouping: GroupingStrategy::ByTimeInterval(Duration::hours(1)),
            filters: VisualizationFilters {
                environments: None,
                test_types: None,
                execution_modes: None,
                branches: None,
                min_duration: None,
                max_duration: None,
                success_only: None,
                failure_only: None,
            },
            styling: ChartStyling {
                theme: VisualizationTheme::Auto,
                colors: vec![
                    "#3498db".to_string(),
                    "#e74c3c".to_string(),
                    "#2ecc71".to_string(),
                    "#f39c12".to_string(),
                    "#9b59b6".to_string(),
                ],
                font_size: 12,
                width: 800,
                height: 400,
                responsive: true,
                animation: true,
            },
        }
    }
}

impl Default for TestVisualizationEngine {
    fn default() -> Self {
        Self::new()
    }
}