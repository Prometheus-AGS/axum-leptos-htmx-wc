# Quickstart Guide: Comprehensive Testing Infrastructure

**Date**: December 31, 2024
**Feature**: [spec.md](./spec.md)

This guide helps you quickly get started with the comprehensive testing infrastructure to achieve 100% code coverage and full system validation.

## Prerequisites

- Docker and Docker Compose installed
- Rust 1.75+ with cargo installed
- Bun (for TypeScript/frontend builds)
- Git for version control

## Quick Start (5 minutes)

### 1. Setup Test Environment

```bash
# Start all test services
docker-compose -f docker-compose.test.yaml up -d

# Verify services are healthy
docker-compose -f docker-compose.test.yaml ps
```

### 2. Run Basic Test Suite

```bash
# Execute comprehensive test suite
./tools/test-runner.sh --mode=full

# Or run specific test types
./tools/test-runner.sh --rust-only    # Rust backend tests only
./tools/test-runner.sh --ts-only      # TypeScript frontend tests only
./tools/test-runner.sh --e2e-only     # End-to-end UI tests only
```

### 3. View Test Results

```bash
# Generate and open coverage report
./tools/coverage-report.sh --open

# View latest test report (JSON format)
cat tests/coverage/latest-report.json | jq '.'
```

## Core Workflows

### For Developers: Fast Development Loop

```bash
# 1. Start test services (once per session)
docker-compose -f docker-compose.test.yaml up -d

# 2. Run changed code tests only (< 2 minutes)
./tools/test-runner.sh --mode=fast --changed-only

# 3. Check coverage for your changes
./tools/coverage-report.sh --diff --format=terminal
```

**Expected Output**:
```
✅ Backend Tests: 15/15 passed (100%)
✅ Frontend Tests: 8/8 passed (100%)
📊 Overall Coverage: 98.5% (+0.2%)
⏱️  Total Time: 1m 23s
```

### For QA: Full System Certification

```bash
# 1. Clean environment setup
./tools/setup-test-env.sh --clean

# 2. Run complete certification suite (10-15 minutes)
./tools/test-runner.sh --mode=certification

# 3. Generate comprehensive report
./tools/coverage-report.sh --format=html --include-trends
```

**Expected Output**:
```
🎯 Test Suite Results:
   ✅ Unit Tests: 145/145 passed
   ✅ Integration Tests: 67/67 passed
   ✅ E2E Tests: 23/23 passed
   ✅ Performance Tests: 12/12 passed

📊 Coverage Summary:
   🦀 Rust Backend: 100.0%
   📜 TypeScript Frontend: 99.8%
   🎯 Overall: 99.9%

⚡ Performance:
   📈 API Response Time: 95ms avg (-5ms from baseline)
   🚀 UI Load Time: 1.2s avg (+0.1s from baseline)
   💾 Memory Usage: 245MB peak

🏆 Quality Gates: ALL PASSED ✅
```

### For CI/CD: Automated Quality Gates

```yaml
# .github/workflows/test.yml example
- name: Run Comprehensive Tests
  run: |
    ./tools/setup-test-env.sh --ci
    ./tools/test-runner.sh --mode=full --output=junit
    ./tools/coverage-report.sh --format=xml

- name: Quality Gate Check
  run: |
    ./tools/quality-gate.sh --coverage-threshold=95 \
                           --performance-threshold=20 \
                           --fail-on-regression
```

## Test Configuration

### test-config.yaml Structure

```yaml
# test-config.yaml
environments:
  development:
    mode: fast
    parallel: true
    timeout: 300s
    services: [postgres, redis]

  integration:
    mode: full
    parallel: true
    timeout: 900s
    services: [postgres, redis, surreal, unstructured]

coverage:
  rust:
    tool: grcov
    format: [html, lcov, json]
    exclude_patterns:
      - "build.rs"
      - "tests/fixtures/*"
      - "target/*"

  typescript:
    tool: playwright-v8
    format: [html, json]
    exclude_patterns:
      - "node_modules/*"
      - "dist/*"
      - "*.config.ts"

quality_gates:
  coverage_threshold: 95.0
  test_reliability_threshold: 98.0
  performance_thresholds:
    response_time_degradation: 20
    throughput_degradation: 30
```

### Docker Compose Test Services

The existing `docker-compose.test.yaml` provides:

- **PostgreSQL**: Test database with pgvector extension
- **SurrealDB**: Document database for vector storage
- **Redis**: Caching and session storage
- **Unstructured API**: File processing service

All services run in isolated test network with tmpfs storage for speed.

## Test Organization

