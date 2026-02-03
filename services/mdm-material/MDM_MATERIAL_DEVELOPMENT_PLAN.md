# MDM-Material 开发计划

## 项目概览

**当前完成度**: ~95% (几乎所有功能已实现)
**代码规模**: 54个Rust文件，约11,813行代码
**架构模式**: Clean Architecture + DDD + CQRS + Event Sourcing
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

#### 2. 数据库迁移 (Database Migrations) - 13个迁移文件
- **17张表**: 3个核心表 + 6个视图表 + 3个关系表 + 1个事件表 + 4个辅助表
- **48+索引**: 租户隔离、业务查询、复合索引、全文搜索
- **行级安全**: 完整的多租户隔离机制
- **约束**: UNIQUE, FOREIGN KEY, CHECK 约束
- **事件表**: material_events 表支持事件溯源

#### 3. Proto转换器 (Proto Converters) - 539行代码
- **6个视图数据转换器**: 所有视图数据的双向转换
- **枚举处理**: 正确的Unspecified变体处理
- **复杂类型支持**: Money, Timestamp, LocalizedText

#### 4. API层 (API Layer) - 100%完成 ✅
**已实现的RPC方法 (33/33):**
- 物料CRUD: CreateMaterial, GetMaterial, UpdateMaterial, DeleteMaterial, ListMaterials
- 状态管理: ActivateMaterial, DeactivateMaterial, BlockMaterial, MarkForDeletion
- 视图扩展: ExtendMaterialToPlant, ExtendMaterialToSalesOrg, ExtendMaterialToPurchaseOrg
- 视图更新: UpdatePlantData, UpdateSalesData, UpdatePurchaseData
- 物料组: CreateMaterialGroup, GetMaterialGroup, UpdateMaterialGroup, DeleteMaterialGroup, ListMaterialGroups
- 物料类型: CreateMaterialType, GetMaterialType, UpdateMaterialType, ListMaterialTypes
- 批量操作: BatchCreateMaterials, BatchUpdateMaterials
- 搜索: SearchMaterials
- **变更历史**: GetMaterialChangeHistory ✅
- **替代物料**: GetAlternativeMaterials, SetAlternativeMaterial, RemoveAlternativeMaterial ✅
- **单位换算**: CreateUnitConversion, DeleteUnitConversion ✅

**代码统计**: 1,524行代码

#### 5. 应用层 (Application Layer) - 100%完成 ✅
- **ServiceHandler**: 1,361行，完整的业务逻辑实现
- **47个公共方法**: 涵盖所有业务场景
- **CQRS实现**: Commands和Queries模块
- **业务逻辑**:
  - 物料完整生命周期管理
  - 6种视图数据扩展和更新
  - 替代物料关系管理 ✅
  - 单位换算管理 ✅
  - 变更历史查询 ✅
- **验证**: 命令验证和业务规则

#### 6. 基础设施层 (Infrastructure Layer) - 100%完成 ✅
- **PostgreSQL仓储**: 2,182行代码
  - MaterialRepository: 完整实现
  - MaterialGroupRepository: 完整实现
  - MaterialTypeRepository: 完整实现
- **所有视图数据操作**:
  - ✅ save_plant_data / get_plant_data
  - ✅ save_sales_data / get_sales_data
  - ✅ save_purchase_data / get_purchase_data
  - ✅ save_storage_data / get_storage_data
  - ✅ save_accounting_data / get_accounting_data
  - ✅ save_quality_data / get_quality_data
- **替代物料操作**: ✅ save_alternative / get_alternatives / delete_alternative
- **单位换算操作**: ✅ save_unit_conversion / get_unit_conversions / delete_unit_conversion
- **替代查询方法**: ✅ find_by_material_number, find_by_code
- **数据转换器**: Row ↔ Domain对象转换 (631行)
- **事务支持**: ACID事务、乐观锁

#### 7. 事件存储 (Event Store) - 100%完成 ✅
- **PostgresEventStore**: 306行代码
- **事件持久化**: save_event 方法
- **事件查询**: get_events, get_events_by_time_range
- **事件版本控制**: aggregate_version 支持
- **分页支持**: 完整的分页查询
- **数据库表**: material_events 表已创建

### ❌ 未实现模块 (0%)

