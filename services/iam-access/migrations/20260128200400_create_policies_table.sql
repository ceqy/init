-- 创建策略表

CREATE TABLE IF NOT EXISTS policies (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    effect VARCHAR(10) NOT NULL CHECK (effect IN ('ALLOW', 'DENY')),
    subjects JSONB NOT NULL DEFAULT '[]',
    resources JSONB NOT NULL DEFAULT '[]',
    actions JSONB NOT NULL DEFAULT '[]',
    conditions JSONB,
    priority INT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,
    
    -- 同一租户内策略名称唯一
    CONSTRAINT uk_policies_tenant_name UNIQUE (tenant_id, name)
);

-- 创建索引
CREATE INDEX idx_policies_tenant_id ON policies(tenant_id);
CREATE INDEX idx_policies_is_active ON policies(is_active);
CREATE INDEX idx_policies_effect ON policies(effect);
CREATE INDEX idx_policies_priority ON policies(priority DESC);

-- GIN 索引用于 JSONB 查询
CREATE INDEX idx_policies_subjects ON policies USING GIN (subjects);
CREATE INDEX idx_policies_resources ON policies USING GIN (resources);
CREATE INDEX idx_policies_actions ON policies USING GIN (actions);

-- 添加注释
COMMENT ON TABLE policies IS '策略表 - 用于 ABAC 访问控制';
COMMENT ON COLUMN policies.effect IS '策略效果: ALLOW 或 DENY';
COMMENT ON COLUMN policies.subjects IS '主体匹配模式，JSON 数组';
COMMENT ON COLUMN policies.resources IS '资源匹配模式，JSON 数组';
COMMENT ON COLUMN policies.actions IS '操作匹配模式，JSON 数组';
COMMENT ON COLUMN policies.conditions IS '条件表达式，JSON 对象';
COMMENT ON COLUMN policies.priority IS '优先级，数值越大优先级越高';
