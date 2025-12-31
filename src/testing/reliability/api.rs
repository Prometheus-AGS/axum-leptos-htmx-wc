use axum::{
    extract::{Query, State, Path},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

use crate::testing::TestExecutionResult;
use super::{
    TestReliabilityEngine, ReliabilityConfig, ReliabilityOverview,
    ReliabilityHealthScore, ReliabilityTrendPoint, FlakyTestSummary,
    FailurePatternSummary, ReliabilityImprovement, DateRange,
    ReliabilityAlertConfig, NotificationChannel, Priority,
};

/// API state for reliability endpoints
#[derive(Clone)]
pub struct ReliabilityApiState {
    pub engine: Arc<RwLock<TestReliabilityEngine>>,
    pub cache: Arc<RwLock<ReliabilityCache>>,
    pub config: ReliabilityApiConfig,
}

/// API configuration
#[derive(Debug, Clone)]
pub struct ReliabilityApiConfig {
    pub cache_ttl_minutes: u64,
    pub max_data_points: usize,
    pub enable_real_time_updates: bool,
    pub alert_config: ReliabilityAlertConfig,
}

/// Internal cache structure
#[derive(Debug, Default)]
pub struct ReliabilityCache {
    pub overview_cache: Option<(DateTime<Utc>, ReliabilityOverview)>,
    pub health_score_cache: Option<(DateTime<Utc>, ReliabilityHealthScore)>,
    pub trend_cache: HashMap<String, (DateTime<Utc>, Vec<ReliabilityTrendPoint>)>,
    pub flaky_tests_cache: Option<(DateTime<Utc>, Vec<FlakyTestSummary>)>,
    pub patterns_cache: Option<(DateTime<Utc>, Vec<FailurePatternSummary>)>,
}

/// Query parameters for reliability endpoints
#[derive(Debug, Deserialize)]
pub struct ReliabilityQuery {
    pub time_window: Option<String>,
    pub environment: Option<String>,
    pub test_category: Option<String>,
    pub force_refresh: Option<bool>,
    pub granularity: Option<String>,
    pub include_predictions: Option<bool>,
}

/// Query parameters for trends
#[derive(Debug, Deserialize)]
pub struct TrendQuery {
    pub metric: String,
    pub time_window: Option<String>,
    pub granularity: Option<String>,
    pub environment: Option<String>,
}

/// Custom analysis request
#[derive(Debug, Deserialize)]
pub struct CustomAnalysisRequest {
    pub config: CustomReliabilityConfig,
    pub filters: AnalysisFilters,
    pub description: String,
}

/// Custom reliability configuration
#[derive(Debug, Deserialize)]
pub struct CustomReliabilityConfig {
    pub flakiness_threshold: f64,
    pub stability_window_hours: u64,
    pub pattern_detection_sensitivity: f64,
    pub environmental_analysis: bool,
    pub predictive_analysis: bool,
}

/// Analysis filters
#[derive(Debug, Deserialize)]
pub struct AnalysisFilters {
    pub test_names: Option<Vec<String>>,
    pub test_suites: Option<Vec<String>>,
    pub environments: Option<Vec<String>>,
    pub time_range: Option<DateRange>,
    pub status_filter: Option<Vec<String>>,
    pub exclude_flaky: Option<bool>,
}

/// Reliability alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityAlert {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub affected_tests: Vec<String>,
    pub threshold_value: f64,
    pub actual_value: f64,
    pub first_detected: DateTime<Utc>,
    pub environment: Option<String>,
    pub recommended_actions: Vec<String>,
    pub acknowledgment_required: bool,
}

/// Types of reliability alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighFlakiness,
    LowStability,
    PatternDetected,
    ConsecutiveFailures,
    EnvironmentalIssue,
    PredictedDegradation,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

/// Reliability recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityRecommendation {
    pub recommendation_type: RecommendationType,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub affected_tests: Vec<String>,
    pub estimated_impact: ImpactEstimate,
    pub implementation_steps: Vec<String>,
    pub success_metrics: Vec<String>,
    pub time_estimate_hours: u32,
}

/// Types of recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    FlakeReduction,
    StabilityImprovement,
    EnvironmentalOptimization,
    TestOptimization,
    MonitoringEnhancement,
    ProcessImprovement,
}

