"""Pydantic models mirroring server DTOs."""

from typing import Any

from pydantic import BaseModel


# =============================================================================
# Chat API Types
# =============================================================================


class ChatResponse(BaseModel):
    """Response from starting a chat."""

    session_id: str
    stream_url: str


class Message(BaseModel):
    """A message in a conversation."""

    role: str
    content: str


# =============================================================================
# Runs API Types
# =============================================================================


class RunResponse(BaseModel):
    """Response from creating a run."""

    id: str
    stream_url: str


# =============================================================================
# Knowledge Base API Types
# =============================================================================


class KnowledgeBaseConfig(BaseModel):
    """Knowledge base configuration."""

    embedding_provider: str = ""
    embedding_model: str = ""
    vector_dimensions: int | None = None
    file_processor: str = ""
    chunk_strategy: str = ""


class KnowledgeBase(BaseModel):
    """A knowledge base."""

    id: str
    name: str
    description: str | None = None
    config: KnowledgeBaseConfig = KnowledgeBaseConfig()
    created_at: str
    updated_at: str


class CreateKnowledgeBaseRequest(BaseModel):
    """Request to create a knowledge base."""

    name: str
    description: str | None = None


class Document(BaseModel):
    """A document in a knowledge base."""

    id: str
    kb_id: str
    filename: str
    mime_type: str | None = None
    chunk_count: int
    status: str
    error_message: str | None = None


class SearchResult(BaseModel):
    """A single search result."""

    content: str
    score: float
    metadata: dict[str, Any]
    document_id: str | None = None


class SearchResponse(BaseModel):
    """Response from a knowledge base search."""

    results: list[SearchResult]


# =============================================================================
# Ingest API Types
# =============================================================================


class IngestResponse(BaseModel):
    """Response from ingestion."""

    success: bool
    chunk_count: int
