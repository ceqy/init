//! gRPC service implementation

use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::application::commands::*;
use crate::application::queries::*;
use crate::application::ServiceHandler;

use super::conversions::*;
use super::proto_converters::*;

// 使用 main.rs 中定义的 proto 模块
use crate::common;
use crate::mdm_material;

use mdm_material::v1::material_service_server::MaterialService;
use mdm_material::v1::*;

pub struct MaterialServiceImpl {
    handler: Arc<ServiceHandler>,
}

impl MaterialServiceImpl {
    pub fn new(handler: Arc<ServiceHandler>) -> Self {
        Self { handler }
    }
}

#[tonic::async_trait]
impl MaterialService for MaterialServiceImpl {
    // ========== 物料基本操作 ==========

    async fn create_material(
        &self,
        request: Request<CreateMaterialRequest>,
    ) -> Result<Response<CreateMaterialResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        // 转换为 Command
        let cmd = CreateMaterialCommand {
            tenant_id: tenant_id.clone(),
            user_id,
            material_number: req.material_number,
            description: req.description,
            localized_description: req.localized_description.map(|l| l.into()),
            material_type_id: parse_material_type_id(&req.material_type_id)
                .map_err(|e| Status::invalid_argument(e.to_string()))?,
            material_group_id: if req.material_group_id.is_empty() {
                None
            } else {
                Some(parse_material_group_id(&req.material_group_id)
                    .map_err(|e| Status::invalid_argument(e.to_string()))?)
            },
            base_unit: req.base_unit,
            gross_weight: if req.gross_weight > 0.0 { Some(req.gross_weight) } else { None },
            net_weight: if req.net_weight > 0.0 { Some(req.net_weight) } else { None },
            weight_unit: if req.weight_unit.is_empty() { None } else { Some(req.weight_unit) },
            volume: if req.volume > 0.0 { Some(req.volume) } else { None },
            volume_unit: if req.volume_unit.is_empty() { None } else { Some(req.volume_unit) },
            length: None,
            width: None,
            height: None,
            dimension_unit: None,
            custom_attributes: None,
        };