#### 8. 测试 (Testing) - 0%完成
- 单元测试
- 集成测试
- 性能测试

## 开发计划

### ✅ 第一阶段：完成核心功能 (优先级：P0) - 已完成

~~所有核心功能已经完成实现~~

#### ✅ 任务1: 完成基础设施层视图数据操作 - 已完成
**状态**: ✅ 已完成
**文件**: `src/infrastructure/persistence/postgres.rs`

已实现的方法 (12个):
- ✅ save_plant_data / get_plant_data (行1076-1155)
- ✅ save_sales_data / get_sales_data (行1157-1227)
- ✅ save_purchase_data / get_purchase_data (行1229-1310)
- ✅ save_storage_data / get_storage_data (行1312-1375)
- ✅ save_accounting_data / get_accounting_data (行1377-1462)
- ✅ save_quality_data / get_quality_data (行1464-1544)

#### ✅ 任务2: 添加替代查询方法 - 已完成
**状态**: ✅ 已完成
**文件**: `src/infrastructure/persistence/postgres.rs`

已实现的方法:
- ✅ MaterialRepository::find_by_material_number
- ✅ MaterialGroupRepository::find_by_code (行66-88)
- ✅ MaterialTypeRepository::find_by_code (行66-88)

#### ✅ 任务3: 更新API层使用新的查询方法 - 已完成
**状态**: ✅ 已完成
**文件**: `src/api/grpc_service.rs`

所有RPC方法已实现并支持多种查询方式。

### ✅ 第二阶段：高级功能 (优先级：P1) - 已完成

~~所有高级功能已经完成实现~~

#### ✅ 任务4: 实现替代物料关系管理 - 已完成
**状态**: ✅ 已完成

已实现的功能:
1. ✅ 数据库操作 (postgres.rs 行1005-1074):
   - save_alternative
   - get_alternatives
   - delete_alternative

2. ✅ 应用层处理 (handler.rs 行1143-1253):
   - get_alternative_materials (行1143)
   - set_alternative_material (行1169)
   - remove_alternative_material (行1224)

3. ✅ API层实现 (grpc_service.rs 行1325-1442):
   - GetAlternativeMaterials (行1325)
   - SetAlternativeMaterial (行1363)
   - RemoveAlternativeMaterial (行1410)

#### ✅ 任务5: 实现单位换算管理 - 已完成
**状态**: ✅ 已完成

已实现的功能:
1. ✅ 数据库操作 (postgres.rs 行1546-1650):
   - save_unit_conversion
   - get_unit_conversions
   - delete_unit_conversion

2. ✅ 应用层处理 (handler.rs 行1255-1320):
   - create_unit_conversion (行1255)
   - delete_unit_conversion (行1294)

3. ✅ API层实现 (grpc_service.rs 行1444-1524):
   - CreateUnitConversion (行1444)
   - DeleteUnitConversion (行1493)

#### ✅ 任务6: 实现变更历史查询 - 已完成
**状态**: ✅ 已完成

已实现的功能:
1. ✅ 事件存储 (event_store.rs 306行):
   - PostgresEventStore 完整实现
   - save_event (行50)
   - get_events (行84)
   - get_events_by_time_range (行109)

2. ✅ 应用层查询 (handler.rs 行1322-1361):
   - get_material_change_history

3. ✅ API层实现 (grpc_service.rs 行1252-1323):
   - GetMaterialChangeHistory

4. ✅ 数据库表: material_events 表已创建 (迁移文件 20260201000013)

### 第三阶段：测试基础设施 (优先级：P1) - 待实现

#### 任务7: 建立测试框架
**状态**: ⏳ 待实现
**预计工作量**: 大

**目录结构**:
```
tests/
├── unit/
│   ├── domain/
│   │   ├── material_tests.rs          # 物料实体测试
│   │   ├── material_group_tests.rs    # 物料组实体测试
│   │   ├── material_type_tests.rs     # 物料类型实体测试
│   │   └── value_objects_tests.rs     # 值对象测试
│   ├── application/
│   │   ├── handler_tests.rs           # 业务处理器测试
│   │   └── command_tests.rs           # 命令验证测试
│   └── api/
│       └── proto_converter_tests.rs   # Proto转换器测试
├── integration/
│   ├── repository_tests.rs            # 仓储集成测试
│   ├── grpc_tests.rs                  # gRPC端到端测试
│   ├── transaction_tests.rs           # 事务测试
│   └── event_store_tests.rs           # 事件存储测试
└── fixtures/
    ├── test_data.rs                   # 测试数据工厂
    └── database_setup.rs              # 数据库测试环境设置
```

