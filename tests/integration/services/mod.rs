#![allow(clippy::pedantic)]

use std::time::Duration;
use tracing::{info, error};

pub mod comprehensive;

pub use comprehensive::*;

/// Execute service certification test by test ID and environment
pub async fn execute_service_test(test_id: &str, environment_id: &str) -> Result<crate::certification::CertificationResult, Box<dyn std::error::Error + Send + Sync>> {
    info!("Executing service certification test: {} in environment: {}", test_id, environment_id);

    match test_id {
        "llm_integration" => execute_llm_integration_test(environment_id).await,
        "mcp_integration" => execute_mcp_integration_test(environment_id).await,
        "external_api_integration" => execute_external_api_test(environment_id).await,
        "service_performance" => execute_service_performance_test(environment_id).await,
        "streaming_integration" => execute_streaming_integration_test(environment_id).await,
        "tool_calling_integration" => execute_tool_calling_test(environment_id).await,
        _ => {
            error!("Unknown service test ID: {}", test_id);
            Ok(crate::certification::CertificationResult {
                test_id: test_id.to_string(),
                success: false,
                duration: Duration::from_millis(1),
                message: format!("Unknown service test ID: {}", test_id),
                details: None,
                artifacts: Vec::new(),
            })
        }
    }
}

/// Execute LLM integration test
async fn execute_llm_integration_test(environment_id: &str) -> Result<crate::certification::CertificationResult, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = std::time::Instant::now();

    info!("Testing LLM integration in environment: {}", environment_id);

    let suite = ServiceIntegrationSuite::new();

    // Test all configured LLM services
    let mut all_results = Vec::new();
    for config in &suite.llm_configs.clone() {
        let results = suite.test_llm_service(config).await?;
        all_results.extend(results);
    }

    let total_tests = all_results.len();
    let successful_tests = all_results.iter().filter(|r| r.success).count();
    let all_successful = successful_tests == total_tests;

    let artifacts = all_results.iter().map(|r| {
        format!("{}: {} ({}ms)", r.test_id, if r.success { "PASS" } else { "FAIL" }, r.duration.as_millis())
    }).collect();

    Ok(crate::certification::CertificationResult {
        test_id: "llm_integration".to_string(),
        success: all_successful,
        duration: start_time.elapsed(),
        message: format!("LLM integration tests: {}/{} passed", successful_tests, total_tests),
        details: Some(serde_json::json!({
            "environment_id": environment_id,
            "total_tests": total_tests,
            "successful_tests": successful_tests,
            "llm_services": suite.llm_configs.len(),
            "test_results": all_results.iter().map(|r| {
                serde_json::json!({
                    "test_id": r.test_id,
                    "service_name": r.service_name,
                    "service_type": format!("{:?}", r.service_type),
                    "scenario_id": r.scenario_id,
                    "success": r.success,
                    "duration_ms": r.duration.as_millis(),
                    "performance_metrics": {
                        "connection_time_ms": r.performance_metrics.connection_time.as_millis(),
                        "total_response_time_ms": r.performance_metrics.total_response_time.as_millis(),
                        "tokens_per_second": r.performance_metrics.tokens_per_second
                    }
                })
            }).collect::<Vec<_>>()
        })),
        artifacts,
    })
}

