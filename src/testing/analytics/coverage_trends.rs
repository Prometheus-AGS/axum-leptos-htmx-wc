use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use chrono::{DateTime, Utc, Duration};
use crate::testing::entities::TestExecutionResult;
use super::{AnalyticsResult, InsightLevel, Insight};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageTrendAnalyzer {
    pub config: CoverageTrendConfig,
    historical_snapshots: Vec<CoverageSnapshot>,
    trend_cache: HashMap<String, CachedTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageTrendConfig {
    pub analysis_window_days: u32,
    pub trend_detection_sensitivity: f64,
    pub regression_threshold: f64,
    pub improvement_threshold: f64,
    pub minimum_data_points: usize,
    pub language_weights: HashMap<String, f64>,
    pub alert_thresholds: CoverageAlertThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageAlertThresholds {
    pub critical_regression: f64,    // > 5% drop
    pub major_regression: f64,       // > 3% drop
    pub minor_regression: f64,       // > 1% drop
    pub improvement_celebration: f64, // > 2% increase
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageSnapshot {
    pub timestamp: DateTime<Utc>,
    pub execution_id: String,
    pub branch: String,
    pub commit_hash: String,
    pub rust_coverage: CoverageMetrics,
    pub typescript_coverage: CoverageMetrics,
    pub integration_coverage: CoverageMetrics,
    pub e2e_coverage: CoverageMetrics,
    pub overall_metrics: OverallCoverageMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageMetrics {
    pub line_coverage: f64,
    pub branch_coverage: f64,
    pub function_coverage: f64,
    pub statement_coverage: f64,
    pub lines_covered: u32,
    pub lines_total: u32,
    pub branches_covered: u32,
    pub branches_total: u32,
    pub functions_covered: u32,
    pub functions_total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallCoverageMetrics {
    pub weighted_coverage: f64,
    pub quality_score: f64,
    pub completeness_index: f64,
    pub test_effectiveness: f64,
    pub coverage_debt: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTrend {
    pub trend_direction: TrendDirection,
    pub confidence_score: f64,
    pub last_updated: DateTime<Utc>,
    pub data_points: Vec<TrendDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
    Volatile,
    InsufficientData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendDataPoint {
    pub timestamp: DateTime<Utc>,
    pub coverage_value: f64,
    pub change_from_previous: f64,
    pub significance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageTrendAnalysis {
    pub overall_trend: TrendSummary,
    pub language_trends: HashMap<String, TrendSummary>,
    pub coverage_predictions: Vec<CoveragePrediction>,
    pub regression_alerts: Vec<CoverageRegression>,
    pub improvement_highlights: Vec<CoverageImprovement>,
    pub recommendations: Vec<CoverageRecommendation>,
    pub quality_gates: Vec<QualityGateStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendSummary {
    pub direction: TrendDirection,
    pub current_coverage: f64,
    pub change_7d: f64,
    pub change_30d: f64,
    pub volatility_score: f64,
    pub confidence_level: f64,
    pub projected_next_week: f64,
    pub projected_next_month: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoveragePrediction {
    pub target_date: DateTime<Utc>,
    pub predicted_coverage: f64,
    pub confidence_interval: (f64, f64),
    pub scenario: PredictionScenario,
    pub assumptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictionScenario {
    Conservative,
    Realistic,
    Optimistic,
    CurrentTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageRegression {
    pub detected_at: DateTime<Utc>,
    pub severity: RegressionSeverity,
    pub affected_language: String,
    pub coverage_drop: f64,
    pub likely_cause: String,
    pub impact_analysis: String,
    pub suggested_actions: Vec<String>,
    pub commit_range: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    Critical,
    Major,
    Minor,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageImprovement {
    pub detected_at: DateTime<Utc>,
    pub language: String,
    pub coverage_increase: f64,
    pub achievement_type: AchievementType,
    pub description: String,
    pub impact_summary: String,
    pub contributing_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementType {
    MajorMilestone,
    SteadyProgress,
    QuickWin,
    TechnicalDebtReduction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageRecommendation {
    pub priority: RecommendationPriority,
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub expected_impact: f64,
    pub effort_estimate: String,
    pub implementation_steps: Vec<String>,
    pub success_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    TestGaps,
    IntegrationCoverage,
    EdgeCases,
    PerformanceTests,
    SecurityTests,
    DocumentationTests,
    RegressionPrevention,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateStatus {
    pub gate_name: String,
    pub status: GateStatus,
    pub current_value: f64,
    pub threshold: f64,
    pub trend: TrendDirection,
    pub last_passed: Option<DateTime<Utc>>,
    pub blocking_release: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GateStatus {
    Passing,
    Warning,
    Failing,
    NotApplicable,
}

impl Default for CoverageTrendConfig {
    fn default() -> Self {
        let mut language_weights = HashMap::new();
        language_weights.insert("rust".to_string(), 0.6);
        language_weights.insert("typescript".to_string(), 0.4);

        Self {
            analysis_window_days: 30,
            trend_detection_sensitivity: 0.7,
            regression_threshold: -2.0,
            improvement_threshold: 1.0,
            minimum_data_points: 5,
            language_weights,
            alert_thresholds: CoverageAlertThresholds {
                critical_regression: -5.0,
                major_regression: -3.0,
                minor_regression: -1.0,
                improvement_celebration: 2.0,
            },
        }
    }
}

impl CoverageTrendAnalyzer {
    pub fn new(config: CoverageTrendConfig) -> Self {
        Self {
            config,
            historical_snapshots: Vec::new(),
            trend_cache: HashMap::new(),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(CoverageTrendConfig::default())
    }

    pub fn load_historical_data(&mut self, test_results: Vec<TestExecutionResult>) -> Result<(), Box<dyn std::error::Error>> {
        self.historical_snapshots = test_results
            .into_iter()
            .filter_map(|result| self.convert_to_coverage_snapshot(result))
            .collect();

        // Sort by timestamp
        self.historical_snapshots.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Clear cache after loading new data
        self.trend_cache.clear();

        Ok(())
    }

    fn convert_to_coverage_snapshot(&self, result: TestExecutionResult) -> Option<CoverageSnapshot> {
        // Extract coverage data from test execution result
        // This would parse coverage reports from various tools
        let rust_coverage = CoverageMetrics {
            line_coverage: result.coverage_data.get("rust_line_coverage").unwrap_or(&0.0).clone(),
            branch_coverage: result.coverage_data.get("rust_branch_coverage").unwrap_or(&0.0).clone(),
            function_coverage: result.coverage_data.get("rust_function_coverage").unwrap_or(&0.0).clone(),
            statement_coverage: result.coverage_data.get("rust_statement_coverage").unwrap_or(&0.0).clone(),
            lines_covered: result.coverage_data.get("rust_lines_covered").unwrap_or(&0.0).clone() as u32,
            lines_total: result.coverage_data.get("rust_lines_total").unwrap_or(&1.0).clone() as u32,
            branches_covered: result.coverage_data.get("rust_branches_covered").unwrap_or(&0.0).clone() as u32,
            branches_total: result.coverage_data.get("rust_branches_total").unwrap_or(&1.0).clone() as u32,
            functions_covered: result.coverage_data.get("rust_functions_covered").unwrap_or(&0.0).clone() as u32,
            functions_total: result.coverage_data.get("rust_functions_total").unwrap_or(&1.0).clone() as u32,
        };

        let typescript_coverage = CoverageMetrics {
            line_coverage: result.coverage_data.get("ts_line_coverage").unwrap_or(&0.0).clone(),
            branch_coverage: result.coverage_data.get("ts_branch_coverage").unwrap_or(&0.0).clone(),
            function_coverage: result.coverage_data.get("ts_function_coverage").unwrap_or(&0.0).clone(),
            statement_coverage: result.coverage_data.get("ts_statement_coverage").unwrap_or(&0.0).clone(),
            lines_covered: result.coverage_data.get("ts_lines_covered").unwrap_or(&0.0).clone() as u32,
            lines_total: result.coverage_data.get("ts_lines_total").unwrap_or(&1.0).clone() as u32,
            branches_covered: result.coverage_data.get("ts_branches_covered").unwrap_or(&0.0).clone() as u32,
            branches_total: result.coverage_data.get("ts_branches_total").unwrap_or(&1.0).clone() as u32,
            functions_covered: result.coverage_data.get("ts_functions_covered").unwrap_or(&0.0).clone() as u32,
            functions_total: result.coverage_data.get("ts_functions_total").unwrap_or(&1.0).clone() as u32,
        };

        let integration_coverage = CoverageMetrics {
            line_coverage: result.coverage_data.get("integration_line_coverage").unwrap_or(&0.0).clone(),
            branch_coverage: result.coverage_data.get("integration_branch_coverage").unwrap_or(&0.0).clone(),
            function_coverage: result.coverage_data.get("integration_function_coverage").unwrap_or(&0.0).clone(),
            statement_coverage: result.coverage_data.get("integration_statement_coverage").unwrap_or(&0.0).clone(),
            lines_covered: 0, lines_total: 1, branches_covered: 0, branches_total: 1,
            functions_covered: 0, functions_total: 1,
        };

        let e2e_coverage = CoverageMetrics {
            line_coverage: result.coverage_data.get("e2e_line_coverage").unwrap_or(&0.0).clone(),
            branch_coverage: result.coverage_data.get("e2e_branch_coverage").unwrap_or(&0.0).clone(),
            function_coverage: result.coverage_data.get("e2e_function_coverage").unwrap_or(&0.0).clone(),
            statement_coverage: result.coverage_data.get("e2e_statement_coverage").unwrap_or(&0.0).clone(),
            lines_covered: 0, lines_total: 1, branches_covered: 0, branches_total: 1,
            functions_covered: 0, functions_total: 1,
        };

        // Calculate weighted overall metrics
        let weighted_coverage = self.calculate_weighted_coverage(&rust_coverage, &typescript_coverage);
        let quality_score = self.calculate_quality_score(&rust_coverage, &typescript_coverage, &integration_coverage, &e2e_coverage);
        let completeness_index = self.calculate_completeness_index(&rust_coverage, &typescript_coverage);
        let test_effectiveness = self.calculate_test_effectiveness(&result);
        let coverage_debt = self.calculate_coverage_debt(&rust_coverage, &typescript_coverage);

        let overall_metrics = OverallCoverageMetrics {
            weighted_coverage,
            quality_score,
            completeness_index,
            test_effectiveness,
            coverage_debt,
        };

        Some(CoverageSnapshot {
            timestamp: result.timestamp,
            execution_id: result.execution_id,
            branch: result.environment.get("branch").unwrap_or(&"main".to_string()).clone(),
            commit_hash: result.environment.get("commit").unwrap_or(&"unknown".to_string()).clone(),
            rust_coverage,
            typescript_coverage,
            integration_coverage,
            e2e_coverage,
            overall_metrics,
        })
    }

    fn calculate_weighted_coverage(&self, rust: &CoverageMetrics, typescript: &CoverageMetrics) -> f64 {
        let rust_weight = self.config.language_weights.get("rust").unwrap_or(&0.6);
        let ts_weight = self.config.language_weights.get("typescript").unwrap_or(&0.4);

        (rust.line_coverage * rust_weight) + (typescript.line_coverage * ts_weight)
    }

    fn calculate_quality_score(&self, rust: &CoverageMetrics, typescript: &CoverageMetrics,
                               integration: &CoverageMetrics, e2e: &CoverageMetrics) -> f64 {
        // Weighted score considering different types of coverage
        let unit_score = (rust.line_coverage + typescript.line_coverage) / 2.0;
        let integration_score = integration.line_coverage;
        let e2e_score = e2e.line_coverage;
        let branch_score = (rust.branch_coverage + typescript.branch_coverage) / 2.0;

        // Quality score formula: 40% unit, 30% integration, 20% e2e, 10% branch
        (unit_score * 0.4) + (integration_score * 0.3) + (e2e_score * 0.2) + (branch_score * 0.1)
    }

    fn calculate_completeness_index(&self, rust: &CoverageMetrics, typescript: &CoverageMetrics) -> f64 {
        // Measure how complete the test coverage is across all dimensions
        let line_completeness = (rust.line_coverage + typescript.line_coverage) / 200.0;
        let branch_completeness = (rust.branch_coverage + typescript.branch_coverage) / 200.0;
        let function_completeness = (rust.function_coverage + typescript.function_coverage) / 200.0;

        (line_completeness + branch_completeness + function_completeness) / 3.0 * 100.0
    }

    fn calculate_test_effectiveness(&self, result: &TestExecutionResult) -> f64 {
        // Measure how effective tests are at catching issues
        let success_rate = if result.total_tests > 0 {
            (result.successful_tests as f64 / result.total_tests as f64) * 100.0
        } else {
            100.0
        };

        // Factor in test diversity, execution speed, and failure detection
        let diversity_factor = if result.test_suites.len() > 1 { 1.1 } else { 1.0 };
        let speed_factor = if result.duration_ms < 300000 { 1.05 } else { 0.95 }; // Bonus for fast tests

        success_rate * diversity_factor * speed_factor
    }

    fn calculate_coverage_debt(&self, rust: &CoverageMetrics, typescript: &CoverageMetrics) -> f64 {
        // Calculate technical debt in terms of uncovered code
        let target_coverage = 90.0;
        let current_coverage = (rust.line_coverage + typescript.line_coverage) / 2.0;
        let gap = target_coverage - current_coverage;

        if gap > 0.0 {
            gap * (rust.lines_total + typescript.lines_total) as f64 / 100.0
        } else {
            0.0
        }
    }

    pub fn analyze_trends(&mut self) -> Result<CoverageTrendAnalysis, Box<dyn std::error::Error>> {
        if self.historical_snapshots.len() < self.config.minimum_data_points {
            return Err("Insufficient historical data for trend analysis".into());
        }

        let overall_trend = self.analyze_overall_trend()?;
        let language_trends = self.analyze_language_trends()?;
        let coverage_predictions = self.generate_coverage_predictions()?;
        let regression_alerts = self.detect_coverage_regressions()?;
        let improvement_highlights = self.identify_coverage_improvements()?;
        let recommendations = self.generate_coverage_recommendations()?;
        let quality_gates = self.evaluate_quality_gates()?;

        Ok(CoverageTrendAnalysis {
            overall_trend,
            language_trends,
            coverage_predictions,
            regression_alerts,
            improvement_highlights,
            recommendations,
            quality_gates,
        })
    }

    fn analyze_overall_trend(&self) -> Result<TrendSummary, Box<dyn std::error::Error>> {
        let recent_snapshots = self.get_recent_snapshots(30);
        if recent_snapshots.is_empty() {
            return Err("No recent data available".into());
        }

        let current_coverage = recent_snapshots.last().unwrap().overall_metrics.weighted_coverage;
        let trend_direction = self.detect_trend_direction(&recent_snapshots, |s| s.overall_metrics.weighted_coverage);

        let change_7d = self.calculate_coverage_change(7, |s| s.overall_metrics.weighted_coverage);
        let change_30d = self.calculate_coverage_change(30, |s| s.overall_metrics.weighted_coverage);

        let volatility_score = self.calculate_volatility(&recent_snapshots, |s| s.overall_metrics.weighted_coverage);
        let confidence_level = self.calculate_confidence_level(&recent_snapshots);

        let projected_next_week = self.project_coverage(7, &recent_snapshots, |s| s.overall_metrics.weighted_coverage);
        let projected_next_month = self.project_coverage(30, &recent_snapshots, |s| s.overall_metrics.weighted_coverage);

        Ok(TrendSummary {
            direction: trend_direction,
            current_coverage,
            change_7d,
            change_30d,
            volatility_score,
            confidence_level,
            projected_next_week,
            projected_next_month,
        })
    }

    fn analyze_language_trends(&self) -> Result<HashMap<String, TrendSummary>, Box<dyn std::error::Error>> {
        let mut language_trends = HashMap::new();

        // Analyze Rust trends
        let rust_trend = self.analyze_language_specific_trend("rust", |s| s.rust_coverage.line_coverage)?;
        language_trends.insert("rust".to_string(), rust_trend);

        // Analyze TypeScript trends
        let ts_trend = self.analyze_language_specific_trend("typescript", |s| s.typescript_coverage.line_coverage)?;
        language_trends.insert("typescript".to_string(), ts_trend);

        // Analyze Integration trends
        let integration_trend = self.analyze_language_specific_trend("integration", |s| s.integration_coverage.line_coverage)?;
        language_trends.insert("integration".to_string(), integration_trend);

        // Analyze E2E trends
        let e2e_trend = self.analyze_language_specific_trend("e2e", |s| s.e2e_coverage.line_coverage)?;
        language_trends.insert("e2e".to_string(), e2e_trend);

        Ok(language_trends)
    }

    fn analyze_language_specific_trend<F>(&self, language: &str, extractor: F) -> Result<TrendSummary, Box<dyn std::error::Error>>
    where
        F: Fn(&CoverageSnapshot) -> f64 + Copy,
    {
        let recent_snapshots = self.get_recent_snapshots(30);
        if recent_snapshots.is_empty() {
            return Err(format!("No recent data available for {}", language).into());
        }

        let current_coverage = extractor(recent_snapshots.last().unwrap());
        let trend_direction = self.detect_trend_direction(&recent_snapshots, extractor);

        let change_7d = self.calculate_coverage_change(7, extractor);
        let change_30d = self.calculate_coverage_change(30, extractor);

        let volatility_score = self.calculate_volatility(&recent_snapshots, extractor);
        let confidence_level = self.calculate_confidence_level(&recent_snapshots);

        let projected_next_week = self.project_coverage(7, &recent_snapshots, extractor);
        let projected_next_month = self.project_coverage(30, &recent_snapshots, extractor);

        Ok(TrendSummary {
            direction: trend_direction,
            current_coverage,
            change_7d,
            change_30d,
            volatility_score,
            confidence_level,
            projected_next_week,
            projected_next_month,
        })
    }

    fn get_recent_snapshots(&self, days: u32) -> Vec<&CoverageSnapshot> {
        let cutoff = Utc::now() - Duration::days(days as i64);
        self.historical_snapshots
            .iter()
            .filter(|s| s.timestamp >= cutoff)
            .collect()
    }

    fn detect_trend_direction<F>(&self, snapshots: &[&CoverageSnapshot], extractor: F) -> TrendDirection
    where
        F: Fn(&CoverageSnapshot) -> f64,
    {
        if snapshots.len() < 3 {
            return TrendDirection::InsufficientData;
        }

        let values: Vec<f64> = snapshots.iter().map(|s| extractor(s)).collect();
        let recent_trend = self.calculate_linear_regression_slope(&values);
        let volatility = self.calculate_volatility(snapshots, extractor);

        match (recent_trend, volatility) {
            (slope, vol) if vol > 5.0 => TrendDirection::Volatile,
            (slope, _) if slope > self.config.improvement_threshold => TrendDirection::Improving,
            (slope, _) if slope < self.config.regression_threshold => TrendDirection::Declining,
            _ => TrendDirection::Stable,
        }
    }

    fn calculate_coverage_change<F>(&self, days: u32, extractor: F) -> f64
    where
        F: Fn(&CoverageSnapshot) -> f64,
    {
        let recent_snapshots = self.get_recent_snapshots(days);
        if recent_snapshots.len() < 2 {
            return 0.0;
        }

        let latest = extractor(recent_snapshots.last().unwrap());
        let earliest = extractor(recent_snapshots.first().unwrap());

        latest - earliest
    }

    fn calculate_volatility<F>(&self, snapshots: &[&CoverageSnapshot], extractor: F) -> f64
    where
        F: Fn(&CoverageSnapshot) -> f64,
    {
        if snapshots.len() < 2 {
            return 0.0;
        }

        let values: Vec<f64> = snapshots.iter().map(|s| extractor(s)).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;

        variance.sqrt()
    }

    fn calculate_confidence_level(&self, snapshots: &[&CoverageSnapshot]) -> f64 {
        // Confidence based on data points, recency, and consistency
        let data_points_factor = (snapshots.len() as f64 / 20.0).min(1.0);
        let recency_factor = if snapshots.last().unwrap().timestamp > Utc::now() - Duration::days(1) {
            1.0
        } else {
            0.8
        };

        (data_points_factor * recency_factor * 100.0).min(95.0)
    }

    fn project_coverage<F>(&self, days_ahead: u32, snapshots: &[&CoverageSnapshot], extractor: F) -> f64
    where
        F: Fn(&CoverageSnapshot) -> f64,
    {
        if snapshots.len() < 3 {
            return extractor(snapshots.last().unwrap());
        }

        let values: Vec<f64> = snapshots.iter().map(|s| extractor(s)).collect();
        let slope = self.calculate_linear_regression_slope(&values);
        let current = values.last().unwrap();

        current + (slope * days_ahead as f64)
    }

    fn calculate_linear_regression_slope(&self, values: &[f64]) -> f64 {
        let n = values.len() as f64;
        if n < 2.0 {
            return 0.0;
        }

        let x_mean = (n - 1.0) / 2.0;
        let y_mean = values.iter().sum::<f64>() / n;

        let numerator: f64 = values
            .iter()
            .enumerate()
            .map(|(i, &y)| (i as f64 - x_mean) * (y - y_mean))
            .sum();

        let denominator: f64 = (0..values.len())
            .map(|i| (i as f64 - x_mean).powi(2))
            .sum();

        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    fn generate_coverage_predictions(&self) -> Result<Vec<CoveragePrediction>, Box<dyn std::error::Error>> {
        let mut predictions = Vec::new();
        let recent_snapshots = self.get_recent_snapshots(30);

        if recent_snapshots.is_empty() {
            return Ok(predictions);
        }

        // Generate predictions for different time horizons
        for (days, scenario) in [(7, PredictionScenario::Conservative), (30, PredictionScenario::Realistic), (90, PredictionScenario::Optimistic)] {
            let target_date = Utc::now() + Duration::days(days);
            let predicted_coverage = self.project_coverage(days as u32, &recent_snapshots, |s| s.overall_metrics.weighted_coverage);

            let confidence_interval = self.calculate_prediction_confidence_interval(predicted_coverage, days as f64);

            predictions.push(CoveragePrediction {
                target_date,
                predicted_coverage,
                confidence_interval,
                scenario,
                assumptions: vec![
                    "Current development velocity continues".to_string(),
                    "No major architectural changes".to_string(),
                    "Test-first development practices maintained".to_string(),
                ],
            });
        }

        Ok(predictions)
    }

    fn calculate_prediction_confidence_interval(&self, predicted: f64, days_ahead: f64) -> (f64, f64) {
        // Wider intervals for longer predictions
        let uncertainty = 2.0 + (days_ahead / 30.0) * 3.0;
        (
            (predicted - uncertainty).max(0.0),
            (predicted + uncertainty).min(100.0)
        )
    }

    fn detect_coverage_regressions(&self) -> Result<Vec<CoverageRegression>, Box<dyn std::error::Error>> {
        let mut regressions = Vec::new();
        let recent_snapshots = self.get_recent_snapshots(7);

        if recent_snapshots.len() < 2 {
            return Ok(regressions);
        }

        for (i, snapshot) in recent_snapshots.iter().enumerate().skip(1) {
            let previous = recent_snapshots[i - 1];

            // Check overall coverage regression
            let coverage_drop = previous.overall_metrics.weighted_coverage - snapshot.overall_metrics.weighted_coverage;
            if coverage_drop > self.config.alert_thresholds.minor_regression.abs() {
                let severity = if coverage_drop > self.config.alert_thresholds.critical_regression.abs() {
                    RegressionSeverity::Critical
                } else if coverage_drop > self.config.alert_thresholds.major_regression.abs() {
                    RegressionSeverity::Major
                } else {
                    RegressionSeverity::Minor
                };

                regressions.push(CoverageRegression {
                    detected_at: snapshot.timestamp,
                    severity,
                    affected_language: "overall".to_string(),
                    coverage_drop,
                    likely_cause: self.analyze_regression_cause(previous, snapshot),
                    impact_analysis: format!("Coverage dropped by {:.2}% affecting overall quality score", coverage_drop),
                    suggested_actions: self.generate_regression_actions(coverage_drop),
                    commit_range: Some(format!("{}..{}", previous.commit_hash, snapshot.commit_hash)),
                });
            }
        }

        Ok(regressions)
    }

    fn analyze_regression_cause(&self, previous: &CoverageSnapshot, current: &CoverageSnapshot) -> String {
        let rust_drop = previous.rust_coverage.line_coverage - current.rust_coverage.line_coverage;
        let ts_drop = previous.typescript_coverage.line_coverage - current.typescript_coverage.line_coverage;

        if rust_drop > ts_drop {
            "Rust code changes without corresponding test updates".to_string()
        } else if ts_drop > rust_drop {
            "TypeScript/frontend changes without adequate test coverage".to_string()
        } else {
            "General reduction in test coverage across multiple areas".to_string()
        }
    }

    fn generate_regression_actions(&self, coverage_drop: f64) -> Vec<String> {
        let mut actions = vec![
            "Review recent commits for untested code paths".to_string(),
            "Run coverage analysis to identify specific gaps".to_string(),
        ];

        if coverage_drop > 3.0 {
            actions.push("Consider blocking deployment until coverage is restored".to_string());
            actions.push("Conduct team review of testing practices".to_string());
        }

        actions.push("Add tests for newly identified uncovered areas".to_string());
        actions.push("Update CI/CD pipeline to prevent future regressions".to_string());

        actions
    }

    fn identify_coverage_improvements(&self) -> Result<Vec<CoverageImprovement>, Box<dyn std::error::Error>> {
        let mut improvements = Vec::new();
        let recent_snapshots = self.get_recent_snapshots(14);

        if recent_snapshots.len() < 2 {
            return Ok(improvements);
        }

        for (i, snapshot) in recent_snapshots.iter().enumerate().skip(1) {
            let previous = recent_snapshots[i - 1];

            // Check for improvements in each language
            let rust_improvement = snapshot.rust_coverage.line_coverage - previous.rust_coverage.line_coverage;
            if rust_improvement >= self.config.alert_thresholds.improvement_celebration {
                improvements.push(CoverageImprovement {
                    detected_at: snapshot.timestamp,
                    language: "rust".to_string(),
                    coverage_increase: rust_improvement,
                    achievement_type: self.categorize_achievement(rust_improvement),
                    description: format!("Rust coverage improved by {:.2}%", rust_improvement),
                    impact_summary: "Enhanced backend reliability and maintainability".to_string(),
                    contributing_factors: vec![
                        "Additional unit tests".to_string(),
                        "Improved integration test coverage".to_string(),
                        "Better error handling tests".to_string(),
                    ],
                });
            }

            let ts_improvement = snapshot.typescript_coverage.line_coverage - previous.typescript_coverage.line_coverage;
            if ts_improvement >= self.config.alert_thresholds.improvement_celebration {
                improvements.push(CoverageImprovement {
                    detected_at: snapshot.timestamp,
                    language: "typescript".to_string(),
                    coverage_increase: ts_improvement,
                    achievement_type: self.categorize_achievement(ts_improvement),
                    description: format!("TypeScript coverage improved by {:.2}%", ts_improvement),
                    impact_summary: "Enhanced frontend reliability and user experience".to_string(),
                    contributing_factors: vec![
                        "Component testing improvements".to_string(),
                        "Better UI interaction tests".to_string(),
                        "Enhanced error boundary testing".to_string(),
                    ],
                });
            }
        }

        Ok(improvements)
    }

    fn categorize_achievement(&self, improvement: f64) -> AchievementType {
        match improvement {
            x if x >= 10.0 => AchievementType::MajorMilestone,
            x if x >= 5.0 => AchievementType::TechnicalDebtReduction,
            x if x >= 3.0 => AchievementType::SteadyProgress,
            _ => AchievementType::QuickWin,
        }
    }

    fn generate_coverage_recommendations(&self) -> Result<Vec<CoverageRecommendation>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();

        if let Some(latest_snapshot) = self.historical_snapshots.last() {
            // Analyze gaps and generate targeted recommendations

            // Rust coverage recommendations
            if latest_snapshot.rust_coverage.line_coverage < 85.0 {
                recommendations.push(CoverageRecommendation {
                    priority: RecommendationPriority::High,
                    category: RecommendationCategory::TestGaps,
                    title: "Improve Rust Unit Test Coverage".to_string(),
                    description: "Rust line coverage is below target threshold of 85%".to_string(),
                    expected_impact: 85.0 - latest_snapshot.rust_coverage.line_coverage,
                    effort_estimate: "2-3 days".to_string(),
                    implementation_steps: vec![
                        "Run cargo tarpaulin to identify uncovered lines".to_string(),
                        "Focus on error handling and edge cases".to_string(),
                        "Add tests for business logic functions".to_string(),
                        "Implement property-based testing for complex algorithms".to_string(),
                    ],
                    success_metrics: vec![
                        "Rust line coverage > 85%".to_string(),
                        "Branch coverage > 80%".to_string(),
                        "All critical paths tested".to_string(),
                    ],
                });
            }

            // Integration testing recommendations
            if latest_snapshot.integration_coverage.line_coverage < 70.0 {
                recommendations.push(CoverageRecommendation {
                    priority: RecommendationPriority::Critical,
                    category: RecommendationCategory::IntegrationCoverage,
                    title: "Enhance Integration Test Suite".to_string(),
                    description: "Integration test coverage is insufficient for reliable deployments".to_string(),
                    expected_impact: 25.0,
                    effort_estimate: "1 week".to_string(),
                    implementation_steps: vec![
                        "Set up Docker Compose test environment".to_string(),
                        "Create API endpoint integration tests".to_string(),
                        "Test database operations and migrations".to_string(),
                        "Validate authentication and authorization flows".to_string(),
                        "Test external service integrations".to_string(),
                    ],
                    success_metrics: vec![
                        "Integration coverage > 70%".to_string(),
                        "All API endpoints tested".to_string(),
                        "Database operations fully covered".to_string(),
                        "External integrations mocked and tested".to_string(),
                    ],
                });
            }

            // Performance testing recommendations
            recommendations.push(CoverageRecommendation {
                priority: RecommendationPriority::Medium,
                category: RecommendationCategory::PerformanceTests,
                title: "Add Performance Regression Tests".to_string(),
                description: "Establish baseline performance tests to prevent regressions".to_string(),
                expected_impact: 10.0,
                effort_estimate: "3-4 days".to_string(),
                implementation_steps: vec![
                    "Set up criterion.rs for Rust benchmarks".to_string(),
                    "Create load testing with Playwright".to_string(),
                    "Establish performance budgets".to_string(),
                    "Integrate performance tests into CI".to_string(),
                ],
                success_metrics: vec![
                    "Performance tests running in CI".to_string(),
                    "Baseline metrics established".to_string(),
                    "Regression alerts configured".to_string(),
                ],
            });
        }

        Ok(recommendations)
    }

    fn evaluate_quality_gates(&self) -> Result<Vec<QualityGateStatus>, Box<dyn std::error::Error>> {
        let mut gates = Vec::new();

        if let Some(latest_snapshot) = self.historical_snapshots.last() {
            // Overall coverage gate
            gates.push(QualityGateStatus {
                gate_name: "Overall Coverage".to_string(),
                status: if latest_snapshot.overall_metrics.weighted_coverage >= 80.0 {
                    GateStatus::Passing
                } else if latest_snapshot.overall_metrics.weighted_coverage >= 70.0 {
                    GateStatus::Warning
                } else {
                    GateStatus::Failing
                },
                current_value: latest_snapshot.overall_metrics.weighted_coverage,
                threshold: 80.0,
                trend: self.detect_trend_direction(&self.get_recent_snapshots(7), |s| s.overall_metrics.weighted_coverage),
                last_passed: self.find_last_gate_pass(80.0, |s| s.overall_metrics.weighted_coverage),
                blocking_release: latest_snapshot.overall_metrics.weighted_coverage < 70.0,
            });

            // Rust coverage gate
            gates.push(QualityGateStatus {
                gate_name: "Rust Coverage".to_string(),
                status: if latest_snapshot.rust_coverage.line_coverage >= 85.0 {
                    GateStatus::Passing
                } else if latest_snapshot.rust_coverage.line_coverage >= 75.0 {
                    GateStatus::Warning
                } else {
                    GateStatus::Failing
                },
                current_value: latest_snapshot.rust_coverage.line_coverage,
                threshold: 85.0,
                trend: self.detect_trend_direction(&self.get_recent_snapshots(7), |s| s.rust_coverage.line_coverage),
                last_passed: self.find_last_gate_pass(85.0, |s| s.rust_coverage.line_coverage),
                blocking_release: latest_snapshot.rust_coverage.line_coverage < 75.0,
            });

            // TypeScript coverage gate
            gates.push(QualityGateStatus {
                gate_name: "TypeScript Coverage".to_string(),
                status: if latest_snapshot.typescript_coverage.line_coverage >= 80.0 {
                    GateStatus::Passing
                } else if latest_snapshot.typescript_coverage.line_coverage >= 70.0 {
                    GateStatus::Warning
                } else {
                    GateStatus::Failing
                },
                current_value: latest_snapshot.typescript_coverage.line_coverage,
                threshold: 80.0,
                trend: self.detect_trend_direction(&self.get_recent_snapshots(7), |s| s.typescript_coverage.line_coverage),
                last_passed: self.find_last_gate_pass(80.0, |s| s.typescript_coverage.line_coverage),
                blocking_release: latest_snapshot.typescript_coverage.line_coverage < 70.0,
            });

            // Quality score gate
            gates.push(QualityGateStatus {
                gate_name: "Quality Score".to_string(),
                status: if latest_snapshot.overall_metrics.quality_score >= 85.0 {
                    GateStatus::Passing
                } else if latest_snapshot.overall_metrics.quality_score >= 75.0 {
                    GateStatus::Warning
                } else {
                    GateStatus::Failing
                },
                current_value: latest_snapshot.overall_metrics.quality_score,
                threshold: 85.0,
                trend: self.detect_trend_direction(&self.get_recent_snapshots(7), |s| s.overall_metrics.quality_score),
                last_passed: self.find_last_gate_pass(85.0, |s| s.overall_metrics.quality_score),
                blocking_release: latest_snapshot.overall_metrics.quality_score < 75.0,
            });
        }

        Ok(gates)
    }

    fn find_last_gate_pass<F>(&self, threshold: f64, extractor: F) -> Option<DateTime<Utc>>
    where
        F: Fn(&CoverageSnapshot) -> f64,
    {
        self.historical_snapshots
            .iter()
            .rev()
            .find(|snapshot| extractor(snapshot) >= threshold)
            .map(|snapshot| snapshot.timestamp)
    }

    pub fn get_current_coverage_summary(&self) -> Option<CoverageSnapshot> {
        self.historical_snapshots.last().cloned()
    }

    pub fn export_trend_data(&self) -> Result<String, Box<dyn std::error::Error>> {
        let analysis = self.analyze_trends()?;
        serde_json::to_string_pretty(&analysis).map_err(|e| e.into())
    }
}

impl Default for CoverageTrendAnalyzer {
    fn default() -> Self {
        Self::with_default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_coverage_trend_analyzer_creation() {
        let analyzer = CoverageTrendAnalyzer::with_default_config();
        assert_eq!(analyzer.config.analysis_window_days, 30);
        assert_eq!(analyzer.historical_snapshots.len(), 0);
    }

    #[test]
    fn test_weighted_coverage_calculation() {
        let analyzer = CoverageTrendAnalyzer::with_default_config();
        let rust_coverage = CoverageMetrics {
            line_coverage: 90.0,
            branch_coverage: 85.0,
            function_coverage: 88.0,
            statement_coverage: 89.0,
            lines_covered: 900, lines_total: 1000,
            branches_covered: 85, branches_total: 100,
            functions_covered: 88, functions_total: 100,
        };
        let typescript_coverage = CoverageMetrics {
            line_coverage: 80.0,
            branch_coverage: 75.0,
            function_coverage: 78.0,
            statement_coverage: 79.0,
            lines_covered: 800, lines_total: 1000,
            branches_covered: 75, branches_total: 100,
            functions_covered: 78, functions_total: 100,
        };

        let weighted = analyzer.calculate_weighted_coverage(&rust_coverage, &typescript_coverage);
        assert_eq!(weighted, 86.0); // 90 * 0.6 + 80 * 0.4
    }

    #[test]
    fn test_trend_direction_detection() {
        let analyzer = CoverageTrendAnalyzer::with_default_config();

        // Create test snapshots with improving trend
        let snapshots: Vec<CoverageSnapshot> = (0..5)
            .map(|i| CoverageSnapshot {
                timestamp: Utc.timestamp_opt(1640995200 + i * 86400, 0).single().unwrap(),
                execution_id: format!("test_{}", i),
                branch: "main".to_string(),
                commit_hash: format!("commit_{}", i),
                rust_coverage: CoverageMetrics {
                    line_coverage: 70.0 + (i as f64 * 2.0),
                    branch_coverage: 0.0, function_coverage: 0.0, statement_coverage: 0.0,
                    lines_covered: 0, lines_total: 1, branches_covered: 0, branches_total: 1,
                    functions_covered: 0, functions_total: 1,
                },
                typescript_coverage: CoverageMetrics {
                    line_coverage: 60.0 + (i as f64 * 2.0),
                    branch_coverage: 0.0, function_coverage: 0.0, statement_coverage: 0.0,
                    lines_covered: 0, lines_total: 1, branches_covered: 0, branches_total: 1,
                    functions_covered: 0, functions_total: 1,
                },
                integration_coverage: CoverageMetrics {
                    line_coverage: 50.0, branch_coverage: 0.0, function_coverage: 0.0, statement_coverage: 0.0,
                    lines_covered: 0, lines_total: 1, branches_covered: 0, branches_total: 1,
                    functions_covered: 0, functions_total: 1,
                },
                e2e_coverage: CoverageMetrics {
                    line_coverage: 40.0, branch_coverage: 0.0, function_coverage: 0.0, statement_coverage: 0.0,
                    lines_covered: 0, lines_total: 1, branches_covered: 0, branches_total: 1,
                    functions_covered: 0, functions_total: 1,
                },
                overall_metrics: OverallCoverageMetrics {
                    weighted_coverage: 65.0 + (i as f64 * 2.0),
                    quality_score: 70.0,
                    completeness_index: 80.0,
                    test_effectiveness: 90.0,
                    coverage_debt: 10.0,
                },
            })
            .collect();

        let snapshot_refs: Vec<&CoverageSnapshot> = snapshots.iter().collect();
        let direction = analyzer.detect_trend_direction(&snapshot_refs, |s| s.overall_metrics.weighted_coverage);

        assert!(matches!(direction, TrendDirection::Improving));
    }
}