-- 物料类型表
-- Material Types Table
-- 定义物料的基本类型，如原材料、半成品、成品等

CREATE TABLE IF NOT EXISTS material_types (
    -- 主键
    id UUID PRIMARY KEY,

    -- 租户隔离
    tenant_id UUID NOT NULL,

    -- 业务字段
    code VARCHAR(10) NOT NULL,                    -- 物料类型代码，如 ROH, HALB, FERT
    name VARCHAR(100) NOT NULL,                   -- 物料类型名称
    localized_name JSONB DEFAULT '{}'::jsonb,     -- 多语言名称 {"zh": "原材料", "en": "Raw Material"}

    -- 控制参数
    quantity_update BOOLEAN NOT NULL DEFAULT TRUE,    -- 是否更新数量
    value_update BOOLEAN NOT NULL DEFAULT TRUE,       -- 是否更新价值
    internal_procurement BOOLEAN NOT NULL DEFAULT FALSE,  -- 内部采购
    external_procurement BOOLEAN NOT NULL DEFAULT TRUE,   -- 外部采购

    -- 默认值
    default_valuation_class VARCHAR(10),          -- 默认评估类
    default_price_control SMALLINT NOT NULL DEFAULT 0,  -- 默认价格控制 (0=未指定, 1=标准价, 2=移动平均价)

    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,

    -- 约束
    CONSTRAINT uk_material_types_tenant_code UNIQUE (tenant_id, code)
);

-- 索引
CREATE INDEX idx_material_types_tenant_id ON material_types(tenant_id);
CREATE INDEX idx_material_types_code ON material_types(code);

-- 注释
COMMENT ON TABLE material_types IS '物料类型表，定义物料的基本分类';
COMMENT ON COLUMN material_types.code IS '物料类型代码，如 ROH(原材料), HALB(半成品), FERT(成品)';
COMMENT ON COLUMN material_types.default_price_control IS '0=未指定, 1=标准价格, 2=移动平均价';
