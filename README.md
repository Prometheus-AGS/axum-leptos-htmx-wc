# Axum + Leptos + HTMX + Web Components

## Agentic Streaming LLM Application (MCP-First, Tauri-Ready)

This repository is a reference implementation and living architecture example for building agentic AI applications that:
*   support tool-first LLM interaction
*   stream rich, typed model output
*   remain HTML-first and inspectable
*   avoid heavyweight SPA frameworks
*   run identically as:
    *   a web app
    *   a desktop app (via Tauri)
    *   a mobile app (via Tauri)

This is not a demo toy.
There are no mocks.
Everything is wired against real protocols, real streaming, and real tools from day one.

---

## 🚀 Using This Template

This project is a GitHub template. Create your own project from it:

### Option 1: GitHub UI
Click **"Use this template"** → The cleanup workflow runs automatically on first push.

### Option 2: cargo-generate
```bash
cargo generate --git https://github.com/Prometheus-AGS/axum-leptos-htmx-wc
```

### Option 3: Bootstrap Script
```bash
git clone https://github.com/Prometheus-AGS/axum-leptos-htmx-wc my-project
cd my-project && ./bootstrap.sh
```

### Option 4: Rust TUI CLI
```bash
cd tools/project-init && cargo run
```

See [TEMPLATE_USAGE.md](./TEMPLATE_USAGE.md) for detailed configuration options.

---

## High-Level Goals

This project exists to prove (and then serve as a template for):

1.  **Always-on tool use with LLMs**
    *   The server is always an MCP client
    *   Tools are discovered dynamically from mcp.json
    *   The model can call any tool at any time
    *   Tool execution is deterministic and server-controlled
2.  **First-class streaming**
    *   Token streaming
    *   Tool call streaming
    *   Tool result streaming
    *   Structured chunk types (thinking, reasoning, citations, memory, errors)
    *   A future-proof AG-UI-style event model
3.  **Local-First Persistence**
    *   **PGlite** (Postgres in WASM) for complete client-side history
    *   Full-text search running in the browser
    *   Offline-capable architecture
4.  **Protocol flexibility without UI changes**
    *   OpenAI Chat Completions
    *   OpenAI Responses
    *   OpenAI-compatible backends (Ollama, vLLM, etc.)
    *   One internal event contract for the UI
5.  **HTML-centric UI composition**
    *   HTMX 2.0.8 for navigation and server interaction
    *   Web Components for client-side programmability
    *   Alpine.js for local UI reactivity
    *   No React, Next.js, Vue, or SPA routers

---

## Architecture Overview

