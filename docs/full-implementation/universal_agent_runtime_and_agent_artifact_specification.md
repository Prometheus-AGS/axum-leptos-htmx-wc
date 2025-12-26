# Universal Agent Runtime

## Purpose
The **Universal Agent Runtime (UAR)** is a single runtime that can host and execute agent instances consistently across:

- **Desktop** (Tauri-embedded Axum server on loopback)
- **Cloud** (Axum service, same router + handlers)
- **Local-first** (optional embedded DB + sync)

It provides:

1. **A consistent execution model** for agents and tools
2. **A consistent streaming event model** compatible with **AG-UI**, **A2UI**, and **Agent-to-Agent (A2A)**
3. **A tool framework** that supports both **internal tools** and **Model Context Protocol (MCP) servers**
4. **A “skills” system** that improves tool selection and tool usage behavior
5. **A metadata-driven “agent artifact specification”** that fully defines agent behavior at runtime

---

## UAR Core Concepts

### 1) Agent Instance
An **agent instance** is a runtime process (logical) composed of:

- **Agent Artifact** (metadata describing behavior)
- **Provider Policy** (model/provider selection rules)
- **Memory Adapters** (RAG / knowledge / conversation state)
- **Tool Registry** (internal + MCP)
- **Skill Registry** (prompt+tool usage overlays)
- **Streaming Event Bus** (AG-UI / A2UI / A2A)

### 2) Runtime Host
The UAR host is an **Axum server** exposing:

- Chat + agent invocation HTTP endpoints
- Streaming (SSE) endpoints
- MCP-over-HTTP endpoints (hybrid: 200 or 202+SSE)
- A2A endpoints for agent-to-agent messaging

### 3) Normalized Event Model
All downstream protocols (AG-UI, A2UI, internal UIs) are fed from a **single normalized event bus**.

**Event invariants**:

- Every stream has a `run_id`
- Events are strictly ordered per `run_id`
- All tool calls have a lifecycle (`tool.start` → `tool.delta*` → `tool.end`)
- UI constructs are represented as chunks/events (forms, artifacts, navigation, etc.)

---

# UAR Functional Specification

## 1. Configuration and Boot

### 1.1 Runtime Modes
- **Desktop mode**: bind to `127.0.0.1`, random high port, token required.
- **Cloud mode**: bind to public interface behind TLS/ingress; same handlers.

### 1.2 App + Agent Mounting
At startup, UAR loads:

- a **runtime config** (providers, storage, ports, security)
- an **artifact registry** (agent artifacts, workflows, UI artifacts)

Agent artifacts can be loaded from:

- local filesystem
- artifact registry service
- database (SurrealDB/Postgres/PGlite)

### 1.3 Multitenant Support
A **tenant** is a partition boundary for:

- agent artifact versions
- skills
- tools (including MCP server configs)
- memory indices / vector stores
- auth/session keys

Minimal required tenant identifiers:

- `tenant_id`
- `workspace_id`
- `user_id`

---

## 2. Execution Lifecycle

### 2.1 Run
A **run** is one agent invocation:

- `run_id` (ULID/UUID)
- `conversation_id` (optional)
- `agent_id` (artifact id)
- `inputs` (structured)
- `context` (memory and files)

### 2.2 Run State Machine

1. **start**
2. **context assembly**
3. **skill selection + prompt assembly**
4. **model selection (provider policy)**
5. **stream start**
6. **LLM streaming**
7. **tool loop** (internal + MCP)
8. **finalize** (store memory, emit artifacts, done)

### 2.3 Cancellation
- HTTP `POST /api/runs/:run_id/cancel`
- A2A cancellation message
- MCP `$/cancelRequest` support (future)

---

## 3. Provider Framework (LLM Provider Layer)

### 3.1 Provider Abstraction
Providers implement a common trait:

- request/response (non-stream)
- streaming (SSE-like internal stream)
- tool calling formats
- model capabilities (reasoning, tool calling, citations)

### 3.2 Provider Policy
Provider policy is applied per run:

- choose provider + model based on:
  - task class
  - latency/cost constraints
  - tool calling requirements
  - context length
  - “reasoning required” signal

### 3.3 Routing Patterns
- **Direct**: single provider
- **Fallback**: provider failover
- **Split**: plan with model A, execute with model B
- **Ensemble** (optional): multiple candidates → ranker

---

## 4. Memory Framework

### 4.1 Memory Adapters
Memory is modular:

