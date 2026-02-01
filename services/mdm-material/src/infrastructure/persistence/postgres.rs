//! PostgreSQL repository implementation

use async_trait::async_trait;
use common::types::{PagedResult, Pagination, TenantId};
use sqlx::PgPool;

use crate::domain::entities::{Material, MaterialFilter, MaterialGroup, MaterialSearchResult, MaterialType};
use crate::domain::repositories::{MaterialGroupRepository, MaterialRepository, MaterialTypeRepository};
use crate::domain::value_objects::{AlternativeMaterial, MaterialGroupId, MaterialId, MaterialNumber, MaterialTypeId};
use crate::error::ServiceResult;

pub struct PostgresMaterialRepository {
    #[allow(dead_code)]
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
    ) -> ServiceResult<Option<Material>> {
        // TODO: Implement
        Ok(None)
    }

    async fn find_by_number(
        &self,
        _number: &MaterialNumber,
        _tenant_id: &TenantId,
    ) -> ServiceResult<Option<Material>> {
        // TODO: Implement
        Ok(None)
    }

    async fn save(&self, _material: &Material) -> ServiceResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn update(&self, _material: &Material) -> ServiceResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn delete(&self, _id: &MaterialId, _tenant_id: &TenantId) -> ServiceResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn list(
        &self,
        _tenant_id: &TenantId,
        _filter: MaterialFilter,
        pagination: Pagination,
    ) -> ServiceResult<PagedResult<Material>> {
        // TODO: Implement
        Ok(PagedResult::new(vec![], 0, &pagination))
    }

    async fn search(
        &self,
        _tenant_id: &TenantId,
        _query: &str,
        _pagination: Pagination,
    ) -> ServiceResult<Vec<MaterialSearchResult>> {
        // TODO: Implement
        Ok(vec![])
    }

    async fn exists_by_number(
        &self,
        _number: &MaterialNumber,
        _tenant_id: &TenantId,
    ) -> ServiceResult<bool> {
        // TODO: Implement
        Ok(false)
    }

    async fn find_alternatives(
        &self,
        _material_id: &MaterialId,
        _tenant_id: &TenantId,
    ) -> ServiceResult<Vec<AlternativeMaterial>> {
        // TODO: Implement
        Ok(vec![])
    }

    async fn save_alternative(
        &self,
        _material_id: &MaterialId,
        _alternative: &AlternativeMaterial,
    ) -> ServiceResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn remove_alternative(
        &self,
        _material_id: &MaterialId,
        _alternative_id: &MaterialId,
    ) -> ServiceResult<()> {
        // TODO: Implement
        Ok(())
    }
}

pub struct PostgresMaterialGroupRepository {
    #[allow(dead_code)]
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
        _id: &MaterialGroupId,
        _tenant_id: &TenantId,
    ) -> ServiceResult<Option<MaterialGroup>> {
        // TODO: Implement
        Ok(None)
    }

    async fn find_by_code(
        &self,
        _code: &str,
        _tenant_id: &TenantId,
    ) -> ServiceResult<Option<MaterialGroup>> {
        // TODO: Implement
        Ok(None)
    }

    async fn save(&self, _group: &MaterialGroup) -> ServiceResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn update(&self, _group: &MaterialGroup) -> ServiceResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn delete(&self, _id: &MaterialGroupId, _tenant_id: &TenantId) -> ServiceResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn list(
        &self,
        _tenant_id: &TenantId,
        _parent_id: Option<&MaterialGroupId>,
        pagination: Pagination,
    ) -> ServiceResult<PagedResult<MaterialGroup>> {
        // TODO: Implement
        Ok(PagedResult::new(vec![], 0, &pagination))
    }

    async fn find_children(
        &self,
        _parent_id: &MaterialGroupId,
        _tenant_id: &TenantId,
    ) -> ServiceResult<Vec<MaterialGroup>> {
        // TODO: Implement
        Ok(vec![])
    }

    async fn exists_by_code(&self, _code: &str, _tenant_id: &TenantId) -> ServiceResult<bool> {
        // TODO: Implement
        Ok(false)
    }
}

pub struct PostgresMaterialTypeRepository {
    #[allow(dead_code)]
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
        _id: &MaterialTypeId,
        _tenant_id: &TenantId,
    ) -> ServiceResult<Option<MaterialType>> {
        // TODO: Implement
        Ok(None)
    }

    async fn find_by_code(
        &self,
        _code: &str,
        _tenant_id: &TenantId,
    ) -> ServiceResult<Option<MaterialType>> {
        // TODO: Implement
        Ok(None)
    }

    async fn save(&self, _material_type: &MaterialType) -> ServiceResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn update(&self, _material_type: &MaterialType) -> ServiceResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn list(
        &self,
        _tenant_id: &TenantId,
        pagination: Pagination,
    ) -> ServiceResult<PagedResult<MaterialType>> {
        // TODO: Implement
        Ok(PagedResult::new(vec![], 0, &pagination))
    }

    async fn exists_by_code(&self, _code: &str, _tenant_id: &TenantId) -> ServiceResult<bool> {
        // TODO: Implement
        Ok(false)
    }
}
