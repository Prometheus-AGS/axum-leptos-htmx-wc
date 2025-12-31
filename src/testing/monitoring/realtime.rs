// Real-time test progress tracking implementation
// T039: Implement real-time test progress tracking

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{broadcast, mpsc},
    time::{interval, Interval},
};
use tracing::{error, info, warn, debug};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{TestExecutionMetrics, TestExecutionMode, PhaseMetrics};

/// Real-time test progress tracker
pub struct RealTimeTracker {
    /// Active test runs being tracked
    active_runs: Arc<RwLock<HashMap<String, ActiveTestRun>>>,
    /// Broadcast channel for real-time updates
    update_sender: broadcast::Sender<TestProgressUpdate>,
    /// Internal channel for receiving progress updates
    progress_receiver: mpsc::Receiver<TestProgressEvent>,
    /// Internal sender for progress events
    progress_sender: mpsc::Sender<TestProgressEvent>,
    /// Ticker for periodic status updates
    status_ticker: Interval,
}

/// Represents an active test run being tracked
#[derive(Debug, Clone, Serialize)]
pub struct ActiveTestRun {
    pub run_id: String,
    pub environment: String,
    pub mode: TestExecutionMode,
    pub start_time: SystemTime,
    pub current_phase: String,
    pub phases: HashMap<String, PhaseStatus>,
    pub overall_progress: f64,
    pub estimated_completion: Option<SystemTime>,
    pub status: TestRunStatus,
    pub live_metrics: LiveMetrics,
}

/// Status of an individual test phase
#[derive(Debug, Clone, Serialize)]
pub struct PhaseStatus {
    pub phase_name: String,
    pub status: PhaseExecutionStatus,
    pub start_time: Option<SystemTime>,
    pub end_time: Option<SystemTime>,
    pub progress_percentage: f64,
    pub current_test: Option<String>,
    pub tests_completed: usize,
    pub tests_total: usize,
    pub failures: usize,
    pub duration: Option<Duration>,
}

/// Real-time metrics for active test runs
#[derive(Debug, Clone, Serialize)]
pub struct LiveMetrics {
    pub tests_completed: usize,
    pub tests_failed: usize,
    pub tests_passed: usize,
    pub tests_skipped: usize,
    pub current_phase_duration: Duration,
    pub total_duration: Duration,
    pub average_test_duration: Duration,
    pub coverage_percentage: Option<f64>,
    pub memory_usage: Option<u64>,
    pub cpu_usage: Option<f64>,
}

