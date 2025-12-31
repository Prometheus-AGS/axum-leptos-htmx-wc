# Comprehensive Testing Infrastructure

This document describes the complete testing infrastructure for the axum-leptos-htmx-wc project, designed to achieve and maintain 100% code coverage across all components.

## Overview

The testing infrastructure provides:

- **100% Code Coverage**: Comprehensive testing for both Rust backend and TypeScript frontend
- **Real Service Integration**: Tests run against actual PostgreSQL, SurrealDB, Redis, and LLM services
- **Multi-Level Testing**: Unit, integration, API, and end-to-end tests
- **Automated CI/CD**: GitHub Actions workflows for continuous testing
- **Docker Integration**: Consistent testing environments via Docker Compose
- **Rich Reporting**: HTML, JSON, XML, and LCOV coverage reports

## Architecture

### Test Categories

1. **Unit Tests**
   - Rust: `cargo test --lib --bins --tests`
   - TypeScript: `bun test`
   - Fast, isolated component testing

2. **Integration Tests**
   - Rust: Tests in `tests/` directory with `_integration` suffix
   - Database integration with real PostgreSQL and SurrealDB
   - Redis caching and session management
   - Single-threaded execution for data consistency

3. **API Tests**
   - HTTP endpoint testing via `tests::api` module
   - Request/response validation
   - Authentication and authorization flows
   - Error handling and edge cases

4. **End-to-End Tests**
   - Playwright automation testing full user workflows
   - Cross-browser compatibility (Chromium focus)
   - UI interaction validation
   - Real application deployment testing

5. **Performance Tests**
   - Load testing and benchmarking
   - Response time validation
   - Resource usage monitoring
   - Regression detection

## Configuration Files

### `test-config.yaml`
Central configuration for test execution parameters, service settings, and coverage thresholds.

### `docker-compose.test.yaml`
Orchestrates test services with proper health checks:
- PostgreSQL with pgvector extension
- SurrealDB in memory mode
- Redis Stack for caching
- Unstructured API for document processing
- Main application with test environment

### Test Coverage Configuration
- **Rust**: `.cargo/config.toml` and `.grcovrc` for LLVM instrumentation
- **TypeScript**: Built-in coverage via Bun and c8
- **Playwright**: V8 coverage integration

## Test Execution

### Local Development

#### Quick Tests (5-10 minutes)
```bash
./tools/test-all.sh --quick
```
- Smoke tests and unit tests only
- Fast feedback for development

#### Full Test Suite (15-30 minutes)
```bash
./tools/test-all.sh --full
```
- All test categories including E2E
- Complete coverage analysis
- Performance benchmarks

#### CI Mode (Sequential execution)
```bash
./tools/test-all.sh --ci
```
- Non-parallel execution for CI environments
- Comprehensive logging
- Strict coverage requirements

### Coverage Reports

#### Generate All Coverage Reports
```bash
./tools/coverage.sh --unified
```

#### Rust Coverage Only
```bash
./tools/coverage.sh --rust-only
```

#### TypeScript Coverage Only
```bash
./tools/coverage.sh --typescript-only
```

#### Open Reports in Browser
```bash
./tools/coverage.sh --unified --open
```

### Docker-Based Testing

#### Full Docker Integration
```bash
docker-compose -f docker-compose.test.yaml up --build
docker-compose -f docker-compose.test.yaml run test-runner
```

#### Individual Services
```bash
# Start dependencies only
docker-compose -f docker-compose.test.yaml up -d postgres redis surreal

# Run tests against external services
./tools/test-all.sh --full
```

## Continuous Integration

### GitHub Actions Workflows

#### 1. Comprehensive Test Suite (`comprehensive-tests.yml`)
**Triggers**: Push to main/develop, PRs to main/develop, scheduled daily runs
**Duration**: 30-45 minutes
**Features**:
- Multi-job parallel execution
- Code quality checks (format, lint, security audit)
- Build verification across multiple Rust versions
- Full test suite with real services
- Performance benchmarking
- Docker integration tests
- Coverage threshold enforcement (80%+ required)
- Artifact uploads and PR comments

