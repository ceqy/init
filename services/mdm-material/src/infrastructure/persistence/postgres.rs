//! PostgreSQL repository implementation

use async_trait::async_trait;
use common::types::{PagedResult, Pagination, TenantId};
use domain_core::{AggregateRoot, Entity};
use errors::{AppError, AppResult};
use sqlx::PgPool;

use crate::domain::entities::{Material, MaterialFilter, MaterialGroup, MaterialSearchResult, MaterialType};
use crate::domain::enums::PriceControl;
use crate::domain::repositories::{MaterialGroupRepository, MaterialRepository, MaterialTypeRepository};
use crate::domain::value_objects::{AlternativeMaterial, MaterialGroupId, MaterialId, MaterialNumber, MaterialTypeId};

use super::converters::{
    accounting_data_from_row, alternative_material_from_row, localized_text_to_json,
    material_from_parts, material_group_from_row, material_type_from_row, plant_data_from_row,
    purchase_data_from_row, quality_data_from_row, sales_data_from_row, storage_data_from_row,
    unit_conversion_from_row,
};
use super::rows::{
    MaterialAccountingDataRow, MaterialAlternativeRow, MaterialGroupRow, MaterialPlantDataRow,
    MaterialPurchaseDataRow, MaterialQualityDataRow, MaterialRow, MaterialSalesDataRow,
    MaterialStorageDataRow, MaterialTypeRow, MaterialUnitConversionRow,
};

// ============================================================================
// MaterialTypeRepository 实现
// ============================================================================

pub struct PostgresMaterialTypeRepository {
    pool: PgPool,
}

impl PostgresMaterialTypeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MaterialTypeRepository for PostgresMaterialTypeRepository {
    async fn find_by_id(
        &self,
        id: &MaterialTypeId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<MaterialType>> {
        let row = sqlx::query_as::<_, MaterialTypeRow>(
            r#"
            SELECT id, tenant_id, code, name, localized_name,
                   quantity_update, value_update, internal_procurement, external_procurement,
                   default_valuation_class, default_price_control,
                   created_at, created_by, updated_at, updated_by
            FROM material_types
            WHERE id = $1 AND tenant_id = $2
            "#,
        )
        .bind(id.0)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("查询物料类型失败: {}", e)))?;

        Ok(row.map(material_type_from_row))
    }

    async fn find_by_code(
        &self,
        code: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<MaterialType>> {
        let row = sqlx::query_as::<_, MaterialTypeRow>(
            r#"
            SELECT id, tenant_id, code, name, localized_name,
                   quantity_update, value_update, internal_procurement, external_procurement,
                   default_valuation_class, default_price_control,
                   created_at, created_by, updated_at, updated_by
            FROM material_types
            WHERE code = $1 AND tenant_id = $2
            "#,
        )
        .bind(code)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("查询物料类型失败: {}", e)))?;

        Ok(row.map(material_type_from_row))
    }

    async fn save(&self, material_type: &MaterialType) -> AppResult<()> {
        let price_control: i16 = match material_type.default_price_control() {
            PriceControl::Unspecified => 0,
            PriceControl::Standard => 1,
            PriceControl::MovingAverage => 2,
        };

        let created_by = material_type.audit_info().created_by.as_ref().map(|u| u.0);
        let updated_by = material_type.audit_info().updated_by.as_ref().map(|u| u.0);

        sqlx::query(
            r#"
            INSERT INTO material_types (
                id, tenant_id, code, name, localized_name,
                quantity_update, value_update, internal_procurement, external_procurement,
                default_valuation_class, default_price_control,
                created_at, created_by, updated_at, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(material_type.id().0)
        .bind(material_type.tenant_id().0)
        .bind(material_type.code())
        .bind(material_type.name())
        .bind(localized_text_to_json(material_type.localized_name()))
        .bind(material_type.quantity_update())
        .bind(material_type.value_update())
        .bind(material_type.internal_procurement())
        .bind(material_type.external_procurement())
        .bind(if material_type.default_valuation_class().is_empty() { None } else { Some(material_type.default_valuation_class()) })
        .bind(price_control)
        .bind(material_type.audit_info().created_at)
        .bind(created_by)
        .bind(material_type.audit_info().updated_at)
        .bind(updated_by)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("保存物料类型失败: {}", e)))?;

        Ok(())
    }

    async fn update(&self, material_type: &MaterialType) -> AppResult<()> {
        let price_control: i16 = match material_type.default_price_control() {
            PriceControl::Unspecified => 0,
            PriceControl::Standard => 1,
            PriceControl::MovingAverage => 2,
        };

        let updated_by = material_type.audit_info().updated_by.as_ref().map(|u| u.0);

        let result = sqlx::query(
            r#"
            UPDATE material_types SET
                name = $1,
                localized_name = $2,
                quantity_update = $3,
                value_update = $4,
                internal_procurement = $5,
                external_procurement = $6,
                default_valuation_class = $7,
                default_price_control = $8,
                updated_at = $9,
                updated_by = $10
            WHERE id = $11 AND tenant_id = $12
            "#,
        )
        .bind(material_type.name())
        .bind(localized_text_to_json(material_type.localized_name()))
        .bind(material_type.quantity_update())
        .bind(material_type.value_update())
        .bind(material_type.internal_procurement())
        .bind(material_type.external_procurement())
        .bind(if material_type.default_valuation_class().is_empty() { None } else { Some(material_type.default_valuation_class()) })
        .bind(price_control)
        .bind(material_type.audit_info().updated_at)
        .bind(updated_by)
        .bind(material_type.id().0)
        .bind(material_type.tenant_id().0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("更新物料类型失败: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("物料类型不存在".to_string()));
        }

        Ok(())
    }

    async fn list(
        &self,
        tenant_id: &TenantId,
        pagination: Pagination,
    ) -> AppResult<PagedResult<MaterialType>> {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM material_types WHERE tenant_id = $1",
        )
        .bind(tenant_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("查询物料类型总数失败: {}", e)))?;

        let rows = sqlx::query_as::<_, MaterialTypeRow>(
            r#"
            SELECT id, tenant_id, code, name, localized_name,
                   quantity_update, value_update, internal_procurement, external_procurement,
                   default_valuation_class, default_price_control,
                   created_at, created_by, updated_at, updated_by
            FROM material_types
            WHERE tenant_id = $1
            ORDER BY code
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id.0)
        .bind(pagination.page_size as i64)
        .bind(((pagination.page - 1) * pagination.page_size) as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("查询物料类型列表失败: {}", e)))?;

        let items: Vec<MaterialType> = rows.into_iter().map(material_type_from_row).collect();

        Ok(PagedResult::new(items, total.0 as u64, &pagination))
    }

    async fn exists_by_code(&self, code: &str, tenant_id: &TenantId) -> AppResult<bool> {
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM material_types WHERE code = $1 AND tenant_id = $2)",
        )
        .bind(code)
        .bind(tenant_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("检查物料类型编码失败: {}", e)))?;

        Ok(result.0)
    }
}