**实现步骤**:
1. 添加测试依赖到 Cargo.toml:
   - `tokio-test`
   - `mockall` (用于 mock)
   - `testcontainers` (用于数据库测试)
   - `fake` (用于生成测试数据)
2. 创建测试数据工厂
3. 设置测试数据库环境

#### 任务8: 编写单元测试
**状态**: ⏳ 待实现
**预计工作量**: 大

**覆盖范围**:

1. **领域实体测试** (优先级最高):
   - Material 实体业务逻辑
     - 创建物料
     - 更新物料
     - 状态转换 (激活/停用/锁定/标记删除)
     - 视图数据扩展
   - MaterialGroup 实体
     - 层级关系验证
     - 编码唯一性
   - MaterialType 实体
     - 类型配置验证

2. **值对象测试**:
   - MaterialNumber 验证规则
   - LocalizedText 多语言处理
   - UnitConversion 换算逻辑
   - AlternativeMaterial 关系验证

3. **命令处理器测试**:
   - 命令验证逻辑
   - 业务规则检查
   - 错误处理

4. **Proto转换器测试**:
   - Domain ↔ Proto 双向转换
   - 枚举类型转换
   - 可选字段处理

**目标覆盖率**: 80%+

#### 任务9: 编写集成测试
**状态**: ⏳ 待实现
**预计工作量**: 大

**测试场景**:

1. **数据库仓储操作**:
   - CRUD 操作完整性
   - 视图数据保存和查询
   - 替代物料关系管理
   - 单位换算管理
   - 事务回滚测试
   - 乐观锁并发控制

2. **gRPC端到端测试**:
   - 所有33个RPC方法
   - 请求验证
   - 错误响应
   - 元数据提取 (tenant_id, user_id)

3. **事务和并发测试**:
   - 并发创建物料
   - 并发更新同一物料
   - 版本冲突处理
   - 死锁检测

4. **多租户隔离测试**:
   - 租户数据隔离
   - 跨租户访问拒绝
   - RLS 策略验证

5. **事件存储测试**:
   - 事件持久化
   - 事件查询
   - 变更历史重建
   - 事件版本控制

### 第四阶段：性能优化 (优先级：P2) - 待实现

#### 任务10: 查询优化
**状态**: ⏳ 待实现
**预计工作量**: 中等

**优化点**:

1. **解决N+1查询问题**:
   - 当前问题: 获取物料列表时，每个物料的视图数据需要单独查询
   - 优化方案: 实现批量加载视图数据
   ```rust
   async fn batch_load_plant_data(&self, material_ids: &[MaterialId]) -> AppResult<HashMap<MaterialId, Vec<PlantData>>>
   async fn batch_load_sales_data(&self, material_ids: &[MaterialId]) -> AppResult<HashMap<MaterialId, Vec<SalesData>>>
   // ... 其他视图数据
   ```

2. **优化复杂查询的执行计划**:
   - 分析慢查询日志
   - 添加缺失的索引
   - 使用 EXPLAIN ANALYZE 优化查询
   - 考虑使用物化视图

3. **查询结果缓存**:
   - 实现查询结果的内存缓存
   - 使用 LRU 策略管理缓存大小
   - 在数据更新时自动失效缓存

4. **分页优化**:
   - 使用游标分页替代 OFFSET/LIMIT
   - 实现 keyset pagination

#### 任务11: 缓存层实现
**状态**: ⏳ 待实现
**预计工作量**: 中等
**文件**: `src/infrastructure/cache/` (新建)

**实现内容**:

1. **缓存接口定义**:
```rust
#[async_trait]
pub trait MaterialCache: Send + Sync {
    async fn get_material(&self, id: &MaterialId) -> AppResult<Option<Material>>;
    async fn set_material(&self, material: &Material, ttl: Duration) -> AppResult<()>;
    async fn invalidate_material(&self, id: &MaterialId) -> AppResult<()>;
    async fn get_material_group(&self, id: &MaterialGroupId) -> AppResult<Option<MaterialGroup>>;
    async fn set_material_group(&self, group: &MaterialGroup, ttl: Duration) -> AppResult<()>;
    async fn get_material_type(&self, id: &MaterialTypeId) -> AppResult<Option<MaterialType>>;
    async fn set_material_type(&self, type_: &MaterialType, ttl: Duration) -> AppResult<()>;
}
```