```mermaid
graph TB
    %% Development & Build Layer
    subgraph DevTools [Development & Build Pipeline]
        subgraph Frontend [Frontend Build]
            Bun[Bun Build System]
            TS[TypeScript Compiler]
            CSS[Tailwind CSS]
            WASM[WASM Assets]
        end

        subgraph RustBuild [Rust Build]
            Cargo[Cargo Build]
            Clippy[Clippy Linter]
            Fmt[rustfmt]
            LLVM[LLVM Coverage]
        end

        subgraph Quality [Code Quality]
            ESLint[ESLint]
            Prettier[Prettier]
            Tests[Test Execution]
            Coverage[Coverage Reports]
        end
    end

    %% Testing Infrastructure Layer
    subgraph TestInfra [Testing Infrastructure]
        subgraph TestRunner [Test Execution]
            TestScript[tools/test-all.sh]
            QuickMode[--quick mode]
            FullMode[--full mode]
            CIMode[--ci mode]
        end

        subgraph TestTypes [Test Categories]
            UnitTests[Unit Tests]
            IntegrationTests[Integration Tests]
            E2ETests[Playwright E2E]
            PerfTests[Performance Tests]
        end

        subgraph DockerTest [Docker Test Environment]
            PostgresTest[PostgreSQL Test]
            RedisTest[Redis Test]
            SurrealTest[SurrealDB Test]
            UnstructuredTest[Unstructured API]
        end
    end

    %% Runtime Application Layer
    subgraph Runtime [Runtime Application Architecture]
        subgraph ClientUI [Client UI Layer]
            HTMX[HTMX 2.0.8<br/>Navigation & Forms]
            WC[Web Components<br/>TypeScript]
            Alpine[Alpine.js<br/>Local State]
            SSE[SSE Event Stream]
            PGLite[PGlite<br/>Client Database]
        end

        subgraph Server [Axum Server Core]
            API[API Endpoints<br/>REST & SSE]
            Orch[LLM Orchestrator<br/>Stream Management]
            Session[Session Store<br/>State Management]
            MCP_Registry[MCP Registry<br/>Tool Discovery]
        end

        subgraph DataLayer [Data Persistence]
            PostgresDB[(PostgreSQL<br/>pgvector)]
            SurrealDB[(SurrealDB<br/>Multi-Model)]
            Redis[(Redis Stack<br/>Caching & Sessions)]
        end

        subgraph MCPEcosystem [MCP Tool Ecosystem]
            MCPClient[MCP Client<br/>rmcp SDK]
            TimeServer[Time Server<br/>@mcpcentral/mcp-time]
            TavilyServer[Tavily Search<br/>Web Search API]
            CustomMCP[Custom MCP Servers]
        end
    end

    %% External Services
    subgraph External [External Services]
        LLMProviders[LLM APIs<br/>OpenAI, Azure, Ollama]
        WebSearch[Tavily Search API]
        UnstructuredAPI[Unstructured.io API<br/>Document Processing]
    end

    %% Deployment Targets
    subgraph Deploy [Deployment Targets]
        WebApp[Web Application<br/>localhost:3001]
        TauriDesktop[Tauri Desktop<br/>Cross-Platform]
        TauriMobile[Tauri Mobile<br/>iOS & Android]
    end

    %% Build Flow
    Bun --> TS
    Bun --> CSS
    Bun --> WASM
    Cargo --> Clippy
    Cargo --> Fmt

    %% Quality Flow
    TS --> ESLint
    TS --> Prettier
    Cargo --> LLVM
    Tests --> Coverage

    %% Test Flow
    TestScript --> QuickMode
    TestScript --> FullMode
    TestScript --> CIMode
    TestRunner --> TestTypes
    TestTypes --> DockerTest

    %% Runtime Data Flow
    HTMX -.->|HTTP Requests| API
    SSE -.->|Event Stream| WC
    WC -.->|DOM Updates| Alpine
    WC -.->|Local Persist| PGLite

    API --> Orch
    Orch --> Session
    Orch --> MCP_Registry

    %% Database Connections
    Server -.->|Primary DB| PostgresDB
    Server -.->|Multi-Model| SurrealDB
    Server -.->|Caching| Redis

    %% MCP Tool Execution
    MCP_Registry --> MCPClient
    MCPClient --> TimeServer
    MCPClient --> TavilyServer
    MCPClient --> CustomMCP

    %% External Integrations
    Orch -.->|LLM Calls| LLMProviders
    TavilyServer -.->|Search Queries| WebSearch
    Server -.->|Document Processing| UnstructuredAPI

    %% Streaming Flow
    LLMProviders -.->|Token Stream| Orch
    MCPClient -.->|Tool Results| Orch
    Orch -.->|Event Stream| API
    API -.->|SSE Events| SSE

    %% Deployment Flow
    Server --> WebApp
    Server --> TauriDesktop
    Server --> TauriMobile

    %% Testing Integration
    TestInfra -.->|Validates| Runtime
    DockerTest -.->|Isolates| DataLayer
    Coverage -.->|Reports| Quality

    %% Styling
    classDef devLayer fill:#e1f5fe
    classDef testLayer fill:#f3e5f5
    classDef runtimeLayer fill:#e8f5e8
    classDef dataLayer fill:#fff3e0
    classDef mcpLayer fill:#fce4ec
    classDef externalLayer fill:#f1f8e9
    classDef deployLayer fill:#e0f2f1

    class DevTools,Frontend,RustBuild,Quality devLayer
    class TestInfra,TestRunner,TestTypes,DockerTest testLayer
    class Runtime,ClientUI,Server runtimeLayer
    class DataLayer,PostgresDB,SurrealDB,Redis dataLayer
    class MCPEcosystem,MCPClient,TimeServer,TavilyServer,CustomMCP mcpLayer
    class External,LLMProviders,WebSearch,UnstructuredAPI externalLayer
    class Deploy,WebApp,TauriDesktop,TauriMobile deployLayer
```

