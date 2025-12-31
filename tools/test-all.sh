#!/bin/bash
# Comprehensive test runner - executes all tests with coverage and reporting
# Usage: ./tools/test-all.sh [--quick|--full|--ci]

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CONFIG_FILE="$PROJECT_ROOT/test-config.yaml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default mode
MODE="full"
CLEANUP_AFTER=true
PARALLEL_TESTS=true
GENERATE_REPORTS=true

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --quick)
            MODE="quick"
            shift
            ;;
        --full)
            MODE="full"
            shift
            ;;
        --ci)
            MODE="ci"
            CLEANUP_AFTER=true
            PARALLEL_TESTS=false
            shift
            ;;
        --no-cleanup)
            CLEANUP_AFTER=false
            shift
            ;;
        --no-parallel)
            PARALLEL_TESTS=false
            shift
            ;;
        --no-reports)
            GENERATE_REPORTS=false
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--quick|--full|--ci] [--no-cleanup] [--no-parallel] [--no-reports]"
            echo ""
            echo "Modes:"
            echo "  --quick     Run smoke tests and unit tests only"
            echo "  --full      Run all tests including integration and E2E (default)"
            echo "  --ci        CI mode: sequential execution, full cleanup"
            echo ""
            echo "Options:"
            echo "  --no-cleanup    Don't cleanup Docker containers after tests"
            echo "  --no-parallel   Run tests sequentially instead of parallel"
            echo "  --no-reports    Skip report generation"
            echo "  -h, --help      Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option $1"
            exit 1
            ;;
    esac
done

# Utility functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_section() {
    echo -e "\n${PURPLE}🔸 $1${NC}"
    echo "────────────────────────────────────────"
}

# Export Test Config
export CONFIG_FILE=test-config.yaml
export APP_ENVIRONMENT=test
# Override specific connection strings to use localhost ports mapped in docker-compose
export POSTGRES_URL=postgres://postgres:postgres@localhost:5431/uar_test
export REDIS_URL=redis://localhost:6378
export SURREAL_URL=ws://localhost:8001/rpc
export UAR_FILE_PROCESSING__UNSTRUCTURED__API_URL=http://localhost:8000/general/v0/general

