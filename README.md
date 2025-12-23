# Axum + Leptos + HTMX + Web Components

## Agentic Streaming LLM Application (MCP-First, Tauri-Ready)

This repository is a reference implementation and living architecture example for building agentic AI applications that:
	•	support tool-first LLM interaction
	•	stream rich, typed model output
	•	remain HTML-first and inspectable
	•	avoid heavyweight SPA frameworks
	•	run identically as:
	•	a web app
	•	a desktop app (via Tauri)
	•	a mobile app (via Tauri)

This is not a demo toy.
There are no mocks.
Everything is wired against real protocols, real streaming, and real tools from day one.

⸻

High-Level Goals

This project exists to prove (and then serve as a template for):

	1.	Always-on tool use with LLMs
	•	The server is always an MCP client
	•	Tools are discovered dynamically from mcp.json
	•	The model can call any tool at any time
	•	Tool execution is deterministic and server-controlled
	2.	First-class streaming
	•	Token streaming
	•	Tool call streaming
	•	Tool result streaming
	•	Structured chunk types (thinking, reasoning, citations, memory, errors)
	•	A future-proof AG-UI-style event model
	3.	Protocol flexibility without UI changes
	•	OpenAI Chat Completions
	•	OpenAI Responses
	•	OpenAI-compatible backends (Ollama, vLLM, etc.)
	•	One internal event contract for the UI
	4.	HTML-centric UI composition
	•	HTMX 2.0.8 for navigation and server interaction
	•	Web Components for client-side programmability
	•	Alpine.js for local UI reactivity
	•	No React, Next.js, Vue, or SPA routers
	5.	Tauri compatibility from day one
	•	No CDN scripts
	•	No secrets in the browser
	•	Same UI for web, desktop, and mobile

⸻

Architecture Overview

┌─────────────────────────────┐
│          Client UI          │
│                             │
│  HTMX 2.0.8                 │
│  Web Components (TS)        │
│  Alpine.js                  │
│                             │
│  AG-UI-style SSE rendering  │
└──────────────┬──────────────┘
               │ SSE (typed events)
┌──────────────▼──────────────┐
│        Axum Server           │
│                              │
│  LLM Orchestrator            │
│  - Chat Completions          │
│  - Responses                 │
│  - Streaming + Non-Streaming │
│                              │
│  MCP Client (rmcp)           │
│  - stdio servers             │
│  - remote HTTP servers       │
│                              │
│  Tool Loop                   │
│  - assemble tool calls       │
│  - execute tools             │
│  - feed results back         │
└──────────────┬──────────────┘
               │ MCP (stdio / HTTP)
┌──────────────▼──────────────┐
│        MCP Servers           │
│                              │
│  time                        │
│  tavily                      │
│  (future: db, fs, graph…)   │
└─────────────────────────────┘


⸻

Core Design Principles

1. Tools Are Non-Optional

This system assumes:
	•	The model will call tools
	•	The model should reason with tools
	•	The model cannot execute tools itself

Therefore:
	•	The server is always an MCP client
	•	Tools are discovered dynamically at startup
	•	Tools are available to every request
	•	Tool execution is deterministic, auditable, and server-side

⸻

2. Streaming Is the Default

All LLM interaction supports:
	•	streaming responses
	•	streaming tool calls
	•	streaming tool results

The server normalizes all upstream streaming into a single internal event model, regardless of whether the upstream protocol is:
	•	Chat Completions
	•	Responses
	•	OpenAI-compatible proxies

The client never has to care which protocol is used.

⸻

3. One Internal Event Contract

Internally, everything becomes typed events:
	•	message.delta
	•	tool_call.delta
	•	tool_call.complete
	•	tool_result
	•	error
	•	done

In parallel, the server mirrors these events into AG-UI-style events:
	•	agui.message.delta
	•	agui.tool_call.delta
	•	agui.tool_call.complete
	•	agui.tool_result
	•	agui.error
	•	agui.done

This allows:
	•	progressive rendering
	•	structured UIs
	•	future AG-UI endpoints without refactoring

⸻

LLM Protocol Support

Supported Today
	•	OpenAI Chat Completions
	•	Streaming
	•	Tool calls
	•	Tool results
	•	OpenAI Responses
	•	Streaming
	•	Rich event types
	•	Tool calls (server-orchestrated)

