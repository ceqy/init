-- Outbox 表 - 用于可靠的事件发布
-- 事件先写入此表（与业务数据同一事务），后台进程异步发布

CREATE TABLE IF NOT EXISTS outbox (
    id UUID PRIMARY KEY,
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_id UUID NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ,
    retry_count INT NOT NULL DEFAULT 0,
    last_error TEXT
);

-- 索引：快速查找未发布的事件
CREATE INDEX idx_outbox_unpublished ON outbox(created_at) WHERE published_at IS NULL;

-- 索引：按聚合类型和 ID 查询
CREATE INDEX idx_outbox_aggregate ON outbox(aggregate_type, aggregate_id);

COMMENT ON TABLE outbox IS '事件发件箱 - Outbox 模式保证事件发布的最终一致性';
COMMENT ON COLUMN outbox.aggregate_type IS '聚合根类型 (Role, Policy 等)';
COMMENT ON COLUMN outbox.aggregate_id IS '聚合根 ID';
COMMENT ON COLUMN outbox.event_type IS '事件类型 (RoleCreated, RoleUpdated 等)';
COMMENT ON COLUMN outbox.payload IS '事件负载 (JSON)';
COMMENT ON COLUMN outbox.published_at IS '发布时间，NULL 表示未发布';
COMMENT ON COLUMN outbox.retry_count IS '重试次数';
COMMENT ON COLUMN outbox.last_error IS '最后错误信息';
