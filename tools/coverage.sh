#!/bin/bash
# Dedicated coverage report generator
# Usage: ./tools/coverage.sh [--rust-only|--typescript-only|--unified]

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Default settings
RUST_COVERAGE=true
TYPESCRIPT_COVERAGE=true
UNIFIED_REPORT=true
OPEN_REPORT=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --rust-only)
            TYPESCRIPT_COVERAGE=false
            UNIFIED_REPORT=false
            shift
            ;;
        --typescript-only)
            RUST_COVERAGE=false
            UNIFIED_REPORT=false
            shift
            ;;
        --unified)
            RUST_COVERAGE=true
            TYPESCRIPT_COVERAGE=true
            UNIFIED_REPORT=true
            shift
            ;;
        --open)
            OPEN_REPORT=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--rust-only|--typescript-only|--unified] [--open]"
            echo ""
            echo "Options:"
            echo "  --rust-only        Generate Rust coverage report only"
            echo "  --typescript-only  Generate TypeScript coverage report only"
            echo "  --unified         Generate unified coverage report (default)"
            echo "  --open            Open reports in browser after generation"
            echo "  -h, --help        Show this help message"
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

# Check for required tools
check_tools() {
    log_section "Checking Coverage Tools"

    if [[ "$RUST_COVERAGE" == true ]]; then
        if ! command -v grcov >/dev/null 2>&1; then
            log_warning "grcov not found. Install with: cargo install grcov"
            log_info "Falling back to cargo-llvm-cov or cargo-tarpaulin if available"

            if ! command -v cargo-llvm-cov >/dev/null 2>&1 && ! command -v cargo-tarpaulin >/dev/null 2>&1; then
                log_error "No Rust coverage tool found. Please install grcov, cargo-llvm-cov, or cargo-tarpaulin."
                exit 1
            fi
        fi
    fi

    if [[ "$TYPESCRIPT_COVERAGE" == true ]]; then
        if ! command -v bun >/dev/null 2>&1; then
            log_error "bun not found. Please install bun for TypeScript coverage."
            exit 1
        fi
    fi

    log_success "Coverage tools check completed"
}

# Generate Rust coverage
generate_rust_coverage() {
    if [[ "$RUST_COVERAGE" != true ]]; then
        return 0
    fi

    log_section "Generating Rust Coverage Report"

    # Create coverage directories
    mkdir -p "$PROJECT_ROOT/tests/coverage/rust"

    cd "$PROJECT_ROOT"

    # Generate coverage report with available tool
    if command -v grcov >/dev/null 2>&1; then
        log_info "Generating coverage report with grcov..."

        # Check if we have profraw files from previous test runs
        local profraw_files
        profraw_files=$(find . -name "*.profraw" 2>/dev/null | wc -l)

        if [[ "$profraw_files" -eq 0 ]]; then
            log_warning "No .profraw files found. Running tests first..."

            # Set coverage environment
            export CARGO_INCREMENTAL=0
            export RUSTFLAGS="-C instrument-coverage"
            export LLVM_PROFILE_FILE="tests/coverage/rust/coverage-%p-%m.profraw"

            # Run tests to generate coverage data
            cargo test --workspace --lib --bins --tests
        fi

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
            --ignore "build.rs" \
            --ignore "**/test_*" \
            --ignore "**/*test.rs" \
            --ignore "**/fixtures/*" \
            --ignore "**/*.pb.rs" \
            --ignore "**/generated/*" \
            -o "tests/coverage/rust"

        log_success "Rust coverage report generated with grcov"
        log_info "HTML Report: file://$PROJECT_ROOT/tests/coverage/rust/html/index.html"

    elif command -v cargo-llvm-cov >/dev/null 2>&1; then
        log_info "Generating coverage report with cargo-llvm-cov..."

        # Set coverage environment
        export CARGO_INCREMENTAL=0

        cargo llvm-cov --workspace \
            --html \
            --lcov \
            --output-dir tests/coverage/rust \
            --ignore-filename-regex "(tests/|test_|_test\.rs|build\.rs|\.pb\.rs|generated/)"

        log_success "Rust coverage report generated with cargo-llvm-cov"
        log_info "HTML Report: file://$PROJECT_ROOT/tests/coverage/rust/html/index.html"

    elif command -v cargo-tarpaulin >/dev/null 2>&1; then
        log_info "Generating coverage report with cargo-tarpaulin..."

        cargo tarpaulin \
            --out Html \
            --out Xml \
            --out Json \
            --ignore-tests \
            --workspace \
            --output-dir tests/coverage/rust

        # Rename tarpaulin report to match expected structure
        if [[ -f "tests/coverage/rust/tarpaulin-report.html" ]]; then
            mkdir -p "tests/coverage/rust/html"
            mv "tests/coverage/rust/tarpaulin-report.html" "tests/coverage/rust/html/index.html"
        fi

        log_success "Rust coverage report generated with cargo-tarpaulin"
        log_info "HTML Report: file://$PROJECT_ROOT/tests/coverage/rust/html/index.html"

    else
        log_error "No Rust coverage tool available"
        return 1
    fi
}

