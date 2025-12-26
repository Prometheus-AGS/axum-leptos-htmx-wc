/**
 * TypeScript SDK for axum-leptos-htmx-wc
 *
 * @example
 * ```typescript
 * import { Client } from '@prometheus-ags/axum-leptos-htmx-wc-sdk';
 *
 * const client = new Client('http://localhost:3000');
 *
 * // Chat API
 * const chat = await client.chat.send('Hello!');
 * console.log('Stream URL:', chat.stream_url);
 *
 * // Knowledge API
 * const kbs = await client.knowledge.list();
 * for (const kb of kbs) {
 *   console.log(`KB: ${kb.name} (${kb.id})`);
 * }
 * ```
 */

// =============================================================================
// Types
// =============================================================================

export interface ChatRequest {
  message: string;
  session_id?: string;
}

export interface ChatResponse {
  session_id: string;
  stream_url: string;
}

export interface Message {
  role: string;
  content: string;
}

export interface CreateRunRequest {
  input: string;
  context?: Record<string, unknown>;
}

export interface RunResponse {
  id: string;
  stream_url: string;
}

export interface KnowledgeBase {
  id: string;
  name: string;
  description?: string;
  config: KnowledgeBaseConfig;
  created_at: string;
  updated_at: string;
}

export interface KnowledgeBaseConfig {
  embedding_provider: string;
  embedding_model: string;
  vector_dimensions?: number;
  file_processor: string;
  chunk_strategy: string;
}

export interface CreateKnowledgeBaseRequest {
  name: string;
  description?: string;
}

export interface Document {
  id: string;
  kb_id: string;
  filename: string;
  mime_type?: string;
  chunk_count: number;
  status: string;
  error_message?: string;
}

export interface SearchRequest {
  query: string;
  limit?: number;
  min_score?: number;
}

export interface SearchResponse {
  results: SearchResult[];
}

export interface SearchResult {
  content: string;
  score: number;
  metadata: Record<string, unknown>;
  document_id?: string;
}

// =============================================================================
// API Error
// =============================================================================

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

// =============================================================================
// Client
// =============================================================================

export class Client {
  private baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl.replace(/\/$/, '');
  }

  // ─────────────────────────────────────────────────────────────────────────
  // Chat API
  // ─────────────────────────────────────────────────────────────────────────

  chat = {
    send: async (message: string, sessionId?: string): Promise<ChatResponse> => {
      const res = await fetch(`${this.baseUrl}/api/chat`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ message, session_id: sessionId }),
      });
      return this.handleResponse(res);
    },

    getMessages: async (sessionId: string): Promise<Message[]> => {
      const res = await fetch(`${this.baseUrl}/api/sessions/${sessionId}/messages`);
      return this.handleResponse(res);
    },
  };

  // ─────────────────────────────────────────────────────────────────────────
  // Runs API
  // ─────────────────────────────────────────────────────────────────────────

  runs = {
    create: async (input: string, context?: Record<string, unknown>): Promise<RunResponse> => {
      const res = await fetch(`${this.baseUrl}/api/runs`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ input, context }),
      });
      return this.handleResponse(res);
    },

    stream: (runId: string): EventSource => {
      return new EventSource(`${this.baseUrl}/api/runs/${runId}/stream`);
    },

    streamUrl: (runId: string): string => {
      return `${this.baseUrl}/api/runs/${runId}/stream`;
    },
  };

  // ─────────────────────────────────────────────────────────────────────────
  // Knowledge API
  // ─────────────────────────────────────────────────────────────────────────

  knowledge = {
    list: async (): Promise<KnowledgeBase[]> => {
      const res = await fetch(`${this.baseUrl}/api/knowledge`);
      return this.handleResponse(res);
    },

    create: async (request: CreateKnowledgeBaseRequest): Promise<KnowledgeBase> => {
      const res = await fetch(`${this.baseUrl}/api/knowledge`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(request),
      });
      return this.handleResponse(res);
    },

    get: async (id: string): Promise<KnowledgeBase> => {
      const res = await fetch(`${this.baseUrl}/api/knowledge/${id}`);
      return this.handleResponse(res);
    },

    delete: async (id: string): Promise<void> => {
      const res = await fetch(`${this.baseUrl}/api/knowledge/${id}`, {
        method: 'DELETE',
      });
      if (!res.ok) {
        const message = await res.text();
        throw new ApiError(res.status, message);
      }
    },

    listDocuments: async (kbId: string): Promise<Document[]> => {
      const res = await fetch(`${this.baseUrl}/api/knowledge/${kbId}/documents`);
      return this.handleResponse(res);
    },

    search: async (
      kbId: string,
      query: string,
      limit = 5,
      minScore = 0.7
    ): Promise<SearchResponse> => {
      const res = await fetch(`${this.baseUrl}/api/knowledge/${kbId}/search`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query, limit, min_score: minScore }),
      });
      return this.handleResponse(res);
    },
  };

  // ─────────────────────────────────────────────────────────────────────────
  // Ingest API
  // ─────────────────────────────────────────────────────────────────────────

  ingest = {
    content: async (
      content: string,
      metadata?: Record<string, unknown>
    ): Promise<{ success: boolean; chunk_count: number }> => {
      const res = await fetch(`${this.baseUrl}/api/ingest`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content, metadata }),
      });
      return this.handleResponse(res);
    },
  };

  // ─────────────────────────────────────────────────────────────────────────
  // Helpers
  // ─────────────────────────────────────────────────────────────────────────

  private async handleResponse<T>(res: Response): Promise<T> {
    if (res.ok) {
      return res.json();
    }
    const message = await res.text();
    throw new ApiError(res.status, message);
  }
}

export default Client;