- conversation store
- document store
- vector store
- knowledge graph

### 4.2 Retrieval Policies
Retrieval is policy-driven:

- top-k
- hybrid keyword+vector
- citation-required mode
- safety filtering

### 4.3 Streaming Citations
When memory contributes to output, UAR can emit:

- `citation` chunks (sources)
- `memory` chunks (what was recalled)

---

## 5. Tool Framework

### 5.1 Tool Types
1) **Internal tools**: implemented as Rust functions / services.
2) **Remote tools**: HTTP endpoints.
3) **MCP tools**: discovered through MCP servers.

### 5.2 Tool Registry
The tool registry is the canonical index:

- `tool_id`
- `name`
- `description`
- `input_schema` / `output_schema` (JSON Schema)
- `transport` (internal/http/mcp)
- `auth` requirements
- `tags`

### 5.3 Tool Execution Contract
Tools must support:

- `tool.start`
- `tool.delta` (optional)
- `tool.end` (success or error)

A tool call must be replayable from:

- tool name
- input payload
- tool version
- environment id

---

# Skills Specification

## 1. What a Skill Is
A **skill** is:

- a **triggering definition** (when it applies)
- a **prompt overlay** (what text to inject)
- a **tool bundle** (tools to prefer, restrictions)
- optional **tool usage templates** (how to call tools)
- optional **post-processing** rules (validation, formatting)

Skills are used to:

- reduce hallucinated tool usage
- enforce consistent tool calling patterns
- make tool usage explainable and repeatable

## 2. Skill Activation
Skill activation happens during **context assembly** before the LLM request:

1. Compute request embedding (or semantic features)
2. Retrieve candidate skills (vector search + keyword filters)
3. Apply gating rules:
   - tenant/workspace scope
   - model capability requirements
   - tool availability
4. Rank skills by:
   - similarity
   - freshness
   - success rate
   - user preferences
5. Select:
   - top N skills (typically 1–5)

## 3. Skill Storage
Skills should be stored in a DB with:

- versioning
- embeddings for `description + triggers`
- references to tool IDs

Minimal schema (conceptual):

- `skill_id`
- `version`
- `title`
- `description`
- `triggers` (keywords, patterns, semantic)
- `prompt_overlay` (markdown)
- `preferred_tools` (tool IDs)
- `tool_templates` (optional)
- `constraints` (denylist/allowlist)
- `metrics` (success/failure)

## 4. Prompt Assembly with Skills
Prompt is assembled as:

1. system base prompt
2. agent artifact prompt
3. selected skill overlays (ordered)
4. tool registry summary (only relevant tools)
5. memory context
6. conversation history
7. user message

---

# Protocol Support

## 1) AG-UI Support
UAR must be able to stream AG-UI compatible chunks.

### 1.1 Event Mapping
UAR normalized events map to AG-UI chunk types:

- `chat.delta` → text chunk
- `reasoning.delta` → reasoning/thinking chunk
- `memory.recall` → memory chunk
- `citations` → citation chunk
- `tool.*` → tool result chunks
- `artifact.*` → artifact chunks
- `error` → error chunk

## 2) A2UI Support
A2UI is treated as a *rendering dialect* of the same event bus.

UAR must provide:

- canonical event schema (normalized)
- an adapter that emits A2UI events for compatible clients

## 3) Agent-to-Agent (A2A) Support
UAR must support agent-to-agent messaging so multiple runtimes can collaborate.

### 3.1 A2A Endpoints
- `POST /api/a2a/inbox` (receive message)
- `POST /api/a2a/send` (send message)
- `GET /api/a2a/stream/:agent_id` (SSE stream for agent messages)

### 3.2 Message Model
- `message_id`
- `from_agent`
- `to_agent`
- `conversation_id`
- `payload` (typed)
- `capabilities` (tools, models)
- `attachments` (optional)

---

# UAR Agent Artifact Specification

## 1. Goals
The **Agent Artifact** defines everything needed to instantiate an agent:

- identity and metadata
- system prompt and policies
- tool access and tool policies
- skill preferences
- memory configuration
- UI interaction rules (forms, artifacts)
- protocol adapters (AG-UI/A2UI)

It is intended to be used as **runtime metadata**, not code.


## 2. Top-Level Structure

