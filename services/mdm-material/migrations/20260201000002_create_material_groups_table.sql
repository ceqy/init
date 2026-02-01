-- 物料组表
-- Material Groups Table
-- 支持层级结构的物料分类

CREATE TABLE IF NOT EXISTS material_groups (
    -- 主键
    id UUID PRIMARY KEY,

    -- 租户隔离
    tenant_id UUID NOT NULL,

    -- 业务字段
    code VARCHAR(20) NOT NULL,                    -- 物料组代码
    name VARCHAR(100) NOT NULL,                   -- 物料组名称
    localized_name JSONB DEFAULT '{}'::jsonb,     -- 多语言名称

    -- 层级结构
    parent_id UUID REFERENCES material_groups(id) ON DELETE SET NULL,  -- 父级物料组
    level INTEGER NOT NULL DEFAULT 0,             -- 层级深度 (0=根节点)
    path TEXT NOT NULL DEFAULT '',                -- 物化路径，如 /root/parent/current
    is_leaf BOOLEAN NOT NULL DEFAULT TRUE,        -- 是否叶子节点

    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,

    -- 约束
    CONSTRAINT uk_material_groups_tenant_code UNIQUE (tenant_id, code),
    CONSTRAINT chk_material_groups_level CHECK (level >= 0)
);

-- 索引
CREATE INDEX idx_material_groups_tenant_id ON material_groups(tenant_id);
CREATE INDEX idx_material_groups_parent_id ON material_groups(parent_id);
CREATE INDEX idx_material_groups_path ON material_groups(path);
CREATE INDEX idx_material_groups_tenant_parent ON material_groups(tenant_id, parent_id);

-- 注释
COMMENT ON TABLE material_groups IS '物料组表，支持层级结构的物料分类';
COMMENT ON COLUMN material_groups.path IS '物化路径，用于快速查询所有子节点';
COMMENT ON COLUMN material_groups.is_leaf IS '是否叶子节点，只有叶子节点可以关联物料';
