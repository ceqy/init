-- 替代物料表
-- Material Alternatives Table
-- 存储物料的替代关系

CREATE TABLE IF NOT EXISTS material_alternatives (
    -- 主键
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- 关联
    material_id UUID NOT NULL REFERENCES materials(id) ON DELETE CASCADE,
    alternative_material_id UUID NOT NULL REFERENCES materials(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,

    -- 替代数据
    priority INTEGER NOT NULL DEFAULT 1,          -- 优先级（数字越小优先级越高）
    usage_probability DECIMAL(5, 2),              -- 使用概率 (%)
    plant VARCHAR(10),                            -- 适用工厂（NULL 表示所有工厂）

    -- 有效期
    valid_from TIMESTAMPTZ,                       -- 生效日期
    valid_to TIMESTAMPTZ,                         -- 失效日期

    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,

    -- 约束
    CONSTRAINT uk_material_alternatives UNIQUE (material_id, alternative_material_id, COALESCE(plant, '')),
    CONSTRAINT chk_not_self_alternative CHECK (material_id != alternative_material_id),
    CONSTRAINT chk_valid_date_range CHECK (valid_from IS NULL OR valid_to IS NULL OR valid_from <= valid_to),
    CONSTRAINT chk_priority_positive CHECK (priority > 0)
);

-- 索引
CREATE INDEX idx_material_alternatives_material_id ON material_alternatives(material_id);
CREATE INDEX idx_material_alternatives_alternative_id ON material_alternatives(alternative_material_id);
CREATE INDEX idx_material_alternatives_tenant_id ON material_alternatives(tenant_id);
CREATE INDEX idx_material_alternatives_valid_period ON material_alternatives(valid_from, valid_to);

-- 注释
COMMENT ON TABLE material_alternatives IS '替代物料表，存储物料的替代关系';
COMMENT ON COLUMN material_alternatives.priority IS '优先级，数字越小优先级越高';