/// Impact estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactEstimate {
    pub reliability_improvement: f64,
    pub affected_test_count: usize,
    pub estimated_failure_reduction: f64,
    pub confidence_level: f64,
}

/// API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub metadata: ResponseMetadata,
}

/// Response metadata
#[derive(Debug, Serialize)]
pub struct ResponseMetadata {
    pub generated_at: DateTime<Utc>,
    pub cache_used: bool,
    pub data_freshness: String,
    pub processing_time_ms: u64,
    pub api_version: String,
}

impl ReliabilityApiState {
    /// Create new API state
    pub fn new() -> Self {
        Self {
            engine: Arc::new(RwLock::new(TestReliabilityEngine::new(ReliabilityConfig::default()))),
            cache: Arc::new(RwLock::new(ReliabilityCache::default())),
            config: ReliabilityApiConfig::default(),
        }
    }

    /// Load test data into the engine
    pub async fn load_test_data(&self, test_results: Vec<TestExecutionResult>) {
        let mut engine = self.engine.write().await;
        engine.load_test_results(test_results);

        // Clear cache when new data is loaded
        let mut cache = self.cache.write().await;
        *cache = ReliabilityCache::default();
    }

    /// Check if cache is valid
    fn is_cache_valid(&self, cached_at: DateTime<Utc>) -> bool {
        let expiry = cached_at + Duration::minutes(self.config.cache_ttl_minutes as i64);
        Utc::now() < expiry
    }
}

impl ReliabilityApiConfig {
    /// Create default configuration
    pub fn default() -> Self {
        Self {
            cache_ttl_minutes: 15,
            max_data_points: 10000,
            enable_real_time_updates: true,
            alert_config: ReliabilityAlertConfig::default(),
        }
    }
}

/// Create reliability API router
pub fn create_reliability_api_router() -> Router<ReliabilityApiState> {
    Router::new()
        .route("/overview", get(get_reliability_overview))
        .route("/health", get(get_health_score))
        .route("/trends/:metric", get(get_reliability_trends))
        .route("/flaky-tests", get(get_flaky_tests))
        .route("/patterns", get(get_failure_patterns))
        .route("/recommendations", get(get_recommendations))
        .route("/alerts", get(get_active_alerts))
        .route("/predictions", get(get_reliability_predictions))
        .route("/analysis/custom", post(run_custom_analysis))
        .route("/test/:test_id/reliability", get(get_test_reliability))
        .route("/environment/:env/reliability", get(get_environment_reliability))
        .route("/status", get(get_api_status))
}