Planned / Implied
	•	Non-streaming calls (same orchestrator, different mode)
	•	OpenAI-compatible backends:
	•	Ollama
	•	vLLM
	•	custom gateways

The UI and SSE contract do not change.

⸻

MCP (Model Context Protocol)

Why MCP?

MCP provides:
	•	a standard tool interface
	•	language-agnostic tooling
	•	dynamic discovery
	•	isolation between model reasoning and execution

This Project Uses:
	•	rmcp (official Rust MCP SDK)
	•	stdio child-process MCP servers
	•	remote streamable HTTP MCP servers

Example mcp.json

{
  "mcpServers": {
    "time": {
      "command": "npx",
      "args": ["-y", "@mcpcentral/mcp-time"]
    },
    "tavily": {
      "url": "https://mcp.tavily.com/mcp/",
      "env": {
        "TAVILY_API_KEY": "${TAVILY_API_KEY}"
      }
    }
  }
}

Tools are namespaced automatically:
	•	time::now
	•	tavily::search

⸻

UI Stack

HTMX 2.0.8

HTMX is used for:
	•	navigation
	•	server interaction
	•	progressive enhancement
	•	HTML fragment replacement

HTMX is not used for:
	•	streaming tokens
	•	tool orchestration
	•	client state management

⸻

Web Components (TypeScript)

Web Components provide:
	•	a programmable client runtime
	•	clean component boundaries
	•	lifecycle hooks
	•	zero framework lock-in

Planned components include:
	•	<chat-shell>
	•	<chat-messages>
	•	<chat-message-bubble>
	•	<chat-stream>
	•	<chat-tool-call>
	•	<chat-code-block>
	•	<chat-mermaid>
	•	<chat-sources>
	•	<copy-to-clipboard>

All components consume typed SSE events.

⸻

Alpine.js

Alpine is used sparingly for:
	•	local UI state
	•	toggles
	•	animations
	•	transitions
	•	UX polish

Alpine never owns:
	•	business state
	•	model state
	•	tool state

⸻

Styling (ShadCN-Inspired)

The visual system follows:
	•	ShadCN-UI design principles
	•	tokenized spacing and colors
	•	accessible contrast
	•	responsive layouts

This ensures:
	•	web UI == desktop UI == mobile UI
	•	minimal drift when wrapped by Tauri

⸻

Streaming UI Capabilities

The UI is designed to support:
	•	incremental token rendering
	•	expandable tool call panels
	•	tool result previews
	•	code blocks with:
	•	syntax highlighting
	•	copy-to-clipboard
	•	Mermaid diagrams
	•	citations / sources panels
	•	memory and reasoning blocks (when exposed)

Everything is progressively rendered.

⸻

Leptos Usage

Leptos is used for:
	•	server-side rendering of the initial HTML shell
	•	layout composition
	•	avoiding hard-coded HTML in Rust

Leptos is not used for:
	•	client-side reactivity
	•	hydration-heavy SPA logic

This keeps the mental model clean:
	•	server renders HTML
	•	client enhances HTML

⸻

Tauri Compatibility

This project is Tauri-ready by design:
	•	no CDN scripts
	•	all assets served locally
	•	no API keys in the browser
	•	SSE works identically in webview
	•	same UI codebase for web + desktop + mobile

The Axum server can be:
	•	bundled inside Tauri
	•	run as a sidecar
	•	or run remotely

⸻

What This Repository Is (and Is Not)

This Is:
	•	a production-shaped reference
	•	a foundation for agentic apps
	•	a template for tool-first AI systems
	•	a demonstration of HTMX + Web Components at scale

This Is Not:
	•	a React/Next.js alternative
	•	a simple chat demo
	•	a mock or prototype
	•	a client-only LLM app

⸻

Roadmap (Implied)
	•	Dedicated AG-UI endpoint
	•	Rich reasoning / memory chunk rendering
	•	Web Worker integration (PGLite, local vector stores)
	•	ONNX / WebGPU client inference
	•	Multi-agent orchestration
	•	CRDT-backed shared sessions
	•	MCP server hot-reload

⸻

Summary

This repository demonstrates that it is possible to build:
	•	deeply agentic
	•	tool-first
	•	streaming-native
	•	HTML-centric
	•	Tauri-compatible

AI applications without heavyweight SPA frameworks, without client-side secrets, and without sacrificing UX or architectural clarity.

If you understand this repo, you understand the foundation of modern, serious AI application architecture.