/// Test run execution status
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum TestRunStatus {
    Starting,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Individual phase execution status
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum PhaseExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

/// Progress update events sent to subscribers
#[derive(Debug, Clone, Serialize)]
pub struct TestProgressUpdate {
    pub update_id: String,
    pub timestamp: DateTime<Utc>,
    pub run_id: String,
    pub update_type: ProgressUpdateType,
    pub data: serde_json::Value,
}

/// Types of progress updates
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ProgressUpdateType {
    RunStarted,
    RunCompleted,
    RunFailed,
    PhaseStarted { phase: String },
    PhaseCompleted { phase: String },
    PhaseProgress { phase: String, progress: f64 },
    TestStarted { test_name: String, phase: String },
    TestCompleted { test_name: String, phase: String, success: bool },
    MetricsUpdate { metrics: LiveMetrics },
    StatusChange { old_status: TestRunStatus, new_status: TestRunStatus },
    ErrorOccurred { error_message: String, phase: Option<String> },
}

/// Internal events for progress tracking
#[derive(Debug, Clone)]
pub enum TestProgressEvent {
    StartRun {
        run_id: String,
        environment: String,
        mode: TestExecutionMode,
        phases: Vec<String>,
    },
    UpdatePhase {
        run_id: String,
        phase: String,
        status: PhaseExecutionStatus,
        progress: Option<f64>,
        current_test: Option<String>,
    },
    UpdateMetrics {
        run_id: String,
        metrics: LiveMetrics,
    },
    UpdateStatus {
        run_id: String,
        status: TestRunStatus,
    },
    CompleteRun {
        run_id: String,
        final_metrics: TestExecutionMetrics,
    },
    FailRun {
        run_id: String,
        error_message: String,
    },
}

/// Configuration for real-time tracking
pub struct RealTimeConfig {
    pub update_interval_ms: u64,
    pub max_active_runs: usize,
    pub enable_metrics_collection: bool,
    pub enable_resource_monitoring: bool,
}

impl Default for RealTimeConfig {
    fn default() -> Self {
        Self {
            update_interval_ms: 1000, // 1 second updates
            max_active_runs: 100,
            enable_metrics_collection: true,
            enable_resource_monitoring: true,
        }
    }
}

impl RealTimeTracker {
    /// Create a new real-time tracker
    pub fn new(config: RealTimeConfig) -> Self {
        let active_runs = Arc::new(RwLock::new(HashMap::new()));
        let (update_sender, _) = broadcast::channel(1000);
        let (progress_sender, progress_receiver) = mpsc::channel(1000);
        let status_ticker = interval(Duration::from_millis(config.update_interval_ms));

        Self {
            active_runs,
            update_sender,
            progress_receiver,
            progress_sender,
            status_ticker,
        }
    }

    /// Get a sender for progress events
    pub fn get_progress_sender(&self) -> mpsc::Sender<TestProgressEvent> {
        self.progress_sender.clone()
    }

    /// Subscribe to real-time updates
    pub fn subscribe(&self) -> broadcast::Receiver<TestProgressUpdate> {
        self.update_sender.subscribe()
    }

    /// Start the real-time tracking service
    pub async fn run(&mut self) {
        info!("Starting real-time test progress tracking service");

        loop {
            tokio::select! {
                // Handle progress events
                event = self.progress_receiver.recv() => {
                    if let Some(event) = event {
                        self.handle_progress_event(event).await;
                    }
                }

                // Periodic status updates
                _ = self.status_ticker.tick() => {
                    self.send_periodic_updates().await;
                }
            }
        }
    }

    /// Handle incoming progress events
    async fn handle_progress_event(&mut self, event: TestProgressEvent) {
        debug!("Handling progress event: {:?}", event);

        match event {
            TestProgressEvent::StartRun { run_id, environment, mode, phases } => {
                self.start_test_run(run_id, environment, mode, phases).await;
            }
            TestProgressEvent::UpdatePhase { run_id, phase, status, progress, current_test } => {
                self.update_phase_status(run_id, phase, status, progress, current_test).await;
            }
            TestProgressEvent::UpdateMetrics { run_id, metrics } => {
                self.update_live_metrics(run_id, metrics).await;
            }
            TestProgressEvent::UpdateStatus { run_id, status } => {
                self.update_run_status(run_id, status).await;
            }
            TestProgressEvent::CompleteRun { run_id, final_metrics } => {
                self.complete_test_run(run_id, final_metrics).await;
            }
            TestProgressEvent::FailRun { run_id, error_message } => {
                self.fail_test_run(run_id, error_message).await;
            }
        }
    }

    /// Start tracking a new test run
    async fn start_test_run(
        &self,
        run_id: String,
        environment: String,
        mode: TestExecutionMode,
        phases: Vec<String>,
    ) {
        let mut active_runs = self.active_runs.write().unwrap();

        // Create phase statuses
        let phase_statuses: HashMap<String, PhaseStatus> = phases
            .iter()
            .enumerate()
            .map(|(i, phase)| {
                let status = if i == 0 { PhaseExecutionStatus::Running } else { PhaseExecutionStatus::Pending };
                (
                    phase.clone(),
                    PhaseStatus {
                        phase_name: phase.clone(),
                        status,
                        start_time: if i == 0 { Some(SystemTime::now()) } else { None },
                        end_time: None,
                        progress_percentage: 0.0,
                        current_test: None,
                        tests_completed: 0,
                        tests_total: 0,
                        failures: 0,
                        duration: None,
                    },
                )
            })
            .collect();

        let active_run = ActiveTestRun {
            run_id: run_id.clone(),
            environment: environment.clone(),
            mode,
            start_time: SystemTime::now(),
            current_phase: phases.first().unwrap_or(&"unknown".to_string()).clone(),
            phases: phase_statuses,
            overall_progress: 0.0,
            estimated_completion: None,
            status: TestRunStatus::Running,
            live_metrics: LiveMetrics::default(),
        };

        active_runs.insert(run_id.clone(), active_run);

        // Send progress update
        let update = TestProgressUpdate {
            update_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            run_id: run_id.clone(),
            update_type: ProgressUpdateType::RunStarted,
            data: serde_json::json!({
                "environment": environment,
                "mode": format!("{:?}", mode),
                "phases": phases
            }),
        };

        if let Err(e) = self.update_sender.send(update) {
            warn!("Failed to send progress update: {}", e);
        }

        info!("Started tracking test run: {} in environment: {}", run_id, environment);
    }

    /// Update phase status
    async fn update_phase_status(
        &self,
        run_id: String,
        phase: String,
        status: PhaseExecutionStatus,
        progress: Option<f64>,
        current_test: Option<String>,
    ) {
        let mut active_runs = self.active_runs.write().unwrap();

        if let Some(run) = active_runs.get_mut(&run_id) {
            if let Some(phase_status) = run.phases.get_mut(&phase) {
                let old_status = phase_status.status.clone();
                phase_status.status = status.clone();

                if let Some(prog) = progress {
                    phase_status.progress_percentage = prog;
                }

                if let Some(test) = current_test {
                    phase_status.current_test = Some(test.clone());
                }

                // Update timestamps based on status changes
                match status {
                    PhaseExecutionStatus::Running if old_status == PhaseExecutionStatus::Pending => {
                        phase_status.start_time = Some(SystemTime::now());
                        run.current_phase = phase.clone();
                    }
                    PhaseExecutionStatus::Completed | PhaseExecutionStatus::Failed => {
                        phase_status.end_time = Some(SystemTime::now());
                        if let Some(start_time) = phase_status.start_time {
                            phase_status.duration = SystemTime::now().duration_since(start_time).ok();
                        }
                    }
                    _ => {}
                }

                // Calculate overall progress
                run.overall_progress = self.calculate_overall_progress(&run.phases);

                // Send progress update
                let update = TestProgressUpdate {
                    update_id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    run_id: run_id.clone(),
                    update_type: match status {
                        PhaseExecutionStatus::Running => ProgressUpdateType::PhaseStarted { phase: phase.clone() },
                        PhaseExecutionStatus::Completed => ProgressUpdateType::PhaseCompleted { phase: phase.clone() },
                        _ => ProgressUpdateType::PhaseProgress { phase: phase.clone(), progress: progress.unwrap_or(0.0) },
                    },
                    data: serde_json::json!({
                        "phase": phase,
                        "status": format!("{:?}", status),
                        "progress": progress,
                        "current_test": current_test,
                        "overall_progress": run.overall_progress
                    }),
                };

                if let Err(e) = self.update_sender.send(update) {
                    warn!("Failed to send phase update: {}", e);
                }
            }
        }
    }

    /// Update live metrics
    async fn update_live_metrics(&self, run_id: String, metrics: LiveMetrics) {
        let mut active_runs = self.active_runs.write().unwrap();

        if let Some(run) = active_runs.get_mut(&run_id) {
            run.live_metrics = metrics.clone();

            let update = TestProgressUpdate {
                update_id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                run_id: run_id.clone(),
                update_type: ProgressUpdateType::MetricsUpdate { metrics },
                data: serde_json::json!({}),
            };

            if let Err(e) = self.update_sender.send(update) {
                warn!("Failed to send metrics update: {}", e);
            }
        }
    }

    /// Update run status
    async fn update_run_status(&self, run_id: String, new_status: TestRunStatus) {
        let mut active_runs = self.active_runs.write().unwrap();

        if let Some(run) = active_runs.get_mut(&run_id) {
            let old_status = run.status.clone();
            run.status = new_status.clone();

            let update = TestProgressUpdate {
                update_id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                run_id: run_id.clone(),
                update_type: ProgressUpdateType::StatusChange { old_status, new_status },
                data: serde_json::json!({}),
            };

            if let Err(e) = self.update_sender.send(update) {
                warn!("Failed to send status update: {}", e);
            }
        }
    }

    /// Complete a test run
    async fn complete_test_run(&self, run_id: String, final_metrics: TestExecutionMetrics) {
        let mut active_runs = self.active_runs.write().unwrap();

        if let Some(mut run) = active_runs.remove(&run_id) {
            run.status = TestRunStatus::Completed;

            let update = TestProgressUpdate {
                update_id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                run_id: run_id.clone(),
                update_type: ProgressUpdateType::RunCompleted,
                data: serde_json::json!({
                    "final_metrics": final_metrics,
                    "duration": run.start_time.elapsed().unwrap_or(Duration::from_secs(0)).as_secs()
                }),
            };

            if let Err(e) = self.update_sender.send(update) {
                warn!("Failed to send completion update: {}", e);
            }

            info!("Completed tracking test run: {}", run_id);
        }
    }

    /// Mark a test run as failed
    async fn fail_test_run(&self, run_id: String, error_message: String) {
        let mut active_runs = self.active_runs.write().unwrap();

        if let Some(mut run) = active_runs.remove(&run_id) {
            run.status = TestRunStatus::Failed;

            let update = TestProgressUpdate {
                update_id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                run_id: run_id.clone(),
                update_type: ProgressUpdateType::RunFailed,
                data: serde_json::json!({
                    "error_message": error_message,
                    "duration": run.start_time.elapsed().unwrap_or(Duration::from_secs(0)).as_secs()
                }),
            };

            if let Err(e) = self.update_sender.send(update) {
                warn!("Failed to send failure update: {}", e);
            }

            error!("Test run failed: {} - {}", run_id, error_message);
        }
    }

    /// Send periodic updates for all active runs
    async fn send_periodic_updates(&self) {
        let active_runs = self.active_runs.read().unwrap();

        for (run_id, run) in active_runs.iter() {
            if run.status == TestRunStatus::Running {
                let update = TestProgressUpdate {
                    update_id: Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    run_id: run_id.clone(),
                    update_type: ProgressUpdateType::MetricsUpdate {
                        metrics: run.live_metrics.clone(),
                    },
                    data: serde_json::json!({
                        "overall_progress": run.overall_progress,
                        "current_phase": run.current_phase,
                        "elapsed_time": run.start_time.elapsed().unwrap_or(Duration::from_secs(0)).as_secs()
                    }),
                };

                if let Err(e) = self.update_sender.send(update) {
                    warn!("Failed to send periodic update for run {}: {}", run_id, e);
                }
            }
        }
    }

    /// Calculate overall progress across all phases
    fn calculate_overall_progress(&self, phases: &HashMap<String, PhaseStatus>) -> f64 {
        if phases.is_empty() {
            return 0.0;
        }

        let total_progress: f64 = phases
            .values()
            .map(|phase| match phase.status {
                PhaseExecutionStatus::Completed => 100.0,
                PhaseExecutionStatus::Failed => 100.0, // Consider failed as completed for progress calculation
                PhaseExecutionStatus::Running => phase.progress_percentage,
                _ => 0.0,
            })
            .sum();

        total_progress / phases.len() as f64
    }

    /// Get current status of all active runs
    pub fn get_active_runs(&self) -> Vec<ActiveTestRun> {
        let active_runs = self.active_runs.read().unwrap();
        active_runs.values().cloned().collect()
    }

    /// Get status of a specific run
    pub fn get_run_status(&self, run_id: &str) -> Option<ActiveTestRun> {
        let active_runs = self.active_runs.read().unwrap();
        active_runs.get(run_id).cloned()
    }

    /// Get count of active runs by status
    pub fn get_run_counts(&self) -> HashMap<TestRunStatus, usize> {
        let active_runs = self.active_runs.read().unwrap();
        let mut counts = HashMap::new();

        for run in active_runs.values() {
            *counts.entry(run.status.clone()).or_insert(0) += 1;
        }

        counts
    }
}

impl Default for LiveMetrics {
    fn default() -> Self {
        Self {
            tests_completed: 0,
            tests_failed: 0,
            tests_passed: 0,
            tests_skipped: 0,
            current_phase_duration: Duration::from_secs(0),
            total_duration: Duration::from_secs(0),
            average_test_duration: Duration::from_secs(0),
            coverage_percentage: None,
            memory_usage: None,
            cpu_usage: None,
        }
    }
}

/// Helper function to create a test progress event sender
pub fn create_progress_reporter() -> (mpsc::Sender<TestProgressEvent>, mpsc::Receiver<TestProgressEvent>) {
    mpsc::channel(1000)
}

/// Convenience struct for reporting test progress
pub struct ProgressReporter {
    sender: mpsc::Sender<TestProgressEvent>,
    run_id: String,
}

impl ProgressReporter {
    pub fn new(sender: mpsc::Sender<TestProgressEvent>, run_id: String) -> Self {
        Self { sender, run_id }
    }

    /// Report test run start
    pub async fn start_run(&self, environment: String, mode: TestExecutionMode, phases: Vec<String>) -> Result<(), mpsc::error::SendError<TestProgressEvent>> {
        let event = TestProgressEvent::StartRun {
            run_id: self.run_id.clone(),
            environment,
            mode,
            phases,
        };
        self.sender.send(event).await
    }

    /// Report phase update
    pub async fn update_phase(
        &self,
        phase: String,
        status: PhaseExecutionStatus,
        progress: Option<f64>,
        current_test: Option<String>,
    ) -> Result<(), mpsc::error::SendError<TestProgressEvent>> {
        let event = TestProgressEvent::UpdatePhase {
            run_id: self.run_id.clone(),
            phase,
            status,
            progress,
            current_test,
        };
        self.sender.send(event).await
    }

    /// Report metrics update
    pub async fn update_metrics(&self, metrics: LiveMetrics) -> Result<(), mpsc::error::SendError<TestProgressEvent>> {
        let event = TestProgressEvent::UpdateMetrics {
            run_id: self.run_id.clone(),
            metrics,
        };
        self.sender.send(event).await
    }

    /// Report run completion
    pub async fn complete_run(&self, final_metrics: TestExecutionMetrics) -> Result<(), mpsc::error::SendError<TestProgressEvent>> {
        let event = TestProgressEvent::CompleteRun {
            run_id: self.run_id.clone(),
            final_metrics,
        };
        self.sender.send(event).await
    }

    /// Report run failure
    pub async fn fail_run(&self, error_message: String) -> Result<(), mpsc::error::SendError<TestProgressEvent>> {
        let event = TestProgressEvent::FailRun {
            run_id: self.run_id.clone(),
            error_message,
        };
        self.sender.send(event).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_real_time_tracker_creation() {
        let config = RealTimeConfig::default();
        let tracker = RealTimeTracker::new(config);

        // Test basic functionality
        assert_eq!(tracker.get_active_runs().len(), 0);
    }

    #[tokio::test]
    async fn test_progress_reporter() {
        let (sender, mut receiver) = create_progress_reporter();
        let reporter = ProgressReporter::new(sender, "test_run_123".to_string());

        // Test sending events
        let phases = vec!["setup".to_string(), "test".to_string(), "teardown".to_string()];
        reporter.start_run("test_env".to_string(), TestExecutionMode::Full, phases).await.unwrap();

        // Verify event received
        let event = receiver.recv().await.unwrap();
        match event {
            TestProgressEvent::StartRun { run_id, environment, .. } => {
                assert_eq!(run_id, "test_run_123");
                assert_eq!(environment, "test_env");
            }
            _ => panic!("Expected StartRun event"),
        }
    }

    #[tokio::test]
    async fn test_phase_progress_calculation() {
        let mut phases = HashMap::new();

        phases.insert("phase1".to_string(), PhaseStatus {
            phase_name: "phase1".to_string(),
            status: PhaseExecutionStatus::Completed,
            start_time: Some(SystemTime::now()),
            end_time: Some(SystemTime::now()),
            progress_percentage: 100.0,
            current_test: None,
            tests_completed: 10,
            tests_total: 10,
            failures: 0,
            duration: Some(Duration::from_secs(30)),
        });

        phases.insert("phase2".to_string(), PhaseStatus {
            phase_name: "phase2".to_string(),
            status: PhaseExecutionStatus::Running,
            start_time: Some(SystemTime::now()),
            end_time: None,
            progress_percentage: 50.0,
            current_test: Some("test_5".to_string()),
            tests_completed: 5,
            tests_total: 10,
            failures: 1,
            duration: None,
        });

        let config = RealTimeConfig::default();
        let tracker = RealTimeTracker::new(config);

        let progress = tracker.calculate_overall_progress(&phases);
        assert_eq!(progress, 75.0); // (100 + 50) / 2
    }
}