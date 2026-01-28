-- 修复 Session 表索引和 Event Store 版本冲突问题
-- 1. 添加 Session 表缺失的复合索引
-- 2. 为 Event Store 添加版本唯一约束

-- ============================================
-- 1. Sessions 表：添加过期时间和撤销状态的复合索引
-- ============================================
-- 查询模式：WHERE tenant_id = ? AND expires_at > NOW()
-- 用于清理过期会话和查询有效会话
CREATE INDEX IF NOT EXISTS idx_sessions_tenant_expires ON sessions(tenant_id, expires_at);

-- 查询模式：WHERE tenant_id = ? AND revoked = false
-- 用于查询未撤销的会话
CREATE INDEX IF NOT EXISTS idx_sessions_tenant_revoked ON sessions(tenant_id, revoked);

-- 查询模式：WHERE tenant_id = ? AND user_id = ? AND revoked = false
-- 用于查询用户的有效会话
CREATE INDEX IF NOT EXISTS idx_sessions_tenant_user_revoked ON sessions(tenant_id, user_id, revoked) WHERE revoked = false;

-- ============================================
-- 2. Event Store 表：添加版本唯一约束
-- ============================================
-- 首先检查 event_store 表是否存在
DO $$
BEGIN
    -- 如果表不存在，创建它
    IF NOT EXISTS (SELECT FROM pg_tables WHERE schemaname = 'public' AND tablename = 'event_store') THEN
        CREATE TABLE event_store (
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

        RAISE NOTICE 'Created event_store table';
    END IF;
END $$;

-- 添加唯一约束：防止同一聚合的版本冲突
-- 这是 Event Sourcing 的核心约束，确保乐观并发控制
ALTER TABLE event_store
    DROP CONSTRAINT IF EXISTS uk_event_store_aggregate_version;

ALTER TABLE event_store
    ADD CONSTRAINT uk_event_store_aggregate_version
    UNIQUE (aggregate_type, aggregate_id, version);

-- 添加索引以优化查询性能
CREATE INDEX IF NOT EXISTS idx_event_store_aggregate ON event_store(aggregate_type, aggregate_id, version DESC);
CREATE INDEX IF NOT EXISTS idx_event_store_occurred_at ON event_store(occurred_at DESC);

-- 添加注释
COMMENT ON CONSTRAINT uk_event_store_aggregate_version ON event_store IS
    '防止版本冲突：同一聚合的版本号必须唯一，实现乐观并发控制';

COMMENT ON INDEX idx_event_store_aggregate IS
    '优化按聚合查询事件历史的性能';

-- ============================================
-- 3. 性能优化说明
-- ============================================
-- Sessions 表：
--   - idx_sessions_tenant_expires: 支持定期清理过期会话的批量操作
--   - idx_sessions_tenant_revoked: 支持查询用户的有效会话列表
--   - idx_sessions_tenant_user_revoked (部分索引): 仅索引未撤销的会话，减少索引大小
--
-- Event Store 表：
--   - uk_event_store_aggregate_version: 在数据库层面保证版本一致性
--   - idx_event_store_aggregate: 支持按聚合 ID 查询事件流
--   - idx_event_store_occurred_at: 支持按时间范围查询事件
