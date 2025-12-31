use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Test environment with Docker orchestration for isolated testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEnvironment {
    pub id: Uuid,
    pub name: String,
    pub environment_type: EnvironmentType,
    pub status: EnvironmentStatus,
    pub config: EnvironmentConfig,
    pub services: Vec<ServiceConfig>,
    pub health_checks: Vec<HealthCheck>,
    pub created_at: SystemTime,
    pub destroyed_at: Option<SystemTime>,
    pub resource_limits: ResourceLimits,
}

/// Type of test environment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EnvironmentType {
    Docker,
    Local,
    Cloud,
}

/// Current status of the test environment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EnvironmentStatus {
    Creating,
    Ready,
    Running,
    Destroying,
    Failed,
    Destroyed,
}

/// Environment configuration with Docker Compose settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub docker_compose_file: String,
    pub network_name: String,
    pub volumes: HashMap<String, String>,
    pub environment_variables: HashMap<String, String>,
    pub ports: HashMap<String, u16>,
    pub timeout: Duration,
    pub cleanup_on_exit: bool,
}

/// Service configuration for Docker containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub image: String,
    pub ports: Vec<String>,
    pub environment: HashMap<String, String>,
    pub volumes: Vec<String>,
    pub depends_on: Vec<String>,
    pub health_check: Option<ServiceHealthCheck>,
    pub resource_limits: ServiceResourceLimits,
}

/// Service-specific health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthCheck {
    pub test: Vec<String>,
    pub interval: Duration,
    pub timeout: Duration,
    pub retries: u32,
    pub start_period: Duration,
}

/// Health check validation for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub service: String,
    pub endpoint: String,
    pub expected_status: u16,
    pub timeout_ms: u64,
    pub retry_count: u32,
    pub check_interval: Duration,
}

/// Resource limits for the entire environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_limit: String,
    pub memory_limit: String,
    pub disk_limit: String,
    pub max_containers: u32,
}

/// Service-specific resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResourceLimits {
    pub cpus: Option<String>,
    pub memory: Option<String>,
    pub memswap_limit: Option<String>,
}

/// Environment lifecycle events for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentEvent {
    pub environment_id: Uuid,
    pub event_type: EnvironmentEventType,
    pub timestamp: SystemTime,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Types of environment events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvironmentEventType {
    Creating,
    ServiceStarting,
    ServiceReady,
    ServiceFailed,
    HealthCheckPassed,
    HealthCheckFailed,
    Ready,
    TestStarted,
    TestCompleted,
    Destroying,
    Destroyed,
    Error,
}

impl TestEnvironment {
    /// Create a new test environment with default configuration
    pub fn new(name: String, environment_type: EnvironmentType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            environment_type,
            status: EnvironmentStatus::Creating,
            config: EnvironmentConfig::default(),
            services: Vec::new(),
            health_checks: Vec::new(),
            created_at: SystemTime::now(),
            destroyed_at: None,
            resource_limits: ResourceLimits::default(),
        }
    }

    /// Add a service to the environment
    pub fn add_service(&mut self, service: ServiceConfig) {
        self.services.push(service);
    }

    /// Add a health check for a service
    pub fn add_health_check(&mut self, health_check: HealthCheck) {
        self.health_checks.push(health_check);
    }

    /// Update environment status
    pub fn set_status(&mut self, status: EnvironmentStatus) {
        self.status = status;
        if status == EnvironmentStatus::Destroyed {
            self.destroyed_at = Some(SystemTime::now());
        }
    }

    /// Check if environment is ready for testing
    pub fn is_ready(&self) -> bool {
        self.status == EnvironmentStatus::Ready
    }

    /// Check if environment is running tests
    pub fn is_running(&self) -> bool {
        self.status == EnvironmentStatus::Running
    }

    /// Check if environment is destroyed or being destroyed
    pub fn is_destroyed(&self) -> bool {
        matches!(self.status, EnvironmentStatus::Destroyed | EnvironmentStatus::Destroying)
    }

    /// Get environment uptime
    pub fn uptime(&self) -> Duration {
        if let Some(destroyed_at) = self.destroyed_at {
            destroyed_at.duration_since(self.created_at).unwrap_or_default()
        } else {
            SystemTime::now().duration_since(self.created_at).unwrap_or_default()
        }
    }

    /// Validate environment configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Environment name cannot be empty".to_string());
        }

        if self.services.is_empty() {
            return Err("Environment must have at least one service".to_string());
        }

        // Validate service dependencies
        let service_names: Vec<&String> = self.services.iter().map(|s| &s.name).collect();
        for service in &self.services {
            for dep in &service.depends_on {
                if !service_names.contains(&dep) {
                    return Err(format!(
                        "Service '{}' depends on '{}' which is not defined",
                        service.name, dep
                    ));
                }
            }
        }

        // Validate health checks reference existing services
        for health_check in &self.health_checks {
            if !service_names.contains(&&health_check.service) {
                return Err(format!(
                    "Health check references undefined service '{}'",
                    health_check.service
                ));
            }
        }

        Ok(())
    }
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            docker_compose_file: "docker-compose.test.yaml".to_string(),
            network_name: "test-network".to_string(),
            volumes: HashMap::new(),
            environment_variables: HashMap::new(),
            ports: HashMap::new(),
            timeout: Duration::from_secs(300), // 5 minutes
            cleanup_on_exit: true,
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_limit: "2.0".to_string(),
            memory_limit: "4GB".to_string(),
            disk_limit: "10GB".to_string(),
            max_containers: 10,
        }
    }
}