        // 调用业务逻辑
        let material_id = self
            .handler
            .create_material(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 获取创建的物料
        let query = GetMaterialQuery {
            material_id: material_id.clone(),
            tenant_id,
        };
        let material = self.handler.get_material(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateMaterialResponse {
            material: Some(material_to_proto(&material)),
        }))
    }

    async fn get_material(
        &self,
        request: Request<GetMaterialRequest>,
    ) -> Result<Response<GetMaterialResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        // GetMaterialRequest uses oneof identifier
        let material_id = if let Some(identifier) = req.identifier {
            match identifier {
                get_material_request::Identifier::Id(id) => {
                    parse_material_id(&id)
                        .map_err(|e| Status::invalid_argument(e.to_string()))?
                },
                get_material_request::Identifier::MaterialNumber(_) => {
                    // TODO: 实现通过物料编号查询
                    return Err(Status::unimplemented("Query by material_number not yet implemented"));
                }
            }
        } else {
            return Err(Status::invalid_argument("Missing identifier"));
        };

        let query = GetMaterialQuery {
            material_id,
            tenant_id,
        };

        let material = self
            .handler
            .get_material(query)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(GetMaterialResponse {
            material: Some(material_to_proto(&material)),
        }))
    }

    async fn update_material(
        &self,
        request: Request<UpdateMaterialRequest>,
    ) -> Result<Response<UpdateMaterialResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let material_id = parse_material_id(&req.id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let cmd = UpdateMaterialCommand {
            material_id: material_id.clone(),
            tenant_id: tenant_id.clone(),
            user_id,
            description: if req.description.is_empty() { None } else { Some(req.description) },
            localized_description: req.localized_description.map(|l| l.into()),
            material_group_id: if req.material_group_id.is_empty() {
                None
            } else {
                Some(parse_material_group_id(&req.material_group_id)
                    .map_err(|e| Status::invalid_argument(e.to_string()))?)
            },
            base_unit: None,
            gross_weight: if req.gross_weight > 0.0 { Some(req.gross_weight) } else { None },
            net_weight: if req.net_weight > 0.0 { Some(req.net_weight) } else { None },
            weight_unit: if req.weight_unit.is_empty() { None } else { Some(req.weight_unit) },
            volume: if req.volume > 0.0 { Some(req.volume) } else { None },
            volume_unit: if req.volume_unit.is_empty() { None } else { Some(req.volume_unit) },
            length: if req.length > 0.0 { Some(req.length) } else { None },
            width: if req.width > 0.0 { Some(req.width) } else { None },
            height: if req.height > 0.0 { Some(req.height) } else { None },
            dimension_unit: if req.dimension_unit.is_empty() { None } else { Some(req.dimension_unit) },
            custom_attributes: if req.custom_attributes.is_empty() { None } else { Some(serde_json::to_value(req.custom_attributes).unwrap()) },
        };

        self.handler
            .update_material(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 获取更新后的物料
        let query = GetMaterialQuery {
            material_id,
            tenant_id,
        };
        let material = self.handler.get_material(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateMaterialResponse {
            material: Some(material_to_proto(&material)),
        }))
    }

    async fn delete_material(
        &self,
        request: Request<DeleteMaterialRequest>,
    ) -> Result<Response<()>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let material_id = parse_material_id(&req.id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let cmd = DeleteMaterialCommand {
            material_id,
            tenant_id,
            user_id,
        };

        self.handler
            .delete_material(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn list_materials(
        &self,
        request: Request<ListMaterialsRequest>,
    ) -> Result<Response<ListMaterialsResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        // ListMaterialsRequest uses pagination field, not page/page_size directly
        let pagination = if let Some(p) = req.pagination {
            parse_pagination(p.page, p.page_size)
        } else {
            parse_pagination(1, 20)
        };

        let query = ListMaterialsQuery {
            tenant_id,
            filter: Default::default(), // TODO: 实现过滤器转换
            pagination,
        };

        let result = self
            .handler
            .list_materials(query)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListMaterialsResponse {
            materials: result.items.iter().map(material_to_proto).collect(),
            pagination: Some(common::v1::PaginationResponse {
                total: result.total as i32,
                page: result.page as i32,
                page_size: result.page_size as i32,
                total_pages: ((result.total as u32 + result.page_size - 1) / result.page_size) as i32,
            }),
        }))
    }

    // ========== 物料状态管理 ==========

    async fn activate_material(
        &self,
        request: Request<ActivateMaterialRequest>,
    ) -> Result<Response<()>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let material_id = parse_material_id(&req.id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let cmd = ActivateMaterialCommand {
            material_id,
            tenant_id,
            user_id,
        };

        self.handler
            .activate_material(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn deactivate_material(
        &self,
        request: Request<DeactivateMaterialRequest>,
    ) -> Result<Response<()>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let material_id = parse_material_id(&req.id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let cmd = DeactivateMaterialCommand {
            material_id,
            tenant_id,
            user_id,
        };

        self.handler
            .deactivate_material(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn block_material(
        &self,
        request: Request<BlockMaterialRequest>,
    ) -> Result<Response<()>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let material_id = parse_material_id(&req.id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let cmd = BlockMaterialCommand {
            material_id,
            tenant_id,
            user_id,
        };

        self.handler
            .block_material(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn mark_for_deletion(
        &self,
        request: Request<MarkForDeletionRequest>,
    ) -> Result<Response<()>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let material_id = parse_material_id(&req.id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let cmd = MarkForDeletionCommand {
            material_id,
            tenant_id,
            user_id,
        };

        self.handler
            .mark_for_deletion(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }

    // ========== 物料组操作 ==========

    async fn create_material_group(
        &self,
        request: Request<CreateMaterialGroupRequest>,
    ) -> Result<Response<CreateMaterialGroupResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        let cmd = CreateMaterialGroupCommand {
            tenant_id: tenant_id.clone(),
            user_id,
            code: req.code,
            name: req.name,
            localized_name: req.localized_name.map(|l| l.into()),
            parent_id: if req.parent_id.is_empty() {
                None
            } else {
                Some(parse_material_group_id(&req.parent_id)
                    .map_err(|e| Status::invalid_argument(e.to_string()))?)
            },
        };

        let group_id = self
            .handler
            .create_material_group(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 获取创建的物料组
        let query = GetMaterialGroupQuery {
            group_id: group_id.clone(),
            tenant_id,
        };
        let group = self.handler.get_material_group(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateMaterialGroupResponse {
            material_group: Some(material_group_to_proto(&group)),
        }))
    }

    async fn get_material_group(
        &self,
        request: Request<GetMaterialGroupRequest>,
    ) -> Result<Response<GetMaterialGroupResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        // GetMaterialGroupRequest uses oneof identifier
        let group_id = if let Some(identifier) = req.identifier {
            match identifier {
                get_material_group_request::Identifier::Id(id) => {
                    parse_material_group_id(&id)
                        .map_err(|e| Status::invalid_argument(e.to_string()))?
                },
                get_material_group_request::Identifier::Code(_) => {
                    // TODO: 实现通过编码查询
                    return Err(Status::unimplemented("Query by code not yet implemented"));
                }
            }
        } else {
            return Err(Status::invalid_argument("Missing identifier"));
        };

        let query = GetMaterialGroupQuery {
            group_id,
            tenant_id,
        };

        let group = self
            .handler
            .get_material_group(query)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(GetMaterialGroupResponse {
            material_group: Some(material_group_to_proto(&group)),
            children: vec![], // TODO: 实现子节点查询
        }))
    }

    async fn update_material_group(
        &self,
        request: Request<UpdateMaterialGroupRequest>,
    ) -> Result<Response<UpdateMaterialGroupResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let group_id = parse_material_group_id(&req.id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let cmd = UpdateMaterialGroupCommand {
            group_id: group_id.clone(),
            tenant_id: tenant_id.clone(),
            user_id,
            name: if req.name.is_empty() { None } else { Some(req.name) },
            localized_name: req.localized_name.map(|l| l.into()),
        };

        self.handler
            .update_material_group(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 获取更新后的物料组
        let query = GetMaterialGroupQuery {
            group_id,
            tenant_id,
        };
        let group = self.handler.get_material_group(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateMaterialGroupResponse {
            material_group: Some(material_group_to_proto(&group)),
        }))
    }

    async fn delete_material_group(
        &self,
        request: Request<DeleteMaterialGroupRequest>,
    ) -> Result<Response<()>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let group_id = parse_material_group_id(&req.id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let cmd = DeleteMaterialGroupCommand {
            group_id,
            tenant_id,
            user_id,
        };

        self.handler
            .delete_material_group(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn list_material_groups(
        &self,
        request: Request<ListMaterialGroupsRequest>,
    ) -> Result<Response<ListMaterialGroupsResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        // ListMaterialGroupsRequest uses pagination field
        let pagination = if let Some(p) = req.pagination {
            parse_pagination(p.page, p.page_size)
        } else {
            parse_pagination(1, 20)
        };

        let query = ListMaterialGroupsQuery {
            tenant_id,
            parent_id: if req.parent_id.is_empty() {
                None
            } else {
                Some(parse_material_group_id(&req.parent_id)
                    .map_err(|e| Status::invalid_argument(e.to_string()))?)
            },
            pagination,
        };

        let result = self
            .handler
            .list_material_groups(query)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListMaterialGroupsResponse {
            material_groups: result.items.iter().map(material_group_to_proto).collect(),
            pagination: Some(common::v1::PaginationResponse {
                total: result.total as i32,
                page: result.page as i32,
                page_size: result.page_size as i32,
                total_pages: ((result.total as u32 + result.page_size - 1) / result.page_size) as i32,
            }),
        }))
    }

    // ========== 物料类型操作 ==========

    async fn create_material_type(
        &self,
        request: Request<CreateMaterialTypeRequest>,
    ) -> Result<Response<CreateMaterialTypeResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        let cmd = CreateMaterialTypeCommand {
            tenant_id: tenant_id.clone(),
            user_id,
            code: req.code,
            name: req.name,
            localized_name: req.localized_name.map(|l| l.into()),
            quantity_update: req.quantity_update,
            value_update: req.value_update,
            internal_procurement: req.internal_procurement,
            external_procurement: req.external_procurement,
        };

        let type_id = self
            .handler
            .create_material_type(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 获取创建的物料类型
        let query = GetMaterialTypeQuery {
            type_id: type_id.clone(),
            tenant_id,
        };
        let material_type = self.handler.get_material_type(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateMaterialTypeResponse {
            material_type: Some(material_type_to_proto(&material_type)),
        }))
    }

    async fn get_material_type(
        &self,
        request: Request<GetMaterialTypeRequest>,
    ) -> Result<Response<GetMaterialTypeResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        // GetMaterialTypeRequest uses oneof identifier
        let type_id = if let Some(identifier) = req.identifier {
            match identifier {
                get_material_type_request::Identifier::Id(id) => {
                    parse_material_type_id(&id)
                        .map_err(|e| Status::invalid_argument(e.to_string()))?
                },
                get_material_type_request::Identifier::Code(_) => {
                    // TODO: 实现通过编码查询
                    return Err(Status::unimplemented("Query by code not yet implemented"));
                }
            }
        } else {
            return Err(Status::invalid_argument("Missing identifier"));
        };

        let query = GetMaterialTypeQuery {
            type_id,
            tenant_id,
        };

        let material_type = self
            .handler
            .get_material_type(query)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        Ok(Response::new(GetMaterialTypeResponse {
            material_type: Some(material_type_to_proto(&material_type)),
        }))
    }

    async fn update_material_type(
        &self,
        request: Request<UpdateMaterialTypeRequest>,
    ) -> Result<Response<UpdateMaterialTypeResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let type_id = parse_material_type_id(&req.id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let cmd = UpdateMaterialTypeCommand {
            type_id: type_id.clone(),
            tenant_id: tenant_id.clone(),
            user_id,
            name: if req.name.is_empty() { None } else { Some(req.name) },
            localized_name: req.localized_name.map(|l| l.into()),
            quantity_update: None,
            value_update: None,
            internal_procurement: None,
            external_procurement: None,
        };

        self.handler
            .update_material_type(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 获取更新后的物料类型
        let query = GetMaterialTypeQuery {
            type_id,
            tenant_id,
        };
        let material_type = self.handler.get_material_type(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateMaterialTypeResponse {
            material_type: Some(material_type_to_proto(&material_type)),
        }))
    }

    async fn list_material_types(
        &self,
        request: Request<ListMaterialTypesRequest>,
    ) -> Result<Response<ListMaterialTypesResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        // ListMaterialTypesRequest uses pagination field
        let pagination = if let Some(p) = req.pagination {
            parse_pagination(p.page, p.page_size)
        } else {
            parse_pagination(1, 20)
        };

        let query = ListMaterialTypesQuery {
            tenant_id,
            pagination,
        };

        let result = self
            .handler
            .list_material_types(query)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListMaterialTypesResponse {
            material_types: result.items.iter().map(material_type_to_proto).collect(),
            pagination: Some(common::v1::PaginationResponse {
                total: result.total as i32,
                page: result.page as i32,
                page_size: result.page_size as i32,
                total_pages: ((result.total as u32 + result.page_size - 1) / result.page_size) as i32,
            }),
        }))
    }

    // ========== 物料视图扩展 ==========

    async fn extend_material_to_plant(
        &self,
        request: Request<ExtendMaterialToPlantRequest>,
    ) -> Result<Response<ExtendMaterialToPlantResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let material_id = parse_material_id(&req.material_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let proto_plant_data = req.plant_data
            .ok_or_else(|| Status::invalid_argument("Missing plant_data"))?;

        // 将 proto PlantData 转换为 domain PlantData
        let plant_data = proto_to_plant_data(proto_plant_data);

        // 创建命令
        let cmd = ExtendToPlantCommand {
            material_id: material_id.clone(),
            tenant_id: tenant_id.clone(),
            user_id,
            plant_data,
        };

        // 执行命令
        self.handler.extend_to_plant(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 获取更新后的物料
        let query = GetMaterialQuery {
            material_id,
            tenant_id,
        };
        let material = self.handler.get_material(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ExtendMaterialToPlantResponse {
            material: Some(material_to_proto(&material)),
        }))
    }

    async fn extend_material_to_sales_org(
        &self,
        request: Request<ExtendMaterialToSalesOrgRequest>,
    ) -> Result<Response<ExtendMaterialToSalesOrgResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let material_id = parse_material_id(&req.material_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let proto_sales_data = req.sales_data
            .ok_or_else(|| Status::invalid_argument("Missing sales_data"))?;

        // 将 proto SalesData 转换为 domain SalesData
        let sales_data = proto_to_sales_data(proto_sales_data);

        // 创建命令
        let cmd = ExtendToSalesCommand {
            material_id: material_id.clone(),
            tenant_id: tenant_id.clone(),
            user_id,
            sales_data,
        };

        // 执行命令
        self.handler.extend_to_sales(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 获取更新后的物料
        let query = GetMaterialQuery {
            material_id,
            tenant_id,
        };
        let material = self.handler.get_material(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ExtendMaterialToSalesOrgResponse {
            material: Some(material_to_proto(&material)),
        }))
    }

    async fn extend_material_to_purchase_org(
        &self,
        request: Request<ExtendMaterialToPurchaseOrgRequest>,
    ) -> Result<Response<ExtendMaterialToPurchaseOrgResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let material_id = parse_material_id(&req.material_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let proto_purchase_data = req.purchase_data
            .ok_or_else(|| Status::invalid_argument("Missing purchase_data"))?;

        // 将 proto PurchaseData 转换为 domain PurchaseData
        let purchase_data = proto_to_purchase_data(proto_purchase_data);

        // 创建命令
        let cmd = ExtendToPurchaseCommand {
            material_id: material_id.clone(),
            tenant_id: tenant_id.clone(),
            user_id,
            purchase_data,
        };

        // 执行命令
        self.handler.extend_to_purchase(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 获取更新后的物料
        let query = GetMaterialQuery {
            material_id,
            tenant_id,
        };
        let material = self.handler.get_material(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ExtendMaterialToPurchaseOrgResponse {
            material: Some(material_to_proto(&material)),
        }))
    }

    async fn update_plant_data(
        &self,
        request: Request<UpdatePlantDataRequest>,
    ) -> Result<Response<UpdatePlantDataResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let material_id = parse_material_id(&req.material_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let proto_plant_data = req.plant_data
            .ok_or_else(|| Status::invalid_argument("Missing plant_data"))?;

        // 将 proto PlantData 转换为 domain PlantData
        let plant_data = proto_to_plant_data(proto_plant_data.clone());
        let plant = proto_plant_data.plant;

        // 创建命令
        let cmd = UpdatePlantDataCommand {
            material_id: material_id.clone(),
            tenant_id: tenant_id.clone(),
            user_id,
            plant,
            plant_data,
        };

        // 执行命令
        self.handler.update_plant_data(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 获取更新后的物料
        let query = GetMaterialQuery {
            material_id,
            tenant_id,
        };
        let material = self.handler.get_material(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdatePlantDataResponse {
            material: Some(material_to_proto(&material)),
        }))
    }

    async fn update_sales_data(
        &self,
        request: Request<UpdateSalesDataRequest>,
    ) -> Result<Response<UpdateSalesDataResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let material_id = parse_material_id(&req.material_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let proto_sales_data = req.sales_data
            .ok_or_else(|| Status::invalid_argument("Missing sales_data"))?;

        // 将 proto SalesData 转换为 domain SalesData
        let sales_data = proto_to_sales_data(proto_sales_data.clone());
        let sales_org = proto_sales_data.sales_org;

        // 创建命令
        let cmd = UpdateSalesDataCommand {
            material_id: material_id.clone(),
            tenant_id: tenant_id.clone(),
            user_id,
            sales_org,
            sales_data,
        };

        // 执行命令
        self.handler.update_sales_data(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 获取更新后的物料
        let query = GetMaterialQuery {
            material_id,
            tenant_id,
        };
        let material = self.handler.get_material(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateSalesDataResponse {
            material: Some(material_to_proto(&material)),
        }))
    }

    async fn update_purchase_data(
        &self,
        request: Request<UpdatePurchaseDataRequest>,
    ) -> Result<Response<UpdatePurchaseDataResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let material_id = parse_material_id(&req.material_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let proto_purchase_data = req.purchase_data
            .ok_or_else(|| Status::invalid_argument("Missing purchase_data"))?;

        // 将 proto PurchaseData 转换为 domain PurchaseData
        let purchase_data = proto_to_purchase_data(proto_purchase_data.clone());
        let purchase_org = proto_purchase_data.purchase_org;

        // 创建命令
        let cmd = UpdatePurchaseDataCommand {
            material_id: material_id.clone(),
            tenant_id: tenant_id.clone(),
            user_id,
            purchase_org,
            purchase_data,
        };

        // 执行命令
        self.handler.update_purchase_data(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 获取更新后的物料
        let query = GetMaterialQuery {
            material_id,
            tenant_id,
        };
        let material = self.handler.get_material(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdatePurchaseDataResponse {
            material: Some(material_to_proto(&material)),
        }))
    }

    // ========== 搜索和批量操作 ==========

    async fn search_materials(
        &self,
        request: Request<SearchMaterialsRequest>,
    ) -> Result<Response<SearchMaterialsResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        let pagination = if let Some(p) = req.pagination {
            parse_pagination(p.page, p.page_size)
        } else {
            parse_pagination(1, 20)
        };

        let query = SearchMaterialsQuery {
            tenant_id,
            query: req.query,
            pagination,
        };

        let search_results = self
            .handler
            .search_materials(query)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 将 MaterialSearchResult 转换为 proto MaterialSearchResult
        let results = search_results.iter().map(|sr| {
            MaterialSearchResult {
                material: Some(material_to_proto(&sr.material)),
                score: sr.score,
                highlights: sr.highlights.clone(),
            }
        }).collect();

        Ok(Response::new(SearchMaterialsResponse {
            results,
            pagination: Some(common::v1::PaginationResponse {
                total: search_results.len() as i32,
                page: 1,
                page_size: search_results.len() as i32,
                total_pages: 1,
            }),
        }))
    }

    async fn batch_create_materials(
        &self,
        request: Request<BatchCreateMaterialsRequest>,
    ) -> Result<Response<BatchCreateMaterialsResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let stop_on_error = req.stop_on_error;

        let mut results = Vec::new();
        let mut success_count = 0;
        let mut error_count = 0;

        for (index, material_req) in req.materials.into_iter().enumerate() {
            // 转换为 Command
            let cmd = CreateMaterialCommand {
                tenant_id: tenant_id.clone(),
                user_id: user_id.clone(),
                material_number: material_req.material_number,
                description: material_req.description,
                localized_description: material_req.localized_description.map(|l| l.into()),
                material_type_id: match parse_material_type_id(&material_req.material_type_id) {
                    Ok(id) => id,
                    Err(e) => {
                        error_count += 1;
                        results.push(common::v1::BatchOperationResult {
                            index: index as i32,
                            success: false,
                            id: String::new(),
                            error_code: String::new(),
                            error_message: e.to_string(),
                        });
                        if stop_on_error {
                            break;
                        }
                        continue;
                    }
                },
                material_group_id: if material_req.material_group_id.is_empty() {
                    None
                } else {
                    match parse_material_group_id(&material_req.material_group_id) {
                        Ok(id) => Some(id),
                        Err(e) => {
                            error_count += 1;
                            results.push(common::v1::BatchOperationResult {
                                index: index as i32,
                                success: false,
                                id: String::new(),
                                error_code: String::new(),
                                error_message: e.to_string(),
                            });
                            if stop_on_error {
                                break;
                            }
                            continue;
                        }
                    }
                },
                base_unit: material_req.base_unit,
                gross_weight: if material_req.gross_weight > 0.0 { Some(material_req.gross_weight) } else { None },
                net_weight: if material_req.net_weight > 0.0 { Some(material_req.net_weight) } else { None },
                weight_unit: if material_req.weight_unit.is_empty() { None } else { Some(material_req.weight_unit) },
                volume: if material_req.volume > 0.0 { Some(material_req.volume) } else { None },
                volume_unit: if material_req.volume_unit.is_empty() { None } else { Some(material_req.volume_unit) },
                length: None,
                width: None,
                height: None,
                dimension_unit: None,
                custom_attributes: None,
            };

            // 调用业务逻辑
            match self.handler.create_material(cmd).await {
                Ok(material_id) => {
                    success_count += 1;
                    results.push(common::v1::BatchOperationResult {
                        index: index as i32,
                        success: true,
                        id: material_id.to_string(),
                        error_code: String::new(),
                        error_message: String::new(),
                    });
                }
                Err(e) => {
                    error_count += 1;
                    results.push(common::v1::BatchOperationResult {
                        index: index as i32,
                        success: false,
                        id: String::new(),
                        error_code: String::new(),
                        error_message: e.to_string(),
                    });
                    if stop_on_error {
                        break;
                    }
                }
            }
        }

        Ok(Response::new(BatchCreateMaterialsResponse {
            results,
            success_count,
            error_count,
        }))
    }

    async fn batch_update_materials(
        &self,
        request: Request<BatchUpdateMaterialsRequest>,
    ) -> Result<Response<BatchUpdateMaterialsResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        let stop_on_error = req.stop_on_error;

        let mut results = Vec::new();
        let mut success_count = 0;
        let mut error_count = 0;

        for (index, material_req) in req.materials.into_iter().enumerate() {
            let material_id = match parse_material_id(&material_req.id) {
                Ok(id) => id,
                Err(e) => {
                    error_count += 1;
                    results.push(common::v1::BatchOperationResult {
                        index: index as i32,
                        success: false,
                        id: String::new(),
                        error_code: String::new(),
                        error_message: e.to_string(),
                    });
                    if stop_on_error {
                        break;
                    }
                    continue;
                }
            };

            let cmd = UpdateMaterialCommand {
                material_id: material_id.clone(),
                tenant_id: tenant_id.clone(),
                user_id: user_id.clone(),
                description: if material_req.description.is_empty() { None } else { Some(material_req.description) },
                localized_description: material_req.localized_description.map(|l| l.into()),
                material_group_id: if material_req.material_group_id.is_empty() {
                    None
                } else {
                    match parse_material_group_id(&material_req.material_group_id) {
                        Ok(id) => Some(id),
                        Err(e) => {
                            error_count += 1;
                            results.push(common::v1::BatchOperationResult {
                                index: index as i32,
                                success: false,
                                id: material_id.to_string(),
                                error_code: String::new(),
                                error_message: e.to_string(),
                            });
                            if stop_on_error {
                                break;
                            }
                            continue;
                        }
                    }
                },
                base_unit: None,
                gross_weight: if material_req.gross_weight > 0.0 { Some(material_req.gross_weight) } else { None },
                net_weight: if material_req.net_weight > 0.0 { Some(material_req.net_weight) } else { None },
                weight_unit: if material_req.weight_unit.is_empty() { None } else { Some(material_req.weight_unit) },
                volume: if material_req.volume > 0.0 { Some(material_req.volume) } else { None },
                volume_unit: if material_req.volume_unit.is_empty() { None } else { Some(material_req.volume_unit) },
                length: if material_req.length > 0.0 { Some(material_req.length) } else { None },
                width: if material_req.width > 0.0 { Some(material_req.width) } else { None },
                height: if material_req.height > 0.0 { Some(material_req.height) } else { None },
                dimension_unit: if material_req.dimension_unit.is_empty() { None } else { Some(material_req.dimension_unit) },
                custom_attributes: if material_req.custom_attributes.is_empty() { None } else { Some(serde_json::to_value(material_req.custom_attributes).unwrap()) },
            };

            match self.handler.update_material(cmd).await {
                Ok(_) => {
                    success_count += 1;
                    results.push(common::v1::BatchOperationResult {
                        index: index as i32,
                        success: true,
                        id: material_id.to_string(),
                        error_code: String::new(),
                        error_message: String::new(),
                    });
                }
                Err(e) => {
                    error_count += 1;
                    results.push(common::v1::BatchOperationResult {
                        index: index as i32,
                        success: false,
                        id: material_id.to_string(),
                        error_code: String::new(),
                        error_message: e.to_string(),
                    });
                    if stop_on_error {
                        break;
                    }
                }
            }
        }

        Ok(Response::new(BatchUpdateMaterialsResponse {
            results,
            success_count,
            error_count,
        }))
    }

    // ========== 其他方法（暂未实现） ==========

    async fn get_material_change_history(
        &self,
        _request: Request<GetMaterialChangeHistoryRequest>,
    ) -> Result<Response<GetMaterialChangeHistoryResponse>, Status> {
        // TODO: 需要实现事件溯源和变更历史查询
        Err(Status::unimplemented("Change history requires event sourcing implementation"))
    }

    async fn get_alternative_materials(
        &self,
        request: Request<GetAlternativeMaterialsRequest>,
    ) -> Result<Response<GetAlternativeMaterialsResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        // 解析物料 ID
        let material_id = parse_material_id(&req.material_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // 创建查询
        let query = GetAlternativeMaterialsQuery {
            material_id,
            tenant_id,
            plant: if req.plant.is_empty() { None } else { Some(req.plant) },
        };

        // 执行查询
        let alternatives = self
            .handler
            .get_alternative_materials(query)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 转换为 Proto
        let proto_alternatives = alternatives
            .iter()
            .map(alternative_material_to_proto)
            .collect();

        Ok(Response::new(GetAlternativeMaterialsResponse {
            alternatives: proto_alternatives,
        }))
    }

    async fn set_alternative_material(
        &self,
        request: Request<SetAlternativeMaterialRequest>,
    ) -> Result<Response<()>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        // 解析物料 ID
        let material_id = parse_material_id(&req.material_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let alternative_material_id = parse_material_id(&req.alternative_material_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // 解析有效期
        let (valid_from, valid_to) = if let Some(validity) = req.validity {
            (
                proto_to_timestamp(validity.valid_from),
                proto_to_timestamp(validity.valid_to),
            )
        } else {
            (None, None)
        };

        // 创建命令
        let cmd = SetAlternativeMaterialCommand {
            material_id,
            alternative_material_id,
            tenant_id,
            user_id,
            plant: if req.plant.is_empty() { None } else { Some(req.plant) },
            priority: req.priority,
            valid_from,
            valid_to,
        };

        // 执行命令
        self.handler
            .set_alternative_material(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn remove_alternative_material(
        &self,
        request: Request<RemoveAlternativeMaterialRequest>,
    ) -> Result<Response<()>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        // 解析物料 ID
        let material_id = parse_material_id(&req.material_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let alternative_material_id = parse_material_id(&req.alternative_material_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // 创建命令
        let cmd = RemoveAlternativeMaterialCommand {
            material_id,
            alternative_material_id,
            tenant_id,
            user_id,
            plant: if req.plant.is_empty() { None } else { Some(req.plant) },
        };

        // 执行命令
        self.handler
            .remove_alternative_material(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }

    async fn create_unit_conversion(
        &self,
        request: Request<CreateUnitConversionRequest>,
    ) -> Result<Response<CreateUnitConversionResponse>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        // 解析物料 ID
        let material_id = parse_material_id(&req.material_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // 解析单位换算
        let conversion = req.conversion
            .ok_or_else(|| Status::invalid_argument("conversion is required"))?;

        // 创建命令
        let cmd = CreateUnitConversionCommand {
            material_id: material_id.clone(),
            tenant_id: tenant_id.clone(),
            user_id,
            from_unit: conversion.from_unit,
            to_unit: conversion.to_unit,
            numerator: conversion.numerator,
            denominator: conversion.denominator,
            ean_upc: if conversion.ean_upc.is_empty() { None } else { Some(conversion.ean_upc) },
        };

        // 执行命令
        self.handler
            .create_unit_conversion(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 获取更新后的物料
        let query = GetMaterialQuery {
            material_id,
            tenant_id,
        };
        let material = self.handler.get_material(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateUnitConversionResponse {
            material: Some(material_to_proto(&material)),
        }))
    }

    async fn delete_unit_conversion(
        &self,
        request: Request<DeleteUnitConversionRequest>,
    ) -> Result<Response<()>, Status> {
        let metadata = request.metadata();
        let tenant_id = extract_tenant_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;
        let user_id = extract_user_id(metadata).map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        // 解析物料 ID
        let material_id = parse_material_id(&req.material_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // 创建命令
        let cmd = DeleteUnitConversionCommand {
            material_id,
            tenant_id,
            user_id,
            from_unit: req.from_unit,
            to_unit: req.to_unit,
        };

        // 执行命令
        self.handler
            .delete_unit_conversion(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(()))
    }
}
