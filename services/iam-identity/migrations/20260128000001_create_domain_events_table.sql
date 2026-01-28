-- 创建领域事件存储表
-- 用于持久化 IAM 服务的所有领域事件

CREATE TABLE IF NOT EXISTS domain_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(100) NOT NULL,
    aggregate_type VARCHAR(50) NOT NULL,
    aggregate_id VARCHAR(100) NOT NULL,
    tenant_id UUID NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引：按租户查询
CREATE INDEX IF NOT EXISTS idx_domain_events_tenant_id ON domain_events(tenant_id);

-- 索引：按聚合类型查询
CREATE INDEX IF NOT EXISTS idx_domain_events_aggregate_type ON domain_events(aggregate_type);

-- 索引：按事件类型查询
CREATE INDEX IF NOT EXISTS idx_domain_events_event_type ON domain_events(event_type);

-- 索引：按时间范围查询
CREATE INDEX IF NOT EXISTS idx_domain_events_created_at ON domain_events(created_at DESC);

-- 复合索引：按聚合 ID 和类型查询事件历史
CREATE INDEX IF NOT EXISTS idx_domain_events_aggregate ON domain_events(aggregate_type, aggregate_id, created_at DESC);

-- 启用 RLS
ALTER TABLE domain_events ENABLE ROW LEVEL SECURITY;

-- RLS 策略：租户隔离
DROP POLICY IF EXISTS domain_events_tenant_policy ON domain_events;
CREATE POLICY domain_events_tenant_policy ON domain_events
    USING (tenant_id::text = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id::text = current_setting('app.current_tenant_id', true));

COMMENT ON TABLE domain_events IS '领域事件存储表，用于审计和事件溯源';
COMMENT ON COLUMN domain_events.event_type IS '事件类型，如 UserCreated, UserLoggedIn';
COMMENT ON COLUMN domain_events.aggregate_type IS '聚合类型，如 User, OAuthClient, Session';
COMMENT ON COLUMN domain_events.aggregate_id IS '聚合根 ID';
COMMENT ON COLUMN domain_events.payload IS 'JSON 格式的事件数据';
