# Data Model: Comprehensive Testing Infrastructure

**Date**: December 31, 2024
**Feature**: [spec.md](./spec.md)

## Core Entities

### TestSuite
**Purpose**: Collection of organized tests covering unit, integration, and end-to-end scenarios with coverage tracking

**Fields**:
- `id`: UUID - Unique identifier for the test suite
- `name`: String - Human-readable name (e.g., "Backend Integration Suite", "UI E2E Suite")
- `type`: Enum - Suite type (`unit`, `integration`, `e2e`, `performance`)
- `language`: Enum - Target language (`rust`, `typescript`, `mixed`)
- `status`: Enum - Current execution status (`pending`, `running`, `completed`, `failed`)
- `created_at`: Timestamp - When the suite was created
- `updated_at`: Timestamp - Last modification time
- `config`: JSON - Suite-specific configuration (timeout, retry count, etc.)

**Relationships**:
- Contains many `TestCase` entities
- Produces one `TestReport` per execution
- References one `TestEnvironment` for execution

**Validation Rules**:
- Name must be unique within project
- Type must match at least one contained test case type
- Config must be valid JSON with required fields per type

**State Transitions**:
```
pending → running → (completed | failed)
completed → running (re-execution)
failed → running (retry)
```

### TestCase
**Purpose**: Individual test with specific functionality validation and coverage contribution

**Fields**:
- `id`: UUID - Unique identifier
- `suite_id`: UUID - Foreign key to parent TestSuite
- `name`: String - Descriptive test name
- `file_path`: String - Relative path to test file
- `line_number`: Integer - Starting line number in file
- `type`: Enum - Test type (`unit`, `integration`, `e2e`, `performance`)
- `status`: Enum - Execution status (`pending`, `running`, `passed`, `failed`, `skipped`)
- `duration_ms`: Integer - Execution time in milliseconds
- `retry_count`: Integer - Number of retry attempts
- `tags`: Array[String] - Classification tags for filtering
- `dependencies`: Array[String] - Required services or other test cases

**Relationships**:
- Belongs to one `TestSuite`
- Can produce multiple `TestResult` entries (retries)
- May reference `CoverageData` entries

**Validation Rules**:
- File path must exist and be readable
- Duration must be non-negative
- Retry count cannot exceed suite configuration limits
- Dependencies must reference valid services or test case IDs

### CoverageReport
**Purpose**: Detailed analysis of code coverage percentages, uncovered lines, and coverage trends over time

**Fields**:
- `id`: UUID - Unique identifier
- `test_report_id`: UUID - Foreign key to associated TestReport
- `language`: Enum - Source language (`rust`, `typescript`)
- `total_lines`: Integer - Total lines of code analyzed
- `covered_lines`: Integer - Lines covered by tests
- `coverage_percentage`: Float - Calculated coverage (covered/total * 100)
- `uncovered_lines`: Array[LineRange] - Specific uncovered line ranges
- `excluded_patterns`: Array[String] - File patterns excluded from coverage
- `generated_at`: Timestamp - When coverage was calculated
- `report_format`: Enum - Output format (`html`, `xml`, `json`, `lcov`)
- `file_path`: String - Path to generated report file

**Relationships**:
- Belongs to one `TestReport`
- Contains many `FileCoverage` entries

**Validation Rules**:
- Coverage percentage must be between 0 and 100
- Covered lines cannot exceed total lines
- Report file must exist if file_path is provided
- Uncovered lines must reference valid source code locations

### TestEnvironment
**Purpose**: Isolated infrastructure setup including all required services, databases, and dependencies

**Fields**:
- `id`: UUID - Unique identifier
- `name`: String - Environment name (e.g., "integration-env-1", "e2e-chrome")
- `type`: Enum - Environment type (`docker`, `local`, `cloud`)
- `status`: Enum - Current status (`creating`, `ready`, `running`, `destroying`, `failed`)
- `config`: JSON - Environment configuration (service ports, resource limits)
- `services`: Array[ServiceConfig] - Required services and their configurations
- `health_checks`: Array[HealthCheck] - Service health validation rules
- `created_at`: Timestamp - Environment creation time
- `destroyed_at`: Timestamp - Environment cleanup time (nullable)
- `resource_limits`: ResourceLimits - CPU/memory constraints

**Relationships**:
- Used by multiple `TestSuite` executions
- Produces `EnvironmentLog` entries

