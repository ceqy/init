-- 物料采购视图表
-- Material Purchase Data Table
-- 存储物料在各采购组织的采购数据

CREATE TABLE IF NOT EXISTS material_purchase_data (
    -- 主键
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- 关联
    material_id UUID NOT NULL REFERENCES materials(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,

    -- 组织数据
    purchase_org VARCHAR(10) NOT NULL,            -- 采购组织
    plant VARCHAR(10),                            -- 工厂（可选）

    -- 采购数据
    purchase_unit VARCHAR(10),                    -- 采购单位
    purchasing_group VARCHAR(10),                 -- 采购组
    order_unit VARCHAR(10),                       -- 订单单位
    planned_delivery_days INTEGER DEFAULT 0,      -- 计划交货天数
    gr_processing_days INTEGER DEFAULT 0,         -- 收货处理天数
    under_delivery_tolerance DECIMAL(5, 2),       -- 欠交容差 (%)
    over_delivery_tolerance DECIMAL(5, 2),        -- 超交容差 (%)
    unlimited_over_delivery BOOLEAN DEFAULT FALSE, -- 无限超交

    -- 供应商
    preferred_vendor_id VARCHAR(40),              -- 首选供应商

    -- 价格
    standard_price_amount DECIMAL(15, 2),         -- 标准价格金额
    standard_price_currency VARCHAR(3),           -- 标准价格货币
    price_unit DECIMAL(15, 3) DEFAULT 1,          -- 价格单位

    -- 状态
    status SMALLINT NOT NULL DEFAULT 0,           -- 采购级物料状态

    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,

    -- 约束
    CONSTRAINT uk_material_purchase_data UNIQUE (material_id, purchase_org, COALESCE(plant, ''))
);

-- 索引
CREATE INDEX idx_material_purchase_data_material_id ON material_purchase_data(material_id);
CREATE INDEX idx_material_purchase_data_tenant_id ON material_purchase_data(tenant_id);
CREATE INDEX idx_material_purchase_data_purchase_org ON material_purchase_data(purchase_org);
CREATE INDEX idx_material_purchase_data_tenant_purchase_org ON material_purchase_data(tenant_id, purchase_org);

-- 注释
COMMENT ON TABLE material_purchase_data IS '物料采购视图，存储物料在各采购组织的采购数据';