#### 2. Quick Test Suite (`quick-tests.yml`)
**Triggers**: Push to feature branches, draft PRs
**Duration**: 10-15 minutes
**Features**:
- Fast validation for development
- Basic compilation and format checks
- Unit tests and quick integration tests
- Draft PR specific validation with helpful comments

#### 3. Release Pipeline (`release.yml`)
**Triggers**: Tag pushes, release publications
**Duration**: 45-60 minutes
**Features**:
- Strict pre-release validation (90% coverage required)
- Multi-platform binary builds (Linux, macOS, Windows)
- Docker image creation and publishing
- GitHub release updates with artifacts
- Comprehensive release notes with test summaries

### Coverage Requirements

| Environment | Rust Backend | TypeScript Frontend | E2E Coverage |
|-------------|--------------|-------------------|--------------|
| Development | 80%+ | 75%+ | 70%+ |
| CI/CD | 85%+ | 80%+ | 75%+ |
| Release | 90%+ | 85%+ | 80%+ |

## Test Environment Setup

### Prerequisites

#### Local Development
```bash
# Rust toolchain with components
rustup component add rustfmt clippy llvm-tools-preview

# Coverage tools
cargo install grcov
cargo install cargo-llvm-cov

# Node.js and Bun
curl -fsSL https://bun.sh/install | bash

# Docker and Docker Compose
# Install via official documentation

# Playwright
npx playwright install --with-deps chromium
```

#### CI Environment
All tools are automatically installed via GitHub Actions.

### Environment Variables

#### Test Execution
```bash
# Database connections
DATABASE_URL=postgres://postgres:postgres@localhost:5431/uar_test
REDIS_URL=redis://localhost:6378
SURREAL_URL=ws://localhost:8001/rpc

# Test configuration
CONFIG_FILE=test-config.yaml
APP_ENVIRONMENT=test
TEST_MODE=true
COVERAGE=true

# Coverage instrumentation
CARGO_INCREMENTAL=0
RUSTFLAGS="-C instrument-coverage"
LLVM_PROFILE_FILE="tests/coverage/rust/coverage-%p-%m.profraw"
```

#### LLM Integration (Optional)
```bash
OPENAI_API_KEY=your_api_key_here
TAVILY_API_KEY=your_api_key_here
```

## Directory Structure

```
tests/
├── integration/           # Rust integration tests
│   ├── api/              # API endpoint tests
│   ├── database/         # Database integration tests
│   └── services/         # Service integration tests
├── e2e/                  # End-to-end tests
│   ├── specs/            # Playwright test specifications
│   ├── pages/            # Page object models
│   └── utils/            # Test utilities
├── performance/          # Performance and benchmark tests
├── coverage/            # Coverage reports
│   ├── rust/            # Rust coverage data
│   ├── typescript/      # TypeScript coverage data
│   ├── e2e/             # E2E coverage data
│   └── unified/         # Combined coverage reports
├── config/              # Test-specific configuration
└── fixtures/            # Test data and fixtures

tools/
├── test-all.sh         # Main test execution script
├── coverage.sh         # Coverage report generator
└── ...

.github/workflows/
├── comprehensive-tests.yml  # Full CI pipeline
├── quick-tests.yml         # Fast validation
└── release.yml             # Release pipeline
```

## Coverage Tools and Formats

### Rust Coverage
**Primary Tool**: `grcov`
**Alternative**: `cargo-llvm-cov`, `cargo-tarpaulin`
**Formats**: HTML, LCOV, JSON, Cobertura XML
**Configuration**: `.grcovrc`, `.cargo/config.toml`

### TypeScript Coverage
**Primary Tool**: Bun's built-in coverage
**Alternative**: `c8`, `nyc`
**Formats**: HTML, JSON, LCOV, text
**Integration**: Playwright V8 coverage for E2E

### Unified Reporting
**Tool**: Custom HTML dashboard
**Features**:
- Multi-format report aggregation
- Interactive coverage browsers
- Historical trend analysis
- Quality gate validation
- Direct links to detailed reports

