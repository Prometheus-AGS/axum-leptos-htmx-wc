use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, broadcast};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use super::{
    Alert, AlertSeverity, AlertStatus, AlertType, EscalationPolicy,
    SuppressionRule, AlertNotification, NotificationStatus, AlertingConfig,
    NotificationChannel, AlertWebhookPayload, WebhookEventType,
};
use crate::testing::{TestExecutionResult, reliability::TestStatus};

/// Core alert engine
pub struct AlertEngine {
    pub alerts: Arc<RwLock<HashMap<String, Alert>>>,
    pub escalation_policies: Arc<RwLock<HashMap<String, EscalationPolicy>>>,
    pub suppression_rules: Arc<RwLock<Vec<SuppressionRule>>>,
    pub notification_channels: Arc<RwLock<HashMap<String, Arc<dyn NotificationChannel>>>>,
    pub pending_notifications: Arc<RwLock<Vec<AlertNotification>>>,
    pub alert_history: Arc<RwLock<AlertHistory>>,
    pub metrics: Arc<RwLock<AlertMetrics>>,
    pub config: AlertingConfig,

    // Internal channels for async processing
    alert_sender: mpsc::Sender<AlertEvent>,
    alert_receiver: Arc<RwLock<Option<mpsc::Receiver<AlertEvent>>>>,
    broadcast_sender: broadcast::Sender<Alert>,
    escalation_scheduler: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

/// Alert engine configuration
#[derive(Debug, Clone)]
pub struct AlertEngineConfig {
    pub alert_processing_capacity: usize,
    pub escalation_check_interval_seconds: u64,
    pub notification_batch_size: usize,
    pub metrics_update_interval_seconds: u64,
    pub cleanup_interval_hours: u64,
}

/// Alert context containing relevant data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertContext {
    pub source_system: String,
    pub test_results: Option<Vec<TestExecutionResult>>,
    pub metrics: HashMap<String, f64>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub environment: Option<String>,
    pub affected_components: Vec<String>,
    pub related_entities: Vec<String>,
}

/// Alert state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertState {
    pub alert_id: String,
    pub escalation_count: u32,
    pub last_escalation: Option<DateTime<Utc>>,
    pub next_escalation: Option<DateTime<Utc>>,
    pub notification_attempts: u32,
    pub last_notification: Option<DateTime<Utc>>,
    pub acknowledgment_required: bool,
    pub auto_resolve_at: Option<DateTime<Utc>>,
}

/// Alert history tracking
#[derive(Debug, Default)]
pub struct AlertHistory {
    pub events: Vec<AlertHistoryEvent>,
    pub alert_count_by_day: HashMap<String, usize>,
    pub resolution_times: HashMap<String, Duration>,
    pub escalation_stats: HashMap<String, EscalationStats>,
}

/// Alert history event
#[derive(Debug, Clone, Serialize)]
pub struct AlertHistoryEvent {
    pub event_id: String,
    pub alert_id: String,
    pub event_type: AlertEventType,
    pub timestamp: DateTime<Utc>,
    pub user: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
}

/// Alert event types for history
#[derive(Debug, Clone, Serialize)]
pub enum AlertEventType {
    Created,
    Acknowledged,
    Resolved,
    Escalated,
    Suppressed,
    NotificationSent,
    NotificationFailed,
    AutoResolved,
    Expired,
}

/// Escalation statistics
#[derive(Debug, Clone, Serialize)]
pub struct EscalationStats {
    pub policy_id: String,
    pub total_escalations: usize,
    pub average_escalation_time: Duration,
    pub successful_escalations: usize,
    pub failed_escalations: usize,
}

/// Alert metrics for monitoring
#[derive(Debug, Default)]
pub struct AlertMetrics {
    pub total_alerts_created: u64,
    pub alerts_by_severity: HashMap<AlertSeverity, u64>,
    pub alerts_by_type: HashMap<String, u64>,
    pub average_resolution_time: Duration,
    pub notification_success_rate: f64,
    pub escalation_rate: f64,
    pub suppression_rate: f64,
    pub false_positive_rate: f64,
    pub last_updated: DateTime<Utc>,
}

