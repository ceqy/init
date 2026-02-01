-- 启用 Row-Level Security (RLS)
-- Enable Row-Level Security for tenant isolation

-- 物料类型表 RLS
ALTER TABLE material_types ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_material_types ON material_types
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- 物料组表 RLS
ALTER TABLE material_groups ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_material_groups ON material_groups
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- 物料主表 RLS
ALTER TABLE materials ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_materials ON materials
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- 物料工厂视图 RLS
ALTER TABLE material_plant_data ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_material_plant_data ON material_plant_data
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- 物料销售视图 RLS
ALTER TABLE material_sales_data ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_material_sales_data ON material_sales_data
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- 物料采购视图 RLS
ALTER TABLE material_purchase_data ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_material_purchase_data ON material_purchase_data
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- 物料仓储视图 RLS
ALTER TABLE material_storage_data ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_material_storage_data ON material_storage_data
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- 物料会计视图 RLS
ALTER TABLE material_accounting_data ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_material_accounting_data ON material_accounting_data
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- 物料质量视图 RLS
ALTER TABLE material_quality_data ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_material_quality_data ON material_quality_data
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- 物料单位换算 RLS
ALTER TABLE material_unit_conversions ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_material_unit_conversions ON material_unit_conversions
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- 替代物料 RLS
ALTER TABLE material_alternatives ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_material_alternatives ON material_alternatives
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

-- 注释
COMMENT ON POLICY tenant_isolation_materials ON materials IS '租户隔离策略，确保每个租户只能访问自己的数据';