**Validation Rules**:
- Name must be unique per type
- Config must contain required service definitions
- Resource limits must be within system constraints
- Health checks must specify valid endpoints and criteria

**State Transitions**:
```
creating → (ready | failed)
ready → running → ready
ready → destroying → destroyed
failed → destroying → destroyed
```

### QualityGate
**Purpose**: Automated checkpoint that validates test results and coverage thresholds before allowing deployments

**Fields**:
- `id`: UUID - Unique identifier
- `name`: String - Gate name (e.g., "deployment-gate", "merge-gate")
- `coverage_threshold`: Float - Minimum coverage percentage required
- `test_reliability_threshold`: Float - Minimum test pass rate required
- `performance_thresholds`: JSON - Performance regression limits
- `critical_tests`: Array[String] - Test case IDs that must pass
- `enabled`: Boolean - Whether gate is active
- `created_at`: Timestamp - Gate creation time
- `updated_at`: Timestamp - Last modification time

**Relationships**:
- Evaluates multiple `TestReport` entries
- Produces `QualityGateResult` entries

**Validation Rules**:
- Coverage threshold must be between 0 and 100
- Test reliability threshold must be between 0 and 100
- Performance thresholds must contain valid metric definitions
- Critical tests must reference existing test cases

### TestReport
**Purpose**: Comprehensive summary including test results, coverage metrics, performance data, and failure analysis

**Fields**:
- `id`: UUID - Unique identifier
- `execution_id`: String - Unique execution identifier for grouping
- `suite_id`: UUID - Foreign key to executed TestSuite
- `environment_id`: UUID - Foreign key to TestEnvironment used
- `status`: Enum - Overall execution status (`passed`, `failed`, `partial`)
- `started_at`: Timestamp - Execution start time
- `completed_at`: Timestamp - Execution completion time
- `total_tests`: Integer - Total number of tests executed
- `passed_tests`: Integer - Number of tests that passed
- `failed_tests`: Integer - Number of tests that failed
- `skipped_tests`: Integer - Number of tests skipped
- `overall_coverage`: Float - Aggregated coverage percentage
- `performance_metrics`: JSON - Response times, throughput, memory usage
- `failure_summary`: JSON - Categorized failure reasons and counts
- `regression_detected`: Boolean - Whether performance regressions found
- `report_url`: String - URL to detailed HTML report

**Relationships**:
- Belongs to one `TestSuite` and one `TestEnvironment`
- Contains one `CoverageReport`
- Evaluated by `QualityGate` entities

**Validation Rules**:
- Total tests must equal sum of passed, failed, and skipped
- Completion time must be after start time
- Overall coverage must be calculated from component coverage reports
- Performance metrics must follow defined schema

## Supporting Types

### LineRange
```json
{
  "file_path": "src/example.rs",
  "start_line": 42,
  "end_line": 58
}
```

### ServiceConfig
```json
{
  "name": "postgres",
  "image": "pgvector/pgvector:pg17",
  "ports": ["5432:5432"],
  "environment": {...},
  "health_check": {...}
}
```

### HealthCheck
```json
{
  "service": "postgres",
  "endpoint": "/health",
  "expected_status": 200,
  "timeout_ms": 5000,
  "retry_count": 3
}
```

### ResourceLimits
```json
{
  "cpu_limit": "2.0",
  "memory_limit": "4GB",
  "disk_limit": "10GB"
}
```

### FileCoverage
```json
{
  "file_path": "src/api/handlers.rs",
  "total_lines": 150,
  "covered_lines": 142,
  "coverage_percentage": 94.67,
  "uncovered_ranges": [...]
}
```

## Data Flow Relationships

1. **Test Execution Flow**:
   ```
   TestEnvironment → TestSuite → TestCase → TestResult → CoverageReport → TestReport
   ```

2. **Quality Validation Flow**:
   ```
   TestReport → QualityGate → QualityGateResult → Deployment Decision
   ```

3. **Trend Analysis Flow**:
   ```
   TestReport (historical) → Coverage Trends → Performance Trends → Health Metrics
   ```

## Storage Considerations

- **TestSuite** and **TestCase**: Relatively static, infrequent updates
- **TestReport** and **CoverageReport**: High volume, append-only for historical tracking
- **TestEnvironment**: Medium churn, frequent creation/destruction cycles
- **QualityGate**: Low volume, configuration-focused with occasional updates

This data model supports the comprehensive testing infrastructure requirements while maintaining clear relationships and enabling both real-time execution tracking and historical trend analysis.