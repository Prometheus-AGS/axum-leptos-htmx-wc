# **PMPO, Universal Agent Runtime, and the LLM Provider Framework**





This document is an **authoritative, execution-oriented artifact** intended to guide AI code-generation agents (including Google AntiGravity IDE agents) in implementing **Phase 1** of the Prometheus Agentic platform inside the **Axum + Leptos + HTMX + Web Components reference playground**.



It explains:



- What **PMPO** is and why it exists
- What a **Universal Agent Runtime (UAR)** is
- What the **LLM Provider Framework** is
- How all three combine into a single coherent execution model
- What exactly must be built in **Phase 1**, and what must *not* yet be built





This document is not marketing. It is an **instructional systems design artifact**.



------





## **1. PMPO (Plan–Monitor–Perform–Observe)**







### **1.1 Definition**





**PMPO** is the execution loop that governs all agent behavior in Prometheus.



PMPO stands for:



1. **Plan** – Decide what to do
2. **Monitor** – Track execution and intermediate state
3. **Perform** – Execute actions (LLM calls, tools, UI emission)
4. **Observe** – Interpret results and update state





PMPO is *not* a prompt. It is a **runtime control loop**.



------





### **1.2 Why PMPO Exists**





Naïve agent systems:



- Stream tokens directly to the UI
- Call tools opportunistically
- Collapse reasoning, execution, and output into a single step





PMPO explicitly **separates decision-making from execution**, which enables:



- Deterministic tool execution
- Multi-agent orchestration
- UI lane separation (tokens vs tools vs artifacts)
- Replayable and inspectable runs





------





### **1.3 PMPO Loop Structure**



```
User Input
   ↓
PLAN
   ↓
MONITOR (state + constraints)
   ↓
PERFORM (LLM + tools + artifacts)
   ↓
OBSERVE (results → state)
   ↓
PLAN (next step or finalize)
```

Each iteration may emit **multiple streaming events**.



------





### **1.4 PMPO in Phase 1**





Phase 1 PMPO requirements:



- Single-agent PMPO loop
- No multi-agent planning yet
- No speculative execution
- No memory graph reasoning





**Allowed**:



- Sequential LLM → tool → LLM cycles
- Streaming token output
- Tool aggregation





**Not allowed yet**:



- Parallel planners
- Agent self-spawning
- Cross-run reasoning





------





## **2. Universal Agent Runtime (UAR)**







### **2.1 Definition**





The **Universal Agent Runtime** is the **execution host** for PMPO.



It is the component that:



- Owns run lifecycle
- Routes input to agents
- Activates skills
- Executes tools
- Emits structured events
- Persists run state





The UAR **does not care about HTTP, SSE, OpenAI, or UI**.



Those are adapters.



------





### **2.2 What the UAR Is** 

### **Not**





The UAR is **not**:



- An OpenAI proxy
- A web framework
- A UI renderer
- A database





It is a **pure execution engine**.



------





### **2.3 Core UAR Contracts**





In Phase 1, the UAR exposes exactly one execution entrypoint:

```
UniversalAgentRuntime::start_run(RunContext, EventSink)
```

Where:



- RunContext describes the execution input
- EventSink receives structured runtime events





------





### **2.4 RunContext**





A run is protocol-neutral.

```
struct RunContext {
  run_id
  session_id
  messages
  user_input
}
```

A RunContext may originate from:



- OpenAI Chat Completions
- OpenAI Responses API
- AG-UI UI input
- Tests or scripts





------





### **2.5 EventSink**





The UAR emits **typed semantic events**, not strings:

```
enum UarEvent {
  TokenDelta
  ToolsProgress
  BundleHtml
  Citation
  Status
  Error
  Done
}
```

These map directly to **UI lanes**.



------





### **2.6 UAR Responsibilities in Phase 1**





The UAR must:



- Select a default agent
- Inject system prompt + skills
- Call the LLM via the provider framework
- Execute tools via MCP
- Stream events through the sink





The UAR must **not**:



- Render HTML
- Serialize SSE
- Format OpenAI responses





------





## **3. LLM Provider Framework**







### **3.1 Definition**





The **LLM Provider Framework** abstracts:



- Model vendors
- Protocol differences
- Streaming semantics
- Tool-call formats





It guarantees that **all upstream LLMs become the same internal event stream**.



------





### **3.2 Providers vs Protocols**





Providers:



- OpenAI
- Azure OpenAI
- Ollama
- vLLM





Protocols:



- Chat Completions
- Responses API
- OpenAI-compatible streaming





Providers ≠ Protocols.



The framework normalizes both.



------





### **3.3 Normalized Events**





All providers emit:

```
NormalizedEvent::MessageDelta
NormalizedEvent::ToolCallDelta
NormalizedEvent::ToolCallComplete
NormalizedEvent::ToolResult
NormalizedEvent::Usage
NormalizedEvent::Done
```

These are **provider-agnostic**.



------





### **3.4 Tool Execution**





Tool calls:



- Are discovered via MCP at startup
- Are executed **only by the server**
- Are never executed by the model





The LLM *requests* tools.



The UAR *controls* tools.



------





## **4. How PMPO + UAR + Provider Framework Combine**







### **4.1 Combined Architecture**



```
HTTP / SSE / OpenAI API
        ↓
   Adapter Layer
        ↓
Universal Agent Runtime
        ↓
   PMPO Loop
        ↓
LLM Provider Framework
        ↓
MCP Tool Execution
```



------





### **4.2 Streaming Model**





During a run, the following streams coexist:



- Token stream → TokenDelta
- Tool lifecycle → ToolsProgress
- Structural UI → BundleHtml
- Errors → Error





These must be **independently addressable**.



------





### **4.3 UI Contract**





The runtime never renders UI.



It emits:



- semantic events
- HTML fragments (only when required)





The frontend decides how to render.



------





## **5. Phase 1 Implementation Requirements (MANDATORY)**







### **5.1 What Must Be Built**





Agents must implement **Phase 1 only**:



- UniversalAgentRuntime module
- Agent registry with default agent
- RunContext + EventSink
- SSE projection adapter
- PMPO loop using existing orchestrator





------





### **5.2 What Must Be Preserved**





DO NOT remove:



- Existing streaming optimizations
- Existing NormalizedEvent model
- Existing MCP registry
- Existing HTMX/Web Component UI





Phase 1 wraps these — it does not replace them.



------





### **5.3 Explicit Non-Goals (Phase 1)**





Do **not** implement:



- Multi-agent planners
- Skill embeddings
- Memory graphs
- Speculative execution
- Cross-run learning





These come later.



------





## **6. Execution Plan for AI Code Agents**







### **Step 1: Create** 

### **src/uar/**

###  **modules**





- runtime.rs
- agent.rs
- registry.rs
- run.rs
- skills.rs
- events.rs
- sinks/sse.rs







### **Step 2: Inject Runtime into AppState**







### **Step 3: Replace direct orchestrator usage in** 

### **/api/chat/stream**







### **Step 4: Adapt runtime events to existing SSE format**







### **Step 5: Verify no UI regressions**





------





## **7. Success Criteria**





Phase 1 is complete when:



- Agents are defined as structs, not prompts
- /api/chat/stream no longer calls the orchestrator directly
- Streaming UX is unchanged
- Tool calls remain deterministic
- The runtime can later support multiple projections





------





## **8. Final Notes for Code-Generation Agents**





- Favor **explicit types over inference**
- Prefer **composition over inheritance**
- Do not inline runtime logic into handlers
- Treat PMPO as a control loop, not a prompt





This repository is the **reference execution playground**.



Do not over-engineer Phase 1.



Correct structure > features.