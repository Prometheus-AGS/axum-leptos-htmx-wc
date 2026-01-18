#![allow(dead_code, clippy::pedantic)]

use axum::http::StatusCode;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;


/// Comprehensive API endpoint testing for certification
pub struct ApiCertificationSuite {
    base_url: String,
    client: reqwest::Client,
    auth_token: Option<String>,
}

/// API test result with detailed information
#[derive(Debug, Clone)]
pub struct ApiTestResult {
    pub endpoint: String,
    pub method: String,
    pub status_code: u16,
    pub response_time: Duration,
    pub success: bool,
    pub message: String,
    pub response_body: Option<Value>,
}

/// Collection of API tests for comprehensive validation
#[derive(Debug)]
pub struct ApiEndpointTest {
    pub name: String,
    pub method: &'static str,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Value>,
    pub expected_status: StatusCode,
    pub timeout: Duration,
    pub requires_auth: bool,
    pub validation_rules: Vec<ResponseValidationRule>,
}

/// Rules for validating API responses
#[derive(Debug, Clone)]
pub enum ResponseValidationRule {
    JsonFieldExists(String),
    JsonFieldEquals(String, Value),
    JsonFieldType(String, String), // field_path, expected_type
    HeaderExists(String),
    HeaderEquals(String, String),
    ResponseTimeMax(Duration),
    BodyContains(String),
    BodyNotEmpty,
}

