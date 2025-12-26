//! HTTP client for interacting with the server.

use crate::{
    error::{Error, Result},
    types::*,
};
use url::Url;

/// HTTP client for the API.
///
/// # Example
///
/// ```rust,no_run
/// use axum_leptos_htmx_wc_sdk::Client;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("http://localhost:3000")?;
///
/// // Chat API
/// let chat = client.chat().send("Hello!").await?;
///
/// // Knowledge API
/// let kbs = client.knowledge().list().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    base_url: Url,
    http: reqwest::Client,
}

impl Client {
    /// Create a new client.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL of the server (e.g., "http://localhost:3000")
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        let base_url = Url::parse(base_url.as_ref())?;
        Ok(Self {
            base_url,
            http: reqwest::Client::new(),
        })
    }

    /// Create a new client with a custom reqwest client.
    pub fn with_client(base_url: impl AsRef<str>, http: reqwest::Client) -> Result<Self> {
        let base_url = Url::parse(base_url.as_ref())?;
        Ok(Self { base_url, http })
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    // ─────────────────────────────────────────────────────────────────────────
    // API Accessors
    // ─────────────────────────────────────────────────────────────────────────

    /// Access the Chat API.
    pub fn chat(&self) -> ChatApi<'_> {
        ChatApi { client: self }
    }

    /// Access the Runs API.
    pub fn runs(&self) -> RunsApi<'_> {
        RunsApi { client: self }
    }

    /// Access the Knowledge Base API.
    pub fn knowledge(&self) -> KnowledgeApi<'_> {
        KnowledgeApi { client: self }
    }

    /// Access the Ingest API.
    pub fn ingest(&self) -> IngestApi<'_> {
        IngestApi { client: self }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Internal helpers
    // ─────────────────────────────────────────────────────────────────────────

    fn url(&self, path: &str) -> Url {
        self.base_url
            .join(path)
            .unwrap_or_else(|_| self.base_url.clone())
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status();
        if status.is_success() {
            Ok(response.json().await?)
        } else {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".into());
            Err(Error::Api {
                status: status.as_u16(),
                message,
            })
        }
    }
}

// =============================================================================
// Chat API
// =============================================================================

/// Chat API client.
#[derive(Debug)]
pub struct ChatApi<'a> {
    client: &'a Client,
}

impl<'a> ChatApi<'a> {
    /// Send a chat message.
    ///
    /// Returns the session ID and stream URL for receiving events.
    pub async fn send(&self, message: impl Into<String>) -> Result<ChatResponse> {
        self.send_with_session(message, None).await
    }

    /// Send a chat message in an existing session.
    pub async fn send_with_session(
        &self,
        message: impl Into<String>,
        session_id: Option<String>,
    ) -> Result<ChatResponse> {
        let req = ChatRequest {
            message: message.into(),
            session_id,
        };
        let response = self
            .client
            .http
            .post(self.client.url("/api/chat"))
            .json(&req)
            .send()
            .await?;
        Client::handle_response(response).await
    }

    /// Get messages for a session.
    pub async fn get_messages(&self, session_id: &str) -> Result<Vec<Message>> {
        let response = self
            .client
            .http
            .get(
                self.client
                    .url(&format!("/api/sessions/{session_id}/messages")),
            )
            .send()
            .await?;
        Client::handle_response(response).await
    }
}

// =============================================================================
// Runs API
// =============================================================================

/// Runs API client.
#[derive(Debug)]
pub struct RunsApi<'a> {
    client: &'a Client,
}

impl<'a> RunsApi<'a> {
    /// Create a new run.
    pub async fn create(&self, input: impl Into<String>) -> Result<RunResponse> {
        let req = CreateRunRequest {
            input: input.into(),
            context: None,
        };
        let response = self
            .client
            .http
            .post(self.client.url("/api/runs"))
            .json(&req)
            .send()
            .await?;
        Client::handle_response(response).await
    }

    /// Get the stream URL for a run.
    pub fn stream_url(&self, run_id: &str) -> String {
        self.client
            .url(&format!("/api/runs/{run_id}/stream"))
            .to_string()
    }
}

// =============================================================================
// Knowledge API
// =============================================================================

/// Knowledge Base API client.
#[derive(Debug)]
pub struct KnowledgeApi<'a> {
    client: &'a Client,
}

impl<'a> KnowledgeApi<'a> {
    /// List all knowledge bases.
    pub async fn list(&self) -> Result<Vec<KnowledgeBase>> {
        let response = self
            .client
            .http
            .get(self.client.url("/api/knowledge"))
            .send()
            .await?;
        Client::handle_response(response).await
    }

    /// Create a new knowledge base.
    pub async fn create(&self, req: CreateKnowledgeBaseRequest) -> Result<KnowledgeBase> {
        let response = self
            .client
            .http
            .post(self.client.url("/api/knowledge"))
            .json(&req)
            .send()
            .await?;
        Client::handle_response(response).await
    }

    /// Get a knowledge base by ID.
    pub async fn get(&self, id: &str) -> Result<KnowledgeBase> {
        let response = self
            .client
            .http
            .get(self.client.url(&format!("/api/knowledge/{id}")))
            .send()
            .await?;
        Client::handle_response(response).await
    }

    /// Delete a knowledge base.
    pub async fn delete(&self, id: &str) -> Result<()> {
        let response = self
            .client
            .http
            .delete(self.client.url(&format!("/api/knowledge/{id}")))
            .send()
            .await?;
        if response.status().is_success() {
            Ok(())
        } else {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".into());
            Err(Error::Api {
                status: response.status().as_u16(),
                message,
            })
        }
    }

    /// List documents in a knowledge base.
    pub async fn list_documents(&self, kb_id: &str) -> Result<Vec<Document>> {
        let response = self
            .client
            .http
            .get(
                self.client
                    .url(&format!("/api/knowledge/{kb_id}/documents")),
            )
            .send()
            .await?;
        Client::handle_response(response).await
    }

    /// Search a knowledge base.
    pub async fn search(&self, kb_id: &str, query: impl Into<String>) -> Result<SearchResponse> {
        self.search_with_options(kb_id, query, 5, 0.7).await
    }

    /// Search a knowledge base with options.
    pub async fn search_with_options(
        &self,
        kb_id: &str,
        query: impl Into<String>,
        limit: usize,
        min_score: f32,
    ) -> Result<SearchResponse> {
        let req = SearchRequest {
            query: query.into(),
            limit,
            min_score,
        };
        let response = self
            .client
            .http
            .post(self.client.url(&format!("/api/knowledge/{kb_id}/search")))
            .json(&req)
            .send()
            .await?;
        Client::handle_response(response).await
    }
}

// =============================================================================
// Ingest API
// =============================================================================

/// Ingest API client.
#[derive(Debug)]
pub struct IngestApi<'a> {
    client: &'a Client,
}

impl<'a> IngestApi<'a> {
    /// Ingest content.
    pub async fn ingest(&self, content: impl Into<String>) -> Result<IngestResponse> {
        let req = IngestRequest {
            content: content.into(),
            metadata: None,
        };
        let response = self
            .client
            .http
            .post(self.client.url("/api/ingest"))
            .json(&req)
            .send()
            .await?;
        Client::handle_response(response).await
    }
}