2. **Redis实现**:
```rust
pub struct RedisMaterialCache {
    redis: RedisPool,
}

impl RedisMaterialCache {
    pub fn new(redis: RedisPool) -> Self {
        Self { redis }
    }

    fn cache_key(&self, prefix: &str, id: &str) -> String {
        format!("mdm:material:{}:{}", prefix, id)
    }
}
```

3. **缓存策略**:
   - 物料基础数据: TTL 1小时
   - 物料组/类型: TTL 24小时 (变更频率低)
   - 视图数据: TTL 30分钟
   - 搜索结果: TTL 5分钟

4. **缓存失效策略**:
   - 写操作自动失效相关缓存
   - 支持手动刷新缓存
   - 实现缓存预热机制

5. **集成到仓储层**:
```rust
pub struct CachedMaterialRepository {
    repo: Arc<dyn MaterialRepository>,
    cache: Arc<dyn MaterialCache>,
}

impl CachedMaterialRepository {
    async fn find_by_id(&self, id: &MaterialId, tenant_id: &TenantId) -> AppResult<Option<Material>> {
        // 先查缓存
        if let Some(material) = self.cache.get_material(id).await? {
            return Ok(Some(material));
        }

        // 缓存未命中，查数据库
        if let Some(material) = self.repo.find_by_id(id, tenant_id).await? {
            // 写入缓存
            self.cache.set_material(&material, Duration::from_secs(3600)).await?;
            return Ok(Some(material));
        }

        Ok(None)
    }
}
```

#### 任务12: 批量操作优化
**状态**: ⏳ 待实现
**预计工作量**: 中等

**优化内容**:

1. **使用PostgreSQL的COPY命令进行批量插入**:
   - 当前: 使用 INSERT ... VALUES 逐条插入
   - 优化: 使用 COPY FROM STDIN 批量导入
   ```rust
   async fn batch_insert_materials(&self, materials: &[Material]) -> AppResult<()> {
       // 使用 COPY 命令批量插入
       let mut writer = self.pool.copy_in_raw(
           "COPY materials (id, tenant_id, material_number, ...) FROM STDIN WITH (FORMAT CSV)"
       ).await?;

       for material in materials {
           // 写入 CSV 格式数据
           writer.write_all(format_material_as_csv(material).as_bytes()).await?;
       }

       writer.finish().await?;
       Ok(())
   }
   ```

2. **批量更新的事务优化**:
   - 使用 CTE (Common Table Expressions) 进行批量更新
   - 减少事务持有时间
   ```rust
   async fn batch_update_materials(&self, updates: &[(MaterialId, MaterialUpdate)]) -> AppResult<()> {
       let mut tx = self.pool.begin().await?;

       // 使用 unnest 和 UPDATE ... FROM 进行批量更新
       sqlx::query(r#"
           UPDATE materials m
           SET description = u.description,
               updated_at = NOW()
           FROM (
               SELECT unnest($1::uuid[]) as id,
                      unnest($2::text[]) as description
           ) u
           WHERE m.id = u.id
       "#)
       .bind(/* ids */)
       .bind(/* descriptions */)
       .execute(&mut *tx)
       .await?;

       tx.commit().await?;
       Ok(())
   }
   ```

3. **批量操作的进度反馈**:
   - 实现流式处理，支持进度回调
   ```rust
   pub struct BatchProgress {
       pub total: usize,
       pub processed: usize,
       pub failed: usize,
   }

   async fn batch_create_with_progress<F>(
       &self,
       materials: Vec<Material>,
       progress_callback: F,
   ) -> AppResult<BatchResult>
   where
       F: Fn(BatchProgress) + Send + Sync,
   {
       let total = materials.len();
       let mut processed = 0;
       let mut failed = 0;

       for chunk in materials.chunks(100) {
           match self.batch_insert_materials(chunk).await {
               Ok(_) => processed += chunk.len(),
               Err(_) => failed += chunk.len(),
           }

           progress_callback(BatchProgress { total, processed, failed });
       }

       Ok(BatchResult { total, processed, failed })
   }
   ```

