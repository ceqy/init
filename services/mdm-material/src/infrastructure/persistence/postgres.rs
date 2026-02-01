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

use super::converters::{localized_text_to_json, material_group_from_row, material_type_from_row};
use super::rows::{MaterialGroupRow, MaterialTypeRow};

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
        _id: &MaterialId,
        _tenant_id: &TenantId,
    ) -> AppResult<Option<Material>> {
        // TODO: 实现完整的物料查询，包括所有视图数据
        // 由于 Material 实体较复杂，需要查询多个表并组装
        Ok(None)
    }

    async fn find_by_number(
        &self,
        _number: &MaterialNumber,
        _tenant_id: &TenantId,
    ) -> AppResult<Option<Material>> {
        // TODO: 实现
        Ok(None)
    }

    async fn save(&self, _material: &Material) -> AppResult<()> {
        // TODO: 实现完整的物料保存，包括所有视图数据
        // 需要在事务中保存主表和所有视图表
        Ok(())
    }

    async fn update(&self, _material: &Material) -> AppResult<()> {
        // TODO: 实现
        Ok(())
    }

    async fn delete(&self, _id: &MaterialId, _tenant_id: &TenantId) -> AppResult<()> {
        // TODO: 实现
        Ok(())
    }

    async fn list(
        &self,
        _tenant_id: &TenantId,
        _filter: MaterialFilter,
        pagination: Pagination,
    ) -> AppResult<PagedResult<Material>> {
        // TODO: 实现
        Ok(PagedResult::new(vec![], 0, &pagination))
    }

    async fn search(
        &self,
        _tenant_id: &TenantId,
        _query: &str,
        _pagination: Pagination,
    ) -> AppResult<Vec<MaterialSearchResult>> {
        // TODO: 实现全文搜索
        Ok(vec![])
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
        _material_id: &MaterialId,
        _tenant_id: &TenantId,
    ) -> AppResult<Vec<AlternativeMaterial>> {
        // TODO: 实现替代物料查询
        Ok(vec![])
    }

    async fn save_alternative(
        &self,
        _material_id: &MaterialId,
        _alternative: &AlternativeMaterial,
    ) -> AppResult<()> {
        // TODO: 实现
        Ok(())
    }

    async fn remove_alternative(
        &self,
        _material_id: &MaterialId,
        _alternative_id: &MaterialId,
    ) -> AppResult<()> {
        // TODO: 实现
        Ok(())
    }
}
