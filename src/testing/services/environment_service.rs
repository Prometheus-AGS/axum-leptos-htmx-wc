use crate::testing::entities::test_environment::*;
use serde_json::Value;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Service for managing test environments with Docker orchestration
#[derive(Debug, Clone)]
pub struct EnvironmentService {
    environments: Arc<RwLock<HashMap<Uuid, TestEnvironment>>>,
    events: Arc<RwLock<Vec<EnvironmentEvent>>>,
}

/// Result type for environment operations
pub type EnvironmentResult<T> = Result<T, EnvironmentError>;

/// Errors that can occur during environment management
#[derive(Debug, thiserror::Error)]
pub enum EnvironmentError {
    #[error("Environment not found: {id}")]
    NotFound { id: Uuid },

    #[error("Environment validation failed: {reason}")]
    ValidationError { reason: String },

    #[error("Docker operation failed: {command} - {error}")]
    DockerError { command: String, error: String },

    #[error("Health check failed for service '{service}': {reason}")]
    HealthCheckFailed { service: String, reason: String },

    #[error("Environment timeout: operation took longer than {timeout_secs} seconds")]
    Timeout { timeout_secs: u64 },

    #[error("Environment in invalid state: expected {expected}, got {actual}")]
    InvalidState { expected: String, actual: String },

    #[error("Resource limit exceeded: {resource} - {current}/{limit}")]
    ResourceLimitExceeded { resource: String, current: String, limit: String },
}

/// Status of environment health check
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub service: String,
    pub healthy: bool,
    pub response_time: Duration,
    pub message: String,
    pub last_checked: SystemTime,
}

/// Environment metrics for monitoring
#[derive(Debug, Clone)]
pub struct EnvironmentMetrics {
    pub environment_id: Uuid,
    pub uptime: Duration,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub service_count: usize,
    pub healthy_services: usize,
    pub failed_services: usize,
}

