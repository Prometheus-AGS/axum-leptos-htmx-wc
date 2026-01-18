use std::time::Duration;
use tracing::{info, error};

pub mod comprehensive;

pub use comprehensive::*;

/// Execute database certification test by test ID and environment
pub async fn execute_database_test(test_id: &str, environment_id: &str) -> Result<crate::certification::CertificationResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Executing database certification test: {} in environment: {}", test_id, environment_id);

    match test_id {
        "database_connectivity" => execute_connectivity_test(environment_id).await,
        "postgres_operations" => execute_postgres_test(environment_id).await,
        "surreal_operations" => execute_surreal_test(environment_id).await,
        "redis_operations" => execute_redis_test(environment_id).await,
        "database_performance" => execute_performance_test(environment_id).await,
        "database_integration" => execute_integration_test(environment_id).await,
        _ => {
            error!("Unknown database test ID: {}", test_id);
            Ok(crate::certification::CertificationResult {
                test_id: test_id.to_string(),
                success: false,
                duration: Duration::from_millis(1),
                message: format!("Unknown database test ID: {test_id}"),
                details: None,
                artifacts: Vec::new(),
            })
        }
    }
}

/// Execute database connectivity test
async fn execute_connectivity_test(environment_id: &str) -> Result<crate::certification::CertificationResult, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = std::time::Instant::now();

    info!("Testing database connectivity in environment: {}", environment_id);

    // Create and execute comprehensive database test suite
    let suite = DatabaseCertificationSuite::new();

    // Test individual database connections
    let postgres_result = suite.test_postgres_connection().await?;
    let surreal_result = suite.test_surreal_connection().await?;
    let redis_result = suite.test_redis_connection().await?;

    let all_successful = postgres_result.success && surreal_result.success && redis_result.success;

    let artifacts = vec![
        format!("postgres_connection_time_ms: {}", postgres_result.performance_metrics.connection_time.as_millis()),
        format!("surreal_connection_time_ms: {}", surreal_result.performance_metrics.connection_time.as_millis()),
        format!("redis_connection_time_ms: {}", redis_result.performance_metrics.connection_time.as_millis()),
    ];

    Ok(crate::certification::CertificationResult {
        test_id: "database_connectivity".to_string(),
        success: all_successful,
        duration: start_time.elapsed(),
        message: if all_successful {
            "All database connections successful".to_string()
        } else {
            "One or more database connections failed".to_string()
        },
        details: Some(serde_json::json!({
            "environment_id": environment_id,
            "postgres": {
                "success": postgres_result.success,
                "connection_time_ms": postgres_result.performance_metrics.connection_time.as_millis()
            },
            "surreal": {
                "success": surreal_result.success,
                "connection_time_ms": surreal_result.performance_metrics.connection_time.as_millis()
            },
            "redis": {
                "success": redis_result.success,
                "connection_time_ms": redis_result.performance_metrics.connection_time.as_millis()
            }
        })),
        artifacts,
    })
}

/// Execute `PostgreSQL` operations test
async fn execute_postgres_test(environment_id: &str) -> Result<crate::certification::CertificationResult, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = std::time::Instant::now();

    info!("Testing PostgreSQL operations in environment: {}", environment_id);

    let suite = DatabaseCertificationSuite::new();
    let results = suite.test_postgres_operations().await?;

    let total_tests = results.len();
    let successful_tests = results.iter().filter(|r| r.success).count();
    let all_successful = successful_tests == total_tests;

    let artifacts = results.iter().map(|r| {
        format!("{}: {} ({}ms)", r.test_id, if r.success { "PASS" } else { "FAIL" }, r.duration.as_millis())
    }).collect();

    Ok(crate::certification::CertificationResult {
        test_id: "postgres_operations".to_string(),
        success: all_successful,
        duration: start_time.elapsed(),
        message: format!("PostgreSQL tests: {successful_tests}/{total_tests} passed"),
        details: Some(serde_json::json!({
            "environment_id": environment_id,
            "total_tests": total_tests,
            "successful_tests": successful_tests,
            "test_results": results.iter().map(|r| {
                serde_json::json!({
                    "test_id": r.test_id,
                    "operation": r.operation,
                    "success": r.success,
                    "duration_ms": r.duration.as_millis(),
                    "records_affected": r.records_affected
                })
            }).collect::<Vec<_>>()
        })),
        artifacts,
    })
}