/// Internal alert events
#[derive(Debug, Clone)]
pub enum AlertEvent {
    CreateAlert(Alert),
    AcknowledgeAlert(String, String),
    ResolveAlert(String, Option<String>),
    SuppressAlert(String, u32),
    EscalateAlert(String),
    SendNotification(String, String),
    CleanupExpiredAlerts,
    UpdateMetrics,
}

impl AlertEngine {
    /// Create a new alert engine
    pub fn new(config: AlertingConfig) -> Self {
        let (alert_sender, alert_receiver) = mpsc::channel(1000);
        let (broadcast_sender, _) = broadcast::channel(100);

        Self {
            alerts: Arc::new(RwLock::new(HashMap::new())),
            escalation_policies: Arc::new(RwLock::new(HashMap::new())),
            suppression_rules: Arc::new(RwLock::new(Vec::new())),
            notification_channels: Arc::new(RwLock::new(HashMap::new())),
            pending_notifications: Arc::new(RwLock::new(Vec::new())),
            alert_history: Arc::new(RwLock::new(AlertHistory::default())),
            metrics: Arc::new(RwLock::new(AlertMetrics::default())),
            config,
            alert_sender,
            alert_receiver: Arc::new(RwLock::new(Some(alert_receiver))),
            broadcast_sender,
            escalation_scheduler: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the alert engine
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🚨 Starting Alert Engine...");

        // Start the main event processing loop
        self.start_event_processor().await?;

        // Start periodic tasks
        self.start_escalation_processor().await;
        self.start_notification_processor().await;
        self.start_metrics_updater().await;
        self.start_cleanup_processor().await;

        println!("✓ Alert Engine started successfully");
        Ok(())
    }

    /// Create a new alert
    pub async fn create_alert(
        &self,
        rule_id: String,
        alert_type: AlertType,
        severity: AlertSeverity,
        title: String,
        description: String,
        context: AlertContext,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let alert = Alert::new(rule_id, alert_type, severity, title, description, context);
        let alert_id = alert.id.clone();

        // Check suppression rules
        if self.is_suppressed(&alert).await {
            println!("Alert {} suppressed by rules", alert_id);
            return Ok(alert_id);
        }

        // Send alert creation event
        self.alert_sender.send(AlertEvent::CreateAlert(alert)).await
            .map_err(|_| "Failed to queue alert creation")?;

        Ok(alert_id)
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(
        &self,
        alert_id: &str,
        acknowledged_by: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.alert_sender
            .send(AlertEvent::AcknowledgeAlert(alert_id.to_string(), acknowledged_by))
            .await
            .map_err(|_| "Failed to queue alert acknowledgment")?;

        Ok(())
    }

    /// Resolve an alert
    pub async fn resolve_alert(
        &self,
        alert_id: &str,
        resolution_reason: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.alert_sender
            .send(AlertEvent::ResolveAlert(alert_id.to_string(), resolution_reason))
            .await
            .map_err(|_| "Failed to queue alert resolution")?;

        Ok(())
    }

    /// Suppress an alert
    pub async fn suppress_alert(
        &self,
        alert_id: &str,
        duration_minutes: u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.alert_sender
            .send(AlertEvent::SuppressAlert(alert_id.to_string(), duration_minutes))
            .await
            .map_err(|_| "Failed to queue alert suppression")?;

        Ok(())
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.values()
            .filter(|alert| alert.status.is_active())
            .cloned()
            .collect()
    }

    /// Get alert by ID
    pub async fn get_alert(&self, alert_id: &str) -> Option<Alert> {
        let alerts = self.alerts.read().await;
        alerts.get(alert_id).cloned()
    }

    /// Get alerts by criteria
    pub async fn get_alerts_by_criteria(
        &self,
        severity: Option<AlertSeverity>,
        status: Option<AlertStatus>,
        alert_type: Option<AlertType>,
        time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    ) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.values()
            .filter(|alert| {
                if let Some(sev) = &severity {
                    if &alert.severity != sev { return false; }
                }
                if let Some(stat) = &status {
                    if &alert.status != stat { return false; }
                }
                if let Some(atype) = &alert_type {
                    if std::mem::discriminant(&alert.alert_type) != std::mem::discriminant(atype) {
                        return false;
                    }
                }
                if let Some((start, end)) = time_range {
                    if alert.created_at < start || alert.created_at > end {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    }

    /// Add escalation policy
    pub async fn add_escalation_policy(&self, policy: EscalationPolicy) {
        let mut policies = self.escalation_policies.write().await;
        policies.insert(policy.policy_id.clone(), policy);
    }

    /// Add suppression rule
    pub async fn add_suppression_rule(&self, rule: SuppressionRule) {
        let mut rules = self.suppression_rules.write().await;
        rules.push(rule);
    }

    /// Add notification channel
    pub async fn add_notification_channel(
        &self,
        channel_id: String,
        channel: Arc<dyn NotificationChannel>,
    ) {
        let mut channels = self.notification_channels.write().await;
        channels.insert(channel_id, channel);
    }

    /// Process test results for alerting
    pub async fn process_test_results(
        &self,
        test_results: &[TestExecutionResult],
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut alert_ids = Vec::new();

        // Analyze test results for various alert conditions
        for result in test_results {
            // Test failure alert
            if matches!(result.status, TestStatus::Failed | TestStatus::TimedOut) {
                let context = AlertContext {
                    source_system: "test_execution".to_string(),
                    test_results: Some(vec![result.clone()]),
                    metrics: HashMap::new(),
                    metadata: HashMap::new(),
                    environment: Some(result.environment.clone()),
                    affected_components: vec![result.test_suite.clone()],
                    related_entities: vec![result.test_id.clone()],
                };

                let severity = if matches!(result.status, TestStatus::TimedOut) {
                    AlertSeverity::High
                } else {
                    AlertSeverity::Medium
                };

                let alert_id = self.create_alert(
                    format!("test_failure_rule_{}", result.test_id),
                    AlertType::TestFailure,
                    severity,
                    format!("Test Failed: {}", result.test_name),
                    format!("Test '{}' failed in environment '{}'", result.test_name, result.environment),
                    context,
                ).await?;

                alert_ids.push(alert_id);
            }

            // Flaky test alert
            if result.was_flaky && result.retry_count > 2 {
                let context = AlertContext {
                    source_system: "reliability_analysis".to_string(),
                    test_results: Some(vec![result.clone()]),
                    metrics: [("retry_count".to_string(), result.retry_count as f64)].into(),
                    metadata: HashMap::new(),
                    environment: Some(result.environment.clone()),
                    affected_components: vec![result.test_suite.clone()],
                    related_entities: vec![result.test_id.clone()],
                };

                let alert_id = self.create_alert(
                    format!("flaky_test_rule_{}", result.test_id),
                    AlertType::Flakiness,
                    AlertSeverity::Medium,
                    format!("Flaky Test Detected: {}", result.test_name),
                    format!("Test '{}' is exhibiting flaky behavior (retry count: {})",
                        result.test_name, result.retry_count),
                    context,
                ).await?;

                alert_ids.push(alert_id);
            }

            // Performance regression alert
            if result.execution_time_ms > 10000.0 {  // 10 second threshold
                let context = AlertContext {
                    source_system: "performance_monitoring".to_string(),
                    test_results: Some(vec![result.clone()]),
                    metrics: [("execution_time_ms".to_string(), result.execution_time_ms)].into(),
                    metadata: HashMap::new(),
                    environment: Some(result.environment.clone()),
                    affected_components: vec![result.test_suite.clone()],
                    related_entities: vec![result.test_id.clone()],
                };

                let alert_id = self.create_alert(
                    format!("performance_rule_{}", result.test_id),
                    AlertType::PerformanceRegression,
                    AlertSeverity::Medium,
                    format!("Slow Test Execution: {}", result.test_name),
                    format!("Test '{}' took {:.2}ms to execute (threshold: 10000ms)",
                        result.test_name, result.execution_time_ms),
                    context,
                ).await?;

                alert_ids.push(alert_id);
            }
        }

        // Batch analysis for system-wide issues
        let failure_rate = test_results.iter()
            .filter(|r| matches!(r.status, TestStatus::Failed))
            .count() as f64 / test_results.len() as f64;

        if failure_rate > 0.2 {  // 20% failure rate threshold
            let context = AlertContext {
                source_system: "system_health".to_string(),
                test_results: Some(test_results.to_vec()),
                metrics: [("failure_rate".to_string(), failure_rate)].into(),
                metadata: HashMap::new(),
                environment: None,
                affected_components: test_results.iter()
                    .map(|r| r.test_suite.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect(),
                related_entities: vec![],
            };

            let alert_id = self.create_alert(
                "system_health_rule_high_failure_rate".to_string(),
                AlertType::SystemHealth,
                AlertSeverity::Critical,
                "High Test Failure Rate Detected".to_string(),
                format!("System-wide test failure rate is {:.1}% (threshold: 20%)", failure_rate * 100.0),
                context,
            ).await?;

            alert_ids.push(alert_id);
        }

        Ok(alert_ids)
    }

    /// Get alert metrics
    pub async fn get_metrics(&self) -> AlertMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Subscribe to alert updates
    pub fn subscribe(&self) -> broadcast::Receiver<Alert> {
        self.broadcast_sender.subscribe()
    }

    /// Check if an alert should be suppressed
    async fn is_suppressed(&self, alert: &Alert) -> bool {
        let rules = self.suppression_rules.read().await;
        rules.iter().any(|rule| rule.matches_alert(alert))
    }

    /// Start the main event processing loop
    async fn start_event_processor(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut receiver = self.alert_receiver.write().await.take()
            .ok_or("Event processor already started")?;

        let alerts = self.alerts.clone();
        let history = self.alert_history.clone();
        let metrics = self.metrics.clone();
        let broadcast_sender = self.broadcast_sender.clone();

        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match event {
                    AlertEvent::CreateAlert(mut alert) => {
                        // Add to history
                        {
                            let mut hist = history.write().await;
                            hist.events.push(AlertHistoryEvent {
                                event_id: uuid::Uuid::new_v4().to_string(),
                                alert_id: alert.id.clone(),
                                event_type: AlertEventType::Created,
                                timestamp: Utc::now(),
                                user: None,
                                details: HashMap::new(),
                            });
                        }

                        // Update metrics
                        {
                            let mut m = metrics.write().await;
                            m.total_alerts_created += 1;
                            *m.alerts_by_severity.entry(alert.severity.clone()).or_insert(0) += 1;
                            *m.alerts_by_type.entry(format!("{:?}", alert.alert_type)).or_insert(0) += 1;
                            m.last_updated = Utc::now();
                        }

                        // Broadcast alert
                        let _ = broadcast_sender.send(alert.clone());

                        // Store alert
                        {
                            let mut alert_map = alerts.write().await;
                            alert_map.insert(alert.id.clone(), alert);
                        }
                    }
                    AlertEvent::AcknowledgeAlert(alert_id, user) => {
                        let mut alert_map = alerts.write().await;
                        if let Some(alert) = alert_map.get_mut(&alert_id) {
                            alert.acknowledge(user);
                            let _ = broadcast_sender.send(alert.clone());
                        }
                    }
                    AlertEvent::ResolveAlert(alert_id, reason) => {
                        let mut alert_map = alerts.write().await;
                        if let Some(alert) = alert_map.get_mut(&alert_id) {
                            alert.resolve(reason);
                            let _ = broadcast_sender.send(alert.clone());
                        }
                    }
                    AlertEvent::SuppressAlert(alert_id, duration) => {
                        let mut alert_map = alerts.write().await;
                        if let Some(alert) = alert_map.get_mut(&alert_id) {
                            alert.suppress(duration);
                            let _ = broadcast_sender.send(alert.clone());
                        }
                    }
                    _ => {
                        // Handle other events as needed
                    }
                }
            }
        });

        Ok(())
    }

    /// Start the escalation processor
    async fn start_escalation_processor(&self) {
        let alerts = self.alerts.clone();
        let policies = self.escalation_policies.clone();
        let alert_sender = self.alert_sender.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60)); // Check every minute

            loop {
                interval.tick().await;

                let alerts_to_check = {
                    let alert_map = alerts.read().await;
                    alert_map.values()
                        .filter(|alert| alert.status == AlertStatus::Active)
                        .cloned()
                        .collect::<Vec<_>>()
                };

                for alert in alerts_to_check {
                    // Check if escalation is due
                    let should_escalate = {
                        let policies_map = policies.read().await;
                        if let Some(policy_id) = policies_map.keys().next() { // Use first policy for now
                            if let Some(policy) = policies_map.get(policy_id) {
                                policy.should_escalate(alert.escalation_level, 1)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    };

                    if should_escalate {
                        let _ = alert_sender.send(AlertEvent::EscalateAlert(alert.id.clone())).await;
                    }
                }
            }
        });
    }

    /// Start the notification processor
    async fn start_notification_processor(&self) {
        let pending_notifications = self.pending_notifications.clone();
        let channels = self.notification_channels.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

            loop {
                interval.tick().await;

                let notifications_to_send = {
                    let mut pending = pending_notifications.write().await;
                    let (to_send, remaining): (Vec<_>, Vec<_>) = pending.drain(..)
                        .partition(|n| n.status == NotificationStatus::Pending);
                    *pending = remaining;
                    to_send
                };

                for mut notification in notifications_to_send {
                    let channels_map = channels.read().await;
                    if let Some(channel) = channels_map.get(&notification.channel_id) {
                        // Send notification
                        notification.status = NotificationStatus::Sent;
                        notification.sent_at = Some(Utc::now());

                        // For now, assume success - in real implementation, handle results
                        notification.status = NotificationStatus::Delivered;
                        notification.delivered_at = Some(Utc::now());
                    } else {
                        notification.status = NotificationStatus::Failed;
                        notification.error_message = Some("Channel not found".to_string());
                    }
                }
            }
        });
    }

    /// Start the metrics updater
    async fn start_metrics_updater(&self) {
        let metrics = self.metrics.clone();
        let alerts = self.alerts.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // Every 5 minutes

            loop {
                interval.tick().await;

                let mut m = metrics.write().await;
                let alert_map = alerts.read().await;

                // Update alert counts by severity
                let mut severity_counts = HashMap::new();
                let mut type_counts = HashMap::new();
                let mut resolution_times = Vec::new();

                for alert in alert_map.values() {
                    *severity_counts.entry(alert.severity.clone()).or_insert(0) += 1;
                    *type_counts.entry(format!("{:?}", alert.alert_type)).or_insert(0) += 1;

                    if let (Some(created), Some(resolved)) = (Some(alert.created_at), alert.resolved_at) {
                        resolution_times.push(resolved - created);
                    }
                }

                m.alerts_by_severity = severity_counts;
                m.alerts_by_type = type_counts;

                if !resolution_times.is_empty() {
                    let total_duration = resolution_times.iter().sum::<Duration>();
                    m.average_resolution_time = total_duration / resolution_times.len() as i32;
                }

                m.last_updated = Utc::now();
            }
        });
    }

    /// Start the cleanup processor
    async fn start_cleanup_processor(&self) {
        let alerts = self.alerts.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Every hour

            loop {
                interval.tick().await;

                let cutoff = Utc::now() - Duration::days(config.alert_retention_days as i64);

                let mut alert_map = alerts.write().await;
                let initial_count = alert_map.len();

                alert_map.retain(|_, alert| {
                    // Keep active alerts and recent alerts
                    alert.status.is_active() || alert.created_at > cutoff
                });

                let removed_count = initial_count - alert_map.len();
                if removed_count > 0 {
                    println!("🧹 Cleaned up {} expired alerts", removed_count);
                }
            }
        });
    }
}

impl Default for AlertEngineConfig {
    fn default() -> Self {
        Self {
            alert_processing_capacity: 1000,
            escalation_check_interval_seconds: 60,
            notification_batch_size: 10,
            metrics_update_interval_seconds: 300,
            cleanup_interval_hours: 24,
        }
    }
}

impl AlertContext {
    /// Create a new alert context
    pub fn new(source_system: String) -> Self {
        Self {
            source_system,
            test_results: None,
            metrics: HashMap::new(),
            metadata: HashMap::new(),
            environment: None,
            affected_components: vec![],
            related_entities: vec![],
        }
    }

    /// Add a metric
    pub fn add_metric(&mut self, key: String, value: f64) {
        self.metrics.insert(key, value);
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Add affected component
    pub fn add_affected_component(&mut self, component: String) {
        if !self.affected_components.contains(&component) {
            self.affected_components.push(component);
        }
    }
}