# Generate TypeScript coverage
generate_typescript_coverage() {
    if [[ "$TYPESCRIPT_COVERAGE" != true ]]; then
        return 0
    fi

    log_section "Generating TypeScript Coverage Report"

    # Create coverage directories
    mkdir -p "$PROJECT_ROOT/tests/coverage/typescript"

    cd "$PROJECT_ROOT"

    # Check if we need to run tests first
    if [[ ! -d "coverage" ]] && [[ ! -f ".nyc_output/out.json" ]]; then
        log_warning "No TypeScript coverage data found. Running tests first..."

        # Run TypeScript tests with coverage
        export COVERAGE=true
        bun test web/tests \
            --coverage \
            --coverage-dir "tests/coverage/typescript" \
            --coverage-reporter text \
            --coverage-reporter lcov || log_warning "TypeScript tests may have failed"
    fi

    # Generate additional coverage formats if c8 is available
    if command -v c8 >/dev/null 2>&1; then
        log_info "Generating additional coverage formats with c8..."

        npx c8 report \
            --reporter=html \
            --reporter=lcov \
            --reporter=json \
            --reporter=text \
            --reports-dir=tests/coverage/typescript \
            --temp-directory=.nyc_output \
            --exclude="node_modules/**" \
            --exclude="tests/**" \
            --exclude="**/*.test.ts" \
            --exclude="**/*.spec.ts" \
            --exclude="static/**" 2>/dev/null || log_warning "c8 report generation failed"
    fi

    log_success "TypeScript coverage report generated"
    log_info "Coverage data available in: $PROJECT_ROOT/tests/coverage/typescript/"
}

generate_e2e_coverage() {
    log_section "Generating E2E Coverage Summary"

    if [[ ! -d "$PROJECT_ROOT/tests/coverage/e2e/raw" ]]; then
        log_warning "No Playwright coverage data found in tests/coverage/e2e/raw"
        return 0
    fi

    node "$PROJECT_ROOT/tools/generate-e2e-coverage.mjs" || log_warning "E2E coverage summary generation failed"
}