/// Execute `SurrealDB` operations test
async fn execute_surreal_test(environment_id: &str) -> Result<crate::certification::CertificationResult, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = std::time::Instant::now();

    info!("Testing SurrealDB operations in environment: {}", environment_id);

    let suite = DatabaseCertificationSuite::new();
    let results = suite.test_surreal_operations().await?;

    let total_tests = results.len();
    let successful_tests = results.iter().filter(|r| r.success).count();
    let all_successful = successful_tests == total_tests;

    let artifacts = results.iter().map(|r| {
        format!("{}: {} ({}ms)", r.test_id, if r.success { "PASS" } else { "FAIL" }, r.duration.as_millis())
    }).collect();

    Ok(crate::certification::CertificationResult {
        test_id: "surreal_operations".to_string(),
        success: all_successful,
        duration: start_time.elapsed(),
        message: format!("SurrealDB tests: {successful_tests}/{total_tests} passed"),
        details: Some(serde_json::json!({
            "environment_id": environment_id,
            "total_tests": total_tests,
            "successful_tests": successful_tests,
            "test_results": results.iter().map(|r| {
                serde_json::json!({
                    "test_id": r.test_id,
                    "operation": r.operation,
                    "success": r.success,
                    "duration_ms": r.duration.as_millis(),
                    "records_affected": r.records_affected
                })
            }).collect::<Vec<_>>()
        })),
        artifacts,
    })
}

/// Execute Redis operations test
async fn execute_redis_test(environment_id: &str) -> Result<crate::certification::CertificationResult, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = std::time::Instant::now();

    info!("Testing Redis operations in environment: {}", environment_id);

    let suite = DatabaseCertificationSuite::new();
    let results = suite.test_redis_operations().await?;

    let total_tests = results.len();
    let successful_tests = results.iter().filter(|r| r.success).count();
    let all_successful = successful_tests == total_tests;

    let artifacts = results.iter().map(|r| {
        format!("{}: {} ({}ms)", r.test_id, if r.success { "PASS" } else { "FAIL" }, r.duration.as_millis())
    }).collect();

    Ok(crate::certification::CertificationResult {
        test_id: "redis_operations".to_string(),
        success: all_successful,
        duration: start_time.elapsed(),
        message: format!("Redis tests: {successful_tests}/{total_tests} passed"),
        details: Some(serde_json::json!({
            "environment_id": environment_id,
            "total_tests": total_tests,
            "successful_tests": successful_tests,
            "test_results": results.iter().map(|r| {
                serde_json::json!({
                    "test_id": r.test_id,
                    "operation": r.operation,
                    "success": r.success,
                    "duration_ms": r.duration.as_millis(),
                    "records_affected": r.records_affected
                })
            }).collect::<Vec<_>>()
        })),
        artifacts,
    })
}

/// Execute database performance test
async fn execute_performance_test(environment_id: &str) -> Result<crate::certification::CertificationResult, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = std::time::Instant::now();

    info!("Testing database performance in environment: {}", environment_id);

    let suite = DatabaseCertificationSuite::new();

    // Execute performance tests across all databases
    let postgres_performance = suite.test_postgres_performance().await?;
    let redis_performance = suite.test_redis_performance().await?;

    let mut all_results = postgres_performance;
    all_results.push(redis_performance);

    let total_tests = all_results.len();
    let successful_tests = all_results.iter().filter(|r| r.success).count();
    let all_successful = successful_tests == total_tests;

    // Check performance thresholds
    let mut threshold_violations = Vec::new();
    for result in &all_results {
        if result.performance_metrics.connection_time > Duration::from_millis(suite.performance_thresholds.connection_time_ms) {
            threshold_violations.push(format!("{}: Connection time exceeded threshold", result.test_id));
        }
        if result.performance_metrics.query_time > Duration::from_millis(suite.performance_thresholds.query_time_ms) {
            threshold_violations.push(format!("{}: Query time exceeded threshold", result.test_id));
        }
    }

    let performance_passed = threshold_violations.is_empty();
    let final_success = all_successful && performance_passed;

    let artifacts = all_results.iter().map(|r| {
        format!("{}: {} (query: {}ms, conn: {}ms)",
            r.test_id,
            if r.success { "PASS" } else { "FAIL" },
            r.performance_metrics.query_time.as_millis(),
            r.performance_metrics.connection_time.as_millis())
    }).chain(threshold_violations.iter().cloned()).collect();

    Ok(crate::certification::CertificationResult {
        test_id: "database_performance".to_string(),
        success: final_success,
        duration: start_time.elapsed(),
        message: if final_success {
            format!("Performance tests passed: {successful_tests}/{total_tests} tests, 0 threshold violations")
        } else {
            format!("Performance issues detected: {}/{} tests passed, {} threshold violations", successful_tests, total_tests, threshold_violations.len())
        },
        details: Some(serde_json::json!({
            "environment_id": environment_id,
            "total_tests": total_tests,
            "successful_tests": successful_tests,
            "threshold_violations": threshold_violations,
            "performance_thresholds": {
                "connection_time_ms": suite.performance_thresholds.connection_time_ms,
                "query_time_ms": suite.performance_thresholds.query_time_ms,
                "transaction_time_ms": suite.performance_thresholds.transaction_time_ms
            }
        })),
        artifacts,
    })
}