4. **批量操作的错误处理**:
   - 部分失败时继续处理
   - 返回详细的错误报告
   - 支持失败重试机制

### 第五阶段：运维和监控 (优先级：P2) - 待实现

#### 任务13: 添加监控指标
**状态**: ⏳ 待实现
**预计工作量**: 中等
**文件**: `src/observability/metrics.rs` (新建)

**实现指标**:

1. **请求指标**:
```rust
use prometheus::{Counter, Histogram, IntGauge, Registry};

pub struct Metrics {
    // 请求计数
    pub request_total: Counter,
    pub request_success: Counter,
    pub request_failed: Counter,

    // 请求延迟
    pub request_duration: Histogram,

    // 按方法分类的指标
    pub method_request_total: CounterVec,
    pub method_request_duration: HistogramVec,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Self {
        let request_total = Counter::new(
            "mdm_material_requests_total",
            "Total number of requests"
        ).unwrap();

        let request_duration = Histogram::with_opts(
            HistogramOpts::new(
                "mdm_material_request_duration_seconds",
                "Request duration in seconds"
            ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
        ).unwrap();

        let method_request_total = CounterVec::new(
            Opts::new("mdm_material_method_requests_total", "Requests by method"),
            &["method", "status"]
        ).unwrap();

        registry.register(Box::new(request_total.clone())).unwrap();
        registry.register(Box::new(request_duration.clone())).unwrap();
        registry.register(Box::new(method_request_total.clone())).unwrap();

        Self {
            request_total,
            request_success: Counter::new("mdm_material_requests_success", "Successful requests").unwrap(),
            request_failed: Counter::new("mdm_material_requests_failed", "Failed requests").unwrap(),
            request_duration,
            method_request_total,
            method_request_duration: HistogramVec::new(
                HistogramOpts::new("mdm_material_method_duration_seconds", "Duration by method"),
                &["method"]
            ).unwrap(),
        }
    }

    pub fn record_request(&self, method: &str, duration: f64, success: bool) {
        self.request_total.inc();
        self.request_duration.observe(duration);

        let status = if success { "success" } else { "error" };
        self.method_request_total.with_label_values(&[method, status]).inc();
        self.method_request_duration.with_label_values(&[method]).observe(duration);

        if success {
            self.request_success.inc();
        } else {
            self.request_failed.inc();
        }
    }
}
```

2. **数据库连接池指标**:
```rust
pub struct DatabaseMetrics {
    pub pool_connections_active: IntGauge,
    pub pool_connections_idle: IntGauge,
    pub pool_connections_max: IntGauge,
    pub query_duration: HistogramVec,
    pub query_errors: CounterVec,
}

impl DatabaseMetrics {
    pub fn update_pool_stats(&self, pool: &PgPool) {
        let size = pool.size();
        let idle = pool.num_idle();
        self.pool_connections_active.set((size - idle) as i64);
        self.pool_connections_idle.set(idle as i64);
    }
}
```

3. **缓存指标**:
```rust
pub struct CacheMetrics {
    pub cache_hits: Counter,
    pub cache_misses: Counter,
    pub cache_hit_rate: Gauge,
    pub cache_size: IntGauge,
    pub cache_evictions: Counter,
}

impl CacheMetrics {
    pub fn record_hit(&self) {
        self.cache_hits.inc();
        self.update_hit_rate();
    }

    pub fn record_miss(&self) {
        self.cache_misses.inc();
        self.update_hit_rate();
    }

    fn update_hit_rate(&self) {
        let hits = self.cache_hits.get();
        let misses = self.cache_misses.get();
        let total = hits + misses;
        if total > 0.0 {
            self.cache_hit_rate.set(hits / total);
        }
    }
}
```

4. **业务指标**:
```rust
pub struct BusinessMetrics {
    // 物料操作统计
    pub materials_created: Counter,
    pub materials_updated: Counter,
    pub materials_deleted: Counter,
    pub materials_activated: Counter,
    pub materials_deactivated: Counter,

    // 当前状态统计
    pub materials_total: IntGauge,
    pub materials_active: IntGauge,
    pub materials_inactive: IntGauge,
    pub materials_blocked: IntGauge,

    // 视图扩展统计
    pub plant_extensions: Counter,
    pub sales_extensions: Counter,
    pub purchase_extensions: Counter,
}
```

