-- 物料事件表（用于事件溯源和变更历史）
CREATE TABLE IF NOT EXISTS material_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL UNIQUE,
    material_id UUID NOT NULL,
    tenant_id UUID NOT NULL,

    -- 事件类型
    event_type VARCHAR(50) NOT NULL,

    -- 事件数据（JSON格式存储完整事件）
    event_data JSONB NOT NULL,

    -- 事件元数据
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id VARCHAR(100),

    -- 聚合版本（用于乐观锁）
    aggregate_version INTEGER NOT NULL DEFAULT 1,

    -- 审计信息
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_material_events_material_id ON material_events(material_id);
CREATE INDEX idx_material_events_tenant_id ON material_events(tenant_id);
CREATE INDEX idx_material_events_event_type ON material_events(event_type);
CREATE INDEX idx_material_events_occurred_at ON material_events(occurred_at DESC);
CREATE INDEX idx_material_events_material_tenant ON material_events(material_id, tenant_id);
CREATE INDEX idx_material_events_material_occurred ON material_events(material_id, occurred_at DESC);

-- 复合索引用于变更历史查询
CREATE INDEX idx_material_events_history_query ON material_events(
    material_id,
    tenant_id,
    occurred_at DESC
);

-- 注释
COMMENT ON TABLE material_events IS '物料领域事件表，用于事件溯源和变更历史追踪';
COMMENT ON COLUMN material_events.event_id IS '事件唯一标识';
COMMENT ON COLUMN material_events.material_id IS '物料ID';
COMMENT ON COLUMN material_events.event_type IS '事件类型：Created, Updated, Activated, Deactivated, Blocked, MarkedForDeletion, Deleted, ExtendedToPlant, ExtendedToSalesOrg';
COMMENT ON COLUMN material_events.event_data IS '完整的事件数据（JSON格式）';
COMMENT ON COLUMN material_events.aggregate_version IS '聚合版本号，用于事件顺序和乐观锁';
