-- 物料仓储视图表
-- Material Storage Data Table
-- 存储物料在各仓库和存储位置的仓储数据

CREATE TABLE IF NOT EXISTS material_storage_data (
    -- 主键
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- 关联
    material_id UUID NOT NULL REFERENCES materials(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,

    -- 组织数据
    plant VARCHAR(10) NOT NULL,                   -- 工厂
    storage_location VARCHAR(10) NOT NULL,        -- 存储位置

    -- 仓库管理
    warehouse_number VARCHAR(10),                 -- 仓库编号
    storage_type VARCHAR(10),                     -- 存储类型
    storage_bin VARCHAR(20),                      -- 存储仓位
    picking_area VARCHAR(10),                     -- 拣货区域

    -- 库存限制
    max_storage_quantity DECIMAL(15, 3),          -- 最大存储数量
    min_storage_quantity DECIMAL(15, 3),          -- 最小存储数量
    replenishment_quantity DECIMAL(15, 3),        -- 补货数量

    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,

    -- 约束
    CONSTRAINT uk_material_storage_data UNIQUE (material_id, plant, storage_location)
);

-- 索引
CREATE INDEX idx_material_storage_data_material_id ON material_storage_data(material_id);
CREATE INDEX idx_material_storage_data_tenant_id ON material_storage_data(tenant_id);
CREATE INDEX idx_material_storage_data_plant ON material_storage_data(plant);
CREATE INDEX idx_material_storage_data_tenant_plant ON material_storage_data(tenant_id, plant);

-- 注释
COMMENT ON TABLE material_storage_data IS '物料仓储视图，存储物料在各仓库的仓储数据';