// ============================================================================
// MaterialGroupRepository 实现
// ============================================================================

pub struct PostgresMaterialGroupRepository {
    pool: PgPool,
}

impl PostgresMaterialGroupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MaterialGroupRepository for PostgresMaterialGroupRepository {
    async fn find_by_id(
        &self,
        id: &MaterialGroupId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<MaterialGroup>> {
        let row = sqlx::query_as::<_, MaterialGroupRow>(
            r#"
            SELECT id, tenant_id, code, name, localized_name,
                   parent_id, level, path, is_leaf,
                   created_at, created_by, updated_at, updated_by
            FROM material_groups
            WHERE id = $1 AND tenant_id = $2
            "#,
        )
        .bind(id.0)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("查询物料组失败: {}", e)))?;

        Ok(row.map(material_group_from_row))
    }

    async fn find_by_code(
        &self,
        code: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<MaterialGroup>> {
        let row = sqlx::query_as::<_, MaterialGroupRow>(
            r#"
            SELECT id, tenant_id, code, name, localized_name,
                   parent_id, level, path, is_leaf,
                   created_at, created_by, updated_at, updated_by
            FROM material_groups
            WHERE code = $1 AND tenant_id = $2
            "#,
        )
        .bind(code)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("查询物料组失败: {}", e)))?;

        Ok(row.map(material_group_from_row))
    }

    async fn save(&self, group: &MaterialGroup) -> AppResult<()> {
        let created_by = group.audit_info().created_by.as_ref().map(|u| u.0);
        let updated_by = group.audit_info().updated_by.as_ref().map(|u| u.0);

        sqlx::query(
            r#"
            INSERT INTO material_groups (
                id, tenant_id, code, name, localized_name,
                parent_id, level, path, is_leaf,
                created_at, created_by, updated_at, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(group.id().0)
        .bind(group.tenant_id().0)
        .bind(group.code())
        .bind(group.name())
        .bind(localized_text_to_json(group.localized_name()))
        .bind(group.parent_id().map(|id| id.0))
        .bind(group.level())
        .bind(group.path())
        .bind(group.is_leaf())
        .bind(group.audit_info().created_at)
        .bind(created_by)
        .bind(group.audit_info().updated_at)
        .bind(updated_by)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("保存物料组失败: {}", e)))?;

        Ok(())
    }

    async fn update(&self, group: &MaterialGroup) -> AppResult<()> {
        let updated_by = group.audit_info().updated_by.as_ref().map(|u| u.0);

        let result = sqlx::query(
            r#"
            UPDATE material_groups SET
                name = $1,
                localized_name = $2,
                is_leaf = $3,
                updated_at = $4,
                updated_by = $5
            WHERE id = $6 AND tenant_id = $7
            "#,
        )
        .bind(group.name())
        .bind(localized_text_to_json(group.localized_name()))
        .bind(group.is_leaf())
        .bind(group.audit_info().updated_at)
        .bind(updated_by)
        .bind(group.id().0)
        .bind(group.tenant_id().0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("更新物料组失败: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("物料组不存在".to_string()));
        }

        Ok(())
    }

    async fn delete(&self, id: &MaterialGroupId, tenant_id: &TenantId) -> AppResult<()> {
        // 检查是否有子节点
        let has_children: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM material_groups WHERE parent_id = $1 AND tenant_id = $2)",
        )
        .bind(id.0)
        .bind(tenant_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("检查子节点失败: {}", e)))?;

        if has_children.0 {
            return Err(AppError::validation("无法删除有子节点的物料组".to_string()));
        }

        // 检查是否有关联的物料
        let has_materials: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM materials WHERE material_group_id = $1 AND tenant_id = $2)",
        )
        .bind(id.0)
        .bind(tenant_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("检查关联物料失败: {}", e)))?;

        if has_materials.0 {
            return Err(AppError::validation("无法删除有关联物料的物料组".to_string()));
        }

        let result = sqlx::query(
            "DELETE FROM material_groups WHERE id = $1 AND tenant_id = $2",
        )
        .bind(id.0)
        .bind(tenant_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("删除物料组失败: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("物料组不存在".to_string()));
        }

        Ok(())
    }

    async fn list(
        &self,
        tenant_id: &TenantId,
        parent_id: Option<&MaterialGroupId>,
        pagination: Pagination,
    ) -> AppResult<PagedResult<MaterialGroup>> {
        let (total, rows) = if let Some(parent) = parent_id {
            let total: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM material_groups WHERE tenant_id = $1 AND parent_id = $2",
            )
            .bind(tenant_id.0)
            .bind(parent.0)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("查询物料组总数失败: {}", e)))?;

            let rows = sqlx::query_as::<_, MaterialGroupRow>(
                r#"
                SELECT id, tenant_id, code, name, localized_name,
                       parent_id, level, path, is_leaf,
                       created_at, created_by, updated_at, updated_by
                FROM material_groups
                WHERE tenant_id = $1 AND parent_id = $2
                ORDER BY code
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(tenant_id.0)
            .bind(parent.0)
            .bind(pagination.page_size as i64)
            .bind(((pagination.page - 1) * pagination.page_size) as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("查询物料组列表失败: {}", e)))?;

            (total, rows)
        } else {
            let total: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM material_groups WHERE tenant_id = $1 AND parent_id IS NULL",
            )
            .bind(tenant_id.0)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("查询物料组总数失败: {}", e)))?;

            let rows = sqlx::query_as::<_, MaterialGroupRow>(
                r#"
                SELECT id, tenant_id, code, name, localized_name,
                       parent_id, level, path, is_leaf,
                       created_at, created_by, updated_at, updated_by
                FROM material_groups
                WHERE tenant_id = $1 AND parent_id IS NULL
                ORDER BY code
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(tenant_id.0)
            .bind(pagination.page_size as i64)
            .bind(((pagination.page - 1) * pagination.page_size) as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("查询物料组列表失败: {}", e)))?;

            (total, rows)
        };

        let items: Vec<MaterialGroup> = rows.into_iter().map(material_group_from_row).collect();

        Ok(PagedResult::new(items, total.0 as u64, &pagination))
    }

    async fn find_children(
        &self,
        parent_id: &MaterialGroupId,
        tenant_id: &TenantId,
    ) -> AppResult<Vec<MaterialGroup>> {
        let rows = sqlx::query_as::<_, MaterialGroupRow>(
            r#"
            SELECT id, tenant_id, code, name, localized_name,
                   parent_id, level, path, is_leaf,
                   created_at, created_by, updated_at, updated_by
            FROM material_groups
            WHERE parent_id = $1 AND tenant_id = $2
            ORDER BY code
            "#,
        )
        .bind(parent_id.0)
        .bind(tenant_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("查询子物料组失败: {}", e)))?;

        Ok(rows.into_iter().map(material_group_from_row).collect())
    }

    async fn exists_by_code(&self, code: &str, tenant_id: &TenantId) -> AppResult<bool> {
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM material_groups WHERE code = $1 AND tenant_id = $2)",
        )
        .bind(code)
        .bind(tenant_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("检查物料组编码失败: {}", e)))?;

        Ok(result.0)
    }
}