---

## Testing Infrastructure

This project includes a comprehensive testing infrastructure designed for reliability, coverage, and continuous integration.

### Test Types and Organization

The testing suite is organized into multiple categories for comprehensive coverage:

#### Unit Tests
- **Rust Unit Tests**: Standard Rust unit tests with `cargo test`
- **TypeScript Unit Tests**: Frontend unit tests with Bun test runner

#### Integration Tests
- **API Integration Tests**: Full HTTP API testing using `axum-test`
- **Database Integration Tests**: Database layer testing with real Postgres/SurrealDB instances
- **Service Integration Tests**: Business logic integration testing
- **MCP Tool Integration Tests**: Testing MCP server connections and tool execution

#### End-to-End Tests
- **Playwright E2E Tests**: Full browser automation testing
- **Multi-browser Support**: Tests run across different browser engines
- **Real User Scenarios**: Complete user workflows from UI to backend

#### Performance Tests
- **Load Testing**: API endpoint performance validation
- **Memory Usage Tests**: Resource consumption monitoring
- **Streaming Performance**: Real-time streaming optimization verification

### Testing Tools and Frameworks

#### Rust Testing Stack
- **axum-test** (v18.4.1): Web service integration testing
- **serial_test**: Sequential test execution for database consistency
- **tokio-test**: Async testing utilities
- **testcontainers**: Docker-based test environment management
- **mockall**: Mock object generation for unit tests
- **grcov**: Code coverage collection and reporting
- **cargo-llvm-cov**: LLVM-based coverage instrumentation

#### Frontend Testing Stack
- **Playwright** (v1.57.0): End-to-end browser automation
- **monocart-coverage-reports**: Advanced coverage reporting
- **c8**: JavaScript/TypeScript coverage collection

### Comprehensive Test Runner

The project includes a sophisticated test runner at `tools/test-all.sh` with multiple execution modes:

#### Test Modes
- **Quick Mode** (`--quick`): Smoke tests and unit tests only
- **Full Mode** (`--full`): Complete test suite including integration and E2E tests
- **CI Mode** (`--ci`): Optimized for continuous integration with sequential execution

#### Test Runner Features
- **Parallel Execution**: Tests run in parallel for faster feedback (configurable)
- **Docker Orchestration**: Automatic test service setup and teardown
- **Coverage Collection**: Unified coverage reporting across Rust and TypeScript
- **Health Checks**: Service readiness validation before test execution
- **Report Generation**: HTML and JSON test reports with detailed metrics

### Docker Testing Environment

The testing infrastructure uses Docker Compose for isolated, reproducible test environments:

#### Test Services (`docker-compose.test.yaml`)
- **PostgreSQL with pgvector**: Full-featured database testing
- **SurrealDB**: Multi-model database testing
- **Redis Stack**: Caching and session testing
- **Unstructured API**: Document processing testing
- **Test Runner Container**: Isolated test execution environment

#### Service Health Monitoring
- Automatic health checks for all services
- Configurable timeout and retry logic
- Resource limits for consistent performance

### Coverage and Reporting

#### Multi-Language Coverage
- **Rust Coverage**: LLVM-based instrumentation with `grcov`
- **TypeScript Coverage**: V8-based coverage collection
- **E2E Coverage**: Browser-based coverage during Playwright tests

