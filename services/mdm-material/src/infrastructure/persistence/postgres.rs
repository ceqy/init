//! PostgreSQL repository implementation

use async_trait::async_trait;
use common::types::{PagedResult, Pagination, TenantId};
use errors::AppResult;
use sqlx::PgPool;

use crate::domain::entities::{Material, MaterialFilter, MaterialGroup, MaterialSearchResult, MaterialType};
use crate::domain::repositories::{MaterialGroupRepository, MaterialRepository, MaterialTypeRepository};
use crate::domain::value_objects::{AlternativeMaterial, MaterialGroupId, MaterialId, MaterialNumber, MaterialTypeId};

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
    ) -> AppResult<Option<Material>> {
        // TODO: Implement
        Ok(None)
    }

    async fn find_by_number(
        &self,
        _number: &MaterialNumber,
        _tenant_id: &TenantId,
    ) -> AppResult<Option<Material>> {
        // TODO: Implement
        Ok(None)
    }

    async fn save(&self, _material: &Material) -> AppResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn update(&self, _material: &Material) -> AppResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn delete(&self, _id: &MaterialId, _tenant_id: &TenantId) -> AppResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn list(
        &self,
        _tenant_id: &TenantId,
        _filter: MaterialFilter,
        pagination: Pagination,
    ) -> AppResult<PagedResult<Material>> {
        // TODO: Implement
        Ok(PagedResult::new(vec![], 0, &pagination))
    }

    async fn search(
        &self,
        _tenant_id: &TenantId,
        _query: &str,
        _pagination: Pagination,
    ) -> AppResult<Vec<MaterialSearchResult>> {
        // TODO: Implement
        Ok(vec![])
    }

    async fn exists_by_number(
        &self,
        _number: &MaterialNumber,
        _tenant_id: &TenantId,
    ) -> AppResult<bool> {
        // TODO: Implement
        Ok(false)
    }

    async fn find_alternatives(
        &self,
        _material_id: &MaterialId,
        _tenant_id: &TenantId,
    ) -> AppResult<Vec<AlternativeMaterial>> {
        // TODO: Implement
        Ok(vec![])
    }

    async fn save_alternative(
        &self,
        _material_id: &MaterialId,
        _alternative: &AlternativeMaterial,
    ) -> AppResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn remove_alternative(
        &self,
        _material_id: &MaterialId,
        _alternative_id: &MaterialId,
    ) -> AppResult<()> {
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
    ) -> AppResult<Option<MaterialGroup>> {
        // TODO: Implement
        Ok(None)
    }

    async fn find_by_code(
        &self,
        _code: &str,
        _tenant_id: &TenantId,
    ) -> AppResult<Option<MaterialGroup>> {
        // TODO: Implement
        Ok(None)
    }

    async fn save(&self, _group: &MaterialGroup) -> AppResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn update(&self, _group: &MaterialGroup) -> AppResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn delete(&self, _id: &MaterialGroupId, _tenant_id: &TenantId) -> AppResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn list(
        &self,
        _tenant_id: &TenantId,
        _parent_id: Option<&MaterialGroupId>,
        pagination: Pagination,
    ) -> AppResult<PagedResult<MaterialGroup>> {
        // TODO: Implement
        Ok(PagedResult::new(vec![], 0, &pagination))
    }

    async fn find_children(
        &self,
        _parent_id: &MaterialGroupId,
        _tenant_id: &TenantId,
    ) -> AppResult<Vec<MaterialGroup>> {
        // TODO: Implement
        Ok(vec![])
    }

    async fn exists_by_code(&self, _code: &str, _tenant_id: &TenantId) -> AppResult<bool> {
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
    ) -> AppResult<Option<MaterialType>> {
        // TODO: Implement
        Ok(None)
    }

    async fn find_by_code(
        &self,
        _code: &str,
        _tenant_id: &TenantId,
    ) -> AppResult<Option<MaterialType>> {
        // TODO: Implement
        Ok(None)
    }

    async fn save(&self, _material_type: &MaterialType) -> AppResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn update(&self, _material_type: &MaterialType) -> AppResult<()> {
        // TODO: Implement
        Ok(())
    }

    async fn list(
        &self,
        _tenant_id: &TenantId,
        pagination: Pagination,
    ) -> AppResult<PagedResult<MaterialType>> {
        // TODO: Implement
        Ok(PagedResult::new(vec![], 0, &pagination))
    }

    async fn exists_by_code(&self, _code: &str, _tenant_id: &TenantId) -> AppResult<bool> {
        // TODO: Implement
        Ok(false)
    }
}