```
tests/
├── integration/          # Rust backend integration tests
│   ├── api/             # API endpoint tests
│   ├── database/        # Database operation tests
│   ├── services/        # Business logic tests
│   └── fixtures/        # Test data and utilities
├── e2e/                 # Frontend end-to-end tests
│   ├── specs/           # Test specifications
│   ├── pages/           # Page object models
│   └── utils/           # Test utilities
├── performance/         # Load and performance tests
└── coverage/           # Generated coverage reports
```

## Common Commands

### Test Execution

```bash
# Full test suite with coverage
./tools/test-runner.sh --mode=full --coverage

# Specific test categories
./tools/test-runner.sh --unit-only         # Unit tests only
./tools/test-runner.sh --integration-only  # Integration tests only
./tools/test-runner.sh --e2e-only         # End-to-end tests only

# Parallel execution (faster)
./tools/test-runner.sh --parallel --max-workers=4

# Debug mode with verbose output
./tools/test-runner.sh --debug --verbose
```

### Coverage Reports

```bash
# Generate all format reports
./tools/coverage-report.sh --all-formats

# Open HTML report in browser
./tools/coverage-report.sh --html --open

# Export for CI/CD systems
./tools/coverage-report.sh --xml --lcov --output=reports/
```

### Environment Management

```bash
# Clean setup (destroys and recreates)
./tools/setup-test-env.sh --clean

# Quick health check
./tools/setup-test-env.sh --health-check

# Cleanup after tests
./tools/cleanup-test-env.sh --preserve-logs
```

## Integration Examples

### API Testing (Rust)

```rust
// tests/integration/api/chat_test.rs
#[tokio::test]
async fn test_chat_streaming_with_tools() {
    let app = test_app().await;
    let client = TestClient::new(app);

    let response = client
        .post("/api/chat/stream")
        .json(&json!({
            "message": "What's the weather in Tokyo?",
            "tools_enabled": true
        }))
        .send()
        .await;

    assert_eq!(response.status(), 200);

    let events: Vec<SseEvent> = collect_sse_events(response).await;
    assert!(events.iter().any(|e| e.event == "tool_call.complete"));
    assert!(events.iter().any(|e| e.event == "message.delta"));
}
```

### UI Testing (TypeScript/Playwright)

```typescript
// tests/e2e/specs/chat-flow.spec.ts
import { test, expect } from '@playwright/test';

test('complete chat flow with tool usage', async ({ page }) => {
  await page.goto('http://localhost:3000');

  // Test streaming chat interface
  await page.locator('#message-input').fill('Search for AI news');
  await page.locator('#send-button').click();

  // Verify streaming updates
  await expect(page.locator('.message-delta')).toBeVisible();
  await expect(page.locator('.tool-call')).toBeVisible();

  // Verify final response
  await expect(page.locator('.message-complete')).toContainText('news');
});
```

## Performance Monitoring

### Regression Detection

The system automatically detects performance regressions using configurable thresholds:

- **Response Time**: 20% slower than baseline triggers alert
- **Throughput**: 30% reduction triggers alert
- **Memory Usage**: 25% increase triggers alert

### Performance Test Example

```rust
#[tokio::test]
async fn test_chat_performance_under_load() {
    let app = test_app().await;
    let client = TestClient::new(app);

    let start = Instant::now();

    // Simulate 100 concurrent requests
    let tasks: Vec<_> = (0..100)
        .map(|_| {
            let client = client.clone();
            tokio::spawn(async move {
                client.post("/api/chat").json(&test_message()).send().await
            })
        })
        .collect();

    let results = join_all(tasks).await;
    let duration = start.elapsed();

    // All requests should succeed
    assert!(results.iter().all(|r| r.is_ok()));

    // Performance assertion
    assert!(duration < Duration::from_secs(5), "Load test exceeded 5s");
}
```

## Troubleshooting

### Common Issues

**Tests timeout or fail to start:**
```bash
# Check service health
docker-compose -f docker-compose.test.yaml logs

# Restart services
docker-compose -f docker-compose.test.yaml restart
```

**Coverage reports missing:**
```bash
# Ensure coverage tools installed
cargo install grcov
npm install -g c8

# Check environment variables
echo $RUSTFLAGS  # Should include "-C instrument-coverage"
```

**E2E tests fail in CI:**
```bash
# Install browser dependencies
npx playwright install --with-deps

# Use headless mode in CI
HEADLESS=true ./tools/test-runner.sh --e2e-only
```

### Getting Help

- Check test logs: `tests/logs/`
- View coverage reports: `tests/coverage/latest-report.html`
- Performance metrics: `tests/performance/latest-metrics.json`

This infrastructure ensures your code changes maintain system quality and functionality through comprehensive automated testing.