#### Report Formats
- **HTML Reports**: Interactive coverage visualization
- **JSON Reports**: Machine-readable coverage data
- **LCOV Format**: IDE integration and CI/CD compatibility
- **Cobertura Format**: Jenkins and Azure DevOps integration

#### Coverage Thresholds
- Line coverage tracking per module
- Branch coverage analysis
- Function coverage validation
- Integration with CI/CD pipelines

### Running Tests

#### Quick Testing
```bash
# Run basic smoke and unit tests
./tools/test-all.sh --quick
```

#### Full Test Suite
```bash
# Run complete test suite with coverage
./tools/test-all.sh --full

# CI mode (sequential, full cleanup)
./tools/test-all.sh --ci
```

#### Individual Test Categories
```bash
# Rust unit tests only
cargo test --lib --bins

# Integration tests only
cargo test --test '*_integration'

# E2E tests only
npx playwright test

# Specific test with coverage
cargo llvm-cov --html --open
```

#### Coverage Reports
After running tests, coverage reports are available at:
- **Rust Coverage**: `tests/coverage/rust/html/index.html`
- **E2E Coverage**: `tests/coverage/e2e/playwright/index.html`
- **Unified Summary**: `tests/coverage/unified/test-summary.json`

---

## Core Design Principles

### 1. Tools Are Non-Optional

This system assumes:
*   The model will call tools
*   The model should reason with tools
*   The model cannot execute tools itself

Therefore:
*   The server is always an MCP client
*   Tools are discovered dynamically at startup
*   Tools are available to every request
*   Tool execution is deterministic, auditable, and server-side

---

### 2. Streaming Is the Default

All LLM interaction supports:
*   streaming responses
*   streaming tool calls
*   streaming tool results

The server normalizes all upstream streaming into a single internal event model, regardless of whether the upstream protocol is:
*   Chat Completions
*   Responses
*   OpenAI-compatible proxies

The client never has to care which protocol is used.

---

### 3. One Internal Event Contract

Internally, everything becomes typed events:
*   `message.delta`
*   `tool_call.delta`
*   `tool_call.complete`
*   `tool_result`
*   `error`
*   `done`

In parallel, the server mirrors these events into AG-UI-style events:
*   `agui.message.delta`
*   `agui.tool_call.delta`
*   `agui.tool_call.complete`
*   `agui.tool_result`
*   `agui.error`
*   `agui.done`
*   `agui.usage` (Token counts)

This allows:
*   progressive rendering
*   structured UIs
*   future AG-UI endpoints without refactoring

---

## S-Tier UI Features

This reference implementation includes production-grade UI features:

### 1. Robust Streaming & Tool Support
*   **Flicker-Free Streaming**: Native EventSource implementation with an optimized `StreamController` ensures smooth text delivery.
*   **Unified Tool Blocks**: Multiple tool calls (e.g., Time + Search) are aggregated into clean, keyed DOM elements.
*   **No Truncation**: Specialized buffering logic ensuring markdown and code blocks are always complete.

### 2. Context-Aware Auto-Naming
*   **Background Generation**: Automatically names conversations after the first turn using a dedicated LLM call.
*   **Contextual**: Uses both the user prompt and the assistant's reply for accurate titles.

### 3. Sidebar Polish
*   **Date Grouping**: Conversations organized by "Today", "Yesterday", "Last 7 Days", etc.
*   **Inline Renaming**: Double-click titles to rename instantly.
*   **Full-Text Search**: Search across all conversation history stored locally in PGlite.

---

## MCP (Model Context Protocol)

### Why MCP?

MCP provides:
*   a standard tool interface
*   language-agnostic tooling
*   dynamic discovery
*   isolation between model reasoning and execution

### This Project Uses:
*   `rmcp` (official Rust MCP SDK)
*   stdio child-process MCP servers
*   remote streamable HTTP MCP servers

---

## UI Stack

### HTMX 2.0.8
Used for navigation, server interaction, and progressive enhancement. Not used for high-frequency streaming updates.

