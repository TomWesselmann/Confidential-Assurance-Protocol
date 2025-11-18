-- Policy Store Schema
-- Migration 001: Create policies table for persistent policy storage

CREATE TABLE IF NOT EXISTS policies (
    id TEXT PRIMARY KEY,              -- UUID v4
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    hash TEXT NOT NULL UNIQUE,        -- SHA3-256 (0x-prefixed)
    status TEXT NOT NULL,             -- 'active', 'deprecated', 'draft'
    created_at TEXT NOT NULL,         -- ISO 8601
    updated_at TEXT NOT NULL,         -- ISO 8601
    description TEXT,
    policy_json TEXT NOT NULL,        -- Original Policy JSON
    compiled_bytes BLOB               -- Optional compiled bytes
);

-- Index for fast hash lookups
CREATE INDEX IF NOT EXISTS idx_policies_hash ON policies(hash);

-- Index for status filtering
CREATE INDEX IF NOT EXISTS idx_policies_status ON policies(status);

-- Index for sorting by created_at
CREATE INDEX IF NOT EXISTS idx_policies_created_at ON policies(created_at DESC);
