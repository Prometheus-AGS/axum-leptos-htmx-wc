use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade, State},
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use chrono::{DateTime, Utc, Duration};
use crate::testing::monitoring::comprehensive::TestExecutionResult;
use super::{ChartData, Dataset, DataPoint, VisualizationConfig};

/// Real-time chart update system
#[derive(Debug, Clone)]
pub struct RealTimeChartSystem {
    pub active_subscriptions: Arc<RwLock<HashMap<String, ChartSubscription>>>,
    pub update_broadcaster: broadcast::Sender<ChartUpdateEvent>,
    pub chart_cache: Arc<RwLock<HashMap<String, TimestampedChartData>>>,
}

#[derive(Debug, Clone)]
pub struct ChartSubscription {
    pub subscription_id: String,
    pub chart_config: VisualizationConfig,
    pub client_id: String,
    pub created_at: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub update_frequency: Duration,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChartUpdateEvent {
    pub event_type: String,
    pub chart_id: String,
    pub updated_data: Option<ChartData>,
    pub metadata: UpdateMetadata,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateMetadata {
    pub total_data_points: usize,
    pub data_source: String,
    pub update_reason: String,
    pub performance_metrics: UpdatePerformanceMetrics,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdatePerformanceMetrics {
    pub generation_time_ms: u64,
    pub data_processing_time_ms: u64,
    pub cache_hit: bool,
    pub memory_usage_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct TimestampedChartData {
    pub chart_data: ChartData,
    pub generated_at: DateTime<Utc>,
    pub subscription_ids: Vec<String>,
    pub access_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct WebSocketMessage {
    pub message_type: String,
    pub chart_config: Option<VisualizationConfig>,
    pub subscription_id: Option<String>,
    pub client_id: String,
}

#[derive(Debug, Serialize)]
pub struct WebSocketResponse {
    pub response_type: String,
    pub success: bool,
    pub subscription_id: Option<String>,
    pub chart_data: Option<ChartData>,
    pub error: Option<String>,
    pub metadata: Option<ResponseMetadata>,
}

#[derive(Debug, Serialize)]
pub struct ResponseMetadata {
    pub server_time: DateTime<Utc>,
    pub data_freshness: String,
    pub update_frequency: String,
    pub next_update_eta: Option<DateTime<Utc>>,
}

impl RealTimeChartSystem {
    pub fn new() -> Self {
        let (update_broadcaster, _) = broadcast::channel(1000);

        Self {
            active_subscriptions: Arc::new(RwLock::new(HashMap::new())),
            update_broadcaster,
            chart_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Handle WebSocket connections for real-time chart updates
    pub async fn handle_websocket_connection(
        &self,
        socket: WebSocket,
        client_id: String,
    ) {
        let (mut sender, mut receiver) = socket.split();

        let mut update_receiver = self.update_broadcaster.subscribe();
        let system = self.clone();

        // Handle incoming messages from client
        let system_clone = system.clone();
        let client_id_clone = client_id.clone();
        let sender_clone = Arc::new(tokio::sync::Mutex::new(sender));
        let sender_for_updates = sender_clone.clone();

        // Spawn task to handle client messages
        tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                if let Ok(msg) = msg {
                    if let Ok(text) = msg.to_text() {
                        if let Ok(parsed_msg) = serde_json::from_str::<WebSocketMessage>(text) {
                            system_clone.handle_client_message(
                                parsed_msg,
                                client_id_clone.clone(),
                                sender_clone.clone(),
                            ).await;
                        }
                    }
                }
            }
        });

        // Handle broadcast updates
        tokio::spawn(async move {
            while let Ok(update_event) = update_receiver.recv().await {
                // Check if this client is subscribed to this chart
                let subscriptions = system.active_subscriptions.read().await;
                let client_subscribed = subscriptions.values()
                    .any(|sub| sub.client_id == client_id &&
                         format!("{:?}", sub.chart_config.chart_type) == update_event.chart_id);

                if client_subscribed {
                    let response = WebSocketResponse {
                        response_type: "chart_update".to_string(),
                        success: true,
                        subscription_id: Some(update_event.chart_id.clone()),
                        chart_data: update_event.updated_data,
                        error: None,
                        metadata: Some(ResponseMetadata {
                            server_time: Utc::now(),
                            data_freshness: "real-time".to_string(),
                            update_frequency: "live".to_string(),
                            next_update_eta: None,
                        }),
                    };

                    if let Ok(json) = serde_json::to_string(&response) {
                        let mut sender = sender_for_updates.lock().await;
                        let _ = sender.send(axum::extract::ws::Message::Text(json)).await;
                    }
                }
            }
        });
    }

    async fn handle_client_message(
        &self,
        message: WebSocketMessage,
        client_id: String,
        sender: Arc<tokio::sync::Mutex<futures::stream::SplitSink<WebSocket, axum::extract::ws::Message>>>,
    ) {
        let response = match message.message_type.as_str() {
            "subscribe" => {
                if let Some(config) = message.chart_config {
                    self.handle_subscribe(client_id, config).await
                } else {
                    WebSocketResponse {
                        response_type: "error".to_string(),
                        success: false,
                        subscription_id: None,
                        chart_data: None,
                        error: Some("Chart configuration required for subscription".to_string()),
                        metadata: None,
                    }
                }
            }
            "unsubscribe" => {
                if let Some(subscription_id) = message.subscription_id {
                    self.handle_unsubscribe(subscription_id).await
                } else {
                    WebSocketResponse {
                        response_type: "error".to_string(),
                        success: false,
                        subscription_id: None,
                        chart_data: None,
                        error: Some("Subscription ID required for unsubscribe".to_string()),
                        metadata: None,
                    }
                }
            }
            "get_chart" => {
                if let Some(config) = message.chart_config {
                    self.handle_get_chart(config).await
                } else {
                    WebSocketResponse {
                        response_type: "error".to_string(),
                        success: false,
                        subscription_id: None,
                        chart_data: None,
                        error: Some("Chart configuration required".to_string()),
                        metadata: None,
                    }
                }
            }
            _ => {
                WebSocketResponse {
                    response_type: "error".to_string(),
                    success: false,
                    subscription_id: None,
                    chart_data: None,
                    error: Some(format!("Unknown message type: {}", message.message_type)),
                    metadata: None,
                }
            }
        };

        if let Ok(json) = serde_json::to_string(&response) {
            let mut sender = sender.lock().await;
            let _ = sender.send(axum::extract::ws::Message::Text(json)).await;
        }
    }

    async fn handle_subscribe(
        &self,
        client_id: String,
        config: VisualizationConfig,
    ) -> WebSocketResponse {
        let subscription_id = format!("{}_{}", client_id, Utc::now().timestamp());

        let subscription = ChartSubscription {
            subscription_id: subscription_id.clone(),
            chart_config: config,
            client_id,
            created_at: Utc::now(),
            last_update: Utc::now(),
            update_frequency: Duration::seconds(30), // Default 30-second updates
        };

        let mut subscriptions = self.active_subscriptions.write().await;
        subscriptions.insert(subscription_id.clone(), subscription);

        WebSocketResponse {
            response_type: "subscription_created".to_string(),
            success: true,
            subscription_id: Some(subscription_id),
            chart_data: None,
            error: None,
            metadata: Some(ResponseMetadata {
                server_time: Utc::now(),
                data_freshness: "live".to_string(),
                update_frequency: "30s".to_string(),
                next_update_eta: Some(Utc::now() + Duration::seconds(30)),
            }),
        }
    }

    async fn handle_unsubscribe(&self, subscription_id: String) -> WebSocketResponse {
        let mut subscriptions = self.active_subscriptions.write().await;
        let removed = subscriptions.remove(&subscription_id);

        WebSocketResponse {
            response_type: "subscription_removed".to_string(),
            success: removed.is_some(),
            subscription_id: Some(subscription_id),
            chart_data: None,
            error: if removed.is_none() {
                Some("Subscription not found".to_string())
            } else {
                None
            },
            metadata: Some(ResponseMetadata {
                server_time: Utc::now(),
                data_freshness: "N/A".to_string(),
                update_frequency: "N/A".to_string(),
                next_update_eta: None,
            }),
        }
    }

    async fn handle_get_chart(&self, _config: VisualizationConfig) -> WebSocketResponse {
        // This would generate chart data based on config
        // For now, return a placeholder response
        WebSocketResponse {
            response_type: "chart_data".to_string(),
            success: true,
            subscription_id: None,
            chart_data: Some(self.generate_sample_chart_data()),
            error: None,
            metadata: Some(ResponseMetadata {
                server_time: Utc::now(),
                data_freshness: "current".to_string(),
                update_frequency: "on-demand".to_string(),
                next_update_eta: None,
            }),
        }
    }

    /// Broadcast update to all subscribed clients
    pub async fn broadcast_chart_update(
        &self,
        chart_id: String,
        chart_data: ChartData,
        update_reason: String,
    ) -> Result<(), broadcast::error::SendError<ChartUpdateEvent>> {
        let update_event = ChartUpdateEvent {
            event_type: "data_update".to_string(),
            chart_id: chart_id.clone(),
            updated_data: Some(chart_data.clone()),
            metadata: UpdateMetadata {
                total_data_points: chart_data.datasets.iter()
                    .map(|d| d.data.len())
                    .sum(),
                data_source: "real_time_monitor".to_string(),
                update_reason,
                performance_metrics: UpdatePerformanceMetrics {
                    generation_time_ms: 150, // Mock value
                    data_processing_time_ms: 50, // Mock value
                    cache_hit: false,
                    memory_usage_bytes: 4096, // Mock value
                },
            },
            timestamp: Utc::now(),
        };

        // Update cache
        let mut cache = self.chart_cache.write().await;
        cache.insert(chart_id, TimestampedChartData {
            chart_data,
            generated_at: Utc::now(),
            subscription_ids: Vec::new(),
            access_count: 0,
        });

        self.update_broadcaster.send(update_event)
    }

    /// Process new test results and trigger chart updates
    pub async fn process_new_test_results(&self, results: Vec<TestExecutionResult>) {
        // Analyze which charts need updating
        let subscriptions = self.active_subscriptions.read().await;

        for subscription in subscriptions.values() {
            if self.should_update_subscription(subscription).await {
                // Generate updated chart data based on new results
                if let Ok(updated_chart_data) = self.generate_chart_from_results(&results, &subscription.chart_config).await {
                    let chart_id = format!("{:?}", subscription.chart_config.chart_type);
                    let _ = self.broadcast_chart_update(
                        chart_id,
                        updated_chart_data,
                        "new_test_results".to_string(),
                    ).await;
                }
            }
        }
    }

    async fn should_update_subscription(&self, subscription: &ChartSubscription) -> bool {
        let time_since_update = Utc::now().signed_duration_since(subscription.last_update);
        time_since_update >= subscription.update_frequency
    }

    async fn generate_chart_from_results(
        &self,
        results: &[TestExecutionResult],
        config: &VisualizationConfig,
    ) -> Result<ChartData, Box<dyn std::error::Error + Send + Sync>> {
        // This would use the TestVisualizationEngine to generate charts
        // For now, return sample data
        Ok(self.generate_sample_chart_data())
    }

    fn generate_sample_chart_data(&self) -> ChartData {
        use super::{ChartMetadata, AxisLabels};

        ChartData {
            datasets: vec![
                Dataset {
                    label: "Sample Data".to_string(),
                    data: vec![
                        DataPoint::Numeric(85.0),
                        DataPoint::Numeric(92.0),
                        DataPoint::Numeric(78.0),
                        DataPoint::Numeric(95.0),
                    ],
                    background_color: Some("#3498db".to_string()),
                    border_color: Some("#2980b9".to_string()),
                    fill: Some(false),
                    tension: Some(0.4),
                },
            ],
            labels: vec!["Hour 1".to_string(), "Hour 2".to_string(), "Hour 3".to_string(), "Hour 4".to_string()],
            metadata: ChartMetadata {
                total_data_points: 4,
                time_range: Some((Utc::now() - Duration::hours(4), Utc::now())),
                aggregation_level: "Hourly".to_string(),
                last_updated: Utc::now(),
                chart_title: "Real-time Sample Chart".to_string(),
                axis_labels: AxisLabels {
                    x_axis: "Time".to_string(),
                    y_axis: "Value".to_string(),
                    z_axis: None,
                },
            },
        }
    }

    /// Get system health metrics
    pub async fn get_system_health(&self) -> RealTimeSystemHealth {
        let subscriptions = self.active_subscriptions.read().await;
        let cache = self.chart_cache.read().await;

        RealTimeSystemHealth {
            active_subscriptions: subscriptions.len(),
            cached_charts: cache.len(),
            memory_usage_estimate: self.estimate_memory_usage(&*cache).await,
            uptime_seconds: 0, // Would track actual uptime
            total_updates_sent: 0, // Would track actual count
            average_update_latency_ms: 0.0, // Would track actual latency
            error_rate: 0.0, // Would track actual error rate
        }
    }

    async fn estimate_memory_usage(&self, cache: &HashMap<String, TimestampedChartData>) -> usize {
        // Rough estimation of memory usage
        cache.len() * 4096 // Simplified calculation
    }
}

#[derive(Debug, Serialize)]
pub struct RealTimeSystemHealth {
    pub active_subscriptions: usize,
    pub cached_charts: usize,
    pub memory_usage_estimate: usize,
    pub uptime_seconds: u64,
    pub total_updates_sent: u64,
    pub average_update_latency_ms: f64,
    pub error_rate: f64,
}

/// WebSocket handler for Axum integration
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(chart_system): State<Arc<RealTimeChartSystem>>,
) -> Response {
    ws.on_upgrade(move |socket| async move {
        let client_id = format!("client_{}", Utc::now().timestamp_micros());
        chart_system.handle_websocket_connection(socket, client_id).await;
    })
}

/// Periodic task to clean up stale subscriptions and cache
pub async fn cleanup_task(chart_system: Arc<RealTimeChartSystem>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes

    loop {
        interval.tick().await;

        // Clean up stale subscriptions (older than 1 hour with no activity)
        let cutoff_time = Utc::now() - Duration::hours(1);
        let mut subscriptions = chart_system.active_subscriptions.write().await;
        subscriptions.retain(|_, subscription| subscription.last_update > cutoff_time);

        // Clean up stale cache entries (older than 30 minutes)
        let cache_cutoff = Utc::now() - Duration::minutes(30);
        let mut cache = chart_system.chart_cache.write().await;
        cache.retain(|_, cached_data| cached_data.generated_at > cache_cutoff);

        tracing::info!(
            "Cleanup completed: {} active subscriptions, {} cached charts",
            subscriptions.len(),
            cache.len()
        );
    }
}

impl Default for RealTimeChartSystem {
    fn default() -> Self {
        Self::new()
    }
}