# MDM-Material 开发计划

## 项目概览

**当前完成度**: ~70% (核心功能已实现)
**代码规模**: 54个Rust文件，约10,423行代码
**架构模式**: Clean Architecture + DDD + CQRS
**技术栈**: Rust + gRPC + PostgreSQL + Redis

## 当前实现状态

### ✅ 已完成模块 (100%)

#### 1. 领域层 (Domain Layer) - 4,193行代码
- **核心实体**: Material (694行), MaterialGroup (182行), MaterialType (214行)
- **值对象**: MaterialNumber, LocalizedText, UnitConversion, AlternativeMaterial
- **枚举类型**: MaterialStatus, DataStatus, PriceControl, ProcurementType
- **业务视图**: PlantData, SalesData, PurchaseData, StorageData, AccountingData, QualityData
- **领域事件**: MaterialEvents 完整事件溯源支持
- **仓储接口**: 3个仓储接口，33个方法定义

#### 2. 数据库迁移 (Database Migrations) - 612行SQL
- **16张表**: 3个核心表 + 6个视图表 + 3个关系表 + 4个辅助表
- **48+索引**: 租户隔离、业务查询、复合索引、全文搜索
- **行级安全**: 完整的多租户隔离机制
- **约束**: UNIQUE, FOREIGN KEY, CHECK 约束

#### 3. Proto转换器 (Proto Converters) - 539行代码
- **6个视图数据转换器**: 所有视图数据的双向转换
- **枚举处理**: 正确的Unspecified变体处理
- **复杂类型支持**: Money, Timestamp, LocalizedText

### 🟡 基本完成模块 (60-85%)

#### 4. API层 (API Layer) - 85%完成
**已实现的RPC方法 (27/32):**
- 物料CRUD: CreateMaterial, GetMaterial, UpdateMaterial, DeleteMaterial, ListMaterials
- 状态管理: ActivateMaterial, DeactivateMaterial, BlockMaterial, MarkForDeletion
- 视图扩展: ExtendMaterialToPlant, ExtendMaterialToSalesOrg, ExtendMaterialToPurchaseOrg
- 视图更新: UpdatePlantData, UpdateSalesData, UpdatePurchaseData
- 物料组: CreateMaterialGroup, GetMaterialGroup, UpdateMaterialGroup, DeleteMaterialGroup, ListMaterialGroups
- 物料类型: CreateMaterialType, GetMaterialType, UpdateMaterialType, ListMaterialTypes
- 批量操作: BatchCreateMaterials, BatchUpdateMaterials
- 搜索: SearchMaterials

**待实现的RPC方法 (5个):**
- GetMaterialChangeHistory (需要事件溯源)
- GetAlternativeMaterials, SetAlternativeMaterial, RemoveAlternativeMaterial (需要关系管理)
- CreateUnitConversion, DeleteUnitConversion (需要单位换算管理)

#### 5. 应用层 (Application Layer) - 80%完成
- **ServiceHandler**: 1,100+行，完整的CRUD操作
- **CQRS实现**: Commands和Queries模块
- **业务逻辑**: 物料创建、更新、视图扩展、状态管理
- **验证**: 命令验证和业务规则

#### 6. 基础设施层 (Infrastructure Layer) - 60%完成
- **PostgreSQL仓储**: 3个实体的完整实现
- **33个仓储方法**: 所有CRUD操作、复杂查询
- **数据转换器**: Row ↔ Domain对象转换 (631行)
- **事务支持**: ACID事务、乐观锁

### ❌ 未实现模块 (0%)

#### 7. 测试 (Testing) - 0%完成
- 单元测试
- 集成测试
- 性能测试

## 开发计划

### 第一阶段：完成核心功能 (优先级：P0)

#### 任务1: 完成基础设施层视图数据操作
**预计工作量**: 中等
**文件**: `src/infrastructure/persistence/material_repository.rs`

