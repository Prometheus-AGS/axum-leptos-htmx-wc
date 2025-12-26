CREATE TABLE IF NOT EXISTS memories (
    id TEXT PRIMARY KEY,
    agent_id TEXT, -- NULL for global/system memory
    content TEXT NOT NULL,
    tags TEXT[],
    embedding VECTOR(384),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_memories_agent_id ON memories(agent_id);
-- Vector index
CREATE INDEX IF NOT EXISTS idx_memories_embedding ON memories USING hnsw (embedding vector_cosine_ops);