### Web Components (TypeScript)
Provide a programmable client runtime for the chat interface. Components include:
*   `<chat-stream>`: Manages SSE connection and persistence.
*   `<transcript-view>`: Handles efficient DOM updates and markdown rendering.
*   `<conversation-sidebar>`: Manages history and PGlite interactions.
*   `<token-counter>`: Displays real-time usage metrics.

### PGlite (Postgres WASM)
Runs a full Postgres instance in the browser for:
*   storing conversation history
*   performing full-text search
*   ensuring offline capability

- **Modular Architecture**: Clean separation of concerns (API, Domain, Persistence, Runtime).
- **Configuration**: Hierarchical config via CLI, Env, and File. See [Configuration Guide](docs/configuration.md).

## Getting Started

### Prerequisites

#### Core Development Tools
- **Rust** (latest stable) - Backend development and building
- **Bun** - Frontend asset building and package management
- **Node.js** (latest LTS) - Required for some development tools

#### Database Systems
- **PostgreSQL** (with pgvector extension) - Primary database
- **SurrealDB** - Alternative multi-model database option
- **Redis** - Caching and session management

#### Testing Infrastructure (Optional)
- **Docker** and **Docker Compose** - For containerized testing environment
- **cargo-llvm-cov** - Rust code coverage collection (`cargo install cargo-llvm-cov`)
- **grcov** - Coverage report generation (`cargo install grcov`)
- **Playwright** - End-to-end testing (installed via `npx playwright install`)

#### Additional Tools
- **Git** - Version control
- **curl** - API testing and health checks
- **jq** - JSON processing for test reports (optional)

### Configuration
Copy `example.config.yaml` to `config.yaml` or use environment variables.
See [docs/configuration.md](docs/configuration.md) for details.

### Quick Start

#### 1. Clone and Setup
```bash
# Clone the repository
git clone https://github.com/Prometheus-AGS/axum-leptos-htmx-wc.git
cd axum-leptos-htmx-wc

# Copy configuration file
cp example.config.yaml config.yaml
```

#### 2. Install Dependencies
```bash
# Install Rust dependencies
cargo build

# Install frontend dependencies
bun install

# Install optional testing tools
cargo install cargo-llvm-cov grcov
npx playwright install
```

#### 3. Build Frontend Assets
```bash
# Build all frontend assets
bun run build
```

#### 4. Run the Application
```bash
# Start the development server
cargo run

# The application will be available at http://localhost:3001
```

#### 5. Verify Setup with Tests
```bash
# Run quick smoke tests to verify everything is working
./tools/test-all.sh --quick

# For full testing including Docker services
./tools/test-all.sh --full
```

### Environment Variables

Create a `.env` file in the project root with your configuration:

```bash
# LLM Provider API Keys
OPENAI_API_KEY=your_openai_api_key_here
TAVILY_API_KEY=your_tavily_api_key_here

# Database Configuration
DATABASE_URL=postgres://user:password@localhost:5432/your_db
REDIS_URL=redis://localhost:6379

# Application Configuration
RUST_LOG=info
APP_ENVIRONMENT=development
```

### Development Commands

#### Rust Backend
```bash
# Run the server in development mode
cargo run

# Build the Rust application
cargo build --release

# Run tests
cargo test

# Check for linting issues (extensive clippy configuration)
cargo clippy

# Format Rust code
cargo fmt
```

#### Frontend Assets
```bash
# Build all frontend assets (TypeScript, CSS, WASM)
bun run build

# Development mode with file watching
bun run dev

# Type check TypeScript without emitting
bun run check

# Lint TypeScript files
bun run lint

# Format frontend code
bun run format
```

#### Individual Asset Building
```bash
# Build TypeScript only
bun run build:ts

# Build CSS with Tailwind
bun run build:css

# Copy WASM files from dependencies
bun run copy:wasm
```