# Generate unified report
generate_unified_report() {
    if [[ "$UNIFIED_REPORT" != true ]]; then
        return 0
    fi

    log_section "Generating Unified Coverage Report"

    # Create unified coverage directory
    mkdir -p "$PROJECT_ROOT/tests/coverage/unified"

    # Create a comprehensive report index
    cat > "$PROJECT_ROOT/tests/coverage/unified/index.html" << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Unified Coverage Report</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 8px;
            padding: 30px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        h1 {
            color: #333;
            text-align: center;
            margin-bottom: 30px;
        }
        .report-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        .report-card {
            border: 1px solid #ddd;
            border-radius: 6px;
            padding: 20px;
            text-align: center;
            transition: transform 0.2s;
        }
        .report-card:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 15px rgba(0,0,0,0.1);
        }
        .report-card h3 {
            margin-top: 0;
            color: #555;
        }
        .rust-card {
            border-left: 4px solid #CE422B;
        }
        .typescript-card {
            border-left: 4px solid #3178C6;
        }
        .e2e-card {
            border-left: 4px solid #45ba4b;
        }
        .report-link {
            display: inline-block;
            padding: 10px 20px;
            background-color: #007acc;
            color: white;
            text-decoration: none;
            border-radius: 4px;
            margin: 5px;
            transition: background-color 0.2s;
        }
        .report-link:hover {
            background-color: #005a9f;
        }
        .report-link.disabled {
            background-color: #ccc;
            pointer-events: none;
        }
        .metadata {
            background-color: #f8f9fa;
            border-radius: 4px;
            padding: 15px;
            margin-top: 20px;
        }
        .metadata h4 {
            margin-top: 0;
            color: #666;
        }
        .metadata p {
            margin: 5px 0;
            font-size: 14px;
            color: #777;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🧪 Comprehensive Test Coverage Report</h1>

        <div class="report-grid">
            <div class="report-card rust-card">
                <h3>🦀 Rust Backend Coverage</h3>
                <p>Unit tests, integration tests, and API coverage</p>
                <a href="../rust/html/index.html" class="report-link" id="rust-html">HTML Report</a>
                <a href="../rust/lcov.info" class="report-link" id="rust-lcov">LCOV Data</a>
                <a href="../rust/coverage.json" class="report-link" id="rust-json">JSON Data</a>
            </div>

            <div class="report-card typescript-card">
                <h3>🌐 TypeScript Frontend Coverage</h3>
                <p>Component tests and frontend unit coverage</p>
                <a href="../typescript/index.html" class="report-link" id="ts-html">HTML Report</a>
                <a href="../typescript/lcov.info" class="report-link" id="ts-lcov">LCOV Data</a>
                <a href="../typescript/coverage-final.json" class="report-link" id="ts-json">JSON Data</a>
            </div>

            <div class="report-card e2e-card">
                <h3>🎭 End-to-End Coverage</h3>
                <p>Playwright E2E tests and UI automation</p>
                <a href="../e2e/playwright/index.html" class="report-link" id="e2e-html">Test Report</a>
                <a href="../e2e/coverage-report/index.html" class="report-link" id="e2e-coverage">Coverage Report</a>
            </div>
        </div>

        <div class="metadata">
            <h4>📊 Report Metadata</h4>
            <p><strong>Generated:</strong> <span id="timestamp">Loading...</span></p>
            <p><strong>Test Run ID:</strong> <span id="run-id">Loading...</span></p>
            <p><strong>Environment:</strong> <span id="environment">Loading...</span></p>
        </div>
    </div>

    <script>
        // Load metadata from test summary
        fetch('./test-summary.json')
            .then(response => response.json())
            .then(data => {
                document.getElementById('timestamp').textContent = new Date(data.timestamp).toLocaleString();
                document.getElementById('run-id').textContent = data.test_run_id;
                document.getElementById('environment').textContent =
                    `${data.environment.rust_version} | ${data.environment.node_version} | ${data.environment.os}`;
            })
            .catch(err => {
                console.log('Could not load test summary:', err);
            });

        // Check if report files exist and disable missing links
        const reportLinks = [
            { id: 'rust-html', path: '../rust/html/index.html' },
            { id: 'rust-lcov', path: '../rust/lcov.info' },
            { id: 'rust-json', path: '../rust/coverage.json' },
            { id: 'ts-html', path: '../typescript/index.html' },
            { id: 'ts-lcov', path: '../typescript/lcov.info' },
            { id: 'ts-json', path: '../typescript/coverage-final.json' },
            { id: 'e2e-html', path: '../e2e/playwright/index.html' },
            { id: 'e2e-coverage', path: '../e2e/coverage-report/index.html' }
        ];

        reportLinks.forEach(link => {
            fetch(link.path, { method: 'HEAD' })
                .then(response => {
                    if (!response.ok) {
                        document.getElementById(link.id).classList.add('disabled');
                        document.getElementById(link.id).textContent += ' (N/A)';
                    }
                })
                .catch(() => {
                    document.getElementById(link.id).classList.add('disabled');
                    document.getElementById(link.id).textContent += ' (N/A)';
                });
        });
    </script>
</body>
</html>
EOF

    # Create a summary JSON report
    cat > "$PROJECT_ROOT/tests/coverage/unified/test-summary.json" << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ)",
  "test_run_id": "$(uuidgen || echo "$(date +%s)")",
  "generator": "coverage.sh",
  "environment": {
    "rust_version": "$(rustc --version 2>/dev/null || echo "unknown")",
    "node_version": "$(node --version 2>/dev/null || echo "unknown")",
    "bun_version": "$(bun --version 2>/dev/null || echo "unknown")",
    "os": "$(uname -a 2>/dev/null || echo "unknown")",
    "grcov_version": "$(grcov --version 2>/dev/null || echo "not available")",
    "cargo_llvm_cov": "$(cargo llvm-cov --version 2>/dev/null || echo "not available")",
    "cargo_tarpaulin": "$(cargo tarpaulin --version 2>/dev/null || echo "not available")"
  },
  "reports_generated": {
    "rust_coverage": $RUST_COVERAGE,
    "typescript_coverage": $TYPESCRIPT_COVERAGE,
    "unified_report": $UNIFIED_REPORT
  },
  "report_locations": {
    "rust_html": "rust/html/index.html",
    "rust_lcov": "rust/lcov.info",
    "typescript_html": "typescript/index.html",
    "typescript_lcov": "typescript/lcov.info",
    "unified_html": "unified/index.html"
  }
}
EOF

    log_success "Unified coverage report generated"
    log_info "Unified Report: file://$PROJECT_ROOT/tests/coverage/unified/index.html"
}