/// Execute MCP integration test
async fn execute_mcp_integration_test(environment_id: &str) -> Result<crate::certification::CertificationResult, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = std::time::Instant::now();

    info!("Testing MCP integration in environment: {}", environment_id);

    let suite = ServiceIntegrationSuite::new();

    // Test all configured MCP services
    let mut all_results = Vec::new();
    for config in &suite.mcp_configs.clone() {
        let results = suite.test_mcp_service(config).await?;
        all_results.extend(results);
    }

    let total_tests = all_results.len();
    let successful_tests = all_results.iter().filter(|r| r.success).count();
    let all_successful = successful_tests == total_tests;

    // Count tools tested
    let tools_tested: usize = suite.mcp_configs.iter().map(|c| c.expected_tools.len()).sum();

    let artifacts = all_results.iter().map(|r| {
        format!("{}: {} ({}ms)", r.test_id, if r.success { "PASS" } else { "FAIL" }, r.duration.as_millis())
    }).collect();

    Ok(crate::certification::CertificationResult {
        test_id: "mcp_integration".to_string(),
        success: all_successful,
        duration: start_time.elapsed(),
        message: format!("MCP integration tests: {}/{} passed, {} tools tested", successful_tests, total_tests, tools_tested),
        details: Some(serde_json::json!({
            "environment_id": environment_id,
            "total_tests": total_tests,
            "successful_tests": successful_tests,
            "mcp_services": suite.mcp_configs.len(),
            "tools_tested": tools_tested,
            "test_results": all_results.iter().map(|r| {
                serde_json::json!({
                    "test_id": r.test_id,
                    "service_name": r.service_name,
                    "service_type": format!("{:?}", r.service_type),
                    "scenario_id": r.scenario_id,
                    "success": r.success,
                    "duration_ms": r.duration.as_millis(),
                    "performance_metrics": {
                        "connection_time_ms": r.performance_metrics.connection_time.as_millis(),
                        "total_response_time_ms": r.performance_metrics.total_response_time.as_millis()
                    }
                })
            }).collect::<Vec<_>>()
        })),
        artifacts,
    })
}

/// Execute external API integration test
async fn execute_external_api_test(environment_id: &str) -> Result<crate::certification::CertificationResult, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = std::time::Instant::now();

    info!("Testing external API integration in environment: {}", environment_id);

    let suite = ServiceIntegrationSuite::new();

    // Test all configured external APIs
    let mut all_results = Vec::new();
    for config in &suite.external_apis.clone() {
        let results = suite.test_external_api(config).await?;
        all_results.extend(results);
    }

    let total_tests = all_results.len();
    let successful_tests = all_results.iter().filter(|r| r.success).count();
    let all_successful = successful_tests == total_tests;

    let artifacts = all_results.iter().map(|r| {
        format!("{}: {} ({}ms)", r.test_id, if r.success { "PASS" } else { "FAIL" }, r.duration.as_millis())
    }).collect();

    Ok(crate::certification::CertificationResult {
        test_id: "external_api_integration".to_string(),
        success: all_successful,
        duration: start_time.elapsed(),
        message: format!("External API integration tests: {}/{} passed", successful_tests, total_tests),
        details: Some(serde_json::json!({
            "environment_id": environment_id,
            "total_tests": total_tests,
            "successful_tests": successful_tests,
            "external_apis": suite.external_apis.len(),
            "test_results": all_results.iter().map(|r| {
                serde_json::json!({
                    "test_id": r.test_id,
                    "service_name": r.service_name,
                    "service_type": format!("{:?}", r.service_type),
                    "scenario_id": r.scenario_id,
                    "success": r.success,
                    "duration_ms": r.duration.as_millis(),
                    "performance_metrics": {
                        "connection_time_ms": r.performance_metrics.connection_time.as_millis(),
                        "total_response_time_ms": r.performance_metrics.total_response_time.as_millis()
                    }
                })
            }).collect::<Vec<_>>()
        })),
        artifacts,
    })
}

/// Execute service performance test
async fn execute_service_performance_test(environment_id: &str) -> Result<crate::certification::CertificationResult, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = std::time::Instant::now();

    info!("Testing service performance in environment: {}", environment_id);

    let mut suite = ServiceIntegrationSuite::new();
    let report = suite.execute_comprehensive_tests().await?;

    // Check performance thresholds
    let threshold_violations = &report.performance_summary.threshold_violations;
    let performance_passed = threshold_violations.is_empty();

    let artifacts = vec![
        format!("total_tests: {}", report.total_tests),
        format!("passed_tests: {}", report.passed_tests),
        format!("failed_tests: {}", report.failed_tests),
        format!("avg_response_time_ms: {}", report.performance_summary.average_response_time.as_millis()),
        format!("fastest_service: {}", report.performance_summary.fastest_service.as_deref().unwrap_or("N/A")),
        format!("slowest_service: {}", report.performance_summary.slowest_service.as_deref().unwrap_or("N/A")),
        format!("threshold_violations: {}", threshold_violations.len()),
    ];

    Ok(crate::certification::CertificationResult {
        test_id: "service_performance".to_string(),
        success: performance_passed && report.is_certified(),
        duration: start_time.elapsed(),
        message: if performance_passed {
            format!("Service performance tests passed: {} violations, avg response {}ms",
                threshold_violations.len(),
                report.performance_summary.average_response_time.as_millis())
        } else {
            format!("Service performance issues detected: {} violations", threshold_violations.len())
        },
        details: Some(serde_json::json!({
            "environment_id": environment_id,
            "performance_report": {
                "average_response_time_ms": report.performance_summary.average_response_time.as_millis(),
                "fastest_service": report.performance_summary.fastest_service,
                "slowest_service": report.performance_summary.slowest_service,
                "threshold_violations": threshold_violations,
                "streaming_performance": report.performance_summary.streaming_performance.as_ref().map(|s| {
                    serde_json::json!({
                        "average_first_token_time_ms": s.average_first_token_time.as_millis(),
                        "average_tokens_per_second": s.average_tokens_per_second,
                        "total_tokens_streamed": s.total_tokens_streamed,
                        "streaming_sessions": s.streaming_sessions
                    })
                })
            },
            "certification_status": format!("{:?}", report.integration_status)
        })),
        artifacts,
    })
}

