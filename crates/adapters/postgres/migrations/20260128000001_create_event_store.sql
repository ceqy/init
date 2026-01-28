-- Create event_store table for testing
CREATE TABLE IF NOT EXISTS event_store (
    id UUID PRIMARY KEY,
    aggregate_type VARCHAR(50) NOT NULL,
    aggregate_id VARCHAR(100) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    version BIGINT NOT NULL,
    payload TEXT NOT NULL,
    metadata TEXT NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add unique constraint for version conflict detection
ALTER TABLE event_store
    ADD CONSTRAINT uk_event_store_aggregate_version
    UNIQUE (aggregate_type, aggregate_id, version);

-- Add indexes for query performance
CREATE INDEX IF NOT EXISTS idx_event_store_aggregate
    ON event_store(aggregate_type, aggregate_id, version DESC);

CREATE INDEX IF NOT EXISTS idx_event_store_occurred_at
    ON event_store(occurred_at DESC);
