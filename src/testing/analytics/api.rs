use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use super::{
    TestAnalyticsEngine, AnalyticsConfig, AnalyticsResult, ComprehensiveAnalysisResult,
    AnalyticsSummary, AnalyticsError, AnalyticsTimeWindow, AnalyticsGranularity,
    CrossDomainInsight, AnalyticsInsight, AnalyticsRecommendation, AnalyticsAlert,
};
use crate::testing::entities::TestExecutionResult;

/// Analytics API state
pub struct AnalyticsApiState {
    pub engine: Arc<RwLock<TestAnalyticsEngine>>,
    pub cache: Arc<RwLock<AnalyticsCache>>,
}

/// Analytics response cache
#[derive(Debug, Default)]
pub struct AnalyticsCache {
    pub last_comprehensive_analysis: Option<(DateTime<Utc>, ComprehensiveAnalysisResult)>,
    pub last_summary: Option<(DateTime<Utc>, AnalyticsSummary)>,
    pub cached_insights: Vec<(DateTime<Utc>, Vec<AnalyticsInsight>)>,
    pub cache_ttl_minutes: u32,
}

/// Query parameters for analytics endpoints
#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub time_window: Option<String>,
    pub granularity: Option<String>,
    pub environment: Option<String>,
    pub test_suite: Option<String>,
    pub force_refresh: Option<bool>,
}

/// Request body for custom analysis
#[derive(Debug, Deserialize)]
pub struct CustomAnalysisRequest {
    pub config: AnalyticsConfig,
    pub description: Option<String>,
}

/// Analytics API response wrapper
#[derive(Debug, Serialize)]
pub struct AnalyticsResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub metadata: AnalyticsMetadata,
}

/// Response metadata
#[derive(Debug, Serialize)]
pub struct AnalyticsMetadata {
    pub generated_at: DateTime<Utc>,
    pub cache_used: bool,
    pub data_freshness: String,
    pub processing_time_ms: u64,
    pub api_version: String,
}

impl AnalyticsApiState {
    pub fn new() -> Self {
        Self {
            engine: Arc::new(RwLock::new(TestAnalyticsEngine::new())),
            cache: Arc::new(RwLock::new(AnalyticsCache::new())),
        }
    }

