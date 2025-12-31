-- Settings Types Table
CREATE TABLE IF NOT EXISTS settings_types (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    key TEXT NOT NULL UNIQUE,
    description TEXT,
    display_mode TEXT NOT NULL DEFAULT 'form',
    schema JSONB,
    icon_url TEXT
);

-- Settings Table
CREATE TABLE IF NOT EXISTS settings (
    id UUID PRIMARY KEY,
    settings_type_id UUID NOT NULL REFERENCES settings_types(id),
    name TEXT NOT NULL,
    key TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    data JSONB NOT NULL
);

-- Index for querying settings by key
CREATE INDEX IF NOT EXISTS idx_settings_key ON settings(key);