5. **指标导出端点**:
```rust
use axum::{routing::get, Router};
use prometheus::{Encoder, TextEncoder};

pub fn metrics_router(registry: Registry) -> Router {
    Router::new()
        .route("/metrics", get(move || async move {
            let encoder = TextEncoder::new();
            let metric_families = registry.gather();
            let mut buffer = vec![];
            encoder.encode(&metric_families, &mut buffer).unwrap();
            String::from_utf8(buffer).unwrap()
        }))
}
```

#### 任务14: 分布式追踪
**状态**: ⏳ 待实现
**预计工作量**: 中等
**集成**: OpenTelemetry

**实现内容**:

1. **添加依赖**:
```toml
[dependencies]
opentelemetry = "0.21"
opentelemetry-otlp = "0.14"
tracing-opentelemetry = "0.22"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

2. **初始化追踪**:
```rust
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing(service_name: &str, otlp_endpoint: &str) -> Result<()> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otlp_endpoint)
        )
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_resource(opentelemetry::sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", service_name.to_string()),
                ]))
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}
```

3. **追踪范围**:

   a. **gRPC请求追踪**:
   ```rust
   use tracing::{info_span, instrument};

   #[instrument(
       name = "grpc.create_material",
       skip(self, request),
       fields(
           tenant_id = %tenant_id,
           material_number = %req.material_number,
       )
   )]
   async fn create_material(
       &self,
       request: Request<CreateMaterialRequest>,
   ) -> Result<Response<CreateMaterialResponse>, Status> {
       // 实现...
   }
   ```

   b. **数据库查询追踪**:
   ```rust
   #[instrument(
       name = "db.query.find_material",
       skip(self),
       fields(
           db.system = "postgresql",
           db.operation = "SELECT",
           material_id = %id.0,
       )
   )]
   async fn find_by_id(&self, id: &MaterialId, tenant_id: &TenantId) -> AppResult<Option<Material>> {
       // 实现...
   }
   ```

   c. **缓存操作追踪**:
   ```rust
   #[instrument(
       name = "cache.get",
       skip(self),
       fields(
           cache.key = %key,
           cache.hit = tracing::field::Empty,
       )
   )]
   async fn get_material(&self, id: &MaterialId) -> AppResult<Option<Material>> {
       let result = self.redis.get(&key).await?;
       tracing::Span::current().record("cache.hit", result.is_some());
       Ok(result)
   }
   ```

   d. **外部服务调用追踪**:
   ```rust
   #[instrument(
       name = "http.client.request",
       skip(self),
       fields(
           http.method = "GET",
           http.url = %url,
           http.status_code = tracing::field::Empty,
       )
   )]
   async fn call_external_service(&self, url: &str) -> AppResult<Response> {
       let response = self.client.get(url).send().await?;
       tracing::Span::current().record("http.status_code", response.status().as_u16());
       Ok(response)
   }
   ```

4. **上下文传播**:
```rust
use opentelemetry::propagation::Extractor;

struct MetadataExtractor<'a>(&'a tonic::metadata::MetadataMap);

impl<'a> Extractor for MetadataExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

// 在 gRPC 拦截器中提取追踪上下文
let parent_cx = global::get_text_map_propagator(|propagator| {
    propagator.extract(&MetadataExtractor(request.metadata()))
});
```

#### 任务15: 健康检查
**状态**: ⏳ 待实现
**预计工作量**: 小
**文件**: `src/api/health.rs` (新建)

**实现内容**:

1. **健康检查接口**:
```rust
use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub checks: HealthChecks,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthChecks {
    pub database: ComponentHealth,
    pub cache: ComponentHealth,
    pub event_store: ComponentHealth,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: String,
    pub message: Option<String>,
    pub response_time_ms: Option<u64>,
}

pub struct HealthCheck {
    db_pool: PgPool,
    redis_pool: Option<RedisPool>,
}

impl HealthCheck {
    pub fn new(db_pool: PgPool, redis_pool: Option<RedisPool>) -> Self {
        Self { db_pool, redis_pool }
    }