// ============================================================================
// MaterialRepository 实现
// ============================================================================

pub struct PostgresMaterialRepository {
    pool: PgPool,
}

impl PostgresMaterialRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MaterialRepository for PostgresMaterialRepository {
    async fn find_by_id(
        &self,
        id: &MaterialId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<Material>> {
        // 查询主表
        let row = sqlx::query_as::<_, MaterialRow>(
            r#"
            SELECT id, tenant_id, material_number, description, localized_description,
                   material_type_id, material_type_code, material_group_id, material_group_code,
                   base_unit, gross_weight, net_weight, weight_unit, volume, volume_unit,
                   length, width, height, dimension_unit, status, custom_attributes,
                   created_at, created_by, updated_at, updated_by
            FROM materials
            WHERE id = $1 AND tenant_id = $2
            "#,
        )
        .bind(id.0)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("查询物料失败: {}", e)))?;

        let Some(row) = row else {
            return Ok(None);
        };

        // 加载所有视图数据
        let material = self.load_material_with_views(row).await?;
        Ok(Some(material))
    }

    async fn find_by_number(
        &self,
        number: &MaterialNumber,
        tenant_id: &TenantId,
    ) -> AppResult<Option<Material>> {
        let row = sqlx::query_as::<_, MaterialRow>(
            r#"
            SELECT id, tenant_id, material_number, description, localized_description,
                   material_type_id, material_type_code, material_group_id, material_group_code,
                   base_unit, gross_weight, net_weight, weight_unit, volume, volume_unit,
                   length, width, height, dimension_unit, status, custom_attributes,
                   created_at, created_by, updated_at, updated_by
            FROM materials
            WHERE material_number = $1 AND tenant_id = $2
            "#,
        )
        .bind(number.as_str())
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("查询物料失败: {}", e)))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let material = self.load_material_with_views(row).await?;
        Ok(Some(material))
    }

    async fn save(&self, material: &Material) -> AppResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::database(format!("开启事务失败: {}", e)))?;

        // 保存主表
        self.insert_material(&mut tx, material).await?;

        // 保存工厂视图
        for plant_data in material.plant_data() {
            self.insert_plant_data(&mut tx, material, plant_data).await?;
        }

        // 保存销售视图
        for sales_data in material.sales_data() {
            self.insert_sales_data(&mut tx, material, sales_data).await?;
        }

        // 保存采购视图
        for purchase_data in material.purchase_data() {
            self.insert_purchase_data(&mut tx, material, purchase_data).await?;
        }

        // 保存仓储视图
        for storage_data in material.storage_data() {
            self.insert_storage_data(&mut tx, material, storage_data).await?;
        }

        // 保存会计视图
        for accounting_data in material.accounting_data() {
            self.insert_accounting_data(&mut tx, material, accounting_data).await?;
        }

        // 保存质量视图
        for quality_data in material.quality_data() {
            self.insert_quality_data(&mut tx, material, quality_data).await?;
        }

        // 保存单位换算
        for conversion in material.unit_conversions() {
            self.insert_unit_conversion(&mut tx, material, conversion).await?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::database(format!("提交事务失败: {}", e)))?;

        Ok(())
    }

    async fn update(&self, material: &Material) -> AppResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::database(format!("开启事务失败: {}", e)))?;

        // 更新主表
        self.update_material(&mut tx, material).await?;

        // 删除旧的视图数据并重新插入
        let material_id = material.id().0;
        let tenant_id = material.tenant_id().0;

        // 删除旧数据
        sqlx::query("DELETE FROM material_plant_data WHERE material_id = $1 AND tenant_id = $2")
            .bind(material_id)
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database(format!("删除工厂视图失败: {}", e)))?;

        sqlx::query("DELETE FROM material_sales_data WHERE material_id = $1 AND tenant_id = $2")
            .bind(material_id)
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database(format!("删除销售视图失败: {}", e)))?;

        sqlx::query("DELETE FROM material_purchase_data WHERE material_id = $1 AND tenant_id = $2")
            .bind(material_id)
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database(format!("删除采购视图失败: {}", e)))?;

        sqlx::query("DELETE FROM material_storage_data WHERE material_id = $1 AND tenant_id = $2")
            .bind(material_id)
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database(format!("删除仓储视图失败: {}", e)))?;

        sqlx::query("DELETE FROM material_accounting_data WHERE material_id = $1 AND tenant_id = $2")
            .bind(material_id)
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database(format!("删除会计视图失败: {}", e)))?;

        sqlx::query("DELETE FROM material_quality_data WHERE material_id = $1 AND tenant_id = $2")
            .bind(material_id)
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database(format!("删除质量视图失败: {}", e)))?;

        sqlx::query("DELETE FROM material_unit_conversions WHERE material_id = $1 AND tenant_id = $2")
            .bind(material_id)
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database(format!("删除单位换算失败: {}", e)))?;

        // 重新插入视图数据
        for plant_data in material.plant_data() {
            self.insert_plant_data(&mut tx, material, plant_data).await?;
        }

        for sales_data in material.sales_data() {
            self.insert_sales_data(&mut tx, material, sales_data).await?;
        }

        for purchase_data in material.purchase_data() {
            self.insert_purchase_data(&mut tx, material, purchase_data).await?;
        }

        for storage_data in material.storage_data() {
            self.insert_storage_data(&mut tx, material, storage_data).await?;
        }

        for accounting_data in material.accounting_data() {
            self.insert_accounting_data(&mut tx, material, accounting_data).await?;
        }

        for quality_data in material.quality_data() {
            self.insert_quality_data(&mut tx, material, quality_data).await?;
        }

        for conversion in material.unit_conversions() {
            self.insert_unit_conversion(&mut tx, material, conversion).await?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::database(format!("提交事务失败: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, id: &MaterialId, tenant_id: &TenantId) -> AppResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::database(format!("开启事务失败: {}", e)))?;

        // 删除所有关联数据
        let tables = [
            "material_plant_data",
            "material_sales_data",
            "material_purchase_data",
            "material_storage_data",
            "material_accounting_data",
            "material_quality_data",
            "material_unit_conversions",
            "material_alternatives",
        ];

        for table in tables {
            sqlx::query(&format!(
                "DELETE FROM {} WHERE material_id = $1 AND tenant_id = $2",
                table
            ))
            .bind(id.0)
            .bind(tenant_id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database(format!("删除 {} 失败: {}", table, e)))?;
        }

        // 删除主表
        let result = sqlx::query("DELETE FROM materials WHERE id = $1 AND tenant_id = $2")
            .bind(id.0)
            .bind(tenant_id.0)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::database(format!("删除物料失败: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("物料不存在".to_string()));
        }

        tx.commit()
            .await
            .map_err(|e| AppError::database(format!("提交事务失败: {}", e)))?;

        Ok(())
    }

    async fn list(
        &self,
        tenant_id: &TenantId,
        filter: MaterialFilter,
        pagination: Pagination,
    ) -> AppResult<PagedResult<Material>> {
        // 构建动态查询
        let mut conditions = vec!["tenant_id = $1".to_string()];
        let mut param_idx = 2;

        if filter.material_type_id.is_some() {
            conditions.push(format!("material_type_id = ${}", param_idx));
            param_idx += 1;
        }

        if filter.material_group_id.is_some() {
            conditions.push(format!("material_group_id = ${}", param_idx));
            param_idx += 1;
        }

        if filter.status.is_some() {
            conditions.push(format!("status = ${}", param_idx));
            param_idx += 1;
        }

        if filter.search_term.is_some() {
            conditions.push(format!(
                "(material_number ILIKE ${} OR description ILIKE ${})",
                param_idx,
                param_idx
            ));
            param_idx += 1;
        }

        let where_clause = conditions.join(" AND ");

        // 查询总数
        let count_sql = format!("SELECT COUNT(*) FROM materials WHERE {}", where_clause);
        let mut count_query = sqlx::query_as::<_, (i64,)>(&count_sql).bind(tenant_id.0);

        if let Some(ref type_id) = filter.material_type_id {
            count_query = count_query.bind(type_id.0);
        }
        if let Some(ref group_id) = filter.material_group_id {
            count_query = count_query.bind(group_id.0);
        }
        if let Some(ref status) = filter.status {
            count_query = count_query.bind(*status as i16);
        }
        if let Some(ref term) = filter.search_term {
            count_query = count_query.bind(format!("%{}%", term));
        }

        let total = count_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("查询物料总数失败: {}", e)))?;

        // 查询数据
        let data_sql = format!(
            r#"
            SELECT id, tenant_id, material_number, description, localized_description,
                   material_type_id, material_type_code, material_group_id, material_group_code,
                   base_unit, gross_weight, net_weight, weight_unit, volume, volume_unit,
                   length, width, height, dimension_unit, status, custom_attributes,
                   created_at, created_by, updated_at, updated_by
            FROM materials
            WHERE {}
            ORDER BY material_number
            LIMIT ${} OFFSET ${}
            "#,
            where_clause, param_idx, param_idx + 1
        );

        let mut data_query = sqlx::query_as::<_, MaterialRow>(&data_sql).bind(tenant_id.0);

        if let Some(ref type_id) = filter.material_type_id {
            data_query = data_query.bind(type_id.0);
        }
        if let Some(ref group_id) = filter.material_group_id {
            data_query = data_query.bind(group_id.0);
        }
        if let Some(ref status) = filter.status {
            data_query = data_query.bind(*status as i16);
        }
        if let Some(ref term) = filter.search_term {
            data_query = data_query.bind(format!("%{}%", term));
        }

        let rows = data_query
            .bind(pagination.page_size as i64)
            .bind(((pagination.page - 1) * pagination.page_size) as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("查询物料列表失败: {}", e)))?;

        // 加载每个物料的视图数据
        let mut materials = Vec::with_capacity(rows.len());
        for row in rows {
            let material = self.load_material_with_views(row).await?;
            materials.push(material);
        }

        Ok(PagedResult::new(materials, total.0 as u64, &pagination))
    }

    async fn search(
        &self,
        tenant_id: &TenantId,
        query: &str,
        pagination: Pagination,
    ) -> AppResult<Vec<MaterialSearchResult>> {
        // 简单的模糊搜索实现
        let rows = sqlx::query_as::<_, MaterialRow>(
            r#"
            SELECT id, tenant_id, material_number, description, localized_description,
                   material_type_id, material_type_code, material_group_id, material_group_code,
                   base_unit, gross_weight, net_weight, weight_unit, volume, volume_unit,
                   length, width, height, dimension_unit, status, custom_attributes,
                   created_at, created_by, updated_at, updated_by
            FROM materials
            WHERE tenant_id = $1
              AND (material_number ILIKE $2 OR description ILIKE $2)
            ORDER BY
                CASE WHEN material_number ILIKE $3 THEN 0 ELSE 1 END,
                material_number
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(tenant_id.0)
        .bind(format!("%{}%", query))
        .bind(format!("{}%", query))
        .bind(pagination.page_size as i64)
        .bind(((pagination.page - 1) * pagination.page_size) as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("搜索物料失败: {}", e)))?;

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            let material = self.load_material_with_views(row).await?;
            let score = if material.material_number().as_str().starts_with(query) {
                1.0
            } else {
                0.5
            };
            results.push(MaterialSearchResult {
                material,
                score,
                highlights: vec![],
            });
        }

        Ok(results)
    }

    async fn exists_by_number(
        &self,
        number: &MaterialNumber,
        tenant_id: &TenantId,
    ) -> AppResult<bool> {
        let result: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM materials WHERE material_number = $1 AND tenant_id = $2)",
        )
        .bind(number.as_str())
        .bind(tenant_id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("检查物料编号失败: {}", e)))?;

        Ok(result.0)
    }

    async fn find_alternatives(
        &self,
        material_id: &MaterialId,
        tenant_id: &TenantId,
    ) -> AppResult<Vec<AlternativeMaterial>> {
        let rows = sqlx::query_as::<_, MaterialAlternativeRow>(
            r#"
            SELECT ma.id, ma.material_id, ma.alternative_material_id, ma.tenant_id,
                   ma.priority, ma.usage_probability, ma.plant, ma.valid_from, ma.valid_to,
                   ma.created_at, ma.created_by, ma.updated_at, ma.updated_by
            FROM material_alternatives ma
            WHERE ma.material_id = $1 AND ma.tenant_id = $2
            ORDER BY ma.priority
            "#,
        )
        .bind(material_id.0)
        .bind(tenant_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("查询替代物料失败: {}", e)))?;

        let mut alternatives = Vec::with_capacity(rows.len());
        for row in rows {
            // 查询替代物料的编号和描述
            let alt_info: Option<(String, String)> = sqlx::query_as(
                "SELECT material_number, description FROM materials WHERE id = $1",
            )
            .bind(row.alternative_material_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::database(format!("查询替代物料信息失败: {}", e)))?;

            if let Some((number, desc)) = alt_info {
                alternatives.push(alternative_material_from_row(row, number, desc));
            }
        }

        Ok(alternatives)
    }

    async fn save_alternative(
        &self,
        material_id: &MaterialId,
        alternative: &AlternativeMaterial,
    ) -> AppResult<()> {
        // 获取物料的 tenant_id
        let tenant_id: Option<(uuid::Uuid,)> =
            sqlx::query_as("SELECT tenant_id FROM materials WHERE id = $1")
                .bind(material_id.0)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| AppError::database(format!("查询物料失败: {}", e)))?;

        let Some((tenant_id,)) = tenant_id else {
            return Err(AppError::not_found("物料不存在".to_string()));
        };

        sqlx::query(
            r#"
            INSERT INTO material_alternatives (
                id, material_id, alternative_material_id, tenant_id,
                priority, plant, valid_from, valid_to,
                created_at, created_by, updated_at, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NULL, NOW(), NULL)
            ON CONFLICT (material_id, alternative_material_id, tenant_id)
            DO UPDATE SET
                priority = EXCLUDED.priority,
                plant = EXCLUDED.plant,
                valid_from = EXCLUDED.valid_from,
                valid_to = EXCLUDED.valid_to,
                updated_at = NOW()
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(material_id.0)
        .bind(alternative.material_id().0)
        .bind(tenant_id)
        .bind(alternative.priority())
        .bind(alternative.plant())
        .bind(alternative.valid_from())
        .bind(alternative.valid_to())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("保存替代物料失败: {}", e)))?;

        Ok(())
    }

    async fn remove_alternative(
        &self,
        material_id: &MaterialId,
        alternative_id: &MaterialId,
    ) -> AppResult<()> {
        let result = sqlx::query(
            "DELETE FROM material_alternatives WHERE material_id = $1 AND alternative_material_id = $2",
        )
        .bind(material_id.0)
        .bind(alternative_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("删除替代物料失败: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("替代物料关系不存在".to_string()));
        }

        Ok(())
    }

    // ========== 视图数据 ==========

    async fn save_plant_data(
        &self,
        material_id: &MaterialId,
        plant_data: &PlantData,
    ) -> AppResult<()> {
        // 使用 UPSERT 语法
        sqlx::query(
            r#"
            INSERT INTO material_plant_data (
                id, material_id, tenant_id, plant, plant_name,
                mrp_type, mrp_controller, reorder_point, safety_stock,
                minimum_lot_size, maximum_lot_size, fixed_lot_size, rounding_value,
                planned_delivery_days, gr_processing_days,
                procurement_type, special_procurement, production_scheduler,
                storage_location, storage_bin, batch_management, serial_number_profile,
                abc_indicator, status, deletion_flag,
                created_at, created_by, updated_at, updated_by
            ) VALUES (
                gen_random_uuid(), $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14,
                $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, NOW(), NULL, NOW(), NULL
            )
            ON CONFLICT (material_id, plant, tenant_id)
            DO UPDATE SET
                plant_name = EXCLUDED.plant_name,
                mrp_type = EXCLUDED.mrp_type,
                mrp_controller = EXCLUDED.mrp_controller,
                reorder_point = EXCLUDED.reorder_point,
                safety_stock = EXCLUDED.safety_stock,
                updated_at = NOW()
            "#,
        )
        .bind(material_id.0)
        .bind(plant_data.plant()) // tenant_id 需要从 material 获取
        .bind(plant_data.plant())
        .bind(plant_data.plant_name())
        .bind(plant_data.mrp_type())
        .bind(plant_data.mrp_controller())
        .bind(plant_data.reorder_point())
        .bind(plant_data.safety_stock())
        .bind(plant_data.minimum_lot_size())
        .bind(plant_data.maximum_lot_size())
        .bind(plant_data.fixed_lot_size())
        .bind(plant_data.rounding_value())
        .bind(plant_data.planned_delivery_days())
        .bind(plant_data.gr_processing_days())
        .bind(plant_data.procurement_type() as i16)
        .bind(plant_data.special_procurement())
        .bind(plant_data.production_scheduler())
        .bind(plant_data.storage_location())
        .bind(plant_data.storage_bin())
        .bind(plant_data.batch_management())
        .bind(plant_data.serial_number_profile())
        .bind(plant_data.abc_indicator())
        .bind(0i16) // status placeholder
        .bind(false)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("保存工厂数据失败: {}", e)))?;

        Ok(())
    }

    async fn get_plant_data(
        &self,
        material_id: &MaterialId,
        plant: &str,
        tenant_id: &TenantId,
    ) -> AppResult<Option<PlantData>> {
        let row = sqlx::query_as::<_, MaterialPlantDataRow>(
            "SELECT * FROM material_plant_data WHERE material_id = $1 AND plant = $2 AND tenant_id = $3"
        )
        .bind(material_id.0)
        .bind(plant)
        .bind(tenant_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("查询工厂数据失败: {}", e)))?;

        Ok(row.map(plant_data_from_row))
    }

    async fn save_sales_data(
        &self,
        _material_id: &MaterialId,
        _sales_data: &SalesData,
    ) -> AppResult<()> {
        // TODO: 实现销售数据保存
        Ok(())
    }

    async fn get_sales_data(
        &self,
        _material_id: &MaterialId,
        _sales_org: &str,
        _tenant_id: &TenantId,
    ) -> AppResult<Option<SalesData>> {
        // TODO: 实现销售数据查询
        Ok(None)
    }

    async fn save_purchase_data(
        &self,
        _material_id: &MaterialId,
        _purchase_data: &PurchaseData,
    ) -> AppResult<()> {
        // TODO: 实现采购数据保存
        Ok(())
    }

    async fn get_purchase_data(
        &self,
        _material_id: &MaterialId,
        _purchase_org: &str,
        _tenant_id: &TenantId,
    ) -> AppResult<Option<PurchaseData>> {
        // TODO: 实现采购数据查询
        Ok(None)
    }

    async fn save_storage_data(
        &self,
        _material_id: &MaterialId,
        _storage_data: &StorageData,
    ) -> AppResult<()> {
        // TODO: 实现仓储数据保存
        Ok(())
    }

    async fn get_storage_data(
        &self,
        _material_id: &MaterialId,
        _tenant_id: &TenantId,
    ) -> AppResult<Option<StorageData>> {
        // TODO: 实现仓储数据查询
        Ok(None)
    }

    async fn save_accounting_data(
        &self,
        _material_id: &MaterialId,
        _accounting_data: &AccountingData,
    ) -> AppResult<()> {
        // TODO: 实现会计数据保存
        Ok(())
    }

    async fn get_accounting_data(
        &self,
        _material_id: &MaterialId,
        _tenant_id: &TenantId,
    ) -> AppResult<Option<AccountingData>> {
        // TODO: 实现会计数据查询
        Ok(None)
    }

    async fn save_quality_data(
        &self,
        _material_id: &MaterialId,
        _quality_data: &QualityData,
    ) -> AppResult<()> {
        // TODO: 实现质量数据保存
        Ok(())
    }

    async fn get_quality_data(
        &self,
        _material_id: &MaterialId,
        _tenant_id: &TenantId,
    ) -> AppResult<Option<QualityData>> {
        // TODO: 实现质量数据查询
        Ok(None)
    }

    async fn find_unit_conversions(
        &self,
        material_id: &MaterialId,
        tenant_id: &TenantId,
    ) -> AppResult<Vec<UnitConversion>> {
        let rows = sqlx::query_as::<_, MaterialUnitConversionRow>(
            "SELECT * FROM material_unit_conversions WHERE material_id = $1 AND tenant_id = $2"
        )
        .bind(material_id.0)
        .bind(tenant_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("查询单位换算失败: {}", e)))?;

        Ok(rows.into_iter().filter_map(unit_conversion_from_row).collect())
    }

    async fn save_unit_conversion(
        &self,
        _material_id: &MaterialId,
        _conversion: &UnitConversion,
    ) -> AppResult<()> {
        // TODO: 实现单位换算保存
        Ok(())
    }
}

