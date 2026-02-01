-- 物料工厂视图表
-- Material Plant Data Table
-- 存储物料在各工厂的 MRP、采购、库存等数据

CREATE TABLE IF NOT EXISTS material_plant_data (
    -- 主键
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- 关联
    material_id UUID NOT NULL REFERENCES materials(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,

    -- 组织数据
    plant VARCHAR(10) NOT NULL,                   -- 工厂代码

    -- MRP 数据
    mrp_type VARCHAR(10),                         -- MRP 类型
    mrp_controller VARCHAR(10),                   -- MRP 控制者
    reorder_point DECIMAL(15, 3) DEFAULT 0,       -- 再订货点
    safety_stock DECIMAL(15, 3) DEFAULT 0,        -- 安全库存
    lot_size VARCHAR(10),                         -- 批量大小
    minimum_lot_size DECIMAL(15, 3),              -- 最小批量
    maximum_lot_size DECIMAL(15, 3),              -- 最大批量
    fixed_lot_size DECIMAL(15, 3),                -- 固定批量
    rounding_value DECIMAL(15, 3),                -- 舍入值
    planned_delivery_days INTEGER DEFAULT 0,      -- 计划交货天数
    gr_processing_days INTEGER DEFAULT 0,         -- 收货处理天数

    -- 采购数据
    procurement_type SMALLINT NOT NULL DEFAULT 0, -- 采购类型 (0=未指定, 1=外部, 2=内部, 3=两者)
    special_procurement VARCHAR(10),              -- 特殊采购类型
    production_storage_location VARCHAR(10),      -- 生产存储位置

    -- 库存管理
    batch_management BOOLEAN NOT NULL DEFAULT FALSE,  -- 批次管理
    serial_number_profile VARCHAR(10),            -- 序列号配置文件
    abc_indicator VARCHAR(1),                     -- ABC 指标

    -- 状态
    status SMALLINT NOT NULL DEFAULT 0,           -- 工厂级物料状态

    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,

    -- 约束
    CONSTRAINT uk_material_plant_data UNIQUE (material_id, plant)
);

-- 索引
CREATE INDEX idx_material_plant_data_material_id ON material_plant_data(material_id);
CREATE INDEX idx_material_plant_data_tenant_id ON material_plant_data(tenant_id);
CREATE INDEX idx_material_plant_data_plant ON material_plant_data(plant);
CREATE INDEX idx_material_plant_data_tenant_plant ON material_plant_data(tenant_id, plant);

-- 注释
COMMENT ON TABLE material_plant_data IS '物料工厂视图，存储物料在各工厂的 MRP 和采购数据';
COMMENT ON COLUMN material_plant_data.procurement_type IS '0=未指定, 1=外部采购, 2=内部生产, 3=两者皆可';
