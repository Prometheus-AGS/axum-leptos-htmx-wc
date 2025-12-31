use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

pub mod engine;
pub mod channels;
pub mod rules;
pub mod api;

/// Core alerting module
pub use engine::{
    AlertEngine, AlertEngineConfig, AlertContext, AlertState,
    AlertHistory, AlertMetrics,
};

pub use channels::{
    NotificationChannel, EmailChannel, SlackChannel, WebhookChannel,
    PagerDutyChannel, DiscordChannel, TeamsChannel, NotificationResult,
    ChannelConfig, ChannelStatus,
};

pub use rules::{
    AlertRule, RuleCondition, RuleAction, RuleTrigger, RuleType,
    ThresholdRule, TrendRule, PatternRule, CompositeRule,
    RuleEvaluator, RuleContext,
};

pub use api::{
    create_alerting_api_router, AlertingApiState, AlertingApiConfig,
};

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Alert status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
    Suppressed,
    Expired,
}

/// Core alert structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub title: String,
    pub description: String,
    pub context: AlertContext,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<String>,
    pub resolution_reason: Option<String>,
    pub suppressed_until: Option<DateTime<Utc>>,
    pub notification_channels: Vec<String>,
    pub tags: HashMap<String, String>,
    pub related_alerts: Vec<String>,
    pub escalation_level: u32,
}

/// Types of alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    // Test execution alerts
    TestFailure,
    TestTimeout,
    Flakiness,
    PerformanceRegression,
    CoverageDecrease,

    // System alerts
    SystemHealth,
    ResourceExhaustion,
    ServiceUnavailable,
    InfrastructureIssue,

    // Pattern-based alerts
    AnomalyDetected,
    TrendAlert,
    ThresholdBreach,
    PatternMatch,

    // Custom alerts
    Custom(String),
}

/// Alert escalation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationPolicy {
    pub policy_id: String,
    pub name: String,
    pub description: String,
    pub escalation_levels: Vec<EscalationLevel>,
    pub repeat_interval_minutes: Option<u32>,
    pub max_escalations: Option<u32>,
    pub enabled: bool,
}

/// Individual escalation level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationLevel {
    pub level: u32,
    pub delay_minutes: u32,
    pub notification_channels: Vec<String>,
    pub required_acknowledgment: bool,
    pub auto_resolve_after_minutes: Option<u32>,
}

/// Alert suppression rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionRule {
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub conditions: Vec<SuppressionCondition>,
    pub suppression_duration_minutes: u32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

/// Suppression condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionCondition {
    pub field: String,
    pub operator: SuppressionOperator,
    pub value: String,
    pub case_sensitive: bool,
}

/// Suppression operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuppressionOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Regex,
}

/// Alert notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotification {
    pub notification_id: String,
    pub alert_id: String,
    pub channel_id: String,
    pub channel_type: String,
    pub status: NotificationStatus,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub escalation_level: u32,
}

/// Notification status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Delivered,
    Failed,
    Retrying,
    Suppressed,
}

/// Alert summary for dashboards
#[derive(Debug, Serialize)]
pub struct AlertSummary {
    pub total_alerts: usize,
    pub active_alerts: usize,
    pub critical_alerts: usize,
    pub acknowledged_alerts: usize,
    pub alerts_by_severity: HashMap<AlertSeverity, usize>,
    pub alerts_by_type: HashMap<String, usize>,
    pub recent_alerts: Vec<Alert>,
    pub top_failing_rules: Vec<RuleFailureStats>,
    pub notification_stats: NotificationStats,
    pub generated_at: DateTime<Utc>,
}

/// Rule failure statistics
#[derive(Debug, Serialize)]
pub struct RuleFailureStats {
    pub rule_id: String,
    pub rule_name: String,
    pub failure_count: usize,
    pub last_failure: DateTime<Utc>,
    pub failure_rate: f64,
}

/// Notification statistics
#[derive(Debug, Serialize)]
pub struct NotificationStats {
    pub total_notifications: usize,
    pub successful_notifications: usize,
    pub failed_notifications: usize,
    pub success_rate: f64,
    pub average_delivery_time_ms: f64,
    pub notifications_by_channel: HashMap<String, usize>,
}

