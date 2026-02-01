-- 物料单位换算表
-- Material Unit Conversions Table
-- 存储物料的单位换算关系

CREATE TABLE IF NOT EXISTS material_unit_conversions (
    -- 主键
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- 关联
    material_id UUID NOT NULL REFERENCES materials(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,

    -- 换算数据
    from_unit VARCHAR(10) NOT NULL,               -- 源单位
    to_unit VARCHAR(10) NOT NULL,                 -- 目标单位
    numerator DECIMAL(15, 6) NOT NULL,            -- 分子
    denominator DECIMAL(15, 6) NOT NULL DEFAULT 1, -- 分母

    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,

    -- 约束
    CONSTRAINT uk_material_unit_conversions UNIQUE (material_id, from_unit, to_unit),
    CONSTRAINT chk_denominator_not_zero CHECK (denominator != 0)
);

-- 索引
CREATE INDEX idx_material_unit_conversions_material_id ON material_unit_conversions(material_id);
CREATE INDEX idx_material_unit_conversions_tenant_id ON material_unit_conversions(tenant_id);

-- 注释
COMMENT ON TABLE material_unit_conversions IS '物料单位换算表，存储物料的单位换算关系';
COMMENT ON COLUMN material_unit_conversions.numerator IS '换算因子分子，换算公式: to_unit = from_unit * (numerator / denominator)';