/// Execute streaming integration test
async fn execute_streaming_integration_test(environment_id: &str) -> Result<crate::certification::CertificationResult, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = std::time::Instant::now();

    info!("Testing streaming integration in environment: {}", environment_id);

    let suite = ServiceIntegrationSuite::new();

    // Test streaming for each LLM service
    let mut streaming_results = Vec::new();
    for config in &suite.llm_configs.clone() {
        let result = suite.test_llm_streaming(config).await?;
        streaming_results.push(result);
    }

    let total_tests = streaming_results.len();
    let successful_tests = streaming_results.iter().filter(|r| r.success).count();
    let all_successful = successful_tests == total_tests;

    // Calculate streaming metrics
    let avg_tokens_per_second = streaming_results
        .iter()
        .filter_map(|r| r.performance_metrics.tokens_per_second)
        .sum::<f64>() / successful_tests as f64;

    let avg_first_token_time = streaming_results
        .iter()
        .map(|r| r.performance_metrics.first_byte_time)
        .sum::<Duration>() / total_tests as u32;

    let streaming_threshold_met = avg_tokens_per_second >= suite.performance_thresholds.streaming_tokens_per_second;
    let first_token_threshold_met = avg_first_token_time <= Duration::from_millis(suite.performance_thresholds.streaming_first_token_ms);

    let final_success = all_successful && streaming_threshold_met && first_token_threshold_met;

    let artifacts = streaming_results.iter().map(|r| {
        let tokens_per_sec = r.performance_metrics.tokens_per_second.unwrap_or(0.0);
        format!("{}: {} ({:.1} tokens/sec, first token: {}ms)",
            r.service_name,
            if r.success { "PASS" } else { "FAIL" },
            tokens_per_sec,
            r.performance_metrics.first_byte_time.as_millis())
    }).collect();

    Ok(crate::certification::CertificationResult {
        test_id: "streaming_integration".to_string(),
        success: final_success,
        duration: start_time.elapsed(),
        message: if final_success {
            format!("Streaming integration tests passed: {:.1} avg tokens/sec, {}ms avg first token",
                avg_tokens_per_second,
                avg_first_token_time.as_millis())
        } else {
            format!("Streaming integration issues: {}/{} services passed, {:.1} tokens/sec",
                successful_tests,
                total_tests,
                avg_tokens_per_second)
        },
        details: Some(serde_json::json!({
            "environment_id": environment_id,
            "streaming_metrics": {
                "total_services_tested": total_tests,
                "successful_services": successful_tests,
                "average_tokens_per_second": avg_tokens_per_second,
                "average_first_token_time_ms": avg_first_token_time.as_millis(),
                "streaming_threshold_met": streaming_threshold_met,
                "first_token_threshold_met": first_token_threshold_met,
                "thresholds": {
                    "min_tokens_per_second": suite.performance_thresholds.streaming_tokens_per_second,
                    "max_first_token_time_ms": suite.performance_thresholds.streaming_first_token_ms
                }
            }
        })),
        artifacts,
    })
}