需要实现的方法 (10个):
```rust
// 销售数据
async fn save_sales_data(&self, material_id: &MaterialId, data: &SalesData) -> Result<()>
async fn get_sales_data(&self, material_id: &MaterialId) -> Result<Vec<SalesData>>

// 采购数据
async fn save_purchase_data(&self, material_id: &MaterialId, data: &PurchaseData) -> Result<()>
async fn get_purchase_data(&self, material_id: &MaterialId) -> Result<Vec<PurchaseData>>

// 仓储数据
async fn save_storage_data(&self, material_id: &MaterialId, data: &StorageData) -> Result<()>
async fn get_storage_data(&self, material_id: &MaterialId) -> Result<Vec<StorageData>>

// 会计数据
async fn save_accounting_data(&self, material_id: &MaterialId, data: &AccountingData) -> Result<()>
async fn get_accounting_data(&self, material_id: &MaterialId) -> Result<Vec<AccountingData>>

// 质量数据
async fn save_quality_data(&self, material_id: &MaterialId, data: &QualityData) -> Result<()>
async fn get_quality_data(&self, material_id: &MaterialId) -> Result<Vec<QualityData>>
```

**实现模式**: 参考已实现的 `save_plant_data` 方法

**涉及的数据库表**:
- `mdm_material_sales_data`
- `mdm_material_purchase_data`
- `mdm_material_storage_data`
- `mdm_material_accounting_data`
- `mdm_material_quality_data`

#### 任务2: 添加替代查询方法
**预计工作量**: 小
**文件**:
- `src/infrastructure/persistence/material_repository.rs`
- `src/infrastructure/persistence/material_group_repository.rs`
- `src/infrastructure/persistence/material_type_repository.rs`

需要实现的方法:
```rust
// MaterialRepository
async fn find_by_material_number(&self, tenant_id: &TenantId, material_number: &MaterialNumber) -> Result<Option<Material>>

// MaterialGroupRepository
async fn find_by_code(&self, tenant_id: &TenantId, code: &str) -> Result<Option<MaterialGroup>>

// MaterialTypeRepository
async fn find_by_code(&self, tenant_id: &TenantId, code: &str) -> Result<Option<MaterialType>>
```

#### 任务3: 更新API层使用新的查询方法
**预计工作量**: 小
**文件**: `src/api/grpc_service.rs`

更新以下RPC方法:
- `GetMaterial`: 支持通过material_number查询
- `GetMaterialGroup`: 支持通过code查询
- `GetMaterialType`: 支持通过code查询

### 第二阶段：高级功能 (优先级：P1)

#### 任务4: 实现替代物料关系管理
**预计工作量**: 中等
**文件**:
- `src/infrastructure/persistence/material_repository.rs`
- `src/application/handler.rs`
- `src/api/grpc_service.rs`

需要实现的功能:
1. 数据库操作:
```rust
async fn add_alternative_material(&self, material_id: &MaterialId, alternative: &AlternativeMaterial) -> Result<()>
async fn remove_alternative_material(&self, material_id: &MaterialId, alternative_id: &MaterialId) -> Result<()>
async fn get_alternative_materials(&self, material_id: &MaterialId) -> Result<Vec<AlternativeMaterial>>
```

2. 应用层处理:
```rust
pub async fn set_alternative_material(&self, cmd: SetAlternativeMaterialCommand) -> Result<()>
pub async fn remove_alternative_material(&self, cmd: RemoveAlternativeMaterialCommand) -> Result<()>
pub async fn get_alternative_materials(&self, query: GetAlternativeMaterialsQuery) -> Result<Vec<AlternativeMaterial>>
```

3. API层实现:
- `GetAlternativeMaterials`
- `SetAlternativeMaterial`
- `RemoveAlternativeMaterial`

**涉及的数据库表**: `mdm_material_alternatives`

#### 任务5: 实现单位换算管理
**预计工作量**: 中等
**文件**:
- `src/infrastructure/persistence/material_repository.rs`
- `src/application/handler.rs`
- `src/api/grpc_service.rs`