/// Execute comprehensive database integration test
async fn execute_integration_test(environment_id: &str) -> Result<crate::certification::CertificationResult, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = std::time::Instant::now();

    info!("Testing database integration in environment: {}", environment_id);

    let mut suite = DatabaseCertificationSuite::new();
    let report = suite.execute_comprehensive_tests().await?;

    let artifacts = vec![
        format!("total_tests: {}", report.total_tests),
        format!("passed_tests: {}", report.passed_tests),
        format!("failed_tests: {}", report.failed_tests),
        format!("postgres_tests: {}", report.postgres_results.len()),
        format!("surreal_tests: {}", report.surreal_results.len()),
        format!("redis_tests: {}", report.redis_results.len()),
        format!("certification_status: {:?}", report.certification_status),
        format!("avg_connection_time_ms: {}", report.performance_summary.average_connection_time.as_millis()),
        format!("avg_query_time_ms: {}", report.performance_summary.average_query_time.as_millis()),
    ];

    Ok(crate::certification::CertificationResult {
        test_id: "database_integration".to_string(),
        success: report.is_certified(),
        duration: start_time.elapsed(),
        message: report.summary(),
        details: Some(serde_json::json!({
            "environment_id": environment_id,
            "certification_report": {
                "executed_at": report.executed_at,
                "total_tests": report.total_tests,
                "passed_tests": report.passed_tests,
                "failed_tests": report.failed_tests,
                "certification_status": format!("{:?}", report.certification_status),
                "performance_summary": {
                    "average_connection_time_ms": report.performance_summary.average_connection_time.as_millis(),
                    "average_query_time_ms": report.performance_summary.average_query_time.as_millis(),
                    "slowest_operations": report.performance_summary.slowest_operations,
                    "threshold_violations": report.performance_summary.threshold_violations
                },
                "validation_summary": {
                    "total_validations": report.validation_summary.total_validations,
                    "passed_validations": report.validation_summary.passed_validations,
                    "critical_failures": report.validation_summary.critical_failures,
                    "failed_rules": report.validation_summary.failed_rules
                }
            }
        })),
        artifacts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connectivity() {
        let result = execute_database_test("database_connectivity", "test_env").await;
        assert!(result.is_ok());

        let cert_result = result.unwrap();
        assert_eq!(cert_result.test_id, "database_connectivity");
        assert!(!cert_result.artifacts.is_empty());
    }

    #[tokio::test]
    async fn test_postgres_operations() {
        let result = execute_database_test("postgres_operations", "test_env").await;
        assert!(result.is_ok());

        let cert_result = result.unwrap();
        assert_eq!(cert_result.test_id, "postgres_operations");
        assert!(cert_result.success);
    }

    #[tokio::test]
    async fn test_surreal_operations() {
        let result = execute_database_test("surreal_operations", "test_env").await;
        assert!(result.is_ok());

        let cert_result = result.unwrap();
        assert_eq!(cert_result.test_id, "surreal_operations");
        assert!(cert_result.success);
    }

    #[tokio::test]
    async fn test_redis_operations() {
        let result = execute_database_test("redis_operations", "test_env").await;
        assert!(result.is_ok());

        let cert_result = result.unwrap();
        assert_eq!(cert_result.test_id, "redis_operations");
        assert!(cert_result.success);
    }

    #[tokio::test]
    async fn test_unknown_test_id() {
        let result = execute_database_test("unknown_test", "test_env").await;
        assert!(result.is_ok());

        let cert_result = result.unwrap();
        assert!(!cert_result.success);
        assert!(cert_result.message.contains("Unknown database test ID"));
    }
}