impl ApiCertificationSuite {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            auth_token: None,
        }
    }

    /// Execute complete API certification suite
    pub async fn execute_comprehensive_tests(&mut self) -> Result<Vec<ApiTestResult>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = Vec::new();

        // Get all test definitions
        let tests = self.define_comprehensive_tests();

        for test in tests {
            let result = self.execute_single_test(test).await?;
            results.push(result);

            // Short delay between tests to avoid overwhelming the server
            sleep(Duration::from_millis(100)).await;
        }

        Ok(results)
    }

    /// Define all comprehensive API tests
    fn define_comprehensive_tests(&self) -> Vec<ApiEndpointTest> {
        vec![
            // Health and Status Tests
            self.health_check_test(),
            self.metrics_endpoint_test(),
            self.readiness_test(),

            // Authentication Tests
            self.login_test(),
            self.logout_test(),
            self.token_refresh_test(),
            self.unauthorized_access_test(),

            // Chat API Tests
            self.create_chat_session_test(),
            self.send_message_test(),
            self.get_chat_history_test(),
            self.delete_chat_session_test(),

            // Streaming Tests
            self.chat_stream_test(),
            self.sse_connection_test(),

            // Tool Integration Tests
            self.list_available_tools_test(),
            self.execute_tool_test(),
            self.tool_call_history_test(),

            // File Upload Tests
            self.upload_file_test(),
            self.file_processing_test(),
            self.get_file_status_test(),

            // Settings and Configuration
            self.get_settings_test(),
            self.update_settings_test(),

            // Error Handling Tests
            self.malformed_request_test(),
            self.large_payload_test(),
            self.rate_limiting_test(),
        ]
    }

    /// Health check endpoint test
    fn health_check_test(&self) -> ApiEndpointTest {
        ApiEndpointTest {
            name: "Health Check".to_string(),
            method: "GET",
            path: "/health".to_string(),
            headers: HashMap::new(),
            body: None,
            expected_status: StatusCode::OK,
            timeout: Duration::from_secs(5),
            requires_auth: false,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("status".to_string()),
                ResponseValidationRule::JsonFieldEquals("status".to_string(), json!("healthy")),
                ResponseValidationRule::ResponseTimeMax(Duration::from_millis(500)),
            ],
        }
    }

    /// Metrics endpoint test
    fn metrics_endpoint_test(&self) -> ApiEndpointTest {
        ApiEndpointTest {
            name: "Metrics Endpoint".to_string(),
            method: "GET",
            path: "/metrics".to_string(),
            headers: HashMap::new(),
            body: None,
            expected_status: StatusCode::OK,
            timeout: Duration::from_secs(10),
            requires_auth: false,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("uptime".to_string()),
                ResponseValidationRule::JsonFieldExists("memory_usage".to_string()),
                ResponseValidationRule::JsonFieldType("uptime".to_string(), "number".to_string()),
            ],
        }
    }

    /// Readiness test
    fn readiness_test(&self) -> ApiEndpointTest {
        ApiEndpointTest {
            name: "Readiness Check".to_string(),
            method: "GET",
            path: "/ready".to_string(),
            headers: HashMap::new(),
            body: None,
            expected_status: StatusCode::OK,
            timeout: Duration::from_secs(10),
            requires_auth: false,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("database".to_string()),
                ResponseValidationRule::JsonFieldExists("redis".to_string()),
                ResponseValidationRule::JsonFieldEquals("database".to_string(), json!("connected")),
                ResponseValidationRule::JsonFieldEquals("redis".to_string(), json!("connected")),
            ],
        }
    }

    /// Login test
    fn login_test(&self) -> ApiEndpointTest {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        ApiEndpointTest {
            name: "User Login".to_string(),
            method: "POST",
            path: "/api/auth/login".to_string(),
            headers,
            body: Some(json!({
                "username": "test_user",
                "password": "test_password"
            })),
            expected_status: StatusCode::OK,
            timeout: Duration::from_secs(10),
            requires_auth: false,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("token".to_string()),
                ResponseValidationRule::JsonFieldExists("user_id".to_string()),
                ResponseValidationRule::JsonFieldExists("expires_at".to_string()),
            ],
        }
    }

    /// Logout test
    fn logout_test(&self) -> ApiEndpointTest {
        ApiEndpointTest {
            name: "User Logout".to_string(),
            method: "POST",
            path: "/api/auth/logout".to_string(),
            headers: HashMap::new(),
            body: None,
            expected_status: StatusCode::OK,
            timeout: Duration::from_secs(5),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("message".to_string()),
            ],
        }
    }

    /// Token refresh test
    fn token_refresh_test(&self) -> ApiEndpointTest {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        ApiEndpointTest {
            name: "Token Refresh".to_string(),
            method: "POST",
            path: "/api/auth/refresh".to_string(),
            headers,
            body: Some(json!({
                "refresh_token": "valid_refresh_token"
            })),
            expected_status: StatusCode::OK,
            timeout: Duration::from_secs(10),
            requires_auth: false,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("token".to_string()),
                ResponseValidationRule::JsonFieldExists("expires_at".to_string()),
            ],
        }
    }

    /// Unauthorized access test
    fn unauthorized_access_test(&self) -> ApiEndpointTest {
        ApiEndpointTest {
            name: "Unauthorized Access".to_string(),
            method: "GET",
            path: "/api/chat/sessions".to_string(),
            headers: HashMap::new(),
            body: None,
            expected_status: StatusCode::UNAUTHORIZED,
            timeout: Duration::from_secs(5),
            requires_auth: false, // Intentionally no auth to test security
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("error".to_string()),
            ],
        }
    }

    /// Create chat session test
    fn create_chat_session_test(&self) -> ApiEndpointTest {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        ApiEndpointTest {
            name: "Create Chat Session".to_string(),
            method: "POST",
            path: "/api/chat/sessions".to_string(),
            headers,
            body: Some(json!({
                "title": "Test Chat Session",
                "model": "gpt-4",
                "temperature": 0.7
            })),
            expected_status: StatusCode::CREATED,
            timeout: Duration::from_secs(10),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("session_id".to_string()),
                ResponseValidationRule::JsonFieldExists("title".to_string()),
                ResponseValidationRule::JsonFieldExists("created_at".to_string()),
            ],
        }
    }

    /// Send message test
    fn send_message_test(&self) -> ApiEndpointTest {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        ApiEndpointTest {
            name: "Send Chat Message".to_string(),
            method: "POST",
            path: "/api/chat/sessions/{session_id}/messages".to_string(),
            headers,
            body: Some(json!({
                "content": "Hello, this is a test message",
                "role": "user"
            })),
            expected_status: StatusCode::OK,
            timeout: Duration::from_secs(30),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("message_id".to_string()),
                ResponseValidationRule::JsonFieldExists("content".to_string()),
                ResponseValidationRule::JsonFieldExists("timestamp".to_string()),
            ],
        }
    }

    /// Get chat history test
    fn get_chat_history_test(&self) -> ApiEndpointTest {
        ApiEndpointTest {
            name: "Get Chat History".to_string(),
            method: "GET",
            path: "/api/chat/sessions/{session_id}/messages".to_string(),
            headers: HashMap::new(),
            body: None,
            expected_status: StatusCode::OK,
            timeout: Duration::from_secs(10),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("messages".to_string()),
                ResponseValidationRule::JsonFieldType("messages".to_string(), "array".to_string()),
            ],
        }
    }

    /// Delete chat session test
    fn delete_chat_session_test(&self) -> ApiEndpointTest {
        ApiEndpointTest {
            name: "Delete Chat Session".to_string(),
            method: "DELETE",
            path: "/api/chat/sessions/{session_id}".to_string(),
            headers: HashMap::new(),
            body: None,
            expected_status: StatusCode::NO_CONTENT,
            timeout: Duration::from_secs(10),
            requires_auth: true,
            validation_rules: vec![],
        }
    }

    /// Chat stream test
    fn chat_stream_test(&self) -> ApiEndpointTest {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Accept".to_string(), "text/event-stream".to_string());

        ApiEndpointTest {
            name: "Chat Streaming".to_string(),
            method: "POST",
            path: "/api/chat/stream".to_string(),
            headers,
            body: Some(json!({
                "message": "Tell me a short story",
                "stream": true
            })),
            expected_status: StatusCode::OK,
            timeout: Duration::from_secs(60),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::HeaderExists("Content-Type".to_string()),
                ResponseValidationRule::HeaderEquals("Content-Type".to_string(), "text/event-stream".to_string()),
            ],
        }
    }

    /// SSE connection test
    fn sse_connection_test(&self) -> ApiEndpointTest {
        let mut headers = HashMap::new();
        headers.insert("Accept".to_string(), "text/event-stream".to_string());

        ApiEndpointTest {
            name: "SSE Connection".to_string(),
            method: "GET",
            path: "/api/events".to_string(),
            headers,
            body: None,
            expected_status: StatusCode::OK,
            timeout: Duration::from_secs(15),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::HeaderExists("Content-Type".to_string()),
                ResponseValidationRule::HeaderEquals("Content-Type".to_string(), "text/event-stream".to_string()),
                ResponseValidationRule::HeaderExists("Cache-Control".to_string()),
            ],
        }
    }

    /// List available tools test
    fn list_available_tools_test(&self) -> ApiEndpointTest {
        ApiEndpointTest {
            name: "List Available Tools".to_string(),
            method: "GET",
            path: "/api/tools".to_string(),
            headers: HashMap::new(),
            body: None,
            expected_status: StatusCode::OK,
            timeout: Duration::from_secs(10),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("tools".to_string()),
                ResponseValidationRule::JsonFieldType("tools".to_string(), "array".to_string()),
            ],
        }
    }

    /// Execute tool test
    fn execute_tool_test(&self) -> ApiEndpointTest {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        ApiEndpointTest {
            name: "Execute Tool".to_string(),
            method: "POST",
            path: "/api/tools/execute".to_string(),
            headers,
            body: Some(json!({
                "tool_name": "time::now",
                "parameters": {}
            })),
            expected_status: StatusCode::OK,
            timeout: Duration::from_secs(30),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("result".to_string()),
                ResponseValidationRule::JsonFieldExists("execution_time".to_string()),
            ],
        }
    }

    /// Tool call history test
    fn tool_call_history_test(&self) -> ApiEndpointTest {
        ApiEndpointTest {
            name: "Tool Call History".to_string(),
            method: "GET",
            path: "/api/tools/history".to_string(),
            headers: HashMap::new(),
            body: None,
            expected_status: StatusCode::OK,
            timeout: Duration::from_secs(10),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("history".to_string()),
                ResponseValidationRule::JsonFieldType("history".to_string(), "array".to_string()),
            ],
        }
    }

    /// Upload file test
    fn upload_file_test(&self) -> ApiEndpointTest {
        ApiEndpointTest {
            name: "File Upload".to_string(),
            method: "POST",
            path: "/api/files/upload".to_string(),
            headers: HashMap::new(),
            body: Some(json!({
                "file_name": "test.txt",
                "content_type": "text/plain",
                "data": "VGVzdCBmaWxlIGNvbnRlbnQ=" // Base64 encoded
            })),
            expected_status: StatusCode::CREATED,
            timeout: Duration::from_secs(30),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("file_id".to_string()),
                ResponseValidationRule::JsonFieldExists("upload_url".to_string()),
            ],
        }
    }

    /// File processing test
    fn file_processing_test(&self) -> ApiEndpointTest {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        ApiEndpointTest {
            name: "File Processing".to_string(),
            method: "POST",
            path: "/api/files/{file_id}/process".to_string(),
            headers,
            body: Some(json!({
                "operation": "analyze",
                "options": {
                    "extract_text": true,
                    "generate_summary": true
                }
            })),
            expected_status: StatusCode::ACCEPTED,
            timeout: Duration::from_secs(30),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("job_id".to_string()),
                ResponseValidationRule::JsonFieldExists("status".to_string()),
            ],
        }
    }

    /// Get file status test
    fn get_file_status_test(&self) -> ApiEndpointTest {
        ApiEndpointTest {
            name: "Get File Status".to_string(),
            method: "GET",
            path: "/api/files/{file_id}/status".to_string(),
            headers: HashMap::new(),
            body: None,
            expected_status: StatusCode::OK,
            timeout: Duration::from_secs(10),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("file_id".to_string()),
                ResponseValidationRule::JsonFieldExists("status".to_string()),
                ResponseValidationRule::JsonFieldExists("processed_at".to_string()),
            ],
        }
    }

    /// Get settings test
    fn get_settings_test(&self) -> ApiEndpointTest {
        ApiEndpointTest {
            name: "Get User Settings".to_string(),
            method: "GET",
            path: "/api/settings".to_string(),
            headers: HashMap::new(),
            body: None,
            expected_status: StatusCode::OK,
            timeout: Duration::from_secs(10),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("user_id".to_string()),
                ResponseValidationRule::JsonFieldExists("preferences".to_string()),
            ],
        }
    }

    /// Update settings test
    fn update_settings_test(&self) -> ApiEndpointTest {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        ApiEndpointTest {
            name: "Update User Settings".to_string(),
            method: "PUT",
            path: "/api/settings".to_string(),
            headers,
            body: Some(json!({
                "preferences": {
                    "theme": "dark",
                    "language": "en",
                    "notifications": true
                }
            })),
            expected_status: StatusCode::OK,
            timeout: Duration::from_secs(10),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("updated_at".to_string()),
                ResponseValidationRule::JsonFieldExists("preferences".to_string()),
            ],
        }
    }

    /// Malformed request test
    fn malformed_request_test(&self) -> ApiEndpointTest {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        ApiEndpointTest {
            name: "Malformed Request Handling".to_string(),
            method: "POST",
            path: "/api/chat/sessions".to_string(),
            headers,
            body: Some(json!("invalid json structure")),
            expected_status: StatusCode::BAD_REQUEST,
            timeout: Duration::from_secs(5),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("error".to_string()),
                ResponseValidationRule::JsonFieldExists("message".to_string()),
            ],
        }
    }

    /// Large payload test
    fn large_payload_test(&self) -> ApiEndpointTest {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let large_content = "x".repeat(1024 * 1024); // 1MB of content

        ApiEndpointTest {
            name: "Large Payload Handling".to_string(),
            method: "POST",
            path: "/api/chat/sessions/{session_id}/messages".to_string(),
            headers,
            body: Some(json!({
                "content": large_content,
                "role": "user"
            })),
            expected_status: StatusCode::PAYLOAD_TOO_LARGE,
            timeout: Duration::from_secs(30),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::JsonFieldExists("error".to_string()),
            ],
        }
    }

    /// Rate limiting test
    fn rate_limiting_test(&self) -> ApiEndpointTest {
        ApiEndpointTest {
            name: "Rate Limiting".to_string(),
            method: "GET",
            path: "/api/chat/sessions".to_string(),
            headers: HashMap::new(),
            body: None,
            expected_status: StatusCode::OK, // First request should succeed
            timeout: Duration::from_secs(5),
            requires_auth: true,
            validation_rules: vec![
                ResponseValidationRule::HeaderExists("X-RateLimit-Limit".to_string()),
                ResponseValidationRule::HeaderExists("X-RateLimit-Remaining".to_string()),
            ],
        }
    }

    /// Execute a single API test
    async fn execute_single_test(&mut self, test: ApiEndpointTest) -> Result<ApiTestResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();

        // Set up authentication if required
        if test.requires_auth && self.auth_token.is_none() {
            self.authenticate().await?;
        }

        // Build request
        let url = format!("{}{}", self.base_url, test.path);
        let mut request_builder = match test.method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "DELETE" => self.client.delete(&url),
            "PATCH" => self.client.patch(&url),
            _ => return Err("Unsupported HTTP method".into()),
        };

        // Add headers
        for (key, value) in &test.headers {
            request_builder = request_builder.header(key, value);
        }

        // Add authentication header
        if test.requires_auth && let Some(token) = &self.auth_token {
            request_builder =
                request_builder.header("Authorization", format!("Bearer {}", token));
        }

        // Add body if present
        if let Some(body) = &test.body {
            request_builder = request_builder.json(body);
        }

        // Execute request
        let response = request_builder.timeout(test.timeout).send().await?;
        let status_code = response.status().as_u16();
        let headers = response.headers().clone();
        let response_body = response.text().await?;

        let duration = start_time.elapsed();

        // Parse response body as JSON if possible
        let parsed_body: Option<Value> = serde_json::from_str(&response_body).ok();

        // Validate response
        let success = self.validate_response(
            &test,
            status_code,
            &headers,
            &response_body,
            &parsed_body,
            duration,
        );

        let message = if success {
            "Test passed".to_string()
        } else {
            format!("Test failed: expected status {}, got {}", test.expected_status, status_code)
        };

        Ok(ApiTestResult {
            endpoint: test.path,
            method: test.method.to_string(),
            status_code,
            response_time: duration,
            success,
            message,
            response_body: parsed_body,
        })
    }

    /// Authenticate and store token
    async fn authenticate(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Mock authentication - in real implementation this would call the actual auth endpoint
        self.auth_token = Some("mock_auth_token_12345".to_string());
        Ok(())
    }

    /// Validate API response against test rules
    fn validate_response(
        &self,
        test: &ApiEndpointTest,
        status_code: u16,
        headers: &reqwest::header::HeaderMap,
        response_body: &str,
        parsed_body: &Option<Value>,
        duration: Duration,
    ) -> bool {
        // Check status code
        if status_code != test.expected_status.as_u16() {
            return false;
        }

        // Apply validation rules
        for rule in &test.validation_rules {
            if !self.apply_validation_rule(rule, headers, response_body, parsed_body, duration) {
                return false;
            }
        }

        true
    }

    /// Apply a single validation rule
    fn apply_validation_rule(
        &self,
        rule: &ResponseValidationRule,
        headers: &reqwest::header::HeaderMap,
        response_body: &str,
        parsed_body: &Option<Value>,
        duration: Duration,
    ) -> bool {
        match rule {
            ResponseValidationRule::JsonFieldExists(field_path) => {
                if let Some(json) = parsed_body {
                    self.json_field_exists(json, field_path)
                } else {
                    false
                }
            }
            ResponseValidationRule::JsonFieldEquals(field_path, expected) => {
                if let Some(json) = parsed_body {
                    self.json_field_equals(json, field_path, expected)
                } else {
                    false
                }
            }
            ResponseValidationRule::JsonFieldType(field_path, expected_type) => {
                if let Some(json) = parsed_body {
                    self.json_field_type(json, field_path, expected_type)
                } else {
                    false
                }
            }
            ResponseValidationRule::HeaderExists(header_name) => {
                headers.contains_key(header_name)
            }
            ResponseValidationRule::HeaderEquals(header_name, expected) => {
                headers.get(header_name)
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v == expected)
                    .unwrap_or(false)
            }
            ResponseValidationRule::ResponseTimeMax(max_duration) => {
                duration <= *max_duration
            }
            ResponseValidationRule::BodyContains(content) => {
                response_body.contains(content)
            }
            ResponseValidationRule::BodyNotEmpty => {
                !response_body.is_empty()
            }
        }
    }

    /// Check if JSON field exists
    fn json_field_exists(&self, json: &Value, field_path: &str) -> bool {
        let parts: Vec<&str> = field_path.split('.').collect();
        self.navigate_json_path(json, &parts).is_some()
    }

    /// Check if JSON field equals expected value
    fn json_field_equals(&self, json: &Value, field_path: &str, expected: &Value) -> bool {
        let parts: Vec<&str> = field_path.split('.').collect();
        self.navigate_json_path(json, &parts)
            .map(|value| value == expected)
            .unwrap_or(false)
    }

    /// Check JSON field type
    fn json_field_type(&self, json: &Value, field_path: &str, expected_type: &str) -> bool {
        let parts: Vec<&str> = field_path.split('.').collect();
        self.navigate_json_path(json, &parts)
            .map(|value| {
                match expected_type {
                    "string" => value.is_string(),
                    "number" => value.is_number(),
                    "boolean" => value.is_boolean(),
                    "array" => value.is_array(),
                    "object" => value.is_object(),
                    "null" => value.is_null(),
                    _ => false,
                }
            })
            .unwrap_or(false)
    }

    /// Navigate JSON path to find nested field
    fn navigate_json_path<'a>(&self, json: &'a Value, path: &[&str]) -> Option<&'a Value> {
        path.iter().try_fold(json, |current, &key| current.get(key))
    }
}