```jsonc
{
  "version": "1.0",
  "kind": "agent",
  "id": "contract_agent",
  "metadata": {
    "title": "Contract Agent",
    "description": "Drafts and refines contracts",
    "tags": ["legal", "writing"]
  },
  "runtime": {
    "entry": "llm.chat",
    "protocols": {
      "ag_ui": { "enabled": true },
      "a2ui": { "enabled": true },
      "a2a": { "enabled": true }
    }
  },
  "policy": {
    "provider": {
      "default": { "provider": "openai", "model": "gpt-5" },
      "fallbacks": [
        { "provider": "anthropic", "model": "claude" }
      ]
    },
    "tools": {
      "allow": ["mcp:*", "internal:kb.*"],
      "deny": ["internal:fs.delete"],
      "max_concurrent": 3
    },
    "skills": {
      "prefer": ["skill:rag_citations", "skill:tool_calling_strict"],
      "max_active": 4
    }
  },
  "schemas": {
    "inputs": { "type": "object", "properties": { "message": {"type": "string"} }, "required": ["message"] },
    "outputs": { "type": "object", "properties": { "answer": {"type": "string"} }, "required": ["answer"] }
  },
  "prompt": {
    "system": "You are a helpful assistant...",
    "instructions": [
      "Always cite sources when using the knowledge base.",
      "Prefer tools over guessing."
    ]
  },
  "memory": {
    "conversation": { "enabled": true },
    "kb": {
      "enabled": true,
      "collections": ["default"],
      "citation_required": true
    }
  },
  "tools": {
    "bundles": [
      {
        "id": "kb_tools",
        "tools": ["internal:kb.search", "internal:kb.ingest"],
        "required": false
      }
    ]
  },
  "ui": {
    "forms": { "enabled": true },
    "artifacts": { "enabled": true, "preferred_types": ["html", "mermaid", "code"] }
  }
}
```


## 3. Artifact Fields (Complete)

### 3.1 `version`, `kind`, `id`
- `version`: artifact schema version
- `kind`: must be `agent`
- `id`: stable identifier

### 3.2 `metadata`
- title, description, tags, author, icon

### 3.3 `runtime`
- `entry`: execution mode (e.g. `llm.chat`, `workflow.run`)
- `protocols`: enable adapters

### 3.4 `policy`
- provider selection rules
- tool allow/deny and concurrency
- skill preferences
- output formatting constraints

### 3.5 `schemas`
- JSON Schema for `inputs` and `outputs`
- optional schema for `state`

### 3.6 `prompt`
- system prompt text
- structured instruction list
- optional template variables

### 3.7 `memory`
- conversation retention
- KB settings
- embedding model policy

### 3.8 `tools`
- tool bundles
- tool schema overrides
- tool call guardrails

### 3.9 `ui`
- form behavior (schema-driven)
- artifact generation policy
- navigation permissions

### 3.10 `extensions`
Reserved for future:

- vendor specific
- experiment flags

---

# Axum Server Reference Implementation

## 1. Workspace Structure (recommended)

```
vp-suite-tauri-axum/
  Cargo.toml
  crates/
    api/              # axum router + protocol adapters
    domain/           # core types (events, runs, tools, skills)
    skills/           # skill store + matching
    tools/            # tool registry + execution
    providers/        # LLM provider framework
    memory/           # RAG / KB adapters
  apps/
    cloud-axum/        # main.rs for cloud
    desktop-tauri/     # (later) tauri host that boots embedded axum
    web-ui/leptos-csr/ # CSR UI
  fixtures/
    ag_ui/
    a2ui/
    a2a/
    mcp/
```

## 2. API Surface

### 2.1 Chat + Runs
- `POST /api/runs` → create run
- `GET /api/runs/:run_id/stream` → SSE stream of normalized events
- `POST /api/runs/:run_id/cancel`

### 2.2 AG-UI
- `POST /api/ag-ui/runs` → create run
- `GET /api/ag-ui/runs/:run_id/stream` → SSE, AG-UI encoded

### 2.3 A2UI
- `GET /api/a2ui/runs/:run_id/stream` → SSE, A2UI encoded

### 2.4 MCP (hybrid 200 / 202+SSE)
- `POST /api/mcp/sessions`
- `GET /api/mcp/sessions/:id/events` (SSE)
- `POST /api/mcp/sessions/:id/messages` (JSON-RPC)

### 2.5 A2A
- `POST /api/a2a/send`
- `POST /api/a2a/inbox`
- `GET /api/a2a/stream/:agent_id` (SSE)

---

# Concrete Rust Example