#### Testing Commands
```bash
# Run comprehensive test suite
./tools/test-all.sh --full

# Quick smoke tests and unit tests only
./tools/test-all.sh --quick

# CI mode (sequential, full cleanup)
./tools/test-all.sh --ci

# Individual test types
cargo test --lib --bins                    # Rust unit tests
cargo test --test '*_integration'          # Integration tests
npx playwright test                        # E2E tests
cargo llvm-cov --html --open              # Coverage with HTML report
```

#### Tauri Desktop App
```bash
# Run in development mode
bun run tauri:dev

# Build desktop application
bun run tauri:build
```

### Key Files and Directories

Understanding the project structure is essential for effective development:

#### Core Application
- `src/main.rs`: Entry point, Axum server configuration
- `src/lib.rs`: Core application logic and orchestrator
- `mcp.json`: MCP tool server configuration
- `config.yaml`: Application configuration
- `Cargo.toml`: Rust dependencies and linting configuration

#### Frontend
- `web/main.ts`: Frontend entry point and application initialization
- `web/components/`: Web Components (TypeScript)
  - `chat-stream/`: Main streaming chat interface
  - `chat-messages/`: Message container and management
  - `chat-tool-call/`: Tool call visualization
- `static/`: Compiled assets and WASM files
- `package.json`: Frontend dependencies and build scripts

#### Testing Infrastructure
- `tests/`: Comprehensive test suite
  - `e2e/`: Playwright end-to-end tests
  - `integration/`: Rust integration tests
  - `fixtures/`: Test data and mock responses
- `tools/test-all.sh`: Comprehensive test runner script
- `docker-compose.test.yaml`: Test environment services
- `playwright.config.ts`: E2E testing configuration

#### Configuration Files
- `.env`: Environment variables (create from template)
- `example.config.yaml`: Configuration template
- `tailwind.config.js`: Styling configuration
- `tsconfig.json`: TypeScript configuration

### Alpine.js
Used sparingly for local UI transitions and toggle states.

---

## Code Quality and Standards

This project maintains high code quality through comprehensive linting, formatting, and testing standards.

### Rust Code Quality
The project uses extensive Clippy linting configured in `Cargo.toml`:

#### Enabled Lint Categories
- **Cargo**: Manifest and dependency analysis
- **Complexity**: Code complexity detection
- **Correctness**: Bug prevention and correctness checks
- **Pedantic**: Strict coding standards
- **Performance**: Performance optimization suggestions
- **Style**: Consistent code styling
- **Suspicious**: Potentially problematic patterns

#### Additional Restriction Lints
- `clone_on_ref_ptr`: Prevents unnecessary cloning
- `empty_drop`: Identifies empty Drop implementations
- `undocumented_unsafe_blocks`: Requires documentation for unsafe code
- `redundant_type_annotations`: Removes unnecessary type annotations

#### Code Quality Tools
```bash
# Run all linting checks
cargo clippy

# Format code according to project standards
cargo fmt

# Run with pedantic lints
cargo clippy -- -W clippy::pedantic
```

### TypeScript Code Quality
Frontend code quality is maintained through:

```bash
# Type checking without compilation
bun run check

# ESLint with TypeScript support
bun run lint

# Prettier formatting
bun run format
```

### Performance Optimizations
- **mimalloc**: High-performance memory allocator
- **Structured Logging**: `tracing` for observability
- **Rust 2024 Edition**: Latest language features and optimizations

### Testing Standards
- **Unit Test Coverage**: Comprehensive unit test coverage with `cargo-llvm-cov`
- **Integration Testing**: Real database and service integration
- **E2E Testing**: Full browser automation with Playwright
- **Performance Testing**: Load testing and memory usage validation

---

## Tauri Compatibility

This project is Tauri-ready by design:
*   no CDN scripts
*   all assets served locally
*   no API keys in the browser
*   SSE works identically in webview
*   same UI codebase for web + desktop + mobile

---

## Summary

This repository demonstrates that it is possible to build **deeply agentic**, **tool-first**, **streaming-native**, **HTML-centric**, **Tauri-compatible** AI applications without heavyweight SPA frameworks, without client-side secrets, and without sacrificing UX or architectural clarity.
