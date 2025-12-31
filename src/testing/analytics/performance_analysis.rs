use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use chrono::{DateTime, Utc, Duration};
use crate::testing::entities::TestExecutionResult;
use super::{AnalyticsResult, InsightLevel, Insight};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalyzer {
    pub config: PerformanceAnalysisConfig,
    historical_data: Vec<PerformanceSnapshot>,
    baseline_metrics: HashMap<String, BaselineMetric>,
    regression_cache: HashMap<String, RegressionAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysisConfig {
    pub regression_threshold_percent: f64,
    pub improvement_threshold_percent: f64,
    pub analysis_window_days: u32,
    pub statistical_confidence: f64,
    pub outlier_detection_enabled: bool,
    pub baseline_update_frequency_days: u32,
    pub performance_categories: Vec<PerformanceCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceCategory {
    pub name: String,
    pub weight: f64,
    pub thresholds: PerformanceThresholds,
    pub metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub excellent: f64,
    pub good: f64,
    pub acceptable: f64,
    pub poor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub execution_id: String,
    pub environment: String,
    pub test_suite: String,
    pub execution_metrics: ExecutionMetrics,
    pub resource_metrics: ResourceMetrics,
    pub throughput_metrics: ThroughputMetrics,
    pub latency_metrics: LatencyMetrics,
    pub quality_metrics: QualityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub total_duration_ms: u64,
    pub setup_duration_ms: u64,
    pub test_duration_ms: u64,
    pub teardown_duration_ms: u64,
    pub parallel_efficiency: f64,
    pub test_count: u32,
    pub avg_test_duration_ms: f64,
    pub slowest_test_duration_ms: u64,
    pub fastest_test_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub max_memory_mb: f64,
    pub avg_memory_mb: f64,
    pub max_cpu_percent: f64,
    pub avg_cpu_percent: f64,
    pub disk_io_mb: f64,
    pub network_io_mb: f64,
    pub gc_collections: u32,
    pub gc_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    pub tests_per_second: f64,
    pub assertions_per_second: f64,
    pub operations_per_second: f64,
    pub concurrent_capacity: u32,
    pub scalability_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetrics {
    pub p50_response_ms: f64,
    pub p90_response_ms: f64,
    pub p95_response_ms: f64,
    pub p99_response_ms: f64,
    pub max_response_ms: f64,
    pub avg_response_ms: f64,
    pub response_time_variability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub stability_score: f64,
    pub reliability_index: f64,
    pub performance_consistency: f64,
    pub resource_efficiency: f64,
    pub scalability_rating: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetric {
    pub metric_name: String,
    pub baseline_value: f64,
    pub confidence_interval: (f64, f64),
    pub last_updated: DateTime<Utc>,
    pub sample_size: usize,
    pub standard_deviation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    pub metric_name: String,
    pub regression_detected: bool,
    pub severity: RegressionSeverity,
    pub current_value: f64,
    pub baseline_value: f64,
    pub percentage_change: f64,
    pub statistical_significance: f64,
    pub detection_timestamp: DateTime<Utc>,
    pub affected_components: Vec<String>,
    pub likely_causes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    Critical,    // >20% degradation
    Major,       // >10% degradation
    Minor,       // >5% degradation
    Warning,     // >2% degradation
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysisReport {
    pub summary: PerformanceSummary,
    pub trend_analysis: TrendAnalysis,
    pub regression_report: Vec<RegressionAnalysis>,
    pub improvement_highlights: Vec<PerformanceImprovement>,
    pub bottleneck_analysis: BottleneckAnalysis,
    pub recommendations: Vec<PerformanceRecommendation>,
    pub benchmarks: BenchmarkComparison,
    pub forecasts: Vec<PerformanceForecast>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub overall_score: f64,
    pub grade: PerformanceGrade,
    pub current_metrics: PerformanceSnapshot,
    pub vs_baseline_percent: f64,
    pub vs_previous_percent: f64,
    pub stability_rating: StabilityRating,
    pub key_insights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceGrade {
    Excellent, // 90-100
    Good,      // 80-89
    Acceptable, // 70-79
    Poor,      // 60-69
    Critical,  // <60
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StabilityRating {
    VeryStable,   // <5% variance
    Stable,       // 5-10% variance
    Moderate,     // 10-20% variance
    Unstable,     // 20-30% variance
    VeryUnstable, // >30% variance
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub execution_trend: TrendDirection,
    pub memory_trend: TrendDirection,
    pub throughput_trend: TrendDirection,
    pub latency_trend: TrendDirection,
    pub overall_trend: TrendDirection,
    pub trend_confidence: f64,
    pub inflection_points: Vec<InflectionPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    StronglyImproving,
    Improving,
    Stable,
    Declining,
    StronglyDeclining,
    Volatile,
    InsufficientData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InflectionPoint {
    pub timestamp: DateTime<Utc>,
    pub metric: String,
    pub change_type: ChangeType,
    pub magnitude: f64,
    pub likely_cause: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Improvement,
    Degradation,
    Spike,
    Drop,
    VolatilityIncrease,
    VolatilityDecrease,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImprovement {
    pub detected_at: DateTime<Utc>,
    pub metric: String,
    pub improvement_percent: f64,
    pub significance: ImprovementSignificance,
    pub description: String,
    pub contributing_factors: Vec<String>,
    pub sustainable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImprovementSignificance {
    Major,      // >15% improvement
    Moderate,   // 5-15% improvement
    Minor,      // 2-5% improvement
    Marginal,   // <2% improvement
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckAnalysis {
    pub primary_bottlenecks: Vec<Bottleneck>,
    pub resource_constraints: Vec<ResourceConstraint>,
    pub scaling_limitations: Vec<ScalingLimitation>,
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    pub component: String,
    pub metric: String,
    pub impact_score: f64,
    pub description: String,
    pub resolution_priority: Priority,
    pub estimated_improvement: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConstraint {
    pub resource_type: ResourceType,
    pub utilization_percent: f64,
    pub constraint_level: ConstraintLevel,
    pub impact_on_performance: String,
    pub mitigation_strategies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Memory,
    CPU,
    DiskIO,
    NetworkIO,
    Database,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintLevel {
    Severe,    // >90% utilization
    High,      // 80-90% utilization
    Moderate,  // 70-80% utilization
    Low,       // <70% utilization
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingLimitation {
    pub component: String,
    pub limitation_type: LimitationType,
    pub threshold: f64,
    pub current_utilization: f64,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LimitationType {
    ConcurrencyLimit,
    MemoryLimit,
    ProcessingLimit,
    IOLimit,
    NetworkLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOpportunity {
    pub area: String,
    pub opportunity_type: OpportunityType,
    pub potential_impact: f64,
    pub implementation_effort: EffortLevel,
    pub description: String,
    pub implementation_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpportunityType {
    AlgorithmicImprovement,
    CachingStrategy,
    ParallelizationImprovement,
    ResourceOptimization,
    ArchitecturalChange,
    ConfigurationTuning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Low,      // <1 day
    Medium,   // 1-3 days
    High,     // 1-2 weeks
    VeryHigh, // >2 weeks
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendation {
    pub priority: Priority,
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub expected_impact: f64,
    pub implementation_effort: EffortLevel,
    pub cost_benefit_ratio: f64,
    pub implementation_steps: Vec<String>,
    pub success_metrics: Vec<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Infrastructure,
    CodeOptimization,
    TestOptimization,
    ResourceManagement,
    Architecture,
    Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparison {
    pub industry_percentile: f64,
    pub vs_similar_projects: ComparisonResult,
    pub historical_best: HistoricalBest,
    pub competitive_analysis: Vec<CompetitorBenchmark>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonResult {
    SignificantlyAbove,
    Above,
    Similar,
    Below,
    SignificantlyBelow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalBest {
    pub timestamp: DateTime<Utc>,
    pub metrics: PerformanceSnapshot,
    pub gap_to_current: f64,
    pub factors_for_best_performance: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitorBenchmark {
    pub competitor: String,
    pub metric_comparison: HashMap<String, f64>,
    pub overall_comparison: ComparisonResult,
    pub key_differentiators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceForecast {
    pub metric: String,
    pub forecast_horizon_days: u32,
    pub predicted_value: f64,
    pub confidence_interval: (f64, f64),
    pub scenario: ForecastScenario,
    pub assumptions: Vec<String>,
    pub risk_factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForecastScenario {
    Optimistic,
    Realistic,
    Pessimistic,
    CurrentTrend,
}

impl Default for PerformanceAnalysisConfig {
    fn default() -> Self {
        Self {
            regression_threshold_percent: -5.0,
            improvement_threshold_percent: 5.0,
            analysis_window_days: 30,
            statistical_confidence: 0.95,
            outlier_detection_enabled: true,
            baseline_update_frequency_days: 7,
            performance_categories: vec![
                PerformanceCategory {
                    name: "Execution Speed".to_string(),
                    weight: 0.4,
                    thresholds: PerformanceThresholds {
                        excellent: 100.0,
                        good: 500.0,
                        acceptable: 2000.0,
                        poor: 5000.0,
                    },
                    metrics: vec!["total_duration_ms".to_string(), "avg_test_duration_ms".to_string()],
                },
                PerformanceCategory {
                    name: "Resource Efficiency".to_string(),
                    weight: 0.3,
                    thresholds: PerformanceThresholds {
                        excellent: 50.0,
                        good: 100.0,
                        acceptable: 200.0,
                        poor: 500.0,
                    },
                    metrics: vec!["max_memory_mb".to_string(), "avg_cpu_percent".to_string()],
                },
                PerformanceCategory {
                    name: "Throughput".to_string(),
                    weight: 0.3,
                    thresholds: PerformanceThresholds {
                        excellent: 100.0,
                        good: 50.0,
                        acceptable: 20.0,
                        poor: 5.0,
                    },
                    metrics: vec!["tests_per_second".to_string(), "operations_per_second".to_string()],
                },
            ],
        }
    }
}

impl PerformanceAnalyzer {
    pub fn new(config: PerformanceAnalysisConfig) -> Self {
        Self {
            config,
            historical_data: Vec::new(),
            baseline_metrics: HashMap::new(),
            regression_cache: HashMap::new(),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(PerformanceAnalysisConfig::default())
    }

    pub fn load_historical_data(&mut self, test_results: Vec<TestExecutionResult>) -> Result<(), Box<dyn std::error::Error>> {
        self.historical_data = test_results
            .into_iter()
            .filter_map(|result| self.convert_to_performance_snapshot(result))
            .collect();

        // Sort by timestamp
        self.historical_data.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Update baselines
        self.update_baselines()?;

        Ok(())
    }

    fn convert_to_performance_snapshot(&self, result: TestExecutionResult) -> Option<PerformanceSnapshot> {
        let execution_metrics = ExecutionMetrics {
            total_duration_ms: result.duration_ms,
            setup_duration_ms: result.metadata.get("setup_duration_ms").unwrap_or(&"0".to_string()).parse().unwrap_or(0),
            test_duration_ms: result.duration_ms - result.metadata.get("setup_duration_ms").unwrap_or(&"0".to_string()).parse().unwrap_or(0),
            teardown_duration_ms: result.metadata.get("teardown_duration_ms").unwrap_or(&"0".to_string()).parse().unwrap_or(0),
            parallel_efficiency: result.metadata.get("parallel_efficiency").unwrap_or(&"1.0".to_string()).parse().unwrap_or(1.0),
            test_count: result.total_tests,
            avg_test_duration_ms: if result.total_tests > 0 { result.duration_ms as f64 / result.total_tests as f64 } else { 0.0 },
            slowest_test_duration_ms: result.metadata.get("slowest_test_ms").unwrap_or(&"0".to_string()).parse().unwrap_or(0),
            fastest_test_duration_ms: result.metadata.get("fastest_test_ms").unwrap_or(&"0".to_string()).parse().unwrap_or(0),
        };

        let resource_metrics = ResourceMetrics {
            max_memory_mb: result.metadata.get("max_memory_mb").unwrap_or(&"0".to_string()).parse().unwrap_or(0.0),
            avg_memory_mb: result.metadata.get("avg_memory_mb").unwrap_or(&"0".to_string()).parse().unwrap_or(0.0),
            max_cpu_percent: result.metadata.get("max_cpu_percent").unwrap_or(&"0".to_string()).parse().unwrap_or(0.0),
            avg_cpu_percent: result.metadata.get("avg_cpu_percent").unwrap_or(&"0".to_string()).parse().unwrap_or(0.0),
            disk_io_mb: result.metadata.get("disk_io_mb").unwrap_or(&"0".to_string()).parse().unwrap_or(0.0),
            network_io_mb: result.metadata.get("network_io_mb").unwrap_or(&"0".to_string()).parse().unwrap_or(0.0),
            gc_collections: result.metadata.get("gc_collections").unwrap_or(&"0".to_string()).parse().unwrap_or(0),
            gc_duration_ms: result.metadata.get("gc_duration_ms").unwrap_or(&"0".to_string()).parse().unwrap_or(0),
        };

        let throughput_metrics = ThroughputMetrics {
            tests_per_second: if result.duration_ms > 0 { (result.total_tests as f64 * 1000.0) / result.duration_ms as f64 } else { 0.0 },
            assertions_per_second: result.metadata.get("assertions_per_second").unwrap_or(&"0".to_string()).parse().unwrap_or(0.0),
            operations_per_second: result.metadata.get("operations_per_second").unwrap_or(&"0".to_string()).parse().unwrap_or(0.0),
            concurrent_capacity: result.metadata.get("concurrent_capacity").unwrap_or(&"1".to_string()).parse().unwrap_or(1),
            scalability_factor: result.metadata.get("scalability_factor").unwrap_or(&"1.0".to_string()).parse().unwrap_or(1.0),
        };

        let latency_metrics = LatencyMetrics {
            p50_response_ms: result.metadata.get("p50_response_ms").unwrap_or(&"0".to_string()).parse().unwrap_or(0.0),
            p90_response_ms: result.metadata.get("p90_response_ms").unwrap_or(&"0".to_string()).parse().unwrap_or(0.0),
            p95_response_ms: result.metadata.get("p95_response_ms").unwrap_or(&"0".to_string()).parse().unwrap_or(0.0),
            p99_response_ms: result.metadata.get("p99_response_ms").unwrap_or(&"0".to_string()).parse().unwrap_or(0.0),
            max_response_ms: result.metadata.get("max_response_ms").unwrap_or(&"0".to_string()).parse().unwrap_or(0.0),
            avg_response_ms: result.metadata.get("avg_response_ms").unwrap_or(&"0".to_string()).parse().unwrap_or(0.0),
            response_time_variability: result.metadata.get("response_time_variability").unwrap_or(&"0".to_string()).parse().unwrap_or(0.0),
        };

        let quality_metrics = QualityMetrics {
            stability_score: self.calculate_stability_score(&result),
            reliability_index: self.calculate_reliability_index(&result),
            performance_consistency: self.calculate_performance_consistency(&execution_metrics),
            resource_efficiency: self.calculate_resource_efficiency(&resource_metrics, &execution_metrics),
            scalability_rating: self.calculate_scalability_rating(&throughput_metrics),
        };

        Some(PerformanceSnapshot {
            timestamp: result.timestamp,
            execution_id: result.execution_id,
            environment: result.environment.get("environment").unwrap_or(&"unknown".to_string()).clone(),
            test_suite: result.environment.get("test_suite").unwrap_or(&"default".to_string()).clone(),
            execution_metrics,
            resource_metrics,
            throughput_metrics,
            latency_metrics,
            quality_metrics,
        })
    }

    fn calculate_stability_score(&self, result: &TestExecutionResult) -> f64 {
        let success_rate = if result.total_tests > 0 {
            (result.successful_tests as f64 / result.total_tests as f64) * 100.0
        } else {
            100.0
        };

        // Factor in flakiness
        let flaky_penalty = result.flaky_tests as f64 * 2.0;
        (success_rate - flaky_penalty).max(0.0)
    }

    fn calculate_reliability_index(&self, result: &TestExecutionResult) -> f64 {
        let base_reliability = self.calculate_stability_score(result);

        // Factor in consistency across runs
        let consistency_bonus = if result.metadata.contains_key("consistency_score") {
            result.metadata.get("consistency_score").unwrap().parse::<f64>().unwrap_or(0.0) * 0.1
        } else {
            0.0
        };

        (base_reliability + consistency_bonus).min(100.0)
    }

    fn calculate_performance_consistency(&self, execution: &ExecutionMetrics) -> f64 {
        if execution.test_count == 0 || execution.slowest_test_duration_ms == 0 {
            return 100.0;
        }

        let variance_ratio = execution.fastest_test_duration_ms as f64 / execution.slowest_test_duration_ms as f64;
        (variance_ratio * 100.0).min(100.0)
    }

    fn calculate_resource_efficiency(&self, resource: &ResourceMetrics, execution: &ExecutionMetrics) -> f64 {
        let memory_efficiency = if resource.max_memory_mb > 0.0 {
            (100.0 - (resource.avg_memory_mb / resource.max_memory_mb * 100.0)).max(0.0)
        } else {
            100.0
        };

        let cpu_efficiency = (100.0 - resource.avg_cpu_percent).max(0.0);
        let time_efficiency = if execution.total_duration_ms > 0 {
            ((execution.test_count as f64 * 1000.0) / execution.total_duration_ms as f64).min(100.0)
        } else {
            0.0
        };

        (memory_efficiency * 0.4 + cpu_efficiency * 0.4 + time_efficiency * 0.2)
    }

    fn calculate_scalability_rating(&self, throughput: &ThroughputMetrics) -> f64 {
        let base_throughput = throughput.tests_per_second;
        let scalability_factor = throughput.scalability_factor;
        let concurrent_factor = (throughput.concurrent_capacity as f64).ln().max(1.0);

        ((base_throughput * scalability_factor * concurrent_factor) / 10.0).min(100.0)
    }

    fn update_baselines(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.historical_data.len() < 5 {
            return Ok(()); // Need minimum data for reliable baselines
        }

        let cutoff = Utc::now() - Duration::days(self.config.baseline_update_frequency_days as i64);
        let baseline_data: Vec<&PerformanceSnapshot> = self.historical_data
            .iter()
            .filter(|s| s.timestamp >= cutoff)
            .collect();

        if baseline_data.is_empty() {
            return Ok(());
        }

        // Update execution time baseline
        self.update_baseline_metric("total_duration_ms",
            baseline_data.iter().map(|s| s.execution_metrics.total_duration_ms as f64).collect())?;

        // Update memory baseline
        self.update_baseline_metric("max_memory_mb",
            baseline_data.iter().map(|s| s.resource_metrics.max_memory_mb).collect())?;

        // Update throughput baseline
        self.update_baseline_metric("tests_per_second",
            baseline_data.iter().map(|s| s.throughput_metrics.tests_per_second).collect())?;

        // Update latency baseline
        self.update_baseline_metric("p95_response_ms",
            baseline_data.iter().map(|s| s.latency_metrics.p95_response_ms).collect())?;

        Ok(())
    }

    fn update_baseline_metric(&mut self, metric_name: &str, values: Vec<f64>) -> Result<(), Box<dyn std::error::Error>> {
        if values.is_empty() {
            return Ok(());
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        let confidence_interval = (
            mean - (1.96 * std_dev / (values.len() as f64).sqrt()),
            mean + (1.96 * std_dev / (values.len() as f64).sqrt())
        );

        self.baseline_metrics.insert(metric_name.to_string(), BaselineMetric {
            metric_name: metric_name.to_string(),
            baseline_value: mean,
            confidence_interval,
            last_updated: Utc::now(),
            sample_size: values.len(),
            standard_deviation: std_dev,
        });

        Ok(())
    }

    pub fn analyze_performance(&mut self) -> Result<PerformanceAnalysisReport, Box<dyn std::error::Error>> {
        if self.historical_data.is_empty() {
            return Err("No historical performance data available".into());
        }

        let summary = self.generate_performance_summary()?;
        let trend_analysis = self.analyze_trends()?;
        let regression_report = self.detect_regressions()?;
        let improvement_highlights = self.identify_improvements()?;
        let bottleneck_analysis = self.analyze_bottlenecks()?;
        let recommendations = self.generate_recommendations(&bottleneck_analysis)?;
        let benchmarks = self.compare_benchmarks()?;
        let forecasts = self.generate_forecasts()?;

        Ok(PerformanceAnalysisReport {
            summary,
            trend_analysis,
            regression_report,
            improvement_highlights,
            bottleneck_analysis,
            recommendations,
            benchmarks,
            forecasts,
        })
    }

    fn generate_performance_summary(&self) -> Result<PerformanceSummary, Box<dyn std::error::Error>> {
        let latest_snapshot = self.historical_data.last().ok_or("No performance data available")?;

        let overall_score = self.calculate_overall_score(latest_snapshot);
        let grade = self.determine_grade(overall_score);

        let vs_baseline_percent = if let Some(baseline) = self.baseline_metrics.get("total_duration_ms") {
            ((baseline.baseline_value - latest_snapshot.execution_metrics.total_duration_ms as f64) / baseline.baseline_value) * 100.0
        } else {
            0.0
        };

        let vs_previous_percent = if self.historical_data.len() > 1 {
            let previous = &self.historical_data[self.historical_data.len() - 2];
            ((previous.execution_metrics.total_duration_ms as f64 - latest_snapshot.execution_metrics.total_duration_ms as f64) / previous.execution_metrics.total_duration_ms as f64) * 100.0
        } else {
            0.0
        };

        let stability_rating = self.determine_stability_rating(latest_snapshot);
        let key_insights = self.generate_key_insights(latest_snapshot, vs_baseline_percent, vs_previous_percent);

        Ok(PerformanceSummary {
            overall_score,
            grade,
            current_metrics: latest_snapshot.clone(),
            vs_baseline_percent,
            vs_previous_percent,
            stability_rating,
            key_insights,
        })
    }

    fn calculate_overall_score(&self, snapshot: &PerformanceSnapshot) -> f64 {
        let mut weighted_score = 0.0;
        let mut total_weight = 0.0;

        for category in &self.config.performance_categories {
            let category_score = self.calculate_category_score(category, snapshot);
            weighted_score += category_score * category.weight;
            total_weight += category.weight;
        }

        if total_weight > 0.0 {
            weighted_score / total_weight
        } else {
            0.0
        }
    }

    fn calculate_category_score(&self, category: &PerformanceCategory, snapshot: &PerformanceSnapshot) -> f64 {
        let mut scores = Vec::new();

        for metric in &category.metrics {
            let value = match metric.as_str() {
                "total_duration_ms" => snapshot.execution_metrics.total_duration_ms as f64,
                "avg_test_duration_ms" => snapshot.execution_metrics.avg_test_duration_ms,
                "max_memory_mb" => snapshot.resource_metrics.max_memory_mb,
                "avg_cpu_percent" => snapshot.resource_metrics.avg_cpu_percent,
                "tests_per_second" => snapshot.throughput_metrics.tests_per_second,
                "operations_per_second" => snapshot.throughput_metrics.operations_per_second,
                _ => continue,
            };

            let score = if metric.contains("duration") || metric.contains("memory") || metric.contains("cpu") {
                // Lower is better
                if value <= category.thresholds.excellent {
                    100.0
                } else if value <= category.thresholds.good {
                    80.0
                } else if value <= category.thresholds.acceptable {
                    60.0
                } else if value <= category.thresholds.poor {
                    40.0
                } else {
                    20.0
                }
            } else {
                // Higher is better
                if value >= category.thresholds.excellent {
                    100.0
                } else if value >= category.thresholds.good {
                    80.0
                } else if value >= category.thresholds.acceptable {
                    60.0
                } else if value >= category.thresholds.poor {
                    40.0
                } else {
                    20.0
                }
            };

            scores.push(score);
        }

        if scores.is_empty() {
            50.0 // Default score if no metrics match
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        }
    }

    fn determine_grade(&self, score: f64) -> PerformanceGrade {
        match score {
            x if x >= 90.0 => PerformanceGrade::Excellent,
            x if x >= 80.0 => PerformanceGrade::Good,
            x if x >= 70.0 => PerformanceGrade::Acceptable,
            x if x >= 60.0 => PerformanceGrade::Poor,
            _ => PerformanceGrade::Critical,
        }
    }

    fn determine_stability_rating(&self, snapshot: &PerformanceSnapshot) -> StabilityRating {
        let consistency = snapshot.quality_metrics.performance_consistency;
        match consistency {
            x if x >= 95.0 => StabilityRating::VeryStable,
            x if x >= 90.0 => StabilityRating::Stable,
            x if x >= 80.0 => StabilityRating::Moderate,
            x if x >= 70.0 => StabilityRating::Unstable,
            _ => StabilityRating::VeryUnstable,
        }
    }

    fn generate_key_insights(&self, snapshot: &PerformanceSnapshot, vs_baseline: f64, vs_previous: f64) -> Vec<String> {
        let mut insights = Vec::new();

        if vs_baseline > 10.0 {
            insights.push(format!("Performance improved by {:.1}% vs baseline", vs_baseline));
        } else if vs_baseline < -10.0 {
            insights.push(format!("Performance regressed by {:.1}% vs baseline", vs_baseline.abs()));
        }

        if vs_previous > 5.0 {
            insights.push("Showing improvement from last run".to_string());
        } else if vs_previous < -5.0 {
            insights.push("Performance declined from last run".to_string());
        }

        if snapshot.resource_metrics.max_memory_mb > 500.0 {
            insights.push("High memory usage detected".to_string());
        }

        if snapshot.throughput_metrics.tests_per_second < 1.0 {
            insights.push("Low test throughput - consider parallelization".to_string());
        }

        if snapshot.quality_metrics.stability_score < 95.0 {
            insights.push("Test stability could be improved".to_string());
        }

        insights
    }

    fn analyze_trends(&self) -> Result<TrendAnalysis, Box<dyn std::error::Error>> {
        let recent_snapshots = self.get_recent_snapshots(14);
        if recent_snapshots.len() < 3 {
            return Ok(TrendAnalysis {
                execution_trend: TrendDirection::InsufficientData,
                memory_trend: TrendDirection::InsufficientData,
                throughput_trend: TrendDirection::InsufficientData,
                latency_trend: TrendDirection::InsufficientData,
                overall_trend: TrendDirection::InsufficientData,
                trend_confidence: 0.0,
                inflection_points: Vec::new(),
            });
        }

        let execution_trend = self.detect_trend(&recent_snapshots, |s| s.execution_metrics.total_duration_ms as f64);
        let memory_trend = self.detect_trend(&recent_snapshots, |s| s.resource_metrics.max_memory_mb);
        let throughput_trend = self.detect_trend(&recent_snapshots, |s| s.throughput_metrics.tests_per_second);
        let latency_trend = self.detect_trend(&recent_snapshots, |s| s.latency_metrics.p95_response_ms);

        let overall_trend = self.determine_overall_trend(vec![execution_trend.clone(), memory_trend.clone(), throughput_trend.clone(), latency_trend.clone()]);
        let trend_confidence = self.calculate_trend_confidence(&recent_snapshots);
        let inflection_points = self.identify_inflection_points(&recent_snapshots)?;

        Ok(TrendAnalysis {
            execution_trend,
            memory_trend,
            throughput_trend,
            latency_trend,
            overall_trend,
            trend_confidence,
            inflection_points,
        })
    }

    fn get_recent_snapshots(&self, days: u32) -> Vec<&PerformanceSnapshot> {
        let cutoff = Utc::now() - Duration::days(days as i64);
        self.historical_data
            .iter()
            .filter(|s| s.timestamp >= cutoff)
            .collect()
    }

    fn detect_trend<F>(&self, snapshots: &[&PerformanceSnapshot], extractor: F) -> TrendDirection
    where
        F: Fn(&PerformanceSnapshot) -> f64,
    {
        if snapshots.len() < 3 {
            return TrendDirection::InsufficientData;
        }

        let values: Vec<f64> = snapshots.iter().map(|s| extractor(s)).collect();
        let slope = self.calculate_trend_slope(&values);
        let volatility = self.calculate_trend_volatility(&values);

        match (slope, volatility) {
            (s, v) if v > 20.0 => TrendDirection::Volatile,
            (s, _) if s > 10.0 => TrendDirection::StronglyImproving,
            (s, _) if s > 2.0 => TrendDirection::Improving,
            (s, _) if s > -2.0 => TrendDirection::Stable,
            (s, _) if s > -10.0 => TrendDirection::Declining,
            _ => TrendDirection::StronglyDeclining,
        }
    }

    fn calculate_trend_slope(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let n = values.len() as f64;
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
            (numerator / denominator) * 100.0 // Convert to percentage
        }
    }

    fn calculate_trend_volatility(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        if mean != 0.0 {
            (std_dev / mean) * 100.0 // Coefficient of variation as percentage
        } else {
            0.0
        }
    }

    fn determine_overall_trend(&self, trends: Vec<TrendDirection>) -> TrendDirection {
        let mut improving = 0;
        let mut declining = 0;
        let mut stable = 0;
        let mut volatile = 0;
        let mut insufficient = 0;

        for trend in trends {
            match trend {
                TrendDirection::StronglyImproving | TrendDirection::Improving => improving += 1,
                TrendDirection::StronglyDeclining | TrendDirection::Declining => declining += 1,
                TrendDirection::Stable => stable += 1,
                TrendDirection::Volatile => volatile += 1,
                TrendDirection::InsufficientData => insufficient += 1,
            }
        }

        if insufficient > 2 {
            TrendDirection::InsufficientData
        } else if volatile > 1 {
            TrendDirection::Volatile
        } else if improving > declining {
            TrendDirection::Improving
        } else if declining > improving {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        }
    }

    fn calculate_trend_confidence(&self, snapshots: &[&PerformanceSnapshot]) -> f64 {
        let data_points_factor = (snapshots.len() as f64 / 14.0).min(1.0); // 14 days ideal
        let recency_factor = if snapshots.last().unwrap().timestamp > Utc::now() - Duration::hours(24) {
            1.0
        } else {
            0.8
        };

        (data_points_factor * recency_factor * 100.0).min(95.0)
    }

    fn identify_inflection_points(&self, snapshots: &[&PerformanceSnapshot]) -> Result<Vec<InflectionPoint>, Box<dyn std::error::Error>> {
        let mut inflection_points = Vec::new();

        if snapshots.len() < 3 {
            return Ok(inflection_points);
        }

        // Look for significant changes in key metrics
        for i in 1..snapshots.len()-1 {
            let prev = snapshots[i-1];
            let curr = snapshots[i];
            let next = snapshots[i+1];

            // Check execution time inflection
            let exec_change = self.detect_inflection_point(
                prev.execution_metrics.total_duration_ms as f64,
                curr.execution_metrics.total_duration_ms as f64,
                next.execution_metrics.total_duration_ms as f64,
            );

            if let Some((change_type, magnitude)) = exec_change {
                inflection_points.push(InflectionPoint {
                    timestamp: curr.timestamp,
                    metric: "execution_time".to_string(),
                    change_type,
                    magnitude,
                    likely_cause: "Performance optimization or regression".to_string(),
                });
            }

            // Check memory inflection
            let memory_change = self.detect_inflection_point(
                prev.resource_metrics.max_memory_mb,
                curr.resource_metrics.max_memory_mb,
                next.resource_metrics.max_memory_mb,
            );

            if let Some((change_type, magnitude)) = memory_change {
                inflection_points.push(InflectionPoint {
                    timestamp: curr.timestamp,
                    metric: "memory_usage".to_string(),
                    change_type,
                    magnitude,
                    likely_cause: "Memory leak or optimization".to_string(),
                });
            }
        }

        Ok(inflection_points)
    }

    fn detect_inflection_point(&self, prev: f64, curr: f64, next: f64) -> Option<(ChangeType, f64)> {
        let change1 = ((curr - prev) / prev) * 100.0;
        let change2 = ((next - curr) / curr) * 100.0;

        // Look for significant changes (>10%)
        if change1.abs() > 10.0 || change2.abs() > 10.0 {
            let avg_change = (change1.abs() + change2.abs()) / 2.0;

            if change1 > 10.0 && change2 < -5.0 {
                Some((ChangeType::Spike, avg_change))
            } else if change1 < -10.0 && change2 > 5.0 {
                Some((ChangeType::Drop, avg_change))
            } else if change1 > 10.0 {
                Some((ChangeType::Improvement, change1))
            } else if change1 < -10.0 {
                Some((ChangeType::Degradation, change1.abs()))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn detect_regressions(&mut self) -> Result<Vec<RegressionAnalysis>, Box<dyn std::error::Error>> {
        let mut regressions = Vec::new();

        if let Some(latest) = self.historical_data.last() {
            // Check against baselines
            for (metric_name, baseline) in &self.baseline_metrics {
                let current_value = match metric_name.as_str() {
                    "total_duration_ms" => latest.execution_metrics.total_duration_ms as f64,
                    "max_memory_mb" => latest.resource_metrics.max_memory_mb,
                    "tests_per_second" => latest.throughput_metrics.tests_per_second,
                    "p95_response_ms" => latest.latency_metrics.p95_response_ms,
                    _ => continue,
                };

                let percentage_change = ((current_value - baseline.baseline_value) / baseline.baseline_value) * 100.0;

                // Determine if this is a regression based on metric type
                let is_regression = match metric_name.as_str() {
                    "total_duration_ms" | "max_memory_mb" | "p95_response_ms" => {
                        // Higher is worse
                        percentage_change > self.config.regression_threshold_percent.abs()
                    },
                    "tests_per_second" => {
                        // Lower is worse
                        percentage_change < self.config.regression_threshold_percent
                    },
                    _ => false,
                };

                if is_regression {
                    let severity = self.determine_regression_severity(percentage_change.abs());
                    let statistical_significance = self.calculate_statistical_significance(current_value, baseline);

                    regressions.push(RegressionAnalysis {
                        metric_name: metric_name.clone(),
                        regression_detected: true,
                        severity,
                        current_value,
                        baseline_value: baseline.baseline_value,
                        percentage_change,
                        statistical_significance,
                        detection_timestamp: Utc::now(),
                        affected_components: self.identify_affected_components(metric_name, latest),
                        likely_causes: self.identify_likely_causes(metric_name, percentage_change),
                    });
                }
            }
        }

        Ok(regressions)
    }

    fn determine_regression_severity(&self, percentage_change: f64) -> RegressionSeverity {
        match percentage_change {
            x if x >= 20.0 => RegressionSeverity::Critical,
            x if x >= 10.0 => RegressionSeverity::Major,
            x if x >= 5.0 => RegressionSeverity::Minor,
            x if x >= 2.0 => RegressionSeverity::Warning,
            _ => RegressionSeverity::None,
        }
    }

    fn calculate_statistical_significance(&self, current: f64, baseline: &BaselineMetric) -> f64 {
        let z_score = (current - baseline.baseline_value) / baseline.standard_deviation;
        // Convert to confidence level
        let p_value = 2.0 * (1.0 - self.standard_normal_cdf(z_score.abs()));
        (1.0 - p_value) * 100.0
    }

    fn standard_normal_cdf(&self, x: f64) -> f64 {
        // Approximation of standard normal CDF
        0.5 * (1.0 + self.erf(x / 2.0_f64.sqrt()))
    }

    fn erf(&self, x: f64) -> f64 {
        // Approximation of error function
        let a = 0.3275911;
        let t = 1.0 / (1.0 + a * x.abs());
        let erf_approx = 1.0 - ((((1.061405429 * t - 1.453152027) * t + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t * (-x * x).exp();

        if x >= 0.0 { erf_approx } else { -erf_approx }
    }

    fn identify_affected_components(&self, metric_name: &str, snapshot: &PerformanceSnapshot) -> Vec<String> {
        match metric_name {
            "total_duration_ms" => vec!["test_execution".to_string(), "parallel_processing".to_string()],
            "max_memory_mb" => vec!["memory_management".to_string(), "garbage_collection".to_string()],
            "tests_per_second" => vec!["throughput".to_string(), "concurrency".to_string()],
            "p95_response_ms" => vec!["response_time".to_string(), "network_latency".to_string()],
            _ => vec!["unknown".to_string()],
        }
    }

    fn identify_likely_causes(&self, metric_name: &str, percentage_change: f64) -> Vec<String> {
        let mut causes = Vec::new();

        match metric_name {
            "total_duration_ms" if percentage_change > 0.0 => {
                causes.push("Increased test complexity".to_string());
                causes.push("Resource contention".to_string());
                causes.push("Infrastructure changes".to_string());
            },
            "max_memory_mb" if percentage_change > 0.0 => {
                causes.push("Memory leaks".to_string());
                causes.push("Inefficient data structures".to_string());
                causes.push("Increased test data size".to_string());
            },
            "tests_per_second" if percentage_change < 0.0 => {
                causes.push("Reduced parallelization".to_string());
                causes.push("Slower test setup".to_string());
                causes.push("Resource bottlenecks".to_string());
            },
            _ => {
                causes.push("Unknown cause".to_string());
            }
        }

        causes
    }

    fn identify_improvements(&self) -> Result<Vec<PerformanceImprovement>, Box<dyn std::error::Error>> {
        let mut improvements = Vec::new();

        if self.historical_data.len() < 2 {
            return Ok(improvements);
        }

        let latest = self.historical_data.last().unwrap();
        let previous = &self.historical_data[self.historical_data.len() - 2];

        // Check execution time improvement
        let exec_improvement = ((previous.execution_metrics.total_duration_ms as f64 - latest.execution_metrics.total_duration_ms as f64) / previous.execution_metrics.total_duration_ms as f64) * 100.0;
        if exec_improvement >= self.config.improvement_threshold_percent {
            improvements.push(PerformanceImprovement {
                detected_at: latest.timestamp,
                metric: "execution_time".to_string(),
                improvement_percent: exec_improvement,
                significance: self.categorize_improvement_significance(exec_improvement),
                description: format!("Test execution time improved by {:.1}%", exec_improvement),
                contributing_factors: vec![
                    "Code optimization".to_string(),
                    "Better test parallelization".to_string(),
                    "Infrastructure improvements".to_string(),
                ],
                sustainable: exec_improvement < 50.0, // Extreme improvements might not be sustainable
            });
        }

        // Check memory improvement
        let memory_improvement = ((previous.resource_metrics.max_memory_mb - latest.resource_metrics.max_memory_mb) / previous.resource_metrics.max_memory_mb) * 100.0;
        if memory_improvement >= self.config.improvement_threshold_percent {
            improvements.push(PerformanceImprovement {
                detected_at: latest.timestamp,
                metric: "memory_usage".to_string(),
                improvement_percent: memory_improvement,
                significance: self.categorize_improvement_significance(memory_improvement),
                description: format!("Memory usage reduced by {:.1}%", memory_improvement),
                contributing_factors: vec![
                    "Memory leak fixes".to_string(),
                    "Better garbage collection".to_string(),
                    "Data structure optimization".to_string(),
                ],
                sustainable: true,
            });
        }

        Ok(improvements)
    }

    fn categorize_improvement_significance(&self, improvement_percent: f64) -> ImprovementSignificance {
        match improvement_percent {
            x if x >= 15.0 => ImprovementSignificance::Major,
            x if x >= 5.0 => ImprovementSignificance::Moderate,
            x if x >= 2.0 => ImprovementSignificance::Minor,
            _ => ImprovementSignificance::Marginal,
        }
    }

    fn analyze_bottlenecks(&self) -> Result<BottleneckAnalysis, Box<dyn std::error::Error>> {
        let primary_bottlenecks = self.identify_primary_bottlenecks()?;
        let resource_constraints = self.identify_resource_constraints()?;
        let scaling_limitations = self.identify_scaling_limitations()?;
        let optimization_opportunities = self.identify_optimization_opportunities()?;

        Ok(BottleneckAnalysis {
            primary_bottlenecks,
            resource_constraints,
            scaling_limitations,
            optimization_opportunities,
        })
    }

    fn identify_primary_bottlenecks(&self) -> Result<Vec<Bottleneck>, Box<dyn std::error::Error>> {
        let mut bottlenecks = Vec::new();

        if let Some(latest) = self.historical_data.last() {
            // Identify execution time bottlenecks
            if latest.execution_metrics.total_duration_ms > 5000 {
                bottlenecks.push(Bottleneck {
                    component: "test_execution".to_string(),
                    metric: "total_duration_ms".to_string(),
                    impact_score: (latest.execution_metrics.total_duration_ms as f64 / 1000.0).min(10.0),
                    description: "Long test execution times affecting development velocity".to_string(),
                    resolution_priority: Priority::High,
                    estimated_improvement: 30.0,
                });
            }

            // Identify memory bottlenecks
            if latest.resource_metrics.max_memory_mb > 1000.0 {
                bottlenecks.push(Bottleneck {
                    component: "memory_management".to_string(),
                    metric: "max_memory_mb".to_string(),
                    impact_score: (latest.resource_metrics.max_memory_mb / 200.0).min(10.0),
                    description: "High memory usage potentially limiting test parallelization".to_string(),
                    resolution_priority: Priority::Medium,
                    estimated_improvement: 25.0,
                });
            }

            // Identify throughput bottlenecks
            if latest.throughput_metrics.tests_per_second < 5.0 {
                bottlenecks.push(Bottleneck {
                    component: "test_parallelization".to_string(),
                    metric: "tests_per_second".to_string(),
                    impact_score: (10.0 - latest.throughput_metrics.tests_per_second).max(0.0),
                    description: "Low test throughput indicating serialization issues".to_string(),
                    resolution_priority: Priority::High,
                    estimated_improvement: 50.0,
                });
            }
        }

        Ok(bottlenecks)
    }

    fn identify_resource_constraints(&self) -> Result<Vec<ResourceConstraint>, Box<dyn std::error::Error>> {
        let mut constraints = Vec::new();

        if let Some(latest) = self.historical_data.last() {
            // CPU constraint analysis
            if latest.resource_metrics.avg_cpu_percent > 80.0 {
                constraints.push(ResourceConstraint {
                    resource_type: ResourceType::CPU,
                    utilization_percent: latest.resource_metrics.avg_cpu_percent,
                    constraint_level: if latest.resource_metrics.avg_cpu_percent > 90.0 {
                        ConstraintLevel::Severe
                    } else {
                        ConstraintLevel::High
                    },
                    impact_on_performance: "High CPU usage limiting test parallelization".to_string(),
                    mitigation_strategies: vec![
                        "Optimize CPU-intensive test operations".to_string(),
                        "Reduce test parallelism level".to_string(),
                        "Upgrade test infrastructure".to_string(),
                    ],
                });
            }

            // Memory constraint analysis
            if latest.resource_metrics.max_memory_mb > 2000.0 {
                constraints.push(ResourceConstraint {
                    resource_type: ResourceType::Memory,
                    utilization_percent: (latest.resource_metrics.max_memory_mb / 4000.0 * 100.0).min(100.0),
                    constraint_level: if latest.resource_metrics.max_memory_mb > 3000.0 {
                        ConstraintLevel::Severe
                    } else {
                        ConstraintLevel::High
                    },
                    impact_on_performance: "High memory usage potentially causing GC pressure".to_string(),
                    mitigation_strategies: vec![
                        "Optimize memory allocation in tests".to_string(),
                        "Implement test data cleanup".to_string(),
                        "Increase available memory".to_string(),
                    ],
                });
            }
        }

        Ok(constraints)
    }

    fn identify_scaling_limitations(&self) -> Result<Vec<ScalingLimitation>, Box<dyn std::error::Error>> {
        let mut limitations = Vec::new();

        if let Some(latest) = self.historical_data.last() {
            // Concurrency limitations
            if latest.throughput_metrics.concurrent_capacity < 8 {
                limitations.push(ScalingLimitation {
                    component: "test_runner".to_string(),
                    limitation_type: LimitationType::ConcurrencyLimit,
                    threshold: 8.0,
                    current_utilization: latest.throughput_metrics.concurrent_capacity as f64,
                    recommended_actions: vec![
                        "Increase test runner parallelism".to_string(),
                        "Optimize test isolation".to_string(),
                        "Review resource sharing patterns".to_string(),
                    ],
                });
            }

            // Memory scaling limitations
            if latest.resource_metrics.max_memory_mb / latest.execution_metrics.test_count as f64 > 50.0 {
                limitations.push(ScalingLimitation {
                    component: "memory_per_test".to_string(),
                    limitation_type: LimitationType::MemoryLimit,
                    threshold: 50.0,
                    current_utilization: latest.resource_metrics.max_memory_mb / latest.execution_metrics.test_count as f64,
                    recommended_actions: vec![
                        "Optimize test data management".to_string(),
                        "Implement memory pooling".to_string(),
                        "Review test isolation strategies".to_string(),
                    ],
                });
            }
        }

        Ok(limitations)
    }

    fn identify_optimization_opportunities(&self) -> Result<Vec<OptimizationOpportunity>, Box<dyn std::error::Error>> {
        let mut opportunities = Vec::new();

        if let Some(latest) = self.historical_data.last() {
            // Parallelization opportunities
            if latest.execution_metrics.parallel_efficiency < 0.8 {
                opportunities.push(OptimizationOpportunity {
                    area: "test_parallelization".to_string(),
                    opportunity_type: OpportunityType::ParallelizationImprovement,
                    potential_impact: (0.8 - latest.execution_metrics.parallel_efficiency) * 100.0,
                    implementation_effort: EffortLevel::Medium,
                    description: "Improve test parallelization efficiency".to_string(),
                    implementation_steps: vec![
                        "Analyze test dependencies".to_string(),
                        "Implement better test isolation".to_string(),
                        "Optimize shared resource access".to_string(),
                    ],
                });
            }

            // Caching opportunities
            if latest.latency_metrics.p95_response_ms > 100.0 {
                opportunities.push(OptimizationOpportunity {
                    area: "response_caching".to_string(),
                    opportunity_type: OpportunityType::CachingStrategy,
                    potential_impact: 40.0,
                    implementation_effort: EffortLevel::Low,
                    description: "Implement response caching to reduce latency".to_string(),
                    implementation_steps: vec![
                        "Identify cacheable responses".to_string(),
                        "Implement cache layer".to_string(),
                        "Configure cache invalidation".to_string(),
                    ],
                });
            }

            // Resource optimization opportunities
            if latest.resource_metrics.avg_cpu_percent < 30.0 && latest.execution_metrics.total_duration_ms > 2000 {
                opportunities.push(OptimizationOpportunity {
                    area: "cpu_utilization".to_string(),
                    opportunity_type: OpportunityType::ResourceOptimization,
                    potential_impact: 25.0,
                    implementation_effort: EffortLevel::Medium,
                    description: "Increase CPU utilization through better parallelization".to_string(),
                    implementation_steps: vec![
                        "Increase parallel test execution".to_string(),
                        "Optimize test scheduling".to_string(),
                        "Review resource allocation".to_string(),
                    ],
                });
            }
        }

        Ok(opportunities)
    }

    fn generate_recommendations(&self, bottleneck_analysis: &BottleneckAnalysis) -> Result<Vec<PerformanceRecommendation>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();

        // Generate recommendations based on bottlenecks
        for bottleneck in &bottleneck_analysis.primary_bottlenecks {
            let recommendation = match bottleneck.component.as_str() {
                "test_execution" => PerformanceRecommendation {
                    priority: Priority::High,
                    category: RecommendationCategory::TestOptimization,
                    title: "Optimize Test Execution Performance".to_string(),
                    description: "Reduce test execution time through parallelization and optimization".to_string(),
                    expected_impact: bottleneck.estimated_improvement,
                    implementation_effort: EffortLevel::Medium,
                    cost_benefit_ratio: bottleneck.estimated_improvement / 3.0, // Medium effort = 3 units
                    implementation_steps: vec![
                        "Profile slow tests to identify bottlenecks".to_string(),
                        "Implement test parallelization where possible".to_string(),
                        "Optimize test setup and teardown".to_string(),
                        "Consider test data mocking strategies".to_string(),
                    ],
                    success_metrics: vec![
                        "Reduce average test execution time by 30%".to_string(),
                        "Achieve >80% parallel efficiency".to_string(),
                        "Maintain test reliability >95%".to_string(),
                    ],
                    risk_level: RiskLevel::Low,
                },
                "memory_management" => PerformanceRecommendation {
                    priority: Priority::Medium,
                    category: RecommendationCategory::ResourceManagement,
                    title: "Optimize Memory Usage".to_string(),
                    description: "Reduce memory consumption through better resource management".to_string(),
                    expected_impact: bottleneck.estimated_improvement,
                    implementation_effort: EffortLevel::Medium,
                    cost_benefit_ratio: bottleneck.estimated_improvement / 3.0,
                    implementation_steps: vec![
                        "Implement memory profiling".to_string(),
                        "Optimize data structure usage".to_string(),
                        "Implement proper cleanup routines".to_string(),
                        "Consider memory pooling strategies".to_string(),
                    ],
                    success_metrics: vec![
                        "Reduce peak memory usage by 25%".to_string(),
                        "Minimize GC pressure".to_string(),
                        "Improve memory efficiency score".to_string(),
                    ],
                    risk_level: RiskLevel::Medium,
                },
                _ => continue,
            };

            recommendations.push(recommendation);
        }

        // Generate recommendations based on optimization opportunities
        for opportunity in &bottleneck_analysis.optimization_opportunities {
            let recommendation = PerformanceRecommendation {
                priority: if opportunity.potential_impact > 30.0 { Priority::High } else { Priority::Medium },
                category: match opportunity.opportunity_type {
                    OpportunityType::ParallelizationImprovement => RecommendationCategory::TestOptimization,
                    OpportunityType::CachingStrategy => RecommendationCategory::Infrastructure,
                    OpportunityType::ResourceOptimization => RecommendationCategory::ResourceManagement,
                    _ => RecommendationCategory::CodeOptimization,
                },
                title: opportunity.area.clone(),
                description: opportunity.description.clone(),
                expected_impact: opportunity.potential_impact,
                implementation_effort: opportunity.implementation_effort.clone(),
                cost_benefit_ratio: opportunity.potential_impact / match opportunity.implementation_effort {
                    EffortLevel::Low => 1.0,
                    EffortLevel::Medium => 3.0,
                    EffortLevel::High => 7.0,
                    EffortLevel::VeryHigh => 15.0,
                },
                implementation_steps: opportunity.implementation_steps.clone(),
                success_metrics: vec![
                    format!("Improve performance by {:.0}%", opportunity.potential_impact),
                    "Maintain system stability".to_string(),
                    "Achieve target performance metrics".to_string(),
                ],
                risk_level: match opportunity.implementation_effort {
                    EffortLevel::Low | EffortLevel::Medium => RiskLevel::Low,
                    EffortLevel::High => RiskLevel::Medium,
                    EffortLevel::VeryHigh => RiskLevel::High,
                },
            };

            recommendations.push(recommendation);
        }

        Ok(recommendations)
    }

    fn compare_benchmarks(&self) -> Result<BenchmarkComparison, Box<dyn std::error::Error>> {
        // This would typically compare against industry standards
        // For now, provide a placeholder implementation

        let industry_percentile = if let Some(latest) = self.historical_data.last() {
            // Simple heuristic based on execution time
            match latest.execution_metrics.total_duration_ms {
                x if x < 1000 => 90.0,
                x if x < 5000 => 70.0,
                x if x < 10000 => 50.0,
                x if x < 30000 => 30.0,
                _ => 10.0,
            }
        } else {
            50.0
        };

        let vs_similar_projects = match industry_percentile {
            x if x >= 80.0 => ComparisonResult::Above,
            x if x >= 60.0 => ComparisonResult::Similar,
            x if x >= 40.0 => ComparisonResult::Below,
            _ => ComparisonResult::SignificantlyBelow,
        };

        let historical_best = self.find_historical_best()?;
        let competitive_analysis = Vec::new(); // Would be populated with real competitor data

        Ok(BenchmarkComparison {
            industry_percentile,
            vs_similar_projects,
            historical_best,
            competitive_analysis,
        })
    }

    fn find_historical_best(&self) -> Result<HistoricalBest, Box<dyn std::error::Error>> {
        let best_snapshot = self.historical_data
            .iter()
            .min_by(|a, b| a.execution_metrics.total_duration_ms.cmp(&b.execution_metrics.total_duration_ms))
            .ok_or("No historical data available")?;

        let current_best = self.historical_data.last().unwrap();
        let gap_to_current = ((current_best.execution_metrics.total_duration_ms as f64 - best_snapshot.execution_metrics.total_duration_ms as f64) / best_snapshot.execution_metrics.total_duration_ms as f64) * 100.0;

        Ok(HistoricalBest {
            timestamp: best_snapshot.timestamp,
            metrics: best_snapshot.clone(),
            gap_to_current,
            factors_for_best_performance: vec![
                "Optimal test parallelization".to_string(),
                "Minimal resource contention".to_string(),
                "Efficient test data setup".to_string(),
            ],
        })
    }

    fn generate_forecasts(&self) -> Result<Vec<PerformanceForecast>, Box<dyn std::error::Error>> {
        let mut forecasts = Vec::new();

        if self.historical_data.len() < 5 {
            return Ok(forecasts);
        }

        let recent_snapshots = self.get_recent_snapshots(14);

        // Forecast execution time
        let exec_times: Vec<f64> = recent_snapshots.iter()
            .map(|s| s.execution_metrics.total_duration_ms as f64)
            .collect();

        if let Some(exec_forecast) = self.create_forecast("execution_time", &exec_times, 30) {
            forecasts.push(exec_forecast);
        }

        // Forecast memory usage
        let memory_usage: Vec<f64> = recent_snapshots.iter()
            .map(|s| s.resource_metrics.max_memory_mb)
            .collect();

        if let Some(memory_forecast) = self.create_forecast("memory_usage", &memory_usage, 30) {
            forecasts.push(memory_forecast);
        }

        // Forecast throughput
        let throughput: Vec<f64> = recent_snapshots.iter()
            .map(|s| s.throughput_metrics.tests_per_second)
            .collect();

        if let Some(throughput_forecast) = self.create_forecast("throughput", &throughput, 30) {
            forecasts.push(throughput_forecast);
        }

        Ok(forecasts)
    }

    fn create_forecast(&self, metric_name: &str, values: &[f64], days_ahead: u32) -> Option<PerformanceForecast> {
        if values.len() < 3 {
            return None;
        }

        let slope = self.calculate_trend_slope(values);
        let current = values.last()?;
        let predicted = current + (slope * days_ahead as f64 / 100.0 * current);

        let volatility = self.calculate_trend_volatility(values);
        let confidence_interval = (
            predicted - (volatility * predicted / 100.0),
            predicted + (volatility * predicted / 100.0),
        );

        Some(PerformanceForecast {
            metric: metric_name.to_string(),
            forecast_horizon_days: days_ahead,
            predicted_value: predicted,
            confidence_interval,
            scenario: ForecastScenario::CurrentTrend,
            assumptions: vec![
                "Current development patterns continue".to_string(),
                "No major infrastructure changes".to_string(),
                "Test suite complexity remains similar".to_string(),
            ],
            risk_factors: vec![
                "Significant code changes".to_string(),
                "Infrastructure modifications".to_string(),
                "Change in testing strategy".to_string(),
            ],
        })
    }

    pub fn export_analysis_report(&self) -> Result<String, Box<dyn std::error::Error>> {
        let report = self.analyze_performance()?;
        serde_json::to_string_pretty(&report).map_err(|e| e.into())
    }
}

impl Default for PerformanceAnalyzer {
    fn default() -> Self {
        Self::with_default_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_analyzer_creation() {
        let analyzer = PerformanceAnalyzer::with_default_config();
        assert_eq!(analyzer.config.analysis_window_days, 30);
        assert_eq!(analyzer.historical_data.len(), 0);
    }

    #[test]
    fn test_trend_slope_calculation() {
        let analyzer = PerformanceAnalyzer::with_default_config();
        let values = vec![100.0, 105.0, 110.0, 115.0, 120.0];
        let slope = analyzer.calculate_trend_slope(&values);
        assert!(slope > 0.0); // Should detect upward trend
    }

    #[test]
    fn test_regression_severity_determination() {
        let analyzer = PerformanceAnalyzer::with_default_config();

        assert!(matches!(analyzer.determine_regression_severity(25.0), RegressionSeverity::Critical));
        assert!(matches!(analyzer.determine_regression_severity(15.0), RegressionSeverity::Major));
        assert!(matches!(analyzer.determine_regression_severity(7.0), RegressionSeverity::Minor));
        assert!(matches!(analyzer.determine_regression_severity(3.0), RegressionSeverity::Warning));
        assert!(matches!(analyzer.determine_regression_severity(1.0), RegressionSeverity::None));
    }
}