/// Alert webhook payload
#[derive(Debug, Serialize)]
pub struct AlertWebhookPayload {
    pub event_type: WebhookEventType,
    pub alert: Alert,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Webhook event types
#[derive(Debug, Serialize)]
pub enum WebhookEventType {
    AlertCreated,
    AlertUpdated,
    AlertAcknowledged,
    AlertResolved,
    AlertEscalated,
    AlertSuppressed,
}

impl AlertSeverity {
    /// Get numeric priority for severity (higher = more severe)
    pub fn priority(&self) -> u8 {
        match self {
            AlertSeverity::Critical => 5,
            AlertSeverity::High => 4,
            AlertSeverity::Medium => 3,
            AlertSeverity::Low => 2,
            AlertSeverity::Info => 1,
        }
    }

    /// Get color representation for UI
    pub fn color(&self) -> &'static str {
        match self {
            AlertSeverity::Critical => "#dc2626",  // red-600
            AlertSeverity::High => "#ea580c",      // orange-600
            AlertSeverity::Medium => "#d97706",    // amber-600
            AlertSeverity::Low => "#65a30d",       // lime-600
            AlertSeverity::Info => "#2563eb",      // blue-600
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            AlertSeverity::Critical => "Critical",
            AlertSeverity::High => "High",
            AlertSeverity::Medium => "Medium",
            AlertSeverity::Low => "Low",
            AlertSeverity::Info => "Info",
        }
    }

    /// Check if this severity requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(self, AlertSeverity::Critical | AlertSeverity::High)
    }
}

impl AlertStatus {
    /// Check if the alert is active (not resolved or suppressed)
    pub fn is_active(&self) -> bool {
        matches!(self, AlertStatus::Active | AlertStatus::Acknowledged)
    }

    /// Check if the alert requires action
    pub fn requires_action(&self) -> bool {
        matches!(self, AlertStatus::Active)
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            AlertStatus::Active => "Active",
            AlertStatus::Acknowledged => "Acknowledged",
            AlertStatus::Resolved => "Resolved",
            AlertStatus::Suppressed => "Suppressed",
            AlertStatus::Expired => "Expired",
        }
    }
}

impl Alert {
    /// Create a new alert
    pub fn new(
        rule_id: String,
        alert_type: AlertType,
        severity: AlertSeverity,
        title: String,
        description: String,
        context: AlertContext,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: format!("alert_{}", uuid::Uuid::new_v4()),
            rule_id,
            alert_type,
            severity,
            status: AlertStatus::Active,
            title,
            description,
            context,
            created_at: now,
            updated_at: now,
            acknowledged_at: None,
            resolved_at: None,
            acknowledged_by: None,
            resolution_reason: None,
            suppressed_until: None,
            notification_channels: vec![],
            tags: HashMap::new(),
            related_alerts: vec![],
            escalation_level: 0,
        }
    }

    /// Acknowledge the alert
    pub fn acknowledge(&mut self, acknowledged_by: String) {
        self.status = AlertStatus::Acknowledged;
        self.acknowledged_at = Some(Utc::now());
        self.acknowledged_by = Some(acknowledged_by);
        self.updated_at = Utc::now();
    }

    /// Resolve the alert
    pub fn resolve(&mut self, resolution_reason: Option<String>) {
        self.status = AlertStatus::Resolved;
        self.resolved_at = Some(Utc::now());
        self.resolution_reason = resolution_reason;
        self.updated_at = Utc::now();
    }

    /// Suppress the alert
    pub fn suppress(&mut self, duration_minutes: u32) {
        self.status = AlertStatus::Suppressed;
        self.suppressed_until = Some(Utc::now() + Duration::minutes(duration_minutes as i64));
        self.updated_at = Utc::now();
    }

    /// Check if the alert is expired
    pub fn is_expired(&self, expiry_duration: Duration) -> bool {
        Utc::now() > self.created_at + expiry_duration
    }

    /// Check if suppression has expired
    pub fn is_suppression_expired(&self) -> bool {
        match self.suppressed_until {
            Some(until) => Utc::now() > until,
            None => true,
        }
    }

    /// Get age of the alert
    pub fn age(&self) -> Duration {
        Utc::now() - self.created_at
    }

    /// Add a tag
    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// Remove a tag
    pub fn remove_tag(&mut self, key: &str) {
        self.tags.remove(key);
        self.updated_at = Utc::now();
    }

    /// Check if alert matches tag criteria
    pub fn matches_tags(&self, required_tags: &HashMap<String, String>) -> bool {
        required_tags.iter().all(|(key, value)| {
            self.tags.get(key).map_or(false, |v| v == value)
        })
    }
}