    pub async fn check_database(&self) -> ComponentHealth {
        let start = std::time::Instant::now();

        match sqlx::query("SELECT 1").execute(&self.db_pool).await {
            Ok(_) => ComponentHealth {
                status: "healthy".to_string(),
                message: None,
                response_time_ms: Some(start.elapsed().as_millis() as u64),
            },
            Err(e) => ComponentHealth {
                status: "unhealthy".to_string(),
                message: Some(format!("Database error: {}", e)),
                response_time_ms: Some(start.elapsed().as_millis() as u64),
            },
        }
    }

    pub async fn check_cache(&self) -> ComponentHealth {
        let start = std::time::Instant::now();

        match &self.redis_pool {
            Some(redis) => {
                match redis.get::<_, Option<String>>("health_check").await {
                    Ok(_) => ComponentHealth {
                        status: "healthy".to_string(),
                        message: None,
                        response_time_ms: Some(start.elapsed().as_millis() as u64),
                    },
                    Err(e) => ComponentHealth {
                        status: "unhealthy".to_string(),
                        message: Some(format!("Cache error: {}", e)),
                        response_time_ms: Some(start.elapsed().as_millis() as u64),
                    },
                }
            }
            None => ComponentHealth {
                status: "disabled".to_string(),
                message: Some("Cache not configured".to_string()),
                response_time_ms: None,
            },
        }
    }

    pub async fn check_event_store(&self) -> ComponentHealth {
        let start = std::time::Instant::now();

        match sqlx::query("SELECT COUNT(*) FROM material_events LIMIT 1")
            .execute(&self.db_pool)
            .await
        {
            Ok(_) => ComponentHealth {
                status: "healthy".to_string(),
                message: None,
                response_time_ms: Some(start.elapsed().as_millis() as u64),
            },
            Err(e) => ComponentHealth {
                status: "unhealthy".to_string(),
                message: Some(format!("Event store error: {}", e)),
                response_time_ms: Some(start.elapsed().as_millis() as u64),
            },
        }
    }

    pub async fn check_overall(&self) -> HealthStatus {
        let database = self.check_database().await;
        let cache = self.check_cache().await;
        let event_store = self.check_event_store().await;

        let overall_status = if database.status == "healthy"
            && (cache.status == "healthy" || cache.status == "disabled")
            && event_store.status == "healthy"
        {
            "healthy"
        } else {
            "unhealthy"
        };

        HealthStatus {
            status: overall_status.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            checks: HealthChecks {
                database,
                cache,
                event_store,
            },
        }
    }
}
```

2. **健康检查端点**:
```rust
pub fn health_router(health_check: Arc<HealthCheck>) -> Router {
    Router::new()
        .route("/health", get({
            let health_check = health_check.clone();
            move || async move {
                let status = health_check.check_overall().await;
                let status_code = if status.status == "healthy" {
                    axum::http::StatusCode::OK
                } else {
                    axum::http::StatusCode::SERVICE_UNAVAILABLE
                };
                (status_code, Json(status))
            }
        }))
        .route("/health/live", get(|| async {
            // Liveness probe - 服务是否运行
            Json(serde_json::json!({ "status": "alive" }))
        }))
        .route("/health/ready", get({
            let health_check = health_check.clone();
            move || async move {
                // Readiness probe - 服务是否准备好接收流量
                let db_health = health_check.check_database().await;
                if db_health.status == "healthy" {
                    (axum::http::StatusCode::OK, Json(serde_json::json!({ "status": "ready" })))
                } else {
                    (
                        axum::http::StatusCode::SERVICE_UNAVAILABLE,
                        Json(serde_json::json!({ "status": "not ready", "reason": db_health.message }))
                    )
                }
            }
        }))
}
```

3. **Kubernetes 集成**:
```yaml
# deployment.yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: mdm-material
    image: mdm-material:latest
    livenessProbe:
      httpGet:
        path: /health/live
        port: 8080
      initialDelaySeconds: 10
      periodSeconds: 10
    readinessProbe:
      httpGet:
        path: /health/ready
        port: 8080
      initialDelaySeconds: 5
      periodSeconds: 5
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