## 1) Normalized Event Types (core)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum NormalizedEvent {
    RunStart { run_id: String, agent_id: String },
    ChatDelta { run_id: String, text_delta: String },
    ReasoningDelta { run_id: String, text_delta: String },

    Citation { run_id: String, sources: Vec<CitationSource> },
    MemoryRecall { run_id: String, items: Vec<MemoryItem> },

    ToolStart { run_id: String, tool_call_id: String, tool: String, input: serde_json::Value },
    ToolDelta { run_id: String, tool_call_id: String, delta: serde_json::Value },
    ToolEnd { run_id: String, tool_call_id: String, output: serde_json::Value, ok: bool },

    Artifact { run_id: String, artifact: ArtifactPayload },

    Error { run_id: String, code: String, message: String },
    RunDone { run_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationSource {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub key: String,
    pub value: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactPayload {
    pub artifact_id: String,
    pub artifact_type: String,
    pub title: String,
    pub content: String,
    pub language: Option<String>,
    pub metadata: serde_json::Value,
}
```

## 2) Axum SSE stream endpoint (normalized)

```rust
use axum::{
    extract::Path,
    response::sse::{Event, Sse},
    routing::get,
    Router,
};
use futures::{Stream, StreamExt};
use std::{convert::Infallible, time::Duration};
use tokio_stream::wrappers::ReceiverStream;

pub fn runs_router() -> Router {
    Router::new().route("/api/runs/:run_id/stream", get(stream_run))
}

async fn stream_run(
    Path(run_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // In real code, you subscribe to your run’s event bus.
    let (tx, rx) = tokio::sync::mpsc::channel(64);

    tokio::spawn(async move {
        let _ = tx.send(NormalizedEvent::RunStart { run_id: run_id.clone(), agent_id: "demo".into() }).await;
        for i in 0..5 {
            let _ = tx.send(NormalizedEvent::ChatDelta { run_id: run_id.clone(), text_delta: format!("hello {i} ") }).await;
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        let _ = tx.send(NormalizedEvent::RunDone { run_id }).await;
    });

    let stream = ReceiverStream::new(rx).map(|evt| {
        let json = serde_json::to_string(&evt).unwrap();
        Ok(Event::default().event("event").data(json))
    });

    Sse::new(stream)
}
```

## 3) AG-UI adapter (concept)

```rust
fn to_ag_ui(evt: &NormalizedEvent) -> Option<(String, serde_json::Value)> {
    match evt {
        NormalizedEvent::ChatDelta { text_delta, .. } => Some(("chat.delta".into(), serde_json::json!({"delta": text_delta}))),
        NormalizedEvent::ReasoningDelta { text_delta, .. } => Some(("thinking.delta".into(), serde_json::json!({"delta": text_delta}))),
        NormalizedEvent::Citation { sources, .. } => Some(("citation".into(), serde_json::json!({"sources": sources}))),
        NormalizedEvent::Artifact { artifact, .. } => Some(("artifact".into(), serde_json::to_value(artifact).ok()?)),
        NormalizedEvent::Error { code, message, .. } => Some(("error".into(), serde_json::json!({"code": code, "message": message}))),
        _ => None,
    }
}
```

## 4) A2UI adapter (concept)

A2UI is handled the same way: map `NormalizedEvent` → A2UI event objects.

---

# How this combines into your reference playground

## Phase 1 Implementation Target
Build a single monorepo playground where **each feature is a compile-valid, testable slice**:

1) **UAR core server (Axum)**
   - run lifecycle, event bus, normalized events
2) **AG-UI endpoint support**
   - stream normalized → AG-UI
3) **MCP-over-HTTP + SSE**
   - sessions + messages + events
4) **Skills system**
   - DB-backed skill registry
   - prompt assembly injection
5) **Tools**
   - internal tools
   - MCP tools
6) **A2A endpoints**
   - agent messaging + SSE
7) **Leptos CSR UI**
   - chat panel
   - chunk renderer (thinking, citations, artifacts)
   - artifact preview (iframe + shadow DOM)
8) **Workflow builder (XYFlow-like)**
   - schema-driven node/edge graph

## What codegen agents need to execute Phase 1
To enable other coding agents (e.g., in an IDE) to plan and execute Phase 1, they should follow:

- Implement `domain` types first (events, tool contracts, artifacts, skills)
- Implement `api` crate next (routes + adapters)
- Implement fixture replay and determinism gates
- Implement the Leptos CSR UI against fixture streams before wiring a real LLM
- Only after UI passes: add provider framework and live upstream proxying

---

# Appendices

## A) “Skill” Prompt Overlay Example

```md
## Skill: Tool Calling Strict

When you decide to call a tool:
- Prefer the tool listed in `preferred_tools`.
- Validate the tool input against its JSON Schema.
- If required fields are missing, emit a `ui.form.request` event.
- Never guess values that can be collected from the user.

Tool usage format:
1) Explain intent in one sentence.
2) Call tool.
3) Summarize tool result.
```

## B) Form Request Pattern (schema-driven)

Use a normalized event:

- `ui.form.request` containing JSON Schema

Client renders a form, submits back to:

- `POST /api/runs/:run_id/forms/:form_id`

## C) Artifact Preview Sandbox Pattern

- render artifact in iframe
- write to iframe document
- attach shadow root
- load limited libraries
- capture console errors and emit as `artifact.error`

---

# Phase 1 Implementation Checklist (Executable by Codegen Agents)

This section is written as an **execution-ready plan** for code generation agents in a VSCode-based agent framework.

## Phase 1 Objective
Deliver a runnable **reference playground** that:

- Boots a **Universal Agent Runtime (Axum)**
- Streams normalized events over **SSE**
- Provides **AG-UI** and **A2UI** streaming endpoints via adapters
- Supports **A2A** agent messaging endpoints
- Supports **Tools** (internal + MCP server discovery/execution)
- Supports **Skills** (DB-backed selection + prompt overlay + tool usage templates)
- Includes a **Leptos CSR** web UI that can replay fixtures and then run live

Success criteria:

- `cargo test` passes
- `cargo run -p cloud-axum` runs and serves SSE endpoints
- UI can render fixture streams without a live model
- A sample run can execute an internal tool and a mocked MCP tool

---

## Repo Layout (Phase 1)

```
vp-uar-playground/
  Cargo.toml
  crates/
    domain/              # types: events, runs, artifacts, tools, skills
    api/                 # axum router + handlers + sse + adapters
    providers/           # provider trait + stub provider (phase 1)
    tools/               # tool registry + internal tool executor + mcp client
    skills/              # skill store + matcher + prompt overlays
    memory/              # convo store + kb stub (phase 1)
    fixtures/            # event stream fixtures for deterministic replay
  apps/
    cloud-axum/          # axum main
  web/
    leptos-csr/          # UI that consumes SSE streams
```

Notes:
- `fixtures/` is a *crate* (or module) so it can be used by both server tests and UI dev.
- The UI should initially run against fixture endpoints before live wiring.

---

## Milestone 1 — Domain Types (compile gate)

### Deliverables
- `NormalizedEvent` and supporting types
- Agent Artifact struct + JSON schema validation
- Tool model (registry item + call + result)
- Skill model (triggers + overlay + templates)
- Run model (run_id, agent_id, context, state machine)

### Acceptance tests
- Parse + validate an agent artifact JSON fixture
- Serialize/deserialize normalized events

### Recommended Rust types

- `domain::events::*`
- `domain::artifact::*`
- `domain::tools::*`
- `domain::skills::*`
- `domain::runs::*`

---

## Milestone 2 — Event Bus + Run Manager

### Deliverables
- Per-run event stream broadcast (tokio broadcast or mpsc fanout)
- Run manager that:
  - creates `run_id`
  - spawns an async task that emits events
  - stores minimal run status

### Implementation guidance
- Use `tokio::sync::broadcast` for fan-out to multiple SSE subscribers
- Maintain a `DashMap<RunId, RunHandle>` for active runs

### Acceptance tests
- Create run → subscribe → observe ordered events
- Cancel run → observe `Error` or `RunDone` depending on policy

---

## Milestone 3 — Axum API (Normalized SSE)

### Endpoints
- `POST /api/runs` → create run
- `GET /api/runs/:run_id/stream` → normalized SSE
- `POST /api/runs/:run_id/cancel`

### SSE contract
- Event name: `event`
- Data: JSON serialized `NormalizedEvent`
- Heartbeats: optional `: ping

` or event `ping`

### Acceptance tests
- `POST /api/runs` returns run id
- `GET /api/runs/:id/stream` returns SSE with `RunStart ... RunDone`

---

## Milestone 4 — AG-UI and A2UI Adapters

### Deliverables
- Adapter functions:
  - `NormalizedEvent -> AG-UI chunk`
  - `NormalizedEvent -> A2UI event`
- Endpoints:
  - `POST /api/ag-ui/runs`
  - `GET /api/ag-ui/runs/:run_id/stream`
  - `GET /api/a2ui/runs/:run_id/stream`

### Design constraints
- Adapters **must not** affect core event ordering
- Unknown normalized events can be dropped or mapped to `meta` chunks

### Acceptance tests
- Given a fixture normalized stream, AG-UI endpoint emits correct chunk types
- Same fixture stream mapped to A2UI

---

## Milestone 5 — Tool Framework (Internal + MCP)

### Deliverables
- Tool registry
- Internal tool executor
- MCP tool discovery and execution (Phase 1: minimal)

#### Internal tool contract
- `ToolFn: async fn(serde_json::Value) -> ToolResult`
- Validate input schema if provided

#### MCP (Phase 1)
Support:
- connect to MCP server over HTTP
- list tools (cache)
- call tool (JSON-RPC)

### Endpoints (optional for phase 1)
- `GET /api/tools`
- `POST /api/tools/call`

### Acceptance tests
- Run triggers internal tool call and emits:
  - `ToolStart` → `ToolEnd`
- Mock MCP server provides one tool; runtime calls it

---

## Milestone 6 — Skills System (DB-backed)

### Deliverables
- Skill store interface:
  - `list`, `get`, `upsert`, `delete`, `search`
- Matcher:
  - keyword filters
  - semantic score hook (Phase 1 can stub or simple embedding)
- Prompt overlay injection into assembled prompt

### Activation timing
Must run **before** provider request:

- request features -> skill candidates -> rank -> select -> inject

### Acceptance tests
- Given an input “search docs”, select `rag_citations` skill
- Prompt assembly contains overlay
- Tool allowlist is narrowed by skill

---

## Milestone 7 — A2A (Agent-to-Agent)

### Deliverables
- Message model
- Inbox store (in-memory Phase 1)
- SSE stream per agent

### Endpoints
- `POST /api/a2a/send`
- `POST /api/a2a/inbox`
- `GET /api/a2a/stream/:agent_id`

### Acceptance tests
- Send message → recipient stream receives it

---

## Milestone 8 — Fixtures + Deterministic Replay

### Why this matters
Fixtures let the UI and protocol adapters be developed **without** a live LLM.

### Deliverables
- `fixtures/streams/*.jsonl` where each line is a `NormalizedEvent`
- Fixture endpoint:
  - `GET /api/fixtures/:name/stream`

### Acceptance tests
- Fixture replay order preserved
- AG-UI/A2UI endpoints can wrap fixture streams

---

## Milestone 9 — Leptos CSR UI (Phase 1)

### UI features
- Connect to SSE stream endpoint
- Render:
  - chat deltas
  - thinking/reasoning
  - tool calls and results
  - citations
  - artifact previews
- Switch between:
  - fixture streams
  - live runs

### Acceptance tests
- UI renders a fixture stream fully
- UI can start a run and stream output

---

# Acceptance Checklist (Phase 1)

- [ ] `POST /api/runs` creates a run
- [ ] `GET /api/runs/:id/stream` streams normalized events
- [ ] `GET /api/ag-ui/runs/:id/stream` streams AG-UI mapped chunks
- [ ] `GET /api/a2ui/runs/:id/stream` streams A2UI mapped events
- [ ] Tools: internal tool executes and emits lifecycle events
- [ ] Tools: mocked MCP tool executes and emits lifecycle events
- [ ] Skills: stored + matched + injected into prompt assembly
- [ ] A2A: messages deliver via endpoint and SSE stream
- [ ] UI: fixture stream renders; live run renders

---

# Concrete TODO List (ordered)

1. Create `crates/domain` with event + artifact + tool + skill + run models
2. Create `crates/api` with:
   - run create
   - normalized SSE
   - AG-UI adapter SSE
   - A2UI adapter SSE
3. Create `RunManager` and `EventBus` abstraction
4. Add `fixtures` and a fixture SSE endpoint
5. Create `crates/tools` internal tool registry and executor
6. Create `crates/skills` store + matcher + overlay injection
7. Add `crates/providers` stub provider (returns fixture or echo)
8. Add `A2A` endpoints and stream
9. Create `web/leptos-csr` stream consumer + renderer
10. Add `cargo test` gates for parsing, ordering, adapters

---

# End of Document

