# Universal Agent Runtime: Vision & Gap Analysis

## Executive Summary
The current codebase has successfully established the **plumbing** for a Universal Agent Runtime (UAR). We have a Phase 1 implementation that separates execution from transport, utilizing Axum 0.10 (0.8+), streaming SSE events, and basic PMPO (Plan-Monitor-Perform-Observe) looping.

However, the **intelligence layer** is nascent. The system currently acts as a passive runner for a single, hardcoded agent (`DefaultAgent`). The vision is to evolve this into a dynamic **Agent Operating System** that actively manages intent, routing, context, and knowledge server-side, enabling rich A2UI experiences driven by data rather than code.

## Current State vs. Vision

| Feature Area | Current State (Phase 1) | Vision (Target State) |
| :--- | :--- | :--- |
| **Framework** | Axum 0.8 (Up to date) | **Axum 0.8** (Maintained) |
| **Agent Spec** | Hardcoded `DefaultAgent` struct with static strings. | **Declarative Specs**: Agents defined by data (JSON/DB) including system prompts, IO metadata, and capability manifests. |
| **Routing** | Direct path to default agent. | **Intent Classification**: Input analysis to route requests to the best-fit agent (e.g., "Code Agent" vs "Creative Agent") based on complexity and skill requirements. |
| **Storage** | In-memory `AgentRegistry` (empty/default). | **Server-Side Behavior**: Postgres-backed storage for Agents, Prompts, and Configs. System behavior is data-driven. |
| **Memory** | Ephemeral or simple chat history session file. | **Semantic Memory**: Configurable memory scopes (short-term, long-term) with user preferences persistence. |
| **Context** | "Send All" strategy (full history passed to LLM). | **Smart Context Management**: Sliding windows, token pruning, and retrieval-augmented context (similar to Cherry Studio patterns). |
| **Knowledge** | None / Stateless. | **Knowledge Base**: Server-side document storage (PGVector) with embeddings for RAG (Retrieval Augmented Generation). |

## Detailed Gap Analysis

### 1. Agent Specification & Metadata
**Gap**: Agents are currently Rust structs. To scale, they must be data entities.
**Requirement**:
- Define agents in a database or config files.
- structured IO metadata: What inputs does this agent accept? What A2UI components does it emit?
- System Prompts stored as data, allowing dynamic updates without recompilation.

### 2. Intent Routing & Complexity Analysis
**Gap**: The system assumes every request is a simple chat turn for the default agent.
**Requirement**:
- Implement an **Ingress Supervisor** (Orchestrator Agent) that analyzes user input *before* execution.
- **Routing Logic**: `(Input) -> [Intent Classifier] -> (Target Agent ID)`.
- **Complexity Assessment**: Determine if a query needs a simple LLM call or a multi-step PMPO plan.

### 3. Server-Side Configuration & Storage
**Gap**: Configuration is static or environment-based. Use of filesystem for persistent state is limited.
**Requirement**:
- **Postgres as Brain**: Store Agent definitions, Skills, and User configurations in Postgres.
- **Behavioral Configuration**: "System prompts" and "Temperature" settings should be user/admin configurable at runtime, stored in the DB.

### 4. Context Management
**Gap**: We currently pass `session.messages()` directly to the LLM. This will fail with long conversations.
**Requirement**:
- **Token Management**: Implement token counting and windowing strategies (e.g., "keep last N tokens", "summarize older turns").
- **Context Construction**: dynamically assemble the context window from:
    1. Active sliding window (recent turns)
    2. Relevant memory/facts (injected system messages)
    3. Retrieved knowledge (RAG snippets)

### 5. Knowledge Base (RAG)
**Gap**: The agent has no access to external documents or persistent knowledge.
**Requirement**:
- **Ingestion Pipeline**: Upload docs -> Chunk -> Embed -> Store in `pgvector`.
- **Retrieval Tool**: Agents need a "Recall" skill to query this knowledge base during the PMPO loop.

## Conclusion
We have built the **Body** (Runtime, Transport, IO), but we are missing the **Brain** (Memory, Knowledge, Decision Making). The next phase of development must focus on moving hardcoded logic into dynamic, database-backed structures and implementing the "Supervisor" patterns that make agents truly autonomous and context-aware.
