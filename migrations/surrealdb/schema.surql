-- SurrealDB Schema for UAR Knowledge Base System
-- Run with: surreal import schema.surql --conn ws://localhost:8000

-- =============================================================================
-- Sessions
-- =============================================================================

DEFINE TABLE sessions SCHEMAFULL;
DEFINE FIELD id ON sessions TYPE string;
DEFINE FIELD data ON sessions TYPE object;
DEFINE FIELD created_at ON sessions TYPE datetime;
DEFINE FIELD updated_at ON sessions TYPE datetime;
DEFINE INDEX idx_sessions_id ON sessions FIELDS id UNIQUE;

-- =============================================================================
-- Skills
-- =============================================================================

DEFINE TABLE skills SCHEMAFULL;
DEFINE FIELD skill_id ON skills TYPE string;
DEFINE FIELD name ON skills TYPE string;
DEFINE FIELD description ON skills TYPE option<string>;
DEFINE FIELD definition ON skills TYPE object;
DEFINE FIELD embedding ON skills TYPE array<float>;
DEFINE FIELD created_at ON skills TYPE datetime;
DEFINE FIELD updated_at ON skills TYPE datetime;
DEFINE INDEX idx_skills_id ON skills FIELDS skill_id UNIQUE;

-- =============================================================================
-- Knowledge Bases
-- =============================================================================

DEFINE TABLE knowledge_bases SCHEMAFULL;
DEFINE FIELD id ON knowledge_bases TYPE string;
DEFINE FIELD name ON knowledge_bases TYPE string;
DEFINE FIELD description ON knowledge_bases TYPE option<string>;
DEFINE FIELD config ON knowledge_bases TYPE object;
DEFINE FIELD created_at ON knowledge_bases TYPE datetime;
DEFINE FIELD updated_at ON knowledge_bases TYPE datetime;
DEFINE INDEX idx_kb_id ON knowledge_bases FIELDS id UNIQUE;
DEFINE INDEX idx_kb_name ON knowledge_bases FIELDS name UNIQUE;

-- =============================================================================
-- Knowledge Documents
-- =============================================================================

DEFINE TABLE knowledge_documents SCHEMAFULL;
DEFINE FIELD id ON knowledge_documents TYPE string;
DEFINE FIELD kb_id ON knowledge_documents TYPE string;
DEFINE FIELD filename ON knowledge_documents TYPE string;
DEFINE FIELD file_path ON knowledge_documents TYPE option<string>;
DEFINE FIELD mime_type ON knowledge_documents TYPE option<string>;
DEFINE FIELD chunk_count ON knowledge_documents TYPE int;
DEFINE FIELD status ON knowledge_documents TYPE object;
DEFINE FIELD error_message ON knowledge_documents TYPE option<string>;
DEFINE FIELD created_at ON knowledge_documents TYPE datetime;
DEFINE FIELD updated_at ON knowledge_documents TYPE datetime;
DEFINE INDEX idx_doc_id ON knowledge_documents FIELDS id UNIQUE;
DEFINE INDEX idx_doc_kb ON knowledge_documents FIELDS kb_id;

-- =============================================================================
-- Knowledge Chunks
-- =============================================================================

DEFINE TABLE knowledge_chunks SCHEMAFULL;
DEFINE FIELD id ON knowledge_chunks TYPE string;
DEFINE FIELD kb_id ON knowledge_chunks TYPE string;
DEFINE FIELD document_id ON knowledge_chunks TYPE option<string>;
DEFINE FIELD content ON knowledge_chunks TYPE string;
DEFINE FIELD metadata ON knowledge_chunks TYPE option<object>;
DEFINE FIELD embedding ON knowledge_chunks TYPE array<float>;
DEFINE FIELD created_at ON knowledge_chunks TYPE datetime;
DEFINE INDEX idx_chunk_id ON knowledge_chunks FIELDS id UNIQUE;
DEFINE INDEX idx_chunk_kb ON knowledge_chunks FIELDS kb_id;
DEFINE INDEX idx_chunk_doc ON knowledge_chunks FIELDS document_id;

-- =============================================================================
-- Agents
-- =============================================================================

DEFINE TABLE agents SCHEMAFULL;
DEFINE FIELD id ON agents TYPE string;
DEFINE FIELD data ON agents TYPE object;
DEFINE FIELD created_at ON agents TYPE datetime;
DEFINE FIELD updated_at ON agents TYPE datetime;
DEFINE INDEX idx_agents_id ON agents FIELDS id UNIQUE;

-- =============================================================================
-- Memories
-- =============================================================================

DEFINE TABLE memories SCHEMAFULL;
DEFINE FIELD id ON memories TYPE string;
DEFINE FIELD data ON memories TYPE object;
DEFINE FIELD embedding ON memories TYPE array<float>;
DEFINE FIELD created_at ON memories TYPE datetime;
DEFINE INDEX idx_memories_id ON memories FIELDS id UNIQUE;

-- =============================================================================
-- GraphRAG: Entities (for future use)
-- =============================================================================

DEFINE TABLE entities SCHEMAFULL;
DEFINE FIELD id ON entities TYPE string;
DEFINE FIELD canonical_name ON entities TYPE string;
DEFINE FIELD entity_type ON entities TYPE string;
DEFINE FIELD description ON entities TYPE option<string>;
DEFINE FIELD embedding ON entities TYPE array<float>;
DEFINE FIELD source_chunk_ids ON entities TYPE array<string>;
DEFINE FIELD created_at ON entities TYPE datetime;
DEFINE INDEX idx_entity_id ON entities FIELDS id UNIQUE;
DEFINE INDEX idx_entity_name ON entities FIELDS canonical_name;

-- =============================================================================
-- GraphRAG: Relationships (Graph Edges)
-- =============================================================================

DEFINE TABLE relates SCHEMAFULL TYPE RELATION IN entities OUT entities;
DEFINE FIELD relation_type ON relates TYPE string;
DEFINE FIELD weight ON relates TYPE float;
DEFINE FIELD description ON relates TYPE option<string>;
DEFINE FIELD source_chunk_id ON relates TYPE string;
DEFINE FIELD created_at ON relates TYPE datetime;

-- =============================================================================
-- GraphRAG: Communities
-- =============================================================================

DEFINE TABLE communities SCHEMAFULL;
DEFINE FIELD id ON communities TYPE string;
DEFINE FIELD level ON communities TYPE int;
DEFINE FIELD entity_ids ON communities TYPE array<string>;
DEFINE FIELD summary ON communities TYPE string;
DEFINE FIELD embedding ON communities TYPE array<float>;
DEFINE FIELD created_at ON communities TYPE datetime;
DEFINE INDEX idx_community_id ON communities FIELDS id UNIQUE;
DEFINE INDEX idx_community_level ON communities FIELDS level;
