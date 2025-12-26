"""axum-leptos-htmx-wc Python SDK.

Provides a typed async HTTP client for interacting with the server.

Example:
    >>> import asyncio
    >>> from axum_leptos_htmx_wc_sdk import Client
    >>>
    >>> async def main():
    ...     client = Client("http://localhost:3000")
    ...     
    ...     # Chat API
    ...     response = await client.chat("Hello!")
    ...     print(f"Stream URL: {response.stream_url}")
    ...     
    ...     # Knowledge API
    ...     kbs = await client.list_knowledge_bases()
    ...     for kb in kbs:
    ...         print(f"KB: {kb.name} ({kb.id})")
    >>>
    >>> asyncio.run(main())
"""

from .client import Client
from .types import (
    ChatResponse,
    CreateKnowledgeBaseRequest,
    Document,
    KnowledgeBase,
    KnowledgeBaseConfig,
    Message,
    RunResponse,
    SearchResponse,
    SearchResult,
)

__version__ = "0.1.0"
__all__ = [
    "Client",
    "ChatResponse",
    "CreateKnowledgeBaseRequest",
    "Document",
    "KnowledgeBase",
    "KnowledgeBaseConfig",
    "Message",
    "RunResponse",
    "SearchResponse",
    "SearchResult",
]