需要实现的功能:
1. 数据库操作:
```rust
async fn save_unit_conversion(&self, material_id: &MaterialId, conversion: &UnitConversion) -> Result<()>
async fn delete_unit_conversion(&self, material_id: &MaterialId, from_unit: &str, to_unit: &str) -> Result<()>
async fn get_unit_conversions(&self, material_id: &MaterialId) -> Result<Vec<UnitConversion>>
```

2. 应用层处理:
```rust
pub async fn create_unit_conversion(&self, cmd: CreateUnitConversionCommand) -> Result<()>
pub async fn delete_unit_conversion(&self, cmd: DeleteUnitConversionCommand) -> Result<()>
```

3. API层实现:
- `CreateUnitConversion`
- `DeleteUnitConversion`

**涉及的数据库表**: `mdm_material_unit_conversions`

#### 任务6: 实现变更历史查询
**预计工作量**: 大
**文件**:
- `src/infrastructure/persistence/event_store.rs` (新建)
- `src/application/handler.rs`
- `src/api/grpc_service.rs`

需要实现的功能:
1. 事件存储:
```rust
pub struct EventStore {
    pool: PgPool,
}

impl EventStore {
    async fn save_event(&self, event: &MaterialEvent) -> Result<()>
    async fn get_events(&self, material_id: &MaterialId) -> Result<Vec<MaterialEvent>>
    async fn get_events_by_time_range(&self, material_id: &MaterialId, from: DateTime, to: DateTime) -> Result<Vec<MaterialEvent>>
}
```

2. 应用层查询:
```rust
pub async fn get_material_change_history(&self, query: GetMaterialChangeHistoryQuery) -> Result<Vec<MaterialChangeRecord>>
```

3. API层实现:
- `GetMaterialChangeHistory`

**涉及的数据库表**: `mdm_material_events` (可能需要新建迁移)

### 第三阶段：测试基础设施 (优先级：P1)

#### 任务7: 建立测试框架
**预计工作量**: 大
**目录结构**:
```
tests/
├── unit/
│   ├── domain/
│   │   ├── material_tests.rs
│   │   ├── material_group_tests.rs
│   │   ├── material_type_tests.rs
│   │   └── value_objects_tests.rs
│   ├── application/
│   │   ├── handler_tests.rs
│   │   └── command_tests.rs
│   └── api/
│       └── proto_converter_tests.rs
├── integration/
│   ├── repository_tests.rs
│   ├── grpc_tests.rs
│   └── transaction_tests.rs
└── fixtures/
    ├── test_data.rs
    └── database_setup.rs
```

#### 任务8: 编写单元测试
**预计工作量**: 大
**覆盖范围**:
- 领域实体业务逻辑
- 值对象验证
- 命令处理器
- Proto转换器

**目标覆盖率**: 80%+

#### 任务9: 编写集成测试
**预计工作量**: 大
**测试场景**:
- 数据库仓储操作
- gRPC端到端测试
- 事务和并发测试
- 多租户隔离测试

### 第四阶段：性能优化 (优先级：P2)

#### 任务10: 查询优化
**预计工作量**: 中等
**优化点**:
- 解决N+1查询问题
- 添加批量加载视图数据
- 优化复杂查询的执行计划
- 添加查询结果缓存

#### 任务11: 缓存层实现
**预计工作量**: 中等
**文件**: `src/infrastructure/cache/` (新建)

实现内容:
```rust
pub struct MaterialCache {
    redis: RedisPool,
}

impl MaterialCache {
    async fn get_material(&self, id: &MaterialId) -> Result<Option<Material>>
    async fn set_material(&self, material: &Material, ttl: Duration) -> Result<()>
    async fn invalidate_material(&self, id: &MaterialId) -> Result<()>
}
```

**缓存策略**:
- 物料基础数据: TTL 1小时
- 物料组/类型: TTL 24小时
- 视图数据: TTL 30分钟