// ============================================================================
// PostgresMaterialRepository 辅助方法
// ============================================================================

use crate::domain::value_objects::UnitConversion;
use crate::domain::views::{
    AccountingData, PlantData, PurchaseData, QualityData, SalesData, StorageData,
};
use rust_decimal::Decimal;

impl PostgresMaterialRepository {
    /// 加载物料及其所有视图数据
    async fn load_material_with_views(&self, row: MaterialRow) -> AppResult<Material> {
        let material_id = row.id;
        let tenant_id = row.tenant_id;

        // 并行加载所有视图数据
        let (plant_rows, sales_rows, purchase_rows, storage_rows, accounting_rows, quality_rows, conversion_rows) = tokio::try_join!(
            self.load_plant_data(material_id, tenant_id),
            self.load_sales_data(material_id, tenant_id),
            self.load_purchase_data(material_id, tenant_id),
            self.load_storage_data(material_id, tenant_id),
            self.load_accounting_data(material_id, tenant_id),
            self.load_quality_data(material_id, tenant_id),
            self.load_unit_conversions(material_id, tenant_id),
        )?;

        // 转换视图数据
        let plant_data: Vec<PlantData> = plant_rows.into_iter().map(plant_data_from_row).collect();
        let sales_data: Vec<SalesData> = sales_rows.into_iter().map(sales_data_from_row).collect();
        let purchase_data: Vec<PurchaseData> = purchase_rows.into_iter().map(purchase_data_from_row).collect();
        let storage_data: Vec<StorageData> = storage_rows.into_iter().map(storage_data_from_row).collect();
        let accounting_data: Vec<AccountingData> = accounting_rows.into_iter().map(accounting_data_from_row).collect();
        let quality_data: Vec<QualityData> = quality_rows.into_iter().map(quality_data_from_row).collect();
        let unit_conversions: Vec<UnitConversion> = conversion_rows.into_iter().filter_map(unit_conversion_from_row).collect();

        Ok(material_from_parts(
            row,
            plant_data,
            sales_data,
            purchase_data,
            storage_data,
            accounting_data,
            quality_data,
            unit_conversions,
        ))
    }

