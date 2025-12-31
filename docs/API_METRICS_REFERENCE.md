# Test Execution Metrics API Reference

## Overview

This document provides comprehensive reference documentation for the Test Execution Metrics API (v2), part of the advanced testing infrastructure monitoring system.

## Base URL

All API endpoints are available under the `/api/v2/` prefix:

```
http://localhost:8080/api/v2/
```

## Authentication

Currently no authentication is required for development environments. Production deployments should implement appropriate authentication mechanisms.

## Core API Endpoints

### 1. Aggregated Metrics

**GET** `/api/v2/metrics/aggregate`

Get aggregated metrics across multiple test runs with advanced filtering capabilities.

**Parameters:**
- `days` (optional, number): Number of days to include (default: 30)
- `environment` (optional, string): Filter by environment name
- `mode` (optional, string): Filter by test execution mode

**Response:**
```json
{
  "time_range": "Last 30 days",
  "total_runs": 145,
  "environments": ["development", "staging", "production"],
  "coverage_stats": {
    "average_coverage": 87.5,
    "min_coverage": 78.2,
    "max_coverage": 95.1,
    "coverage_trend": "Up"
  },
  "performance_stats": {
    "average_duration": 245000,
    "fastest_run": 180000,
    "slowest_run": 420000,
    "regression_count": 3
  },
  "reliability_stats": {
    "overall_success_rate": 94.8,
    "flaky_test_count": 12,
    "total_test_failures": 23
  },
  "generated_at": "2024-01-15T10:30:00Z"
}
```

### 2. Test Run Comparison

**GET** `/api/v2/metrics/compare`

Compare metrics between multiple test runs to identify patterns and differences.

**Parameters:**
- `run_ids` (required, string): Comma-separated list of test run IDs

**Response:**
```json
{
  "run_ids": ["run_123", "run_124"],
  "comparison_type": "multi_run",
  "coverage_comparison": [
    {
      "run_id": "run_123",
      "overall_coverage": 85.2,
      "rust_coverage": 88.1,
      "typescript_coverage": 82.3,
      "delta_from_previous": null
    }
  ],
  "performance_comparison": [...],
  "reliability_comparison": [...],
  "recommendations": [
    "Some test runs have significantly lower coverage. Investigate test execution completeness."
  ],
  "generated_at": "2024-01-15T10:30:00Z"
}
```

### 3. Pattern Analysis

**GET** `/api/v2/analytics/test-patterns`

Analyze test execution patterns and identify insights about testing behavior.

**Parameters:**
- `days` (optional, number): Analysis period in days (default: 30)

**Response:**
```json
{
  "analysis_period": "Last 30 days",
  "total_runs_analyzed": 98,
  "execution_patterns": {
    "peak_execution_hours": [9, 10, 14, 15],
    "common_execution_modes": ["Full", "Integration", "Unit"],
    "environment_distribution": {
      "development": 45,
      "staging": 32,
      "production": 21
    },
    "duration_patterns": [
      "Average duration: 245000.00ms",
      "Standard deviation: 85000.00ms",
      "High duration variability detected"
    ]
  },
  "failure_patterns": {
    "most_common_failures": [
      "Database connection timeout",
      "Network timeout in integration tests"
    ],
    "failure_by_environment": {
      "staging": 5,
      "development": 2
    },
    "failure_trends": ["Failure rate stable over time"],
    "repeat_failures": ["test_database_connectivity failing repeatedly"]
  },
  "coverage_patterns": {
    "coverage_by_environment": {
      "development": 85.2,
      "staging": 87.8
    },
    "coverage_stability": 92.3,
    "areas_needing_improvement": [
      "Error handling paths under-tested",
      "Edge cases in validation logic"
    ]
  },
  "performance_patterns": {
    "slowest_test_categories": ["Database integration tests"],
    "performance_by_environment": {
      "development": 180000.0,
      "staging": 245000.0
    },
    "regression_frequency": 3.2
  },
  "recommendations": [
    "Coverage is below 80%. Focus on increasing test coverage."
  ],
  "generated_at": "2024-01-15T10:30:00Z"
}
```