#### 任务12: 批量操作优化
**预计工作量**: 中等
**优化内容**:
- 使用PostgreSQL的COPY命令进行批量插入
- 实现批量更新的事务优化
- 添加批量操作的进度反馈

### 第五阶段：运维和监控 (优先级：P2)

#### 任务13: 添加监控指标
**预计工作量**: 中等
**文件**: `src/observability/metrics.rs` (新建)

实现指标:
- 请求计数和延迟
- 数据库连接池状态
- 缓存命中率
- 业务指标 (物料创建/更新数量)

#### 任务14: 分布式追踪
**预计工作量**: 中等
**集成**: OpenTelemetry

追踪范围:
- gRPC请求
- 数据库查询
- 缓存操作
- 外部服务调用

#### 任务15: 健康检查
**预计工作量**: 小
**文件**: `src/api/health.rs` (新建)

实现检查:
```rust
pub struct HealthCheck {
    db_pool: PgPool,
    redis_pool: RedisPool,
}

impl HealthCheck {
    async fn check_database(&self) -> HealthStatus
    async fn check_cache(&self) -> HealthStatus
    async fn check_overall(&self) -> HealthStatus
}
```

### 第六阶段：文档和部署 (优先级：P3)

#### 任务16: API文档
**预计工作量**: 中等
**内容**:
- gRPC API完整文档
- 请求/响应示例
- 错误码说明
- 使用指南

#### 任务17: 运维文档
**预计工作量**: 中等
**内容**:
- 部署指南
- 配置说明
- 监控告警规则
- 故障排查手册

#### 任务18: 性能基准测试
**预计工作量**: 中等
**测试场景**:
- 单个物料CRUD性能
- 批量操作性能
- 并发请求性能
- 数据库查询性能

## 里程碑

### 里程碑1: 核心功能完整 (第1-2周)
- ✅ 完成所有视图数据操作
- ✅ 完成替代查询方法
- ✅ 完成替代物料和单位换算管理
- **交付物**: 功能完整的服务，支持所有基础操作

### 里程碑2: 测试覆盖 (第3-4周)
- ✅ 单元测试覆盖率 > 80%
- ✅ 集成测试覆盖核心场景
- ✅ 性能测试基准建立
- **交付物**: 高质量、可靠的代码库

### 里程碑3: 生产就绪 (第5-6周)
- ✅ 性能优化完成
- ✅ 监控和追踪就绪
- ✅ 文档完整
- **交付物**: 可部署到生产环境的服务

## 风险和依赖

### 技术风险
1. **数据库性能**: 大量视图数据可能导致查询性能问题
   - 缓解: 提前进行性能测试，优化索引和查询

2. **并发控制**: 乐观锁可能在高并发下导致大量冲突
   - 缓解: 考虑使用悲观锁或分布式锁

3. **缓存一致性**: Redis缓存与数据库数据不一致
   - 缓解: 实现缓存失效策略和一致性检查

### 外部依赖
1. **数据库**: PostgreSQL 14+
2. **缓存**: Redis 6+
3. **消息队列**: (如果需要事件发布)
4. **监控系统**: Prometheus + Grafana

## 资源需求

### 开发资源
- **后端开发**: 1-2人
- **测试**: 1人 (兼职)
- **DevOps**: 1人 (兼职)

### 基础设施
- **开发环境**: PostgreSQL + Redis
- **测试环境**: 完整的服务栈
- **生产环境**: 高可用部署

## 总结

MDM-Material模块已经完成了核心架构和大部分功能实现，当前的主要任务是：

1. **补全基础功能** (P0): 完成视图数据操作和查询方法
2. **实现高级特性** (P1): 关系管理、单位换算、变更历史
3. **建立测试体系** (P1): 确保代码质量和可靠性
4. **优化和监控** (P2): 提升性能和可观测性

按照此计划，预计在6周内可以完成所有核心功能并达到生产就绪状态。
