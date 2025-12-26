"""Async HTTP client for the API."""

from typing import Any

import httpx

from .types import (
    ChatResponse,
    CreateKnowledgeBaseRequest,
    Document,
    IngestResponse,
    KnowledgeBase,
    Message,
    RunResponse,
    SearchResponse,
)


class ApiError(Exception):
    """API error with status code."""

    def __init__(self, status: int, message: str) -> None:
        super().__init__(message)
        self.status = status
        self.message = message


class Client:
    """Async HTTP client for the API.

    Example:
        >>> async with Client("http://localhost:3000") as client:
        ...     response = await client.chat("Hello!")
        ...     print(response.stream_url)
    """

    def __init__(self, base_url: str) -> None:
        """Initialize the client.

        Args:
            base_url: The base URL of the server (e.g., "http://localhost:3000")
        """
        self.base_url = base_url.rstrip("/")
        self._http = httpx.AsyncClient()

    async def __aenter__(self) -> "Client":
        return self

    async def __aexit__(self, *args: object) -> None:
        await self._http.aclose()

    async def close(self) -> None:
        """Close the HTTP client."""
        await self._http.aclose()

    # ─────────────────────────────────────────────────────────────────────────
    # Chat API
    # ─────────────────────────────────────────────────────────────────────────

    async def chat(
        self, message: str, session_id: str | None = None
    ) -> ChatResponse:
        """Send a chat message.

        Args:
            message: The user's message.
            session_id: Optional session ID to continue an existing conversation.

        Returns:
            ChatResponse with session_id and stream_url.
        """
        res = await self._http.post(
            f"{self.base_url}/api/chat",
            json={"message": message, "session_id": session_id},
        )
        self._check_response(res)
        return ChatResponse.model_validate(res.json())

    async def get_messages(self, session_id: str) -> list[Message]:
        """Get messages for a session.

        Args:
            session_id: The session ID.

        Returns:
            List of messages in the session.
        """
        res = await self._http.get(
            f"{self.base_url}/api/sessions/{session_id}/messages"
        )
        self._check_response(res)
        return [Message.model_validate(m) for m in res.json()]

    # ─────────────────────────────────────────────────────────────────────────
    # Runs API
    # ─────────────────────────────────────────────────────────────────────────

    async def create_run(
        self, input_text: str, context: dict[str, Any] | None = None
    ) -> RunResponse:
        """Create a new run.

        Args:
            input_text: The input prompt for the run.
            context: Optional context or configuration.

        Returns:
            RunResponse with id and stream_url.
        """
        res = await self._http.post(
            f"{self.base_url}/api/runs",
            json={"input": input_text, "context": context},
        )
        self._check_response(res)
        return RunResponse.model_validate(res.json())

    def run_stream_url(self, run_id: str) -> str:
        """Get the stream URL for a run.

        Args:
            run_id: The run ID.

        Returns:
            The SSE stream URL.
        """
        return f"{self.base_url}/api/runs/{run_id}/stream"

    # ─────────────────────────────────────────────────────────────────────────
    # Knowledge Base API
    # ─────────────────────────────────────────────────────────────────────────

    async def list_knowledge_bases(self) -> list[KnowledgeBase]:
        """List all knowledge bases.

        Returns:
            List of knowledge bases.
        """
        res = await self._http.get(f"{self.base_url}/api/knowledge")
        self._check_response(res)
        return [KnowledgeBase.model_validate(kb) for kb in res.json()]

    async def create_knowledge_base(
        self, request: CreateKnowledgeBaseRequest
    ) -> KnowledgeBase:
        """Create a new knowledge base.

        Args:
            request: The creation request.

        Returns:
            The created knowledge base.
        """
        res = await self._http.post(
            f"{self.base_url}/api/knowledge",
            json=request.model_dump(exclude_none=True),
        )
        self._check_response(res)
        return KnowledgeBase.model_validate(res.json())

    async def get_knowledge_base(self, kb_id: str) -> KnowledgeBase:
        """Get a knowledge base by ID.

        Args:
            kb_id: The knowledge base ID.

        Returns:
            The knowledge base.
        """
        res = await self._http.get(f"{self.base_url}/api/knowledge/{kb_id}")
        self._check_response(res)
        return KnowledgeBase.model_validate(res.json())

    async def delete_knowledge_base(self, kb_id: str) -> None:
        """Delete a knowledge base.

        Args:
            kb_id: The knowledge base ID.
        """
        res = await self._http.delete(f"{self.base_url}/api/knowledge/{kb_id}")
        self._check_response(res)

    async def list_documents(self, kb_id: str) -> list[Document]:
        """List documents in a knowledge base.

        Args:
            kb_id: The knowledge base ID.

        Returns:
            List of documents.
        """
        res = await self._http.get(
            f"{self.base_url}/api/knowledge/{kb_id}/documents"
        )
        self._check_response(res)
        return [Document.model_validate(d) for d in res.json()]

    async def search(
        self,
        kb_id: str,
        query: str,
        limit: int = 5,
        min_score: float = 0.7,
    ) -> SearchResponse:
        """Search a knowledge base.

        Args:
            kb_id: The knowledge base ID.
            query: The search query.
            limit: Maximum number of results.
            min_score: Minimum similarity score.

        Returns:
            Search response with results.
        """
        res = await self._http.post(
            f"{self.base_url}/api/knowledge/{kb_id}/search",
            json={"query": query, "limit": limit, "min_score": min_score},
        )
        self._check_response(res)
        return SearchResponse.model_validate(res.json())

    # ─────────────────────────────────────────────────────────────────────────
    # Ingest API
    # ─────────────────────────────────────────────────────────────────────────

    async def ingest(
        self, content: str, metadata: dict[str, Any] | None = None
    ) -> IngestResponse:
        """Ingest content.

        Args:
            content: The content to ingest.
            metadata: Optional metadata.

        Returns:
            Ingest response.
        """
        res = await self._http.post(
            f"{self.base_url}/api/ingest",
            json={"content": content, "metadata": metadata},
        )
        self._check_response(res)
        return IngestResponse.model_validate(res.json())

    # ─────────────────────────────────────────────────────────────────────────
    # Helpers
    # ─────────────────────────────────────────────────────────────────────────

    def _check_response(self, res: httpx.Response) -> None:
        """Check response and raise ApiError if not successful."""
        if not res.is_success:
            raise ApiError(res.status_code, res.text)
