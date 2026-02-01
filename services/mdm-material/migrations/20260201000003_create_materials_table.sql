-- 物料主表
-- Materials Table
-- 物料主数据的核心表

CREATE TABLE IF NOT EXISTS materials (
    -- 主键
    id UUID PRIMARY KEY,

    -- 租户隔离
    tenant_id UUID NOT NULL,

    -- 基本信息
    material_number VARCHAR(40) NOT NULL,         -- 物料编号
    description VARCHAR(200) NOT NULL,            -- 物料描述
    localized_description JSONB DEFAULT '{}'::jsonb,  -- 多语言描述

    -- 分类信息
    material_type_id UUID NOT NULL REFERENCES material_types(id),
    material_type_code VARCHAR(10) NOT NULL,      -- 冗余存储，避免频繁 JOIN
    material_group_id UUID REFERENCES material_groups(id),
    material_group_code VARCHAR(20),              -- 冗余存储

    -- 基本单位
    base_unit VARCHAR(10) NOT NULL,               -- 基本计量单位

    -- 尺寸和重量
    gross_weight DECIMAL(15, 3),                  -- 毛重
    net_weight DECIMAL(15, 3),                    -- 净重
    weight_unit VARCHAR(10),                      -- 重量单位
    volume DECIMAL(15, 3),                        -- 体积
    volume_unit VARCHAR(10),                      -- 体积单位
    length DECIMAL(15, 3),                        -- 长度
    width DECIMAL(15, 3),                         -- 宽度
    height DECIMAL(15, 3),                        -- 高度
    dimension_unit VARCHAR(10),                   -- 尺寸单位

    -- 状态
    status SMALLINT NOT NULL DEFAULT 0,           -- 数据状态 (0=草稿, 1=活跃, 2=停用, 3=冻结, 4=标记删除)

    -- 扩展属性
    custom_attributes JSONB DEFAULT '{}'::jsonb,  -- 自定义属性

    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,

    -- 约束
    CONSTRAINT uk_materials_tenant_number UNIQUE (tenant_id, material_number)
);

-- 索引
CREATE INDEX idx_materials_tenant_id ON materials(tenant_id);
CREATE INDEX idx_materials_material_number ON materials(material_number);
CREATE INDEX idx_materials_material_type_id ON materials(material_type_id);
CREATE INDEX idx_materials_material_group_id ON materials(material_group_id);
CREATE INDEX idx_materials_status ON materials(status);
CREATE INDEX idx_materials_tenant_type ON materials(tenant_id, material_type_id);
CREATE INDEX idx_materials_tenant_group ON materials(tenant_id, material_group_id);
CREATE INDEX idx_materials_tenant_status ON materials(tenant_id, status);

-- 全文搜索索引
CREATE INDEX idx_materials_description_gin ON materials USING gin(to_tsvector('simple', description));

-- 注释
COMMENT ON TABLE materials IS '物料主表，存储物料的基本信息';
COMMENT ON COLUMN materials.material_number IS '物料编号，租户内唯一';
COMMENT ON COLUMN materials.status IS '0=草稿, 1=活跃, 2=停用, 3=冻结, 4=标记删除';
COMMENT ON COLUMN materials.custom_attributes IS '自定义扩展属性，JSON 格式';
