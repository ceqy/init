-- 物料质量视图表
-- Material Quality Data Table
-- 存储物料在各工厂的质量管理数据

CREATE TABLE IF NOT EXISTS material_quality_data (
    -- 主键
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- 关联
    material_id UUID NOT NULL REFERENCES materials(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,

    -- 组织数据
    plant VARCHAR(10) NOT NULL,                   -- 工厂

    -- 质量检验
    inspection_active BOOLEAN NOT NULL DEFAULT FALSE,  -- 是否启用检验
    inspection_type VARCHAR(10),                  -- 检验类型
    inspection_interval INTEGER,                  -- 检验间隔（天）
    sample_percentage DECIMAL(5, 2),              -- 抽样百分比

    -- 保质期
    shelf_life_days INTEGER,                      -- 保质期（天）
    remaining_shelf_life_days INTEGER,            -- 剩余保质期（天）

    -- 证书
    certificate_type VARCHAR(10),                 -- 证书类型
    certificate_required BOOLEAN DEFAULT FALSE,   -- 是否需要证书

    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,

    -- 约束
    CONSTRAINT uk_material_quality_data UNIQUE (material_id, plant)
);

-- 索引
CREATE INDEX idx_material_quality_data_material_id ON material_quality_data(material_id);
CREATE INDEX idx_material_quality_data_tenant_id ON material_quality_data(tenant_id);
CREATE INDEX idx_material_quality_data_plant ON material_quality_data(plant);
CREATE INDEX idx_material_quality_data_tenant_plant ON material_quality_data(tenant_id, plant);

-- 注释
COMMENT ON TABLE material_quality_data IS '物料质量视图，存储物料在各工厂的质量管理数据';
