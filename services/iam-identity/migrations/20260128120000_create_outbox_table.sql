-- Create outbox table for transactional outbox pattern
CREATE TABLE IF NOT EXISTS outbox (
    id VARCHAR(255) PRIMARY KEY,
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_id VARCHAR(255) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    payload TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMP WITH TIME ZONE,

    -- Index for querying pending messages
    CONSTRAINT check_processed_at CHECK (processed_at IS NULL OR processed_at >= created_at)
);

-- Index for efficient pending message queries
CREATE INDEX idx_outbox_pending ON outbox (created_at) WHERE processed_at IS NULL;

-- Index for cleanup of processed messages
CREATE INDEX idx_outbox_processed ON outbox (processed_at) WHERE processed_at IS NOT NULL;

-- Index for aggregate tracking
CREATE INDEX idx_outbox_aggregate ON outbox (aggregate_type, aggregate_id);

-- Comment on table
COMMENT ON TABLE outbox IS 'Transactional outbox pattern for reliable event publishing';
COMMENT ON COLUMN outbox.id IS 'Unique message identifier (UUID)';
COMMENT ON COLUMN outbox.aggregate_type IS 'Type of aggregate that generated the event';
COMMENT ON COLUMN outbox.aggregate_id IS 'ID of the aggregate instance';
COMMENT ON COLUMN outbox.event_type IS 'Type of domain event';
COMMENT ON COLUMN outbox.payload IS 'JSON serialized event payload';
COMMENT ON COLUMN outbox.created_at IS 'When the message was created';
COMMENT ON COLUMN outbox.processed_at IS 'When the message was successfully processed and published';