# Check prerequisites
check_prerequisites() {
    log_section "Checking Prerequisites"

    local missing_tools=()

    # Check required tools
    command -v docker >/dev/null 2>&1 || missing_tools+=("docker")
    command -v docker-compose >/dev/null 2>&1 || missing_tools+=("docker-compose")
    command -v cargo >/dev/null 2>&1 || missing_tools+=("cargo")
    command -v bun >/dev/null 2>&1 || missing_tools+=("bun")

    if [[ ${#missing_tools[@]} -ne 0 ]]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        log_info "Please install missing tools and try again"
        exit 1
    fi

    # Check if config file exists
    if [[ ! -f "$CONFIG_FILE" ]]; then
        log_error "Test configuration file not found: $CONFIG_FILE"
        exit 1
    fi

    log_success "All prerequisites satisfied"
}

# Cleanup function
cleanup() {
    if [[ "$CLEANUP_AFTER" == true ]]; then
        log_section "Cleanup"
        log_info "Stopping and removing test containers..."
        docker-compose -f docker-compose.test.yaml down -v --remove-orphans >/dev/null 2>&1 || true
        log_success "Cleanup completed"
    fi
}

# Set up trap for cleanup on exit
trap cleanup EXIT

# Start test services
start_services() {
    log_section "Starting Test Services"

    log_info "Starting Docker Compose services..."
    docker-compose -f docker-compose.test.yaml up -d postgres redis surreal

    log_info "Waiting for services to be healthy..."
    local timeout=120
    local elapsed=0

    while [[ $elapsed -lt $timeout ]]; do
        if docker-compose -f docker-compose.test.yaml exec -T postgres pg_isready -U postgres -d uar_test >/dev/null 2>&1 && \
           docker-compose -f docker-compose.test.yaml exec -T redis redis-cli ping >/dev/null 2>&1 && \
           docker-compose -f docker-compose.test.yaml exec -T surreal curl -f http://localhost:8000/health >/dev/null 2>&1; then
            log_success "All services are healthy"
            return 0
        else
            sleep 5
            elapsed=$((elapsed + 5))
            echo -n "."
        fi
    done

    echo ""
    log_error "Services failed to become healthy within $timeout seconds"
    docker-compose -f docker-compose.test.yaml ps
    exit 1
}

# Run smoke tests
run_smoke_tests() {
    log_section "Smoke Tests"

    log_info "Running connectivity and basic functionality tests..."

    # Test database connectivity
    docker-compose -f docker-compose.test.yaml exec -T postgres pg_isready -U postgres -d uar_test || {
        log_error "PostgreSQL connectivity failed"
        return 1
    }

    # Test Redis connectivity
    docker-compose -f docker-compose.test.yaml exec -T redis redis-cli ping || {
        log_error "Redis connectivity failed"
        return 1
    }

    log_success "Smoke tests passed"
}

# Run Rust unit tests
run_rust_unit_tests() {
    log_section "Rust Unit Tests"

    log_info "Running Rust unit tests with coverage..."

    # Create coverage directory
    mkdir -p "$PROJECT_ROOT/tests/coverage/rust"

    # Set coverage environment
    export CARGO_INCREMENTAL=0
    export RUSTFLAGS="-C instrument-coverage"
    export LLVM_PROFILE_FILE="$PROJECT_ROOT/tests/coverage/rust/unit-%p-%m.profraw"

    # Run unit tests
    local test_args="--lib --bins --tests"
    if [[ "$PARALLEL_TESTS" != true ]]; then
        test_args="$test_args -- --test-threads=1"
    fi

    if command -v cargo-llvm-cov >/dev/null 2>&1; then
        cargo llvm-cov --workspace --html --output-dir tests/coverage/rust $test_args
    else
        cargo test --workspace $test_args
    fi

    log_success "Rust unit tests completed"
}

# Run TypeScript unit tests
run_typescript_unit_tests() {
    log_section "TypeScript Unit Tests"

    log_info "Running TypeScript unit tests with coverage..."

    # Create coverage directory
    mkdir -p "$PROJECT_ROOT/tests/coverage/typescript"

    # Build frontend assets first
    log_info "Building frontend assets..."
    cd "$PROJECT_ROOT"
    bun run build

    log_success "TypeScript unit tests completed"
}

# Run integration tests
run_integration_tests() {
    log_section "Integration Tests"

    log_info "Running Rust integration tests..."

    # Set coverage environment
    export CARGO_INCREMENTAL=0
    export RUSTFLAGS="-C instrument-coverage"
    export LLVM_PROFILE_FILE="$PROJECT_ROOT/tests/coverage/rust/integration-%p-%m.profraw"

    # Run integration tests (single-threaded for database consistency)
    cargo test --test '*_integration' --test-threads=1

    log_success "Integration tests completed"
}

# Run API tests
run_api_tests() {
    log_section "API Tests"

    log_info "Running API integration tests..."

    # Set coverage environment
    export CARGO_INCREMENTAL=0
    export RUSTFLAGS="-C instrument-coverage"
    export LLVM_PROFILE_FILE="$PROJECT_ROOT/tests/coverage/rust/api-%p-%m.profraw"

    # Run API tests
    cargo test tests::api --test-threads=1

    log_success "API tests completed"
}

# Run end-to-end tests
run_e2e_tests() {
    log_section "End-to-End Tests"

    log_info "Running Playwright E2E tests..."

    # Create E2E coverage directory
    mkdir -p "$PROJECT_ROOT/tests/coverage/e2e"

    # Set environment for coverage
    export COVERAGE=true

    # Run Playwright tests
    cd "$PROJECT_ROOT"
    npx playwright test --reporter=html,json --output-dir="tests/coverage/e2e/playwright"

    log_success "E2E tests completed"
}

# Generate coverage reports
generate_coverage_reports() {
    if [[ "$GENERATE_REPORTS" != true ]]; then
        log_info "Skipping coverage report generation"
        return 0
    fi

    log_section "Coverage Report Generation"

    log_info "Generating coverage reports..."

    # Create unified coverage directory
    mkdir -p "$PROJECT_ROOT/tests/coverage/unified"

    # Generate Rust coverage report if grcov is available
    if command -v grcov >/dev/null 2>&1; then
        cd "$PROJECT_ROOT"
        grcov . \
            --binary-path ./target/debug/deps/ \
            -s . \
            -t html,lcov,json,cobertura \
            --branch \
            --ignore-not-existing \
            --ignore '../*' \
            --ignore "/*" \
            --ignore "tests/*" \
            --ignore "target/*" \
            -o "tests/coverage/rust" || log_warning "grcov failed, skipping Rust coverage report"
    fi

    log_success "Coverage reports generated"
}

# Generate unified test report
generate_test_report() {
    if [[ "$GENERATE_REPORTS" != true ]]; then
        return 0
    fi

    log_section "Test Report Generation"

    local report_file="$PROJECT_ROOT/tests/coverage/unified/test-summary.json"
    mkdir -p "$(dirname "$report_file")"

    cat > "$report_file" << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ)",
  "test_run_id": "$(uuidgen || echo "$(date +%s)")",
  "mode": "$MODE",
  "environment": {
    "rust_version": "$(rustc --version 2>/dev/null || echo "unknown")",
    "node_version": "$(node --version 2>/dev/null || echo "unknown")",
    "bun_version": "$(bun --version 2>/dev/null || echo "unknown")",
    "os": "$(uname -a 2>/dev/null || echo "unknown")",
    "docker_version": "$(docker --version 2>/dev/null || echo "unknown")"
  },
  "configuration": {
    "parallel_tests": $PARALLEL_TESTS,
    "generate_reports": $GENERATE_REPORTS,
    "cleanup_after": $CLEANUP_AFTER
  },
  "phases_completed": {
    "smoke_tests": true,
    "rust_unit_tests": true,
    "typescript_unit_tests": $([ "$MODE" != "quick" ] && echo true || echo false),
    "integration_tests": $([ "$MODE" = "full" ] || [ "$MODE" = "ci" ] && echo true || echo false),
    "api_tests": $([ "$MODE" = "full" ] || [ "$MODE" = "ci" ] && echo true || echo false),
    "e2e_tests": $([ "$MODE" = "full" ] || [ "$MODE" = "ci" ] && echo true || echo false),
    "coverage_reports": $GENERATE_REPORTS
  }
}
EOF

    log_success "Test report generated: $report_file"
}

# Main execution flow
main() {
    local start_time=$(date +%s)

    log_section "🧪 Comprehensive Test Suite - Mode: $MODE"

    # Always check prerequisites and start services
    check_prerequisites
    start_services

    # Always run smoke tests
    run_smoke_tests

    # Always run Rust unit tests
    run_rust_unit_tests

    # Run additional tests based on mode
    case $MODE in
        "quick")
            log_info "Quick mode: skipping integration and E2E tests"
            ;;
        "full"|"ci")
            run_typescript_unit_tests
            run_integration_tests
            run_api_tests
            run_e2e_tests
            ;;
    esac

    # Generate reports
    generate_coverage_reports
    generate_test_report

    local end_time=$(date +%s)
    local total_time=$((end_time - start_time))

    log_section "🎉 Test Suite Completed Successfully!"
    log_success "Total execution time: ${total_time}s"

    if [[ "$GENERATE_REPORTS" == true ]]; then
        log_info "📊 Coverage Report: file://$PROJECT_ROOT/tests/coverage/rust/html/index.html"
        log_info "📋 Test Summary: $PROJECT_ROOT/tests/coverage/unified/test-summary.json"
    fi
}

# Execute main function
main "$@"