/// Execute API certification tests for the given environment
pub async fn execute_api_test(
    test_id: &str,
    _environment_id: &str,
) -> Result<crate::certification::CertificationResult, Box<dyn std::error::Error + Send + Sync>> {
    let base_url = match std::env::var("TEST_BASE_URL") {
        Ok(url) => url,
        Err(_) => "http://localhost:3001".to_string(),
    };

    let mut suite = ApiCertificationSuite::new(base_url);
    let results = suite.execute_comprehensive_tests().await?;

    // Find the specific test result
    let test_result = results.iter()
        .find(|r| r.endpoint.contains(test_id) || r.method.to_lowercase().contains(test_id))
        .cloned()
        .unwrap_or_else(|| ApiTestResult {
            endpoint: test_id.to_string(),
            method: "UNKNOWN".to_string(),
            status_code: 500,
            response_time: Duration::from_secs(0),
            success: false,
            message: "Test not found".to_string(),
            response_body: None,
        });

    Ok(crate::certification::CertificationResult {
        test_id: test_id.to_string(),
        success: test_result.success,
        duration: test_result.response_time,
        message: test_result.message,
        details: Some(json!({
            "endpoint": test_result.endpoint,
            "method": test_result.method,
            "status_code": test_result.status_code,
            "response_body": test_result.response_body
        })),
        artifacts: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_suite_creation() {
        let suite = ApiCertificationSuite::new("http://localhost:3001".to_string());
        assert_eq!(suite.base_url, "http://localhost:3001");
    }

    #[test]
    fn test_comprehensive_tests_defined() {
        let suite = ApiCertificationSuite::new("http://localhost:3001".to_string());
        let tests = suite.define_comprehensive_tests();
        assert!(!tests.is_empty());

        // Should include all major test categories
        let test_names: Vec<String> = tests.iter().map(|t| t.name.clone()).collect();
        assert!(test_names.iter().any(|n| n.contains("Health")));
        assert!(test_names.iter().any(|n| n.contains("Login")));
        assert!(test_names.iter().any(|n| n.contains("Chat")));
        assert!(test_names.iter().any(|n| n.contains("Tool")));
    }

    #[test]
    fn test_validation_rules() {
        let suite = ApiCertificationSuite::new("http://localhost:3001".to_string());

        // Test JSON field existence
        let json = json!({"status": "healthy", "nested": {"value": 123}});
        assert!(suite.json_field_exists(&json, "status"));
        assert!(suite.json_field_exists(&json, "nested.value"));
        assert!(!suite.json_field_exists(&json, "nonexistent"));

        // Test JSON field equality
        assert!(suite.json_field_equals(&json, "status", &json!("healthy")));
        assert!(!suite.json_field_equals(&json, "status", &json!("unhealthy")));

        // Test JSON field type
        assert!(suite.json_field_type(&json, "status", "string"));
        assert!(suite.json_field_type(&json, "nested.value", "number"));
        assert!(!suite.json_field_type(&json, "status", "number"));
    }
}