impl ServiceConfig {
    /// Create a PostgreSQL service configuration
    pub fn postgres() -> Self {
        let mut environment = HashMap::new();
        environment.insert("POSTGRES_PASSWORD".to_string(), "postgres".to_string());
        environment.insert("POSTGRES_USER".to_string(), "postgres".to_string());
        environment.insert("POSTGRES_DB".to_string(), "uar_test".to_string());

        Self {
            name: "postgres".to_string(),
            image: "pgvector/pgvector:pg17".to_string(),
            ports: vec!["5432:5432".to_string()],
            environment,
            volumes: vec![],
            depends_on: vec![],
            health_check: Some(ServiceHealthCheck {
                test: vec!["CMD".to_string(), "pg_isready".to_string()],
                interval: Duration::from_secs(10),
                timeout: Duration::from_secs(5),
                retries: 5,
                start_period: Duration::from_secs(30),
            }),
            resource_limits: ServiceResourceLimits {
                cpus: Some("1.0".to_string()),
                memory: Some("1GB".to_string()),
                memswap_limit: None,
            },
        }
    }

    /// Create a Redis service configuration
    pub fn redis() -> Self {
        Self {
            name: "redis".to_string(),
            image: "redis/redis-stack:latest".to_string(),
            ports: vec!["6379:6379".to_string()],
            environment: HashMap::new(),
            volumes: vec![],
            depends_on: vec![],
            health_check: Some(ServiceHealthCheck {
                test: vec!["CMD".to_string(), "redis-cli".to_string(), "ping".to_string()],
                interval: Duration::from_secs(10),
                timeout: Duration::from_secs(5),
                retries: 5,
                start_period: Duration::from_secs(10),
            }),
            resource_limits: ServiceResourceLimits {
                cpus: Some("0.5".to_string()),
                memory: Some("512MB".to_string()),
                memswap_limit: None,
            },
        }
    }

    /// Create a SurrealDB service configuration
    pub fn surrealdb() -> Self {
        Self {
            name: "surreal".to_string(),
            image: "surrealdb/surrealdb:latest".to_string(),
            ports: vec!["8000:8000".to_string()],
            environment: HashMap::new(),
            volumes: vec![],
            depends_on: vec![],
            health_check: Some(ServiceHealthCheck {
                test: vec![
                    "CMD".to_string(),
                    "curl".to_string(),
                    "-f".to_string(),
                    "http://localhost:8000/health".to_string(),
                ],
                interval: Duration::from_secs(10),
                timeout: Duration::from_secs(5),
                retries: 5,
                start_period: Duration::from_secs(15),
            }),
            resource_limits: ServiceResourceLimits {
                cpus: Some("0.5".to_string()),
                memory: Some("512MB".to_string()),
                memswap_limit: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_environment() {
        let env = TestEnvironment::new("test-env".to_string(), EnvironmentType::Docker);

        assert_eq!(env.name, "test-env");
        assert_eq!(env.environment_type, EnvironmentType::Docker);
        assert_eq!(env.status, EnvironmentStatus::Creating);
        assert!(!env.is_ready());
        assert!(!env.is_running());
        assert!(!env.is_destroyed());
    }

    #[test]
    fn test_status_transitions() {
        let mut env = TestEnvironment::new("test-env".to_string(), EnvironmentType::Docker);

        env.set_status(EnvironmentStatus::Ready);
        assert!(env.is_ready());
        assert!(!env.is_running());

        env.set_status(EnvironmentStatus::Running);
        assert!(!env.is_ready());
        assert!(env.is_running());

        env.set_status(EnvironmentStatus::Destroyed);
        assert!(env.is_destroyed());
        assert!(env.destroyed_at.is_some());
    }

    #[test]
    fn test_add_services() {
        let mut env = TestEnvironment::new("test-env".to_string(), EnvironmentType::Docker);

        env.add_service(ServiceConfig::postgres());
        env.add_service(ServiceConfig::redis());

        assert_eq!(env.services.len(), 2);
        assert_eq!(env.services[0].name, "postgres");
        assert_eq!(env.services[1].name, "redis");
    }

    #[test]
    fn test_validate_environment() {
        let mut env = TestEnvironment::new("test-env".to_string(), EnvironmentType::Docker);

        // Should fail without services
        assert!(env.validate().is_err());

        // Add a service
        env.add_service(ServiceConfig::postgres());
        assert!(env.validate().is_ok());

        // Test invalid dependency
        let mut invalid_service = ServiceConfig::redis();
        invalid_service.depends_on.push("nonexistent".to_string());
        env.add_service(invalid_service);
        assert!(env.validate().is_err());
    }

    #[test]
    fn test_service_configurations() {
        let postgres = ServiceConfig::postgres();
        assert_eq!(postgres.name, "postgres");
        assert_eq!(postgres.image, "pgvector/pgvector:pg17");
        assert!(postgres.health_check.is_some());

        let redis = ServiceConfig::redis();
        assert_eq!(redis.name, "redis");
        assert_eq!(redis.image, "redis/redis-stack:latest");
        assert!(redis.health_check.is_some());

        let surreal = ServiceConfig::surrealdb();
        assert_eq!(surreal.name, "surreal");
        assert_eq!(surreal.image, "surrealdb/surrealdb:latest");
        assert!(surreal.health_check.is_some());
    }

    #[test]
    fn test_environment_uptime() {
        let env = TestEnvironment::new("test-env".to_string(), EnvironmentType::Docker);
        let uptime = env.uptime();

        // Should be very small but non-zero
        assert!(uptime.as_millis() < 100);
    }
}