impl EscalationPolicy {
    /// Create a new escalation policy
    pub fn new(name: String, description: String) -> Self {
        Self {
            policy_id: format!("policy_{}", uuid::Uuid::new_v4()),
            name,
            description,
            escalation_levels: vec![],
            repeat_interval_minutes: None,
            max_escalations: None,
            enabled: true,
        }
    }

    /// Add an escalation level
    pub fn add_level(&mut self, level: EscalationLevel) {
        self.escalation_levels.push(level);
        // Sort levels by level number
        self.escalation_levels.sort_by_key(|l| l.level);
    }

    /// Get the escalation level for a given level number
    pub fn get_level(&self, level: u32) -> Option<&EscalationLevel> {
        self.escalation_levels.iter().find(|l| l.level == level)
    }

    /// Get the next escalation level
    pub fn get_next_level(&self, current_level: u32) -> Option<&EscalationLevel> {
        self.escalation_levels.iter()
            .find(|l| l.level > current_level)
    }

    /// Check if escalation should continue
    pub fn should_escalate(&self, current_level: u32, escalation_count: u32) -> bool {
        if !self.enabled {
            return false;
        }

        if let Some(max) = self.max_escalations {
            if escalation_count >= max {
                return false;
            }
        }

        self.get_next_level(current_level).is_some()
    }
}

impl SuppressionRule {
    /// Create a new suppression rule
    pub fn new(
        name: String,
        description: String,
        created_by: String,
    ) -> Self {
        Self {
            rule_id: format!("suppression_{}", uuid::Uuid::new_v4()),
            name,
            description,
            conditions: vec![],
            suppression_duration_minutes: 60, // Default 1 hour
            enabled: true,
            created_at: Utc::now(),
            created_by,
        }
    }

    /// Add a condition to the suppression rule
    pub fn add_condition(&mut self, condition: SuppressionCondition) {
        self.conditions.push(condition);
    }

    /// Check if an alert matches this suppression rule
    pub fn matches_alert(&self, alert: &Alert) -> bool {
        if !self.enabled {
            return false;
        }

        // All conditions must match
        self.conditions.iter().all(|condition| {
            self.evaluate_condition(condition, alert)
        })
    }

    /// Evaluate a single condition against an alert
    fn evaluate_condition(&self, condition: &SuppressionCondition, alert: &Alert) -> bool {
        let field_value = match condition.field.as_str() {
            "title" => &alert.title,
            "description" => &alert.description,
            "alert_type" => &format!("{:?}", alert.alert_type),
            "severity" => &format!("{:?}", alert.severity),
            "rule_id" => &alert.rule_id,
            _ => {
                // Check tags
                if let Some(tag_value) = alert.tags.get(&condition.field) {
                    tag_value
                } else {
                    return false;
                }
            }
        };

        let target_value = if condition.case_sensitive {
            condition.value.as_str()
        } else {
            &condition.value.to_lowercase()
        };

        let compare_value = if condition.case_sensitive {
            field_value.clone()
        } else {
            field_value.to_lowercase()
        };

        match condition.operator {
            SuppressionOperator::Equals => compare_value == target_value,
            SuppressionOperator::NotEquals => compare_value != target_value,
            SuppressionOperator::Contains => compare_value.contains(target_value),
            SuppressionOperator::NotContains => !compare_value.contains(target_value),
            SuppressionOperator::StartsWith => compare_value.starts_with(target_value),
            SuppressionOperator::EndsWith => compare_value.ends_with(target_value),
            SuppressionOperator::Regex => {
                if let Ok(regex) = regex::Regex::new(&condition.value) {
                    regex.is_match(field_value)
                } else {
                    false
                }
            }
        }
    }
}

/// Configuration for alerting system
#[derive(Debug, Clone)]
pub struct AlertingConfig {
    pub enabled: bool,
    pub default_escalation_policy: Option<String>,
    pub alert_retention_days: u32,
    pub notification_retry_count: u32,
    pub notification_retry_delay_seconds: u32,
    pub batch_notification_size: usize,
    pub rate_limit_per_minute: Option<u32>,
    pub enable_webhook_notifications: bool,
    pub webhook_timeout_seconds: u32,
}

impl Default for AlertingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_escalation_policy: None,
            alert_retention_days: 90,
            notification_retry_count: 3,
            notification_retry_delay_seconds: 30,
            batch_notification_size: 10,
            rate_limit_per_minute: Some(100),
            enable_webhook_notifications: true,
            webhook_timeout_seconds: 30,
        }
    }
}