-- 物料会计视图表
-- Material Accounting Data Table
-- 存储物料在各工厂的评估和会计数据

CREATE TABLE IF NOT EXISTS material_accounting_data (
    -- 主键
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- 关联
    material_id UUID NOT NULL REFERENCES materials(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,

    -- 组织数据
    plant VARCHAR(10) NOT NULL,                   -- 工厂
    valuation_area VARCHAR(10) NOT NULL,          -- 评估范围

    -- 评估数据
    valuation_class VARCHAR(10),                  -- 评估类
    valuation_category VARCHAR(10),               -- 评估类别
    price_control SMALLINT NOT NULL DEFAULT 0,    -- 价格控制 (0=未指定, 1=标准价, 2=移动平均价)

    -- 价格
    standard_price_amount DECIMAL(15, 2),         -- 标准价格金额
    standard_price_currency VARCHAR(3),           -- 标准价格货币
    moving_average_price_amount DECIMAL(15, 2),   -- 移动平均价金额
    moving_average_price_currency VARCHAR(3),     -- 移动平均价货币
    price_unit DECIMAL(15, 3) DEFAULT 1,          -- 价格单位

    -- 会计科目
    inventory_account VARCHAR(20),                -- 库存科目
    price_difference_account VARCHAR(20),         -- 价差科目
    cost_element VARCHAR(20),                     -- 成本要素

    -- 成本核算
    costing_lot_size DECIMAL(15, 3),              -- 成本核算批量
    with_qty_structure BOOLEAN DEFAULT FALSE,     -- 是否有数量结构

    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,

    -- 约束
    CONSTRAINT uk_material_accounting_data UNIQUE (material_id, plant, valuation_area)
);

-- 索引
CREATE INDEX idx_material_accounting_data_material_id ON material_accounting_data(material_id);
CREATE INDEX idx_material_accounting_data_tenant_id ON material_accounting_data(tenant_id);
CREATE INDEX idx_material_accounting_data_plant ON material_accounting_data(plant);
CREATE INDEX idx_material_accounting_data_tenant_plant ON material_accounting_data(tenant_id, plant);

-- 注释
COMMENT ON TABLE material_accounting_data IS '物料会计视图，存储物料在各工厂的评估和会计数据';
COMMENT ON COLUMN material_accounting_data.price_control IS '0=未指定, 1=标准价格, 2=移动平均价';