### 4. Failure Analysis

**GET** `/api/v2/analytics/failure-analysis`

Analyze failure patterns and root causes to improve system reliability.

**Parameters:**
- `days` (optional, number): Analysis period in days (default: 7)

**Response:**
```json
{
  "analysis_period": "Last 7 days",
  "total_failures": 15,
  "failure_rate": 8.5,
  "root_cause_analysis": {
    "infrastructure_failures": 5,
    "test_code_failures": 3,
    "environment_issues": 2,
    "dependency_failures": 1
  },
  "failure_timeline": [
    "2024-01-01 09:00: Database connection failure",
    "2024-01-01 14:30: Memory allocation error"
  ],
  "impact_analysis": {
    "affected_environments": ["staging", "development"],
    "blocked_features": ["User authentication"],
    "estimated_recovery_time": 3600
  },
  "recommended_actions": [
    "Increase database connection timeout",
    "Add retry logic for network operations"
  ],
  "generated_at": "2024-01-15T10:30:00Z"
}
```

### 5. Summary Reports

**GET** `/api/v2/reports/summary`

Generate comprehensive summary report with executive insights.

**Parameters:**
- `days` (optional, number): Report period in days (default: 30)

**Response:**
```json
{
  "report_period": "Last 30 days",
  "executive_summary": {
    "total_test_runs": 145,
    "overall_success_rate": 94.8,
    "average_coverage": 87.5,
    "total_test_time": 35550000,
    "critical_issues": 3
  },
  "quality_metrics": {
    "test_coverage_trend": "Up",
    "performance_trend": "Stable",
    "reliability_score": 94.8,
    "maintainability_score": 75.0
  },
  "key_insights": [
    "Average test coverage is 87.5%",
    "Overall test success rate is 94.8%"
  ],
  "action_items": [
    {
      "priority": "High",
      "category": "Coverage",
      "description": "Increase test coverage in critical paths",
      "estimated_effort": "2-3 days"
    }
  ],
  "generated_at": "2024-01-15T10:30:00Z",
  "generated_by": "Test Execution Monitoring System"
}
```

### 6. Search and Discovery

**GET** `/api/v2/search`

Search test runs with advanced filtering and full-text search capabilities.

**Parameters:**
- `q` (optional, string): Search query
- `limit` (optional, number): Results per page (default: 50)
- `offset` (optional, number): Pagination offset (default: 0)

**Response:**
```json
{
  "query": "database",
  "total_results": 23,
  "page_size": 50,
  "current_page": 0,
  "results": [...],
  "search_facets": {
    "environments": ["development", "staging"],
    "execution_modes": ["Full", "Unit", "Integration"],
    "date_ranges": ["Last 24 hours", "Last 7 days", "Last 30 days"]
  }
}
```

## Advanced Features

### 7. Custom Analytics Queries

**POST** `/api/v2/query`

Execute custom analytics queries with flexible parameters.

**Request Body:**
```json
{
  "query_name": "Custom Coverage Analysis",
  "query_type": "aggregation",
  "parameters": {
    "environment": "production",
    "metric_type": "coverage"
  },
  "output_format": "json"
}
```

**Response:**
```json
{
  "query_id": "query_abc123",
  "query_name": "Custom Coverage Analysis",
  "result_data": {
    "total_runs": 45,
    "average_coverage": 89.2,
    "environments": 1
  },
  "execution_time_ms": 150,
  "row_count": 45,
  "executed_at": "2024-01-15T10:30:00Z"
}
```

### 8. Real-Time Alerts

**POST** `/api/v2/alerts/subscribe`

Subscribe to real-time alerts for test execution events.

**Request Body:**
```json
{
  "channels": ["email", "slack"],
  "filters": {
    "environment": "production",
    "failure_threshold": "5"
  },
  "alert_types": ["regression", "failure", "coverage_drop"]
}
```