## Quality Gates

### Automated Checks
1. **Format Compliance**: `cargo fmt`, `prettier`
2. **Linting**: `clippy`, `eslint`
3. **Type Safety**: TypeScript strict mode
4. **Security**: `cargo audit`, `bun audit`
5. **Coverage Thresholds**: Configurable per environment
6. **Performance**: Response time and throughput validation
7. **Docker Health**: Service connectivity and readiness

### Manual Gates
1. **Code Review**: Required for all PRs
2. **Architecture Review**: For significant changes
3. **Security Review**: For authentication/authorization changes
4. **Performance Review**: For database schema or API changes

## Troubleshooting

### Common Issues

#### Docker Services Not Starting
```bash
# Check service health
docker-compose -f docker-compose.test.yaml ps

# View service logs
docker-compose -f docker-compose.test.yaml logs postgres
docker-compose -f docker-compose.test.yaml logs redis
docker-compose -f docker-compose.test.yaml logs surreal

# Reset containers
docker-compose -f docker-compose.test.yaml down -v
docker-compose -f docker-compose.test.yaml up -d --build
```

#### Coverage Tool Issues
```bash
# Verify tool installation
grcov --version
cargo llvm-cov --version

# Check profraw files generated
find . -name "*.profraw" -type f

# Clear coverage cache
rm -rf tests/coverage/
rm -f *.profraw
```

#### Test Failures
```bash
# Run specific test category
cargo test --test config_integration -- --nocapture
cargo test tests::api -- --test-threads=1 --nocapture

# Enable debug logging
RUST_LOG=debug cargo test

# Playwright debugging
npx playwright test --debug
npx playwright test --headed --slow-mo=1000
```

#### Performance Issues
```bash
# Check resource usage
docker stats

# Analyze slow tests
cargo test -- --nocapture | grep -E "(test|elapsed)"

# Profile test execution
time ./tools/test-all.sh --quick
```

## Best Practices

### Test Writing
1. **Isolation**: Each test should be independent
2. **Determinism**: Tests should produce consistent results
3. **Clarity**: Test names should describe expected behavior
4. **Coverage**: Test both happy paths and error conditions
5. **Performance**: Keep unit tests fast (<10ms each)

### Coverage Goals
1. **Completeness**: Test all code paths including error handling
2. **Quality**: Focus on meaningful tests, not just coverage percentage
3. **Maintenance**: Keep tests updated with code changes
4. **Documentation**: Use tests as behavioral documentation

### CI/CD Integration
1. **Fast Feedback**: Quick tests for feature branches
2. **Comprehensive Validation**: Full tests for main branches
3. **Release Quality**: Strict requirements for releases
4. **Monitoring**: Track test execution times and success rates

## Metrics and Monitoring

### Key Metrics
- **Test Execution Time**: Track performance trends
- **Coverage Percentage**: Monitor quality improvements
- **Failure Rate**: Identify unstable tests
- **Resource Usage**: Optimize CI costs

### Dashboards
- **Coverage Reports**: `tests/coverage/unified/index.html`
- **Test Results**: GitHub Actions summary pages
- **Performance Trends**: Artifact-based historical data

### Alerts
- **Coverage Drop**: Below threshold warnings
- **Test Failures**: Immediate CI notifications
- **Performance Regression**: Automated benchmark comparisons

## Contributing

### Adding New Tests
1. Choose appropriate test category (unit/integration/e2e)
2. Follow naming conventions (`*_test.rs`, `*.test.ts`, `*.spec.ts`)
3. Ensure tests are deterministic and isolated
4. Update coverage thresholds if needed
5. Document complex test scenarios

### Modifying Test Infrastructure
1. Update configuration files as needed
2. Test changes locally before committing
3. Update documentation for new features
4. Verify CI workflows still pass
5. Consider backward compatibility

### Performance Optimization
1. Profile test execution to identify bottlenecks
2. Optimize Docker image sizes and startup times
3. Use caching effectively in CI
4. Balance thoroughness with execution speed
5. Monitor resource usage trends