### ✅ 里程碑1: 核心功能完整 - 已完成
- ✅ 完成所有视图数据操作
- ✅ 完成替代查询方法
- ✅ 完成替代物料和单位换算管理
- ✅ 完成变更历史查询
- ✅ 完成事件存储实现
- **交付物**: 功能完整的服务，支持所有33个RPC方法

**实际完成度**: 100%
**代码统计**: 11,813行代码，54个Rust文件

### ⏳ 里程碑2: 测试覆盖 (第1-2周)
- ⏳ 单元测试覆盖率 > 80%
- ⏳ 集成测试覆盖核心场景
- ⏳ 性能测试基准建立
- **交付物**: 高质量、可靠的代码库

**当前状态**: 未开始
**预计工作量**: 2周

### ⏳ 里程碑3: 性能优化 (第3-4周)
- ⏳ 查询优化完成
- ⏳ 缓存层实现
- ⏳ 批量操作优化
- **交付物**: 高性能服务

**当前状态**: 未开始
**预计工作量**: 2周

### ⏳ 里程碑4: 生产就绪 (第5-6周)
- ⏳ 监控和追踪就绪
- ⏳ 健康检查实现
- ⏳ 文档完整
- **交付物**: 可部署到生产环境的服务

**当前状态**: 未开始
**预计工作量**: 2周

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

MDM-Material模块的**核心功能已经100%完成**，远超之前文档记录的70%完成度。

### 🎉 已完成的工作

1. **完整的领域模型** (100%)
   - 3个核心实体，7个值对象，6个业务视图
   - 完整的领域事件支持

2. **完整的数据库架构** (100%)
   - 17张表，13个迁移文件
   - 48+个索引，完整的RLS策略
   - 事件表支持事件溯源

3. **完整的API实现** (100%)
   - 33个RPC方法全部实现
   - 包括变更历史、替代物料、单位换算等高级功能

4. **完整的应用层** (100%)
   - 47个业务方法
   - 完整的CQRS实现

5. **完整的基础设施层** (100%)
   - 所有视图数据操作
   - 替代查询方法
   - 事件存储实现

### 📋 待完成的工作

当前的主要任务集中在**非功能性需求**：

1. **测试** (优先级：P1)
   - 单元测试
   - 集成测试
   - 性能测试

2. **性能优化** (优先级：P2)
   - 查询优化
   - 缓存层
   - 批量操作优化

3. **运维监控** (优先级：P2)
   - 指标收集
   - 分布式追踪
   - 健康检查

4. **文档** (优先级：P3)
   - API文档
   - 运维手册
   - 性能基准

### 🚀 下一步行动

**立即可以开始的工作**：

1. **编写测试** (最高优先级)
   - 从领域实体的单元测试开始
   - 逐步添加集成测试
   - 目标：80%+代码覆盖率

2. **性能测试和优化**
   - 建立性能基准
   - 识别瓶颈
   - 实施优化

3. **添加监控**
   - 集成 Prometheus 指标
   - 添加 OpenTelemetry 追踪
   - 实现健康检查端点

### 📊 项目状态总结

| 模块 | 完成度 | 代码行数 | 状态 |
|------|--------|----------|------|
| 领域层 | 100% | 4,193 | ✅ 完成 |
| 应用层 | 100% | 1,361 | ✅ 完成 |
| API层 | 100% | 1,524 | ✅ 完成 |
| 基础设施层 | 100% | 2,488 | ✅ 完成 |
| 事件存储 | 100% | 306 | ✅ 完成 |
| 数据库迁移 | 100% | 13个文件 | ✅ 完成 |
| 测试 | 0% | 0 | ⏳ 待实现 |
| 性能优化 | 0% | 0 | ⏳ 待实现 |
| 监控 | 0% | 0 | ⏳ 待实现 |

**总体完成度**: 核心功能 100%，整体项目约 60%（考虑测试和运维）

### 🎯 建议的开发顺序

按照优先级，建议按以下顺序进行开发：

1. **第1-2周**: 测试基础设施
   - 建立测试框架
   - 编写单元测试
   - 编写集成测试

2. **第3-4周**: 性能优化
   - 查询优化
   - 缓存层实现
   - 批量操作优化

3. **第5-6周**: 运维和监控
   - 监控指标
   - 分布式追踪
   - 健康检查
   - 文档完善

**预计6周后可以达到生产就绪状态。**