# Open reports in browser
open_reports() {
    if [[ "$OPEN_REPORT" != true ]]; then
        return 0
    fi

    log_section "Opening Coverage Reports"

    # Determine which report to open
    if [[ "$UNIFIED_REPORT" == true ]]; then
        local report_url="file://$PROJECT_ROOT/tests/coverage/unified/index.html"
    elif [[ "$RUST_COVERAGE" == true ]]; then
        local report_url="file://$PROJECT_ROOT/tests/coverage/rust/html/index.html"
    elif [[ "$TYPESCRIPT_COVERAGE" == true ]]; then
        local report_url="file://$PROJECT_ROOT/tests/coverage/typescript/index.html"
    else
        log_warning "No reports to open"
        return 0
    fi

    # Try to open the report in the default browser
    if command -v open >/dev/null 2>&1; then
        open "$report_url"
    elif command -v xdg-open >/dev/null 2>&1; then
        xdg-open "$report_url"
    elif command -v start >/dev/null 2>&1; then
        start "$report_url"
    else
        log_warning "Could not open browser automatically. Open this URL manually:"
        log_info "$report_url"
    fi
}

# Main execution
main() {
    local start_time=$(date +%s)

    log_section "🧪 Coverage Report Generator"

    check_tools

    if [[ "$RUST_COVERAGE" == true ]]; then
        generate_rust_coverage
    fi

    if [[ "$TYPESCRIPT_COVERAGE" == true ]]; then
        generate_typescript_coverage
    fi

    generate_e2e_coverage

    if [[ "$UNIFIED_REPORT" == true ]]; then
        generate_unified_report
    fi

    open_reports

    local end_time=$(date +%s)
    local total_time=$((end_time - start_time))

    log_section "🎉 Coverage Report Generation Complete!"
    log_success "Total execution time: ${total_time}s"

    if [[ "$UNIFIED_REPORT" == true ]]; then
        log_info "📊 Main Report: file://$PROJECT_ROOT/tests/coverage/unified/index.html"
    fi
}

# Execute main function
main "$@"