impl EnvironmentService {
    /// Create a new environment service
    pub fn new() -> Self {
        Self {
            environments: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a new test environment
    pub async fn create_environment(&self, name: String, environment_type: EnvironmentType) -> EnvironmentResult<Uuid> {
        let mut environment = TestEnvironment::new(name.clone(), environment_type);

        // Set up default services based on type
        match environment.environment_type {
            EnvironmentType::Docker => {
                self.setup_default_docker_services(&mut environment);
            }
            EnvironmentType::Local => {
                self.setup_local_services(&mut environment);
            }
            EnvironmentType::Cloud => {
                return Err(EnvironmentError::ValidationError {
                    reason: "Cloud environments not yet supported".to_string(),
                });
            }
        }

        // Validate configuration
        environment.validate().map_err(|reason| EnvironmentError::ValidationError { reason })?;

        let environment_id = environment.id;

        // Log creation event
        self.log_event(EnvironmentEvent {
            environment_id,
            event_type: EnvironmentEventType::Creating,
            timestamp: SystemTime::now(),
            message: format!("Creating environment '{}'", name),
            details: Some(serde_json::json!({
                "type": environment.environment_type,
                "services": environment.services.len()
            })),
        }).await;

        // Store environment
        {
            let mut environments = self.environments.write().unwrap();
            environments.insert(environment_id, environment);
        }

        // Start environment creation process
        self.start_environment_creation(environment_id).await?;

        Ok(environment_id)
    }

    /// Start the environment creation process
    async fn start_environment_creation(&self, environment_id: Uuid) -> EnvironmentResult<()> {
        let environment = {
            let environments = self.environments.read().unwrap();
            environments.get(&environment_id).cloned()
                .ok_or(EnvironmentError::NotFound { id: environment_id })?
        };

        match environment.environment_type {
            EnvironmentType::Docker => {
                self.create_docker_environment(environment_id).await?;
            }
            EnvironmentType::Local => {
                self.create_local_environment(environment_id).await?;
            }
            _ => {
                return Err(EnvironmentError::ValidationError {
                    reason: "Unsupported environment type".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Create Docker-based test environment
    async fn create_docker_environment(&self, environment_id: Uuid) -> EnvironmentResult<()> {
        info!("Creating Docker environment: {}", environment_id);

        // Get environment configuration
        let config = {
            let environments = self.environments.read().unwrap();
            let environment = environments.get(&environment_id)
                .ok_or(EnvironmentError::NotFound { id: environment_id })?;
            environment.config.clone()
        };

        // Stop any existing containers
        self.cleanup_docker_environment(&config).await?;

        // Start Docker Compose
        let compose_result = self.run_docker_compose_up(&config).await;

        match compose_result {
            Ok(_) => {
                self.log_event(EnvironmentEvent {
                    environment_id,
                    event_type: EnvironmentEventType::ServiceStarting,
                    timestamp: SystemTime::now(),
                    message: "Docker Compose services starting".to_string(),
                    details: None,
                }).await;

                // Wait for services to become ready
                self.wait_for_services_ready(environment_id).await?;

                // Update environment status
                self.update_environment_status(environment_id, EnvironmentStatus::Ready).await?;

                info!("Docker environment ready: {}", environment_id);
                Ok(())
            }
            Err(e) => {
                self.update_environment_status(environment_id, EnvironmentStatus::Failed).await?;
                error!("Failed to create Docker environment {}: {}", environment_id, e);
                Err(e)
            }
        }
    }

    /// Run Docker Compose up command
    async fn run_docker_compose_up(&self, config: &EnvironmentConfig) -> EnvironmentResult<()> {
        let output = Command::new("docker-compose")
            .args([
                "-f", &config.docker_compose_file,
                "up", "-d", "--wait", "--build"
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| EnvironmentError::DockerError {
                command: "docker-compose up".to_string(),
                error: e.to_string(),
            })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(EnvironmentError::DockerError {
                command: "docker-compose up".to_string(),
                error: error_msg.to_string(),
            });
        }

        debug!("Docker Compose up completed successfully");
        Ok(())
    }

    /// Wait for all services to become ready
    async fn wait_for_services_ready(&self, environment_id: Uuid) -> EnvironmentResult<()> {
        let (services, timeout) = {
            let environments = self.environments.read().unwrap();
            let environment = environments.get(&environment_id)
                .ok_or(EnvironmentError::NotFound { id: environment_id })?;
            (environment.services.clone(), environment.config.timeout)
        };

        let start_time = Instant::now();
        let mut all_services_ready = false;

        while !all_services_ready && start_time.elapsed() < timeout {
            all_services_ready = true;

            for service in &services {
                let health_result = self.check_service_health(environment_id, &service.name).await;

                match health_result {
                    Ok(result) if result.healthy => {
                        self.log_event(EnvironmentEvent {
                            environment_id,
                            event_type: EnvironmentEventType::ServiceReady,
                            timestamp: SystemTime::now(),
                            message: format!("Service '{}' is ready", service.name),
                            details: Some(serde_json::json!({
                                "response_time_ms": result.response_time.as_millis()
                            })),
                        }).await;
                    }
                    Ok(_) | Err(_) => {
                        all_services_ready = false;
                        debug!("Service '{}' not ready yet", service.name);
                    }
                }
            }

            if !all_services_ready {
                sleep(Duration::from_secs(2)).await;
            }
        }

        if !all_services_ready {
            return Err(EnvironmentError::Timeout {
                timeout_secs: timeout.as_secs(),
            });
        }

        self.log_event(EnvironmentEvent {
            environment_id,
            event_type: EnvironmentEventType::Ready,
            timestamp: SystemTime::now(),
            message: "All services are ready".to_string(),
            details: Some(serde_json::json!({
                "startup_time_ms": start_time.elapsed().as_millis()
            })),
        }).await;

        Ok(())
    }

    /// Check health of a specific service
    async fn check_service_health(&self, environment_id: Uuid, service_name: &str) -> EnvironmentResult<HealthCheckResult> {
        let health_checks = {
            let environments = self.environments.read().unwrap();
            let environment = environments.get(&environment_id)
                .ok_or(EnvironmentError::NotFound { id: environment_id })?;
            environment.health_checks.clone()
        };

        let health_check = health_checks.iter()
            .find(|hc| hc.service == service_name)
            .ok_or_else(|| EnvironmentError::HealthCheckFailed {
                service: service_name.to_string(),
                reason: "No health check configured".to_string(),
            })?;

        let start_time = Instant::now();

        // Perform HTTP health check
        let client = reqwest::Client::new();
        let url = format!("http://localhost{}", health_check.endpoint);

        match client.get(&url)
            .timeout(Duration::from_millis(health_check.timeout_ms))
            .send()
            .await
        {
            Ok(response) => {
                let healthy = response.status().as_u16() == health_check.expected_status;
                Ok(HealthCheckResult {
                    service: service_name.to_string(),
                    healthy,
                    response_time: start_time.elapsed(),
                    message: if healthy {
                        "Service healthy".to_string()
                    } else {
                        format!("Unexpected status: {}", response.status())
                    },
                    last_checked: SystemTime::now(),
                })
            }
            Err(e) => {
                Ok(HealthCheckResult {
                    service: service_name.to_string(),
                    healthy: false,
                    response_time: start_time.elapsed(),
                    message: format!("Health check failed: {}", e),
                    last_checked: SystemTime::now(),
                })
            }
        }
    }

    /// Create local test environment
    async fn create_local_environment(&self, environment_id: Uuid) -> EnvironmentResult<()> {
        info!("Creating local environment: {}", environment_id);

        // For local environments, we assume services are already running
        // This is mainly for development scenarios

        self.update_environment_status(environment_id, EnvironmentStatus::Ready).await?;

        self.log_event(EnvironmentEvent {
            environment_id,
            event_type: EnvironmentEventType::Ready,
            timestamp: SystemTime::now(),
            message: "Local environment ready".to_string(),
            details: None,
        }).await;

        Ok(())
    }

    /// Destroy a test environment
    pub async fn destroy_environment(&self, environment_id: Uuid) -> EnvironmentResult<()> {
        info!("Destroying environment: {}", environment_id);

        // Update status to destroying
        self.update_environment_status(environment_id, EnvironmentStatus::Destroying).await?;

        let environment_type = {
            let environments = self.environments.read().unwrap();
            let environment = environments.get(&environment_id)
                .ok_or(EnvironmentError::NotFound { id: environment_id })?;
            environment.environment_type.clone()
        };

        match environment_type {
            EnvironmentType::Docker => {
                self.destroy_docker_environment(environment_id).await?;
            }
            EnvironmentType::Local => {
                // Local environments don't need cleanup
                debug!("Local environment cleanup not required");
            }
            EnvironmentType::Cloud => {
                return Err(EnvironmentError::ValidationError {
                    reason: "Cloud environment cleanup not implemented".to_string(),
                });
            }
        }

        // Update status to destroyed
        self.update_environment_status(environment_id, EnvironmentStatus::Destroyed).await?;

        self.log_event(EnvironmentEvent {
            environment_id,
            event_type: EnvironmentEventType::Destroyed,
            timestamp: SystemTime::now(),
            message: "Environment destroyed".to_string(),
            details: None,
        }).await;

        // Remove from active environments
        {
            let mut environments = self.environments.write().unwrap();
            environments.remove(&environment_id);
        }

        Ok(())
    }

    /// Destroy Docker environment
    async fn destroy_docker_environment(&self, environment_id: Uuid) -> EnvironmentResult<()> {
        let config = {
            let environments = self.environments.read().unwrap();
            let environment = environments.get(&environment_id)
                .ok_or(EnvironmentError::NotFound { id: environment_id })?;
            environment.config.clone()
        };

        self.cleanup_docker_environment(&config).await
    }

    /// Clean up Docker containers and resources
    async fn cleanup_docker_environment(&self, config: &EnvironmentConfig) -> EnvironmentResult<()> {
        debug!("Cleaning up Docker environment");

        let output = Command::new("docker-compose")
            .args([
                "-f", &config.docker_compose_file,
                "down", "-v", "--remove-orphans"
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| EnvironmentError::DockerError {
                command: "docker-compose down".to_string(),
                error: e.to_string(),
            })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            warn!("Docker cleanup had issues: {}", error_msg);
        }

        Ok(())
    }

    /// Get environment by ID
    pub async fn get_environment(&self, environment_id: Uuid) -> EnvironmentResult<TestEnvironment> {
        let environments = self.environments.read().unwrap();
        environments.get(&environment_id)
            .cloned()
            .ok_or(EnvironmentError::NotFound { id: environment_id })
    }

    /// List all environments
    pub async fn list_environments(&self) -> Vec<TestEnvironment> {
        let environments = self.environments.read().unwrap();
        environments.values().cloned().collect()
    }

    /// Update environment status
    async fn update_environment_status(&self, environment_id: Uuid, status: EnvironmentStatus) -> EnvironmentResult<()> {
        let mut environments = self.environments.write().unwrap();
        let environment = environments.get_mut(&environment_id)
            .ok_or(EnvironmentError::NotFound { id: environment_id })?;

        environment.set_status(status);
        Ok(())
    }

    /// Set up default Docker services
    fn setup_default_docker_services(&self, environment: &mut TestEnvironment) {
        environment.add_service(ServiceConfig::postgres());
        environment.add_service(ServiceConfig::redis());
        environment.add_service(ServiceConfig::surrealdb());

        // Add corresponding health checks
        environment.add_health_check(HealthCheck {
            service: "postgres".to_string(),
            endpoint: ":5432".to_string(),
            expected_status: 200,
            timeout_ms: 5000,
            retry_count: 5,
            check_interval: Duration::from_secs(10),
        });

        environment.add_health_check(HealthCheck {
            service: "redis".to_string(),
            endpoint: ":6379".to_string(),
            expected_status: 200,
            timeout_ms: 5000,
            retry_count: 5,
            check_interval: Duration::from_secs(10),
        });

        environment.add_health_check(HealthCheck {
            service: "surreal".to_string(),
            endpoint: ":8000/health".to_string(),
            expected_status: 200,
            timeout_ms: 5000,
            retry_count: 5,
            check_interval: Duration::from_secs(10),
        });
    }

    /// Set up local services configuration
    fn setup_local_services(&self, environment: &mut TestEnvironment) {
        // For local development, assume services are running on default ports
        environment.config.ports.insert("postgres".to_string(), 5432);
        environment.config.ports.insert("redis".to_string(), 6379);
        environment.config.ports.insert("surreal".to_string(), 8000);
    }

    /// Log an environment event
    async fn log_event(&self, event: EnvironmentEvent) {
        debug!("Environment event: {} - {}", event.event_type_str(), event.message);

        let mut events = self.events.write().unwrap();
        events.push(event);

        // Keep only the last 1000 events to prevent memory growth
        if events.len() > 1000 {
            events.drain(..events.len() - 1000);
        }
    }

    /// Get environment events
    pub async fn get_environment_events(&self, environment_id: Uuid) -> Vec<EnvironmentEvent> {
        let events = self.events.read().unwrap();
        events.iter()
            .filter(|e| e.environment_id == environment_id)
            .cloned()
            .collect()
    }

    /// Get environment metrics
    pub async fn get_environment_metrics(&self, environment_id: Uuid) -> EnvironmentResult<EnvironmentMetrics> {
        let environment = self.get_environment(environment_id).await?;

        // For now, return basic metrics
        // In a real implementation, this would query Docker/system metrics
        Ok(EnvironmentMetrics {
            environment_id,
            uptime: environment.uptime(),
            cpu_usage: 0.0, // TODO: Implement actual CPU monitoring
            memory_usage: 0.0, // TODO: Implement actual memory monitoring
            disk_usage: 0.0, // TODO: Implement actual disk monitoring
            service_count: environment.services.len(),
            healthy_services: environment.services.len(), // TODO: Track actual health
            failed_services: 0, // TODO: Track actual failures
        })
    }
}

impl Default for EnvironmentService {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentEvent {
    /// Get string representation of event type
    pub fn event_type_str(&self) -> &'static str {
        match self.event_type {
            EnvironmentEventType::Creating => "Creating",
            EnvironmentEventType::ServiceStarting => "ServiceStarting",
            EnvironmentEventType::ServiceReady => "ServiceReady",
            EnvironmentEventType::ServiceFailed => "ServiceFailed",
            EnvironmentEventType::HealthCheckPassed => "HealthCheckPassed",
            EnvironmentEventType::HealthCheckFailed => "HealthCheckFailed",
            EnvironmentEventType::Ready => "Ready",
            EnvironmentEventType::TestStarted => "TestStarted",
            EnvironmentEventType::TestCompleted => "TestCompleted",
            EnvironmentEventType::Destroying => "Destroying",
            EnvironmentEventType::Destroyed => "Destroyed",
            EnvironmentEventType::Error => "Error",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_create_environment_service() {
        let service = EnvironmentService::new();

        // Should start with no environments
        let environments = service.list_environments().await;
        assert_eq!(environments.len(), 0);
    }

    #[tokio::test]
    async fn test_create_local_environment() {
        let service = EnvironmentService::new();

        let env_id = service.create_environment(
            "test-local".to_string(),
            EnvironmentType::Local
        ).await.expect("Failed to create local environment");

        let environment = service.get_environment(env_id).await
            .expect("Environment not found");

        assert_eq!(environment.name, "test-local");
        assert_eq!(environment.environment_type, EnvironmentType::Local);
    }

    #[tokio::test]
    async fn test_environment_lifecycle() {
        let service = EnvironmentService::new();

        let env_id = service.create_environment(
            "test-lifecycle".to_string(),
            EnvironmentType::Local
        ).await.expect("Failed to create environment");

        // Environment should exist
        let environment = service.get_environment(env_id).await
            .expect("Environment not found");
        assert_eq!(environment.name, "test-lifecycle");

        // Destroy environment
        service.destroy_environment(env_id).await
            .expect("Failed to destroy environment");

        // Environment should no longer exist
        let result = service.get_environment(env_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_environment_events() {
        let service = EnvironmentService::new();

        let env_id = service.create_environment(
            "test-events".to_string(),
            EnvironmentType::Local
        ).await.expect("Failed to create environment");

        // Should have creation events
        let events = service.get_environment_events(env_id).await;
        assert!(!events.is_empty());

        let creation_event = events.iter()
            .find(|e| matches!(e.event_type, EnvironmentEventType::Creating));
        assert!(creation_event.is_some());
    }
}