/// GET /reliability/overview - Get comprehensive reliability overview
async fn get_reliability_overview(
    State(state): State<ReliabilityApiState>,
    Query(query): Query<ReliabilityQuery>,
) -> Result<Json<ApiResponse<ReliabilityOverview>>, StatusCode> {
    let start_time = std::time::Instant::now();
    let mut cache_used = false;

    // Check cache first
    if !query.force_refresh.unwrap_or(false) {
        let cache = state.cache.read().await;
        if let Some((cached_at, cached_overview)) = &cache.overview_cache {
            if state.is_cache_valid(*cached_at) {
                cache_used = true;
                let metadata = ResponseMetadata {
                    generated_at: Utc::now(),
                    cache_used,
                    data_freshness: "cached".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                };

                return Ok(Json(ApiResponse {
                    success: true,
                    data: cached_overview.clone(),
                    metadata,
                }));
            }
        }
    }

    // Generate fresh analysis
    let engine = state.engine.read().await;

    // Parse time window
    let time_range = parse_time_window(&query.time_window);

    // Run comprehensive reliability analysis
    let overview = engine.generate_reliability_overview(
        time_range,
        query.environment.as_deref(),
        query.test_category.as_deref(),
        query.include_predictions.unwrap_or(true),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update cache
    {
        let mut cache = state.cache.write().await;
        cache.overview_cache = Some((Utc::now(), overview.clone()));
    }

    let metadata = ResponseMetadata {
        generated_at: Utc::now(),
        cache_used,
        data_freshness: "real-time".to_string(),
        processing_time_ms: start_time.elapsed().as_millis() as u64,
        api_version: "v2".to_string(),
    };

    Ok(Json(ApiResponse {
        success: true,
        data: overview,
        metadata,
    }))
}

/// GET /reliability/health - Get current health score
async fn get_health_score(
    State(state): State<ReliabilityApiState>,
    Query(query): Query<ReliabilityQuery>,
) -> Result<Json<ApiResponse<ReliabilityHealthScore>>, StatusCode> {
    let start_time = std::time::Instant::now();
    let mut cache_used = false;

    // Check cache first
    if !query.force_refresh.unwrap_or(false) {
        let cache = state.cache.read().await;
        if let Some((cached_at, cached_score)) = &cache.health_score_cache {
            if state.is_cache_valid(*cached_at) {
                cache_used = true;
                let metadata = ResponseMetadata {
                    generated_at: Utc::now(),
                    cache_used,
                    data_freshness: "cached".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                };

                return Ok(Json(ApiResponse {
                    success: true,
                    data: cached_score.clone(),
                    metadata,
                }));
            }
        }
    }

    let engine = state.engine.read().await;
    let time_range = parse_time_window(&query.time_window);

    let health_score = engine.calculate_reliability_health_score(
        time_range,
        query.environment.as_deref(),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update cache
    {
        let mut cache = state.cache.write().await;
        cache.health_score_cache = Some((Utc::now(), health_score.clone()));
    }

    let metadata = ResponseMetadata {
        generated_at: Utc::now(),
        cache_used,
        data_freshness: "real-time".to_string(),
        processing_time_ms: start_time.elapsed().as_millis() as u64,
        api_version: "v2".to_string(),
    };

    Ok(Json(ApiResponse {
        success: true,
        data: health_score,
        metadata,
    }))
}

/// GET /reliability/trends/:metric - Get trend data for specific metric
async fn get_reliability_trends(
    State(state): State<ReliabilityApiState>,
    Path(metric): Path<String>,
    Query(query): Query<TrendQuery>,
) -> Result<Json<ApiResponse<Vec<ReliabilityTrendPoint>>>, StatusCode> {
    let start_time = std::time::Instant::now();
    let cache_key = format!("{}:{:?}", metric, query);

    // Check cache
    {
        let cache = state.cache.read().await;
        if let Some((cached_at, cached_trends)) = cache.trend_cache.get(&cache_key) {
            if state.is_cache_valid(*cached_at) {
                let metadata = ResponseMetadata {
                    generated_at: Utc::now(),
                    cache_used: true,
                    data_freshness: "cached".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                };

                return Ok(Json(ApiResponse {
                    success: true,
                    data: cached_trends.clone(),
                    metadata,
                }));
            }
        }
    }

    let engine = state.engine.read().await;
    let time_range = parse_time_window(&query.time_window);

    let trends = engine.get_reliability_trends(
        &metric,
        time_range,
        query.granularity.as_deref(),
        query.environment.as_deref(),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update cache
    {
        let mut cache = state.cache.write().await;
        cache.trend_cache.insert(cache_key, (Utc::now(), trends.clone()));
    }

    let metadata = ResponseMetadata {
        generated_at: Utc::now(),
        cache_used: false,
        data_freshness: "real-time".to_string(),
        processing_time_ms: start_time.elapsed().as_millis() as u64,
        api_version: "v2".to_string(),
    };

    Ok(Json(ApiResponse {
        success: true,
        data: trends,
        metadata,
    }))
}

/// GET /reliability/flaky-tests - Get list of flaky tests
async fn get_flaky_tests(
    State(state): State<ReliabilityApiState>,
    Query(query): Query<ReliabilityQuery>,
) -> Result<Json<ApiResponse<Vec<FlakyTestSummary>>>, StatusCode> {
    let start_time = std::time::Instant::now();
    let mut cache_used = false;

    // Check cache first
    if !query.force_refresh.unwrap_or(false) {
        let cache = state.cache.read().await;
        if let Some((cached_at, cached_tests)) = &cache.flaky_tests_cache {
            if state.is_cache_valid(*cached_at) {
                cache_used = true;
                let metadata = ResponseMetadata {
                    generated_at: Utc::now(),
                    cache_used,
                    data_freshness: "cached".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                };

                return Ok(Json(ApiResponse {
                    success: true,
                    data: cached_tests.clone(),
                    metadata,
                }));
            }
        }
    }

    let engine = state.engine.read().await;
    let time_range = parse_time_window(&query.time_window);

    let flaky_tests = engine.identify_flaky_tests(
        time_range,
        query.environment.as_deref(),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update cache
    {
        let mut cache = state.cache.write().await;
        cache.flaky_tests_cache = Some((Utc::now(), flaky_tests.clone()));
    }

    let metadata = ResponseMetadata {
        generated_at: Utc::now(),
        cache_used,
        data_freshness: "real-time".to_string(),
        processing_time_ms: start_time.elapsed().as_millis() as u64,
        api_version: "v2".to_string(),
    };

    Ok(Json(ApiResponse {
        success: true,
        data: flaky_tests,
        metadata,
    }))
}

/// GET /reliability/patterns - Get failure patterns
async fn get_failure_patterns(
    State(state): State<ReliabilityApiState>,
    Query(query): Query<ReliabilityQuery>,
) -> Result<Json<ApiResponse<Vec<FailurePatternSummary>>>, StatusCode> {
    let start_time = std::time::Instant::now();
    let mut cache_used = false;

    // Check cache first
    if !query.force_refresh.unwrap_or(false) {
        let cache = state.cache.read().await;
        if let Some((cached_at, cached_patterns)) = &cache.patterns_cache {
            if state.is_cache_valid(*cached_at) {
                cache_used = true;
                let metadata = ResponseMetadata {
                    generated_at: Utc::now(),
                    cache_used,
                    data_freshness: "cached".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                };

                return Ok(Json(ApiResponse {
                    success: true,
                    data: cached_patterns.clone(),
                    metadata,
                }));
            }
        }
    }

    let engine = state.engine.read().await;
    let time_range = parse_time_window(&query.time_window);

    let patterns = engine.detect_failure_patterns(
        time_range,
        query.environment.as_deref(),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update cache
    {
        let mut cache = state.cache.write().await;
        cache.patterns_cache = Some((Utc::now(), patterns.clone()));
    }

    let metadata = ResponseMetadata {
        generated_at: Utc::now(),
        cache_used,
        data_freshness: "real-time".to_string(),
        processing_time_ms: start_time.elapsed().as_millis() as u64,
        api_version: "v2".to_string(),
    };

    Ok(Json(ApiResponse {
        success: true,
        data: patterns,
        metadata,
    }))
}

/// GET /reliability/recommendations - Get improvement recommendations
async fn get_recommendations(
    State(state): State<ReliabilityApiState>,
    Query(query): Query<ReliabilityQuery>,
) -> Result<Json<ApiResponse<Vec<ReliabilityRecommendation>>>, StatusCode> {
    let start_time = std::time::Instant::now();

    let engine = state.engine.read().await;
    let time_range = parse_time_window(&query.time_window);

    let recommendations = engine.generate_recommendations(
        time_range,
        query.environment.as_deref(),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let metadata = ResponseMetadata {
        generated_at: Utc::now(),
        cache_used: false,
        data_freshness: "real-time".to_string(),
        processing_time_ms: start_time.elapsed().as_millis() as u64,
        api_version: "v2".to_string(),
    };

    Ok(Json(ApiResponse {
        success: true,
        data: recommendations,
        metadata,
    }))
}

/// GET /reliability/alerts - Get active alerts
async fn get_active_alerts(
    State(state): State<ReliabilityApiState>,
) -> Result<Json<ApiResponse<Vec<ReliabilityAlert>>>, StatusCode> {
    let start_time = std::time::Instant::now();

    let engine = state.engine.read().await;
    let alerts = engine.get_active_alerts().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let metadata = ResponseMetadata {
        generated_at: Utc::now(),
        cache_used: false,
        data_freshness: "real-time".to_string(),
        processing_time_ms: start_time.elapsed().as_millis() as u64,
        api_version: "v2".to_string(),
    };

    Ok(Json(ApiResponse {
        success: true,
        data: alerts,
        metadata,
    }))
}

/// GET /reliability/predictions - Get reliability predictions
async fn get_reliability_predictions(
    State(state): State<ReliabilityApiState>,
    Query(query): Query<ReliabilityQuery>,
) -> Result<Json<ApiResponse<Vec<crate::testing::reliability::ReliabilityPrediction>>>, StatusCode> {
    let start_time = std::time::Instant::now();

    let engine = state.engine.read().await;
    let predictions = engine.generate_predictions(
        query.environment.as_deref(),
        30, // 30-day forecast
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let metadata = ResponseMetadata {
        generated_at: Utc::now(),
        cache_used: false,
        data_freshness: "real-time".to_string(),
        processing_time_ms: start_time.elapsed().as_millis() as u64,
        api_version: "v2".to_string(),
    };

    Ok(Json(ApiResponse {
        success: true,
        data: predictions,
        metadata,
    }))
}

/// POST /reliability/analysis/custom - Run custom analysis
async fn run_custom_analysis(
    State(state): State<ReliabilityApiState>,
    Json(request): Json<CustomAnalysisRequest>,
) -> Result<Json<ApiResponse<ReliabilityOverview>>, StatusCode> {
    let start_time = std::time::Instant::now();

    let engine = state.engine.read().await;
    let overview = engine.run_custom_analysis(&request).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let metadata = ResponseMetadata {
        generated_at: Utc::now(),
        cache_used: false,
        data_freshness: "real-time".to_string(),
        processing_time_ms: start_time.elapsed().as_millis() as u64,
        api_version: "v2".to_string(),
    };

    Ok(Json(ApiResponse {
        success: true,
        data: overview,
        metadata,
    }))
}

/// GET /reliability/test/:test_id/reliability - Get reliability for specific test
async fn get_test_reliability(
    State(state): State<ReliabilityApiState>,
    Path(test_id): Path<String>,
    Query(query): Query<ReliabilityQuery>,
) -> Result<Json<ApiResponse<FlakyTestSummary>>, StatusCode> {
    let start_time = std::time::Instant::now();

    let engine = state.engine.read().await;
    let time_range = parse_time_window(&query.time_window);

    let test_reliability = engine.analyze_test_reliability(
        &test_id,
        time_range,
        query.environment.as_deref(),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let metadata = ResponseMetadata {
        generated_at: Utc::now(),
        cache_used: false,
        data_freshness: "real-time".to_string(),
        processing_time_ms: start_time.elapsed().as_millis() as u64,
        api_version: "v2".to_string(),
    };

    Ok(Json(ApiResponse {
        success: true,
        data: test_reliability,
        metadata,
    }))
}

/// GET /reliability/environment/:env/reliability - Get environment reliability
async fn get_environment_reliability(
    State(state): State<ReliabilityApiState>,
    Path(env): Path<String>,
    Query(query): Query<ReliabilityQuery>,
) -> Result<Json<ApiResponse<super::EnvironmentalReliabilityAnalysis>>, StatusCode> {
    let start_time = std::time::Instant::now();

    let engine = state.engine.read().await;
    let time_range = parse_time_window(&query.time_window);

    let env_reliability = engine.analyze_environment_reliability(
        &env,
        time_range,
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let metadata = ResponseMetadata {
        generated_at: Utc::now(),
        cache_used: false,
        data_freshness: "real-time".to_string(),
        processing_time_ms: start_time.elapsed().as_millis() as u64,
        api_version: "v2".to_string(),
    };

    Ok(Json(ApiResponse {
        success: true,
        data: env_reliability,
        metadata,
    }))
}

/// GET /reliability/status - Get API status
async fn get_api_status(
    State(state): State<ReliabilityApiState>,
) -> Result<Json<ApiResponse<HashMap<String, serde_json::Value>>>, StatusCode> {
    let start_time = std::time::Instant::now();

    let engine = state.engine.read().await;
    let cache = state.cache.read().await;

    let mut status = HashMap::new();
    status.insert("status".to_string(), serde_json::Value::String("healthy".to_string()));
    status.insert("total_test_results".to_string(), serde_json::Value::Number(engine.historical_data.len().into()));
    status.insert("cache_entries".to_string(), serde_json::Value::Number(
        (cache.overview_cache.is_some() as usize +
         cache.health_score_cache.is_some() as usize +
         cache.trend_cache.len() +
         cache.flaky_tests_cache.is_some() as usize +
         cache.patterns_cache.is_some() as usize).into()
    ));
    status.insert("api_version".to_string(), serde_json::Value::String("v2".to_string()));
    status.insert("cache_ttl_minutes".to_string(), serde_json::Value::Number(state.config.cache_ttl_minutes.into()));

    let metadata = ResponseMetadata {
        generated_at: Utc::now(),
        cache_used: false,
        data_freshness: "real-time".to_string(),
        processing_time_ms: start_time.elapsed().as_millis() as u64,
        api_version: "v2".to_string(),
    };

    Ok(Json(ApiResponse {
        success: true,
        data: status,
        metadata,
    }))
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