/// Execute tool calling integration test
async fn execute_tool_calling_test(environment_id: &str) -> Result<crate::certification::CertificationResult, Box<dyn std::error::Error + Send + Sync>> {
    let start_time = std::time::Instant::now();

    info!("Testing tool calling integration in environment: {}", environment_id);

    let suite = ServiceIntegrationSuite::new();

    // Test tool calling for services that support it
    let mut tool_calling_results = Vec::new();
    for config in &suite.llm_configs.clone() {
        if suite.supports_tool_calling(config) {
            let result = suite.test_llm_tool_calling(config).await?;
            tool_calling_results.push(result);
        }
    }

    // Test MCP tool execution
    for config in &suite.mcp_configs.clone() {
        for tool in &config.expected_tools.clone() {
            let result = suite.test_mcp_tool_execution(config, tool).await?;
            tool_calling_results.push(result);
        }
    }

    let total_tests = tool_calling_results.len();
    let successful_tests = tool_calling_results.iter().filter(|r| r.success).count();
    let all_successful = successful_tests == total_tests;

    let artifacts = tool_calling_results.iter().map(|r| {
        format!("{}: {} ({}ms)", r.test_id, if r.success { "PASS" } else { "FAIL" }, r.duration.as_millis())
    }).collect();

    Ok(crate::certification::CertificationResult {
        test_id: "tool_calling_integration".to_string(),
        success: all_successful,
        duration: start_time.elapsed(),
        message: format!("Tool calling integration tests: {}/{} passed", successful_tests, total_tests),
        details: Some(serde_json::json!({
            "environment_id": environment_id,
            "tool_calling_metrics": {
                "total_tests": total_tests,
                "successful_tests": successful_tests,
                "llm_tool_calling_tests": suite.llm_configs.iter().filter(|c| suite.supports_tool_calling(c)).count(),
                "mcp_tool_execution_tests": suite.mcp_configs.iter().map(|c| c.expected_tools.len()).sum::<usize>(),
            },
            "test_results": tool_calling_results.iter().map(|r| {
                serde_json::json!({
                    "test_id": r.test_id,
                    "service_name": r.service_name,
                    "service_type": format!("{:?}", r.service_type),
                    "success": r.success,
                    "duration_ms": r.duration.as_millis()
                })
            }).collect::<Vec<_>>()
        })),
        artifacts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_integration() {
        let result = execute_service_test("llm_integration", "test_env").await;
        assert!(result.is_ok());

        let cert_result = result.unwrap();
        assert_eq!(cert_result.test_id, "llm_integration");
        assert!(!cert_result.artifacts.is_empty());
    }

    #[tokio::test]
    async fn test_mcp_integration() {
        let result = execute_service_test("mcp_integration", "test_env").await;
        assert!(result.is_ok());

        let cert_result = result.unwrap();
        assert_eq!(cert_result.test_id, "mcp_integration");
        assert!(cert_result.success);
    }

    #[tokio::test]
    async fn test_external_api_integration() {
        let result = execute_service_test("external_api_integration", "test_env").await;
        assert!(result.is_ok());

        let cert_result = result.unwrap();
        assert_eq!(cert_result.test_id, "external_api_integration");
        assert!(cert_result.success);
    }

    #[tokio::test]
    async fn test_service_performance() {
        let result = execute_service_test("service_performance", "test_env").await;
        assert!(result.is_ok());

        let cert_result = result.unwrap();
        assert_eq!(cert_result.test_id, "service_performance");
    }

    #[tokio::test]
    async fn test_streaming_integration() {
        let result = execute_service_test("streaming_integration", "test_env").await;
        assert!(result.is_ok());

        let cert_result = result.unwrap();
        assert_eq!(cert_result.test_id, "streaming_integration");
    }

    #[tokio::test]
    async fn test_tool_calling_integration() {
        let result = execute_service_test("tool_calling_integration", "test_env").await;
        assert!(result.is_ok());

        let cert_result = result.unwrap();
        assert_eq!(cert_result.test_id, "tool_calling_integration");
    }

    #[tokio::test]
    async fn test_unknown_service_test() {
        let result = execute_service_test("unknown_test", "test_env").await;
        assert!(result.is_ok());

        let cert_result = result.unwrap();
        assert!(!cert_result.success);
        assert!(cert_result.message.contains("Unknown service test ID"));
    }
}