**Response:**
```json
{
  "subscription_id": "sub_xyz789",
  "status": "active",
  "channels": ["email", "slack"],
  "filters": {
    "environment": "production",
    "failure_threshold": "5"
  },
  "created_at": "2024-01-15T10:30:00Z"
}
```

### 9. Webhook Configuration

**POST** `/api/v2/alerts/webhook`

Configure webhooks for automated alert notifications.

**Request Body:**
```json
{
  "url": "https://your-app.com/webhook/test-alerts",
  "events": ["test_failure", "performance_regression", "coverage_drop"],
  "secret_token": "your-secret-token"
}
```

**Response:**
```json
{
  "webhook_id": "webhook_456",
  "status": "configured",
  "url": "https://your-app.com/webhook/test-alerts",
  "events": ["test_failure", "performance_regression", "coverage_drop"],
  "created_at": "2024-01-15T10:30:00Z",
  "last_test": null
}
```

## Query Types for Custom Analytics

### Aggregation Queries
Calculate summary statistics across test runs:
- Total runs by environment
- Average coverage across time periods
- Performance metrics aggregation

### Filtering Queries
Apply complex filters to narrow down results:
- Environment-specific filtering
- Date range filtering
- Test mode filtering

### Trend Analysis Queries
Analyze trends over time:
- Coverage trend analysis
- Performance trend analysis
- Failure rate trends

### Comparison Queries
Compare metrics between different time periods or environments:
- Before/after comparisons
- Environment comparisons
- Feature branch comparisons

## Error Handling

All API endpoints return appropriate HTTP status codes:

- `200 OK`: Successful request
- `400 Bad Request`: Invalid parameters or malformed request
- `404 Not Found`: Requested resource not found
- `500 Internal Server Error`: Server-side error

Error responses include details:

```json
{
  "error": "Invalid query parameters",
  "details": "Parameter 'days' must be a positive integer",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## Rate Limits

Current rate limits (development environment):
- 1000 requests per hour per client
- 10 concurrent requests per client

## Examples

### Example 1: Get Recent Performance Trends

```bash
curl -X GET "http://localhost:8080/api/v2/analytics/test-patterns?days=7" \
  -H "Accept: application/json"
```

### Example 2: Compare Two Test Runs

```bash
curl -X GET "http://localhost:8080/api/v2/metrics/compare?run_ids=run_123,run_124" \
  -H "Accept: application/json"
```

### Example 3: Execute Custom Query

```bash
curl -X POST "http://localhost:8080/api/v2/query" \
  -H "Content-Type: application/json" \
  -d '{
    "query_name": "Production Coverage Analysis",
    "query_type": "aggregation",
    "parameters": {
      "environment": "production",
      "days": 30
    }
  }'
```

### Example 4: Set Up Webhook

```bash
curl -X POST "http://localhost:8080/api/v2/alerts/webhook" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK",
    "events": ["test_failure", "performance_regression"],
    "secret_token": "your-secret-token"
  }'
```

## Integration Guide

### Using with CI/CD Pipelines

The metrics API can be integrated with CI/CD pipelines to:

1. **Automatic Reporting**: Set up post-build hooks to query summary reports
2. **Quality Gates**: Use coverage and reliability metrics as quality gates
3. **Trend Monitoring**: Track performance trends over deployments
4. **Alert Integration**: Configure webhooks for build notifications

### Dashboard Integration

The API powers the real-time monitoring dashboard with:

1. **Live Updates**: Real-time metrics for active test runs
2. **Historical Analysis**: Trend analysis and pattern recognition
3. **Interactive Exploration**: Search and filtering capabilities
4. **Export Features**: Download reports in multiple formats

### Third-Party Integration

Integrate with external tools:

1. **Monitoring Systems**: Send metrics to Prometheus/Grafana
2. **Chat Platforms**: Configure Slack/Teams notifications
3. **Issue Tracking**: Auto-create tickets for critical failures
4. **Analytics Platforms**: Export data for advanced analytics

This API provides comprehensive visibility into test execution patterns, enabling data-driven decisions for continuous improvement of testing infrastructure and code quality.