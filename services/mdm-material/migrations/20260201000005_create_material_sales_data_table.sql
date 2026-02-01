-- 物料销售视图表
-- Material Sales Data Table
-- 存储物料在各销售组织和分销渠道的销售数据

CREATE TABLE IF NOT EXISTS material_sales_data (
    -- 主键
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- 关联
    material_id UUID NOT NULL REFERENCES materials(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,

    -- 组织数据
    sales_org VARCHAR(10) NOT NULL,               -- 销售组织
    distribution_channel VARCHAR(10) NOT NULL,    -- 分销渠道

    -- 销售数据
    sales_unit VARCHAR(10),                       -- 销售单位
    minimum_order_quantity DECIMAL(15, 3),        -- 最小订单数量
    minimum_delivery_quantity DECIMAL(15, 3),     -- 最小交货数量
    delivery_unit VARCHAR(10),                    -- 交货单位
    delivery_unit_quantity DECIMAL(15, 3),        -- 交货单位数量

    -- 定价
    pricing_reference_material VARCHAR(40),       -- 定价参考物料
    item_category_group VARCHAR(10),              -- 项目类别组
    account_assignment_group VARCHAR(10),         -- 科目分配组

    -- 税务
    tax_classification VARCHAR(10),               -- 税务分类

    -- 状态
    status SMALLINT NOT NULL DEFAULT 0,           -- 销售级物料状态

    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,

    -- 约束
    CONSTRAINT uk_material_sales_data UNIQUE (material_id, sales_org, distribution_channel)
);

-- 索引
CREATE INDEX idx_material_sales_data_material_id ON material_sales_data(material_id);
CREATE INDEX idx_material_sales_data_tenant_id ON material_sales_data(tenant_id);
CREATE INDEX idx_material_sales_data_sales_org ON material_sales_data(sales_org);
CREATE INDEX idx_material_sales_data_tenant_sales_org ON material_sales_data(tenant_id, sales_org);

-- 注释
COMMENT ON TABLE material_sales_data IS '物料销售视图，存储物料在各销售组织的销售数据';