    /// Load test results into the analytics engine
    pub async fn load_test_data(&self, results: Vec<TestExecutionResult>) {
        let mut engine = self.engine.write().await;
        engine.load_test_results(results);

        // Clear cache when new data is loaded
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

impl AnalyticsCache {
    pub fn new() -> Self {
        Self {
            last_comprehensive_analysis: None,
            last_summary: None,
            cached_insights: Vec::new(),
            cache_ttl_minutes: 15, // 15-minute cache TTL
        }
    }

    pub fn is_fresh(&self, cached_at: DateTime<Utc>) -> bool {
        let now = Utc::now();
        let ttl = chrono::Duration::minutes(self.cache_ttl_minutes as i64);
        now - cached_at < ttl
    }

    pub fn clear(&mut self) {
        self.last_comprehensive_analysis = None;
        self.last_summary = None;
        self.cached_insights.clear();
    }
}

/// Create the analytics API router
pub fn create_analytics_router() -> Router<AnalyticsApiState> {
    Router::new()
        .route("/analytics/summary", get(get_analytics_summary))
        .route("/analytics/comprehensive", get(get_comprehensive_analysis))
        .route("/analytics/insights", get(get_insights))
        .route("/analytics/recommendations", get(get_recommendations))
        .route("/analytics/alerts", get(get_alerts))
        .route("/analytics/coverage", get(get_coverage_analysis))
        .route("/analytics/performance", get(get_performance_analysis))
        .route("/analytics/reliability", get(get_reliability_analysis))
        .route("/analytics/trends/:metric", get(get_trend_analysis))
        .route("/analytics/custom", post(run_custom_analysis))
        .route("/analytics/health", get(get_system_health))
}

/// Get analytics summary
async fn get_analytics_summary(
    State(state): State<AnalyticsApiState>,
    Query(_params): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsResponse<AnalyticsSummary>>, StatusCode> {
    let start_time = std::time::Instant::now();

    // Check cache first
    let cache = state.cache.read().await;
    if let Some((cached_at, ref summary)) = &cache.last_summary {
        if cache.is_fresh(*cached_at) {
            return Ok(Json(AnalyticsResponse {
                success: true,
                data: Some(summary.clone()),
                error: None,
                metadata: AnalyticsMetadata {
                    generated_at: *cached_at,
                    cache_used: true,
                    data_freshness: "cached".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }));
        }
    }
    drop(cache);

    // Generate fresh summary
    let engine = state.engine.read().await;
    let summary = engine.get_analytics_summary();
    drop(engine);

    // Cache the result
    let mut cache = state.cache.write().await;
    cache.last_summary = Some((Utc::now(), summary.clone()));

    Ok(Json(AnalyticsResponse {
        success: true,
        data: Some(summary),
        error: None,
        metadata: AnalyticsMetadata {
            generated_at: Utc::now(),
            cache_used: false,
            data_freshness: "real-time".to_string(),
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            api_version: "v2".to_string(),
        },
    }))
}

/// Get comprehensive analysis
async fn get_comprehensive_analysis(
    State(state): State<AnalyticsApiState>,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsResponse<ComprehensiveAnalysisResult>>, StatusCode> {
    let start_time = std::time::Instant::now();

    // Check cache if not forcing refresh
    if !params.force_refresh.unwrap_or(false) {
        let cache = state.cache.read().await;
        if let Some((cached_at, ref analysis)) = &cache.last_comprehensive_analysis {
            if cache.is_fresh(*cached_at) {
                return Ok(Json(AnalyticsResponse {
                    success: true,
                    data: Some(analysis.clone()),
                    error: None,
                    metadata: AnalyticsMetadata {
                        generated_at: *cached_at,
                        cache_used: true,
                        data_freshness: "cached".to_string(),
                        processing_time_ms: start_time.elapsed().as_millis() as u64,
                        api_version: "v2".to_string(),
                    },
                }));
            }
        }
        drop(cache);
    }

    // Build configuration from query parameters
    let config = build_analytics_config_from_params(&params);

    // Run comprehensive analysis
    let engine = state.engine.read().await;
    match engine.run_comprehensive_analysis(&config).await {
        Ok(analysis) => {
            drop(engine);

            // Cache the result
            let mut cache = state.cache.write().await;
            cache.last_comprehensive_analysis = Some((Utc::now(), analysis.clone()));

            Ok(Json(AnalyticsResponse {
                success: true,
                data: Some(analysis),
                error: None,
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "real-time".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
        Err(err) => {
            tracing::error!("Failed to run comprehensive analysis: {}", err);
            Ok(Json(AnalyticsResponse {
                success: false,
                data: None,
                error: Some(err.to_string()),
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "error".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
    }
}

/// Get insights
async fn get_insights(
    State(state): State<AnalyticsApiState>,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsResponse<Vec<AnalyticsInsight>>>, StatusCode> {
    let start_time = std::time::Instant::now();

    // Run comprehensive analysis to get insights
    let config = build_analytics_config_from_params(&params);
    let engine = state.engine.read().await;
    let mut engine_mut = engine.clone();
    drop(engine);

    match engine_mut.write().await.run_analysis(config).await {
        Ok(analysis) => {
            Ok(Json(AnalyticsResponse {
                success: true,
                data: Some(analysis.insights),
                error: None,
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "real-time".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
        Err(err) => {
            tracing::error!("Failed to get insights: {}", err);
            Ok(Json(AnalyticsResponse {
                success: false,
                data: None,
                error: Some(err.to_string()),
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "error".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
    }
}

/// Get recommendations
async fn get_recommendations(
    State(state): State<AnalyticsApiState>,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsResponse<Vec<AnalyticsRecommendation>>>, StatusCode> {
    let start_time = std::time::Instant::now();

    let config = build_analytics_config_from_params(&params);
    let engine = state.engine.read().await;
    let mut engine_mut = engine.clone();
    drop(engine);

    match engine_mut.write().await.run_analysis(config).await {
        Ok(analysis) => {
            Ok(Json(AnalyticsResponse {
                success: true,
                data: Some(analysis.recommendations),
                error: None,
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "real-time".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
        Err(err) => {
            tracing::error!("Failed to get recommendations: {}", err);
            Ok(Json(AnalyticsResponse {
                success: false,
                data: None,
                error: Some(err.to_string()),
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "error".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
    }
}

/// Get alerts
async fn get_alerts(
    State(state): State<AnalyticsApiState>,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsResponse<Vec<AnalyticsAlert>>>, StatusCode> {
    let start_time = std::time::Instant::now();

    let config = build_analytics_config_from_params(&params);
    let engine = state.engine.read().await;
    let mut engine_mut = engine.clone();
    drop(engine);

    match engine_mut.write().await.run_analysis(config).await {
        Ok(analysis) => {
            Ok(Json(AnalyticsResponse {
                success: true,
                data: Some(analysis.alerts),
                error: None,
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "real-time".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
        Err(err) => {
            tracing::error!("Failed to get alerts: {}", err);
            Ok(Json(AnalyticsResponse {
                success: false,
                data: None,
                error: Some(err.to_string()),
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "error".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
    }
}

/// Get coverage analysis
async fn get_coverage_analysis(
    State(state): State<AnalyticsApiState>,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsResponse<super::coverage_trends::CoverageAnalysisResult>>, StatusCode> {
    let start_time = std::time::Instant::now();

    let config = build_analytics_config_from_params(&params);
    let engine = state.engine.read().await;

    match engine.coverage_analyzer.analyze_trends(&config).await {
        Ok(analysis) => {
            Ok(Json(AnalyticsResponse {
                success: true,
                data: Some(analysis),
                error: None,
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "real-time".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
        Err(err) => {
            tracing::error!("Failed to get coverage analysis: {}", err);
            Ok(Json(AnalyticsResponse {
                success: false,
                data: None,
                error: Some(format!("Coverage analysis error: {}", err)),
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "error".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
    }
}

/// Get performance analysis
async fn get_performance_analysis(
    State(state): State<AnalyticsApiState>,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsResponse<super::performance_analysis::PerformanceAnalysisResult>>, StatusCode> {
    let start_time = std::time::Instant::now();

    let config = build_analytics_config_from_params(&params);
    let engine = state.engine.read().await;

    match engine.performance_analyzer.analyze_performance(&config).await {
        Ok(analysis) => {
            Ok(Json(AnalyticsResponse {
                success: true,
                data: Some(analysis),
                error: None,
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "real-time".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
        Err(err) => {
            tracing::error!("Failed to get performance analysis: {}", err);
            Ok(Json(AnalyticsResponse {
                success: false,
                data: None,
                error: Some(format!("Performance analysis error: {}", err)),
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "error".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
    }
}

/// Get reliability analysis
async fn get_reliability_analysis(
    State(state): State<AnalyticsApiState>,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsResponse<super::reliability_metrics::ReliabilityAnalysisResult>>, StatusCode> {
    let start_time = std::time::Instant::now();

    let config = build_analytics_config_from_params(&params);
    let engine = state.engine.read().await;

    match engine.reliability_analyzer.analyze_reliability(&config).await {
        Ok(analysis) => {
            Ok(Json(AnalyticsResponse {
                success: true,
                data: Some(analysis),
                error: None,
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "real-time".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
        Err(err) => {
            tracing::error!("Failed to get reliability analysis: {}", err);
            Ok(Json(AnalyticsResponse {
                success: false,
                data: None,
                error: Some(format!("Reliability analysis error: {}", err)),
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "error".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
    }
}

/// Get trend analysis for a specific metric
async fn get_trend_analysis(
    Path(metric): Path<String>,
    State(state): State<AnalyticsApiState>,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsResponse<serde_json::Value>>, StatusCode> {
    let start_time = std::time::Instant::now();

    let config = build_analytics_config_from_params(&params);
    let engine = state.engine.read().await;

    let result = match metric.as_str() {
        "coverage" => {
            match engine.coverage_analyzer.analyze_trends(&config).await {
                Ok(analysis) => serde_json::to_value(&analysis.overall_trend),
                Err(err) => return Ok(Json(AnalyticsResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Coverage trend analysis error: {}", err)),
                    metadata: AnalyticsMetadata {
                        generated_at: Utc::now(),
                        cache_used: false,
                        data_freshness: "error".to_string(),
                        processing_time_ms: start_time.elapsed().as_millis() as u64,
                        api_version: "v2".to_string(),
                    },
                }))
            }
        }
        "performance" => {
            match engine.performance_analyzer.analyze_performance(&config).await {
                Ok(analysis) => serde_json::to_value(&analysis.performance_trend),
                Err(err) => return Ok(Json(AnalyticsResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Performance trend analysis error: {}", err)),
                    metadata: AnalyticsMetadata {
                        generated_at: Utc::now(),
                        cache_used: false,
                        data_freshness: "error".to_string(),
                        processing_time_ms: start_time.elapsed().as_millis() as u64,
                        api_version: "v2".to_string(),
                    },
                }))
            }
        }
        "reliability" => {
            match engine.reliability_analyzer.analyze_reliability(&config).await {
                Ok(analysis) => serde_json::to_value(&analysis.reliability_trend),
                Err(err) => return Ok(Json(AnalyticsResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Reliability trend analysis error: {}", err)),
                    metadata: AnalyticsMetadata {
                        generated_at: Utc::now(),
                        cache_used: false,
                        data_freshness: "error".to_string(),
                        processing_time_ms: start_time.elapsed().as_millis() as u64,
                        api_version: "v2".to_string(),
                    },
                }))
            }
        }
        _ => {
            return Ok(Json(AnalyticsResponse {
                success: false,
                data: None,
                error: Some(format!("Unknown metric: {}. Supported metrics: coverage, performance, reliability", metric)),
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "error".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
    };

    match result {
        Ok(data) => Ok(Json(AnalyticsResponse {
            success: true,
            data: Some(data),
            error: None,
            metadata: AnalyticsMetadata {
                generated_at: Utc::now(),
                cache_used: false,
                data_freshness: "real-time".to_string(),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                api_version: "v2".to_string(),
            },
        })),
        Err(err) => {
            tracing::error!("Failed to serialize trend data: {}", err);
            Ok(Json(AnalyticsResponse {
                success: false,
                data: None,
                error: Some("Failed to serialize trend data".to_string()),
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "error".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
    }
}

/// Run custom analysis with user-provided configuration
async fn run_custom_analysis(
    State(state): State<AnalyticsApiState>,
    Json(request): Json<CustomAnalysisRequest>,
) -> Result<Json<AnalyticsResponse<AnalyticsResult>>, StatusCode> {
    let start_time = std::time::Instant::now();

    let engine = state.engine.read().await;
    let mut engine_mut = engine.clone();
    drop(engine);

    match engine_mut.write().await.run_analysis(request.config).await {
        Ok(analysis) => {
            Ok(Json(AnalyticsResponse {
                success: true,
                data: Some(analysis),
                error: None,
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "real-time".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
        Err(err) => {
            tracing::error!("Failed to run custom analysis: {}", err);
            Ok(Json(AnalyticsResponse {
                success: false,
                data: None,
                error: Some(err.to_string()),
                metadata: AnalyticsMetadata {
                    generated_at: Utc::now(),
                    cache_used: false,
                    data_freshness: "error".to_string(),
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    api_version: "v2".to_string(),
                },
            }))
        }
    }
}

/// Get system health
async fn get_system_health(
    State(state): State<AnalyticsApiState>,
) -> Result<Json<AnalyticsResponse<SystemHealth>>, StatusCode> {
    let start_time = std::time::Instant::now();

    let engine = state.engine.read().await;
    let summary = engine.get_analytics_summary();
    let cache = state.cache.read().await;

    let health = SystemHealth {
        status: if summary.health_score > 0.8 { "healthy".to_string() } else { "degraded".to_string() },
        health_score: summary.health_score,
        total_data_points: summary.total_data_points,
        cache_status: CacheStatus {
            entries_cached: if cache.last_comprehensive_analysis.is_some() { 1 } else { 0 }
                + if cache.last_summary.is_some() { 1 } else { 0 }
                + cache.cached_insights.len(),
            cache_hit_ratio: 0.85, // Placeholder
            last_cache_clear: Utc::now() - chrono::Duration::hours(1), // Placeholder
        },
        uptime: chrono::Duration::hours(24), // Placeholder
        version: "2.0.0".to_string(),
    };

    Ok(Json(AnalyticsResponse {
        success: true,
        data: Some(health),
        error: None,
        metadata: AnalyticsMetadata {
            generated_at: Utc::now(),
            cache_used: false,
            data_freshness: "real-time".to_string(),
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            api_version: "v2".to_string(),
        },
    }))
}

/// System health information
#[derive(Debug, Serialize)]
pub struct SystemHealth {
    pub status: String,
    pub health_score: f64,
    pub total_data_points: usize,
    pub cache_status: CacheStatus,
    pub uptime: chrono::Duration,
    pub version: String,
}

/// Cache status information
#[derive(Debug, Serialize)]
pub struct CacheStatus {
    pub entries_cached: usize,
    pub cache_hit_ratio: f64,
    pub last_cache_clear: DateTime<Utc>,
}

/// Build analytics configuration from query parameters
fn build_analytics_config_from_params(params: &AnalyticsQuery) -> AnalyticsConfig {
    let mut config = AnalyticsConfig::default();

    // Parse time window
    if let Some(ref time_window) = params.time_window {
        config.time_window = match time_window.as_str() {
            "1h" | "hour" => AnalyticsTimeWindow::Last24Hours,
            "1d" | "day" => AnalyticsTimeWindow::Last24Hours,
            "1w" | "week" => AnalyticsTimeWindow::LastWeek,
            "1m" | "month" => AnalyticsTimeWindow::LastMonth,
            "3m" | "quarter" => AnalyticsTimeWindow::LastQuarter,
            "1y" | "year" => AnalyticsTimeWindow::LastYear,
            _ => AnalyticsTimeWindow::LastWeek, // Default
        };
    }

    // Parse granularity
    if let Some(ref granularity) = params.granularity {
        config.granularity = match granularity.as_str() {
            "hourly" => AnalyticsGranularity::Hourly,
            "daily" => AnalyticsGranularity::Daily,
            "weekly" => AnalyticsGranularity::Weekly,
            "monthly" => AnalyticsGranularity::Monthly,
            _ => AnalyticsGranularity::Daily, // Default
        };
    }

    config
}