    async fn load_plant_data(&self, material_id: uuid::Uuid, tenant_id: uuid::Uuid) -> AppResult<Vec<MaterialPlantDataRow>> {
        sqlx::query_as::<_, MaterialPlantDataRow>(
            r#"
            SELECT id, material_id, tenant_id, plant, mrp_type, mrp_controller,
                   reorder_point, safety_stock, lot_size, minimum_lot_size, maximum_lot_size,
                   fixed_lot_size, rounding_value, planned_delivery_days, gr_processing_days,
                   procurement_type, special_procurement, production_storage_location,
                   batch_management, serial_number_profile, abc_indicator, status,
                   created_at, created_by, updated_at, updated_by
            FROM material_plant_data
            WHERE material_id = $1 AND tenant_id = $2
            "#,
        )
        .bind(material_id)
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("加载工厂视图失败: {}", e)))
    }

    async fn load_sales_data(&self, material_id: uuid::Uuid, tenant_id: uuid::Uuid) -> AppResult<Vec<MaterialSalesDataRow>> {
        sqlx::query_as::<_, MaterialSalesDataRow>(
            r#"
            SELECT id, material_id, tenant_id, sales_org, distribution_channel,
                   sales_unit, minimum_order_quantity, minimum_delivery_quantity,
                   delivery_unit, delivery_unit_quantity, pricing_reference_material,
                   item_category_group, account_assignment_group, tax_classification, status,
                   created_at, created_by, updated_at, updated_by
            FROM material_sales_data
            WHERE material_id = $1 AND tenant_id = $2
            "#,
        )
        .bind(material_id)
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("加载销售视图失败: {}", e)))
    }

    async fn load_purchase_data(&self, material_id: uuid::Uuid, tenant_id: uuid::Uuid) -> AppResult<Vec<MaterialPurchaseDataRow>> {
        sqlx::query_as::<_, MaterialPurchaseDataRow>(
            r#"
            SELECT id, material_id, tenant_id, purchase_org, plant, purchase_unit,
                   purchasing_group, order_unit, planned_delivery_days, gr_processing_days,
                   under_delivery_tolerance, over_delivery_tolerance, unlimited_over_delivery,
                   preferred_vendor_id, standard_price_amount, standard_price_currency,
                   price_unit, status, created_at, created_by, updated_at, updated_by
            FROM material_purchase_data
            WHERE material_id = $1 AND tenant_id = $2
            "#,
        )
        .bind(material_id)
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("加载采购视图失败: {}", e)))
    }

    async fn load_storage_data(&self, material_id: uuid::Uuid, tenant_id: uuid::Uuid) -> AppResult<Vec<MaterialStorageDataRow>> {
        sqlx::query_as::<_, MaterialStorageDataRow>(
            r#"
            SELECT id, material_id, tenant_id, plant, storage_location,
                   warehouse_number, storage_type, storage_bin, picking_area,
                   max_storage_quantity, min_storage_quantity, replenishment_quantity,
                   created_at, created_by, updated_at, updated_by
            FROM material_storage_data
            WHERE material_id = $1 AND tenant_id = $2
            "#,
        )
        .bind(material_id)
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("加载仓储视图失败: {}", e)))
    }

    async fn load_accounting_data(&self, material_id: uuid::Uuid, tenant_id: uuid::Uuid) -> AppResult<Vec<MaterialAccountingDataRow>> {
        sqlx::query_as::<_, MaterialAccountingDataRow>(
            r#"
            SELECT id, material_id, tenant_id, plant, valuation_area,
                   valuation_class, valuation_category, price_control,
                   standard_price_amount, standard_price_currency,
                   moving_average_price_amount, moving_average_price_currency,
                   price_unit, inventory_account, price_difference_account,
                   cost_element, costing_lot_size, with_qty_structure,
                   created_at, created_by, updated_at, updated_by
            FROM material_accounting_data
            WHERE material_id = $1 AND tenant_id = $2
            "#,
        )
        .bind(material_id)
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("加载会计视图失败: {}", e)))
    }

    async fn load_quality_data(&self, material_id: uuid::Uuid, tenant_id: uuid::Uuid) -> AppResult<Vec<MaterialQualityDataRow>> {
        sqlx::query_as::<_, MaterialQualityDataRow>(
            r#"
            SELECT id, material_id, tenant_id, plant, inspection_active,
                   inspection_type, inspection_interval, sample_percentage,
                   shelf_life_days, remaining_shelf_life_days,
                   certificate_type, certificate_required,
                   created_at, created_by, updated_at, updated_by
            FROM material_quality_data
            WHERE material_id = $1 AND tenant_id = $2
            "#,
        )
        .bind(material_id)
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("加载质量视图失败: {}", e)))
    }

    async fn load_unit_conversions(&self, material_id: uuid::Uuid, tenant_id: uuid::Uuid) -> AppResult<Vec<MaterialUnitConversionRow>> {
        sqlx::query_as::<_, MaterialUnitConversionRow>(
            r#"
            SELECT id, material_id, tenant_id, from_unit, to_unit,
                   numerator, denominator,
                   created_at, created_by, updated_at, updated_by
            FROM material_unit_conversions
            WHERE material_id = $1 AND tenant_id = $2
            "#,
        )
        .bind(material_id)
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database(format!("加载单位换算失败: {}", e)))
    }

    /// 插入物料主表
    async fn insert_material(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        material: &Material,
    ) -> AppResult<()> {
        let created_by = material.audit_info().created_by.as_ref().map(|u| u.0);
        let updated_by = material.audit_info().updated_by.as_ref().map(|u| u.0);
        let status = material.status() as i16;

        sqlx::query(
            r#"
            INSERT INTO materials (
                id, tenant_id, material_number, description, localized_description,
                material_type_id, material_type_code, material_group_id, material_group_code,
                base_unit, gross_weight, net_weight, weight_unit, volume, volume_unit,
                length, width, height, dimension_unit, status, custom_attributes,
                created_at, created_by, updated_at, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25)
            "#,
        )
        .bind(material.id().0)
        .bind(material.tenant_id().0)
        .bind(material.material_number().as_str())
        .bind(material.description())
        .bind(localized_text_to_json(material.localized_description()))
        .bind(material.material_type_id().0)
        .bind(material.material_type_code())
        .bind(material.material_group_id().map(|id| id.0))
        .bind(if material.material_group_code().is_empty() { None } else { Some(material.material_group_code()) })
        .bind(material.base_unit())
        .bind(Decimal::try_from(material.gross_weight()).ok())
        .bind(Decimal::try_from(material.net_weight()).ok())
        .bind(if material.weight_unit().is_empty() { None } else { Some(material.weight_unit()) })
        .bind(Decimal::try_from(material.volume()).ok())
        .bind(if material.volume_unit().is_empty() { None } else { Some(material.volume_unit()) })
        .bind(Decimal::try_from(material.length()).ok())
        .bind(Decimal::try_from(material.width()).ok())
        .bind(Decimal::try_from(material.height()).ok())
        .bind(if material.dimension_unit().is_empty() { None } else { Some(material.dimension_unit()) })
        .bind(status)
        .bind(serde_json::to_value(material.custom_attributes()).ok())
        .bind(material.audit_info().created_at)
        .bind(created_by)
        .bind(material.audit_info().updated_at)
        .bind(updated_by)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("保存物料失败: {}", e)))?;

        Ok(())
    }

    /// 更新物料主表
    async fn update_material(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        material: &Material,
    ) -> AppResult<()> {
        let updated_by = material.audit_info().updated_by.as_ref().map(|u| u.0);
        let status = material.status() as i16;

        let result = sqlx::query(
            r#"
            UPDATE materials SET
                description = $1,
                localized_description = $2,
                material_group_id = $3,
                material_group_code = $4,
                gross_weight = $5,
                net_weight = $6,
                weight_unit = $7,
                volume = $8,
                volume_unit = $9,
                length = $10,
                width = $11,
                height = $12,
                dimension_unit = $13,
                status = $14,
                custom_attributes = $15,
                updated_at = $16,
                updated_by = $17
            WHERE id = $18 AND tenant_id = $19
            "#,
        )
        .bind(material.description())
        .bind(localized_text_to_json(material.localized_description()))
        .bind(material.material_group_id().map(|id| id.0))
        .bind(if material.material_group_code().is_empty() { None } else { Some(material.material_group_code()) })
        .bind(Decimal::try_from(material.gross_weight()).ok())
        .bind(Decimal::try_from(material.net_weight()).ok())
        .bind(if material.weight_unit().is_empty() { None } else { Some(material.weight_unit()) })
        .bind(Decimal::try_from(material.volume()).ok())
        .bind(if material.volume_unit().is_empty() { None } else { Some(material.volume_unit()) })
        .bind(Decimal::try_from(material.length()).ok())
        .bind(Decimal::try_from(material.width()).ok())
        .bind(Decimal::try_from(material.height()).ok())
        .bind(if material.dimension_unit().is_empty() { None } else { Some(material.dimension_unit()) })
        .bind(status)
        .bind(serde_json::to_value(material.custom_attributes()).ok())
        .bind(material.audit_info().updated_at)
        .bind(updated_by)
        .bind(material.id().0)
        .bind(material.tenant_id().0)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("更新物料失败: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("物料不存在".to_string()));
        }

        Ok(())
    }

    /// 插入工厂视图
    async fn insert_plant_data(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        material: &Material,
        data: &PlantData,
    ) -> AppResult<()> {
        let status = data.status() as i16;
        let procurement_type = data.procurement_type() as i16;

        sqlx::query(
            r#"
            INSERT INTO material_plant_data (
                id, material_id, tenant_id, plant, mrp_type, mrp_controller,
                reorder_point, safety_stock, minimum_lot_size, maximum_lot_size,
                fixed_lot_size, rounding_value, planned_delivery_days, gr_processing_days,
                procurement_type, special_procurement, production_storage_location,
                batch_management, abc_indicator, status,
                created_at, created_by, updated_at, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, NOW(), NULL, NOW(), NULL)
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(material.id().0)
        .bind(material.tenant_id().0)
        .bind(data.plant())
        .bind(if data.mrp_type().is_empty() { None } else { Some(data.mrp_type()) })
        .bind(if data.mrp_controller().is_empty() { None } else { Some(data.mrp_controller()) })
        .bind(Decimal::try_from(data.reorder_point()).ok())
        .bind(Decimal::try_from(data.safety_stock()).ok())
        .bind(Decimal::try_from(data.minimum_lot_size()).ok())
        .bind(Decimal::try_from(data.maximum_lot_size()).ok())
        .bind(Decimal::try_from(data.fixed_lot_size()).ok())
        .bind(Decimal::try_from(data.rounding_value()).ok())
        .bind(data.planned_delivery_days())
        .bind(data.gr_processing_days())
        .bind(procurement_type)
        .bind(if data.special_procurement().is_empty() { None } else { Some(data.special_procurement()) })
        .bind(if data.storage_location().is_empty() { None } else { Some(data.storage_location()) })
        .bind(data.batch_management())
        .bind(if data.abc_indicator().is_empty() { None } else { Some(data.abc_indicator()) })
        .bind(status)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("保存工厂视图失败: {}", e)))?;

        Ok(())
    }

    /// 插入销售视图
    async fn insert_sales_data(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        material: &Material,
        data: &SalesData,
    ) -> AppResult<()> {
        let status = data.status() as i16;

        sqlx::query(
            r#"
            INSERT INTO material_sales_data (
                id, material_id, tenant_id, sales_org, distribution_channel,
                sales_unit, minimum_order_quantity, minimum_delivery_quantity,
                delivery_unit, pricing_reference_material,
                account_assignment_group, tax_classification, status,
                created_at, created_by, updated_at, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW(), NULL, NOW(), NULL)
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(material.id().0)
        .bind(material.tenant_id().0)
        .bind(data.sales_org())
        .bind(data.distribution_channel())
        .bind(if data.sales_unit().is_empty() { None } else { Some(data.sales_unit()) })
        .bind(Decimal::try_from(data.minimum_order_quantity()).ok())
        .bind(Decimal::try_from(data.minimum_delivery_quantity()).ok())
        .bind(if data.delivery_unit().is_empty() { None } else { Some(data.delivery_unit()) })
        .bind(if data.pricing_reference_material().is_empty() { None } else { Some(data.pricing_reference_material()) })
        .bind(if data.account_assignment_group().is_empty() { None } else { Some(data.account_assignment_group()) })
        .bind(if data.tax_classification().is_empty() { None } else { Some(data.tax_classification()) })
        .bind(status)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("保存销售视图失败: {}", e)))?;

        Ok(())
    }

    /// 插入采购视图
    async fn insert_purchase_data(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        material: &Material,
        data: &PurchaseData,
    ) -> AppResult<()> {
        let status = data.status() as i16;

        sqlx::query(
            r#"
            INSERT INTO material_purchase_data (
                id, material_id, tenant_id, purchase_org, plant, purchase_unit,
                purchasing_group, planned_delivery_days,
                under_delivery_tolerance, over_delivery_tolerance, unlimited_over_delivery,
                preferred_vendor_id, status,
                created_at, created_by, updated_at, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NOW(), NULL, NOW(), NULL)
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(material.id().0)
        .bind(material.tenant_id().0)
        .bind(data.purchase_org())
        .bind(if data.plant().is_empty() { None } else { Some(data.plant()) })
        .bind(if data.purchase_unit().is_empty() { None } else { Some(data.purchase_unit()) })
        .bind(if data.purchasing_group().is_empty() { None } else { Some(data.purchasing_group()) })
        .bind(data.planned_delivery_days())
        .bind(Decimal::try_from(data.under_delivery_tolerance()).ok())
        .bind(Decimal::try_from(data.over_delivery_tolerance()).ok())
        .bind(data.unlimited_over_delivery())
        .bind(if data.preferred_vendor_id().is_empty() { None } else { Some(data.preferred_vendor_id()) })
        .bind(status)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("保存采购视图失败: {}", e)))?;

        Ok(())
    }

    /// 插入仓储视图
    async fn insert_storage_data(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        material: &Material,
        data: &StorageData,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO material_storage_data (
                id, material_id, tenant_id, plant, storage_location,
                warehouse_number, storage_type, storage_bin, picking_area,
                max_storage_quantity, min_storage_quantity,
                created_at, created_by, updated_at, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NULL, NOW(), NULL)
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(material.id().0)
        .bind(material.tenant_id().0)
        .bind(data.plant())
        .bind(data.storage_location())
        .bind(if data.warehouse_number().is_empty() { None } else { Some(data.warehouse_number()) })
        .bind(if data.storage_type().is_empty() { None } else { Some(data.storage_type()) })
        .bind(if data.storage_bin().is_empty() { None } else { Some(data.storage_bin()) })
        .bind(if data.picking_area().is_empty() { None } else { Some(data.picking_area()) })
        .bind(Decimal::try_from(data.max_storage_quantity()).ok())
        .bind(Decimal::try_from(data.min_storage_quantity()).ok())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("保存仓储视图失败: {}", e)))?;

        Ok(())
    }

    /// 插入会计视图
    async fn insert_accounting_data(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        material: &Material,
        data: &AccountingData,
    ) -> AppResult<()> {
        let price_control = data.price_control() as i16;

        sqlx::query(
            r#"
            INSERT INTO material_accounting_data (
                id, material_id, tenant_id, plant, valuation_area,
                valuation_class, price_control,
                inventory_account, price_difference_account,
                cost_element, with_qty_structure,
                created_at, created_by, updated_at, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NULL, NOW(), NULL)
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(material.id().0)
        .bind(material.tenant_id().0)
        .bind(data.plant())
        .bind(data.valuation_area())
        .bind(if data.valuation_class().is_empty() { None } else { Some(data.valuation_class()) })
        .bind(price_control)
        .bind(if data.inventory_account().is_empty() { None } else { Some(data.inventory_account()) })
        .bind(if data.price_difference_account().is_empty() { None } else { Some(data.price_difference_account()) })
        .bind(if data.cost_element().is_empty() { None } else { Some(data.cost_element()) })
        .bind(data.has_qty_structure())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("保存会计视图失败: {}", e)))?;

        Ok(())
    }

    /// 插入质量视图
    async fn insert_quality_data(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        material: &Material,
        data: &QualityData,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO material_quality_data (
                id, material_id, tenant_id, plant, inspection_active,
                inspection_type, sample_percentage,
                shelf_life_days, remaining_shelf_life_days, certificate_required,
                created_at, created_by, updated_at, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NULL, NOW(), NULL)
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(material.id().0)
        .bind(material.tenant_id().0)
        .bind(data.plant())
        .bind(data.inspection_active())
        .bind(if data.inspection_type().is_empty() { None } else { Some(data.inspection_type()) })
        .bind(Decimal::try_from(data.sample_percentage()).ok())
        .bind(data.shelf_life_days())
        .bind(data.remaining_shelf_life())
        .bind(data.certificate_required())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("保存质量视图失败: {}", e)))?;

        Ok(())
    }

    /// 插入单位换算
    async fn insert_unit_conversion(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        material: &Material,
        conversion: &UnitConversion,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO material_unit_conversions (
                id, material_id, tenant_id, from_unit, to_unit,
                numerator, denominator,
                created_at, created_by, updated_at, updated_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NULL, NOW(), NULL)
            "#,
        )
        .bind(uuid::Uuid::new_v4())
        .bind(material.id().0)
        .bind(material.tenant_id().0)
        .bind(conversion.source_unit())
        .bind(conversion.target_unit())
        .bind(Decimal::try_from(conversion.numerator()).ok().unwrap_or(Decimal::ONE))
        .bind(Decimal::try_from(conversion.denominator()).ok().unwrap_or(Decimal::ONE))
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::database(format!("保存单位换算失败: {}", e)))?;

        Ok(())
    }
}
