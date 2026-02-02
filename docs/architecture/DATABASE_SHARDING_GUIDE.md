# Cuba ERP 微服务数据库架构设计指南

## 问题背景

项目采用微服务架构，规划有 **46 个服务**。当前数据库配置：
- 单 PostgreSQL 实例
- 支持读写分离
- 多租户架构（TenantId 隔离）

**核心问题：** 四十多个服务是否需要对应的单个分库/分区？

---

## 一、微服务数据库架构策略对比

### 1.1 策略概览

```
┌─────────────────────────────────────────────────────────────┐
│              微服务数据库架构策略金字塔                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│          Level 4: 完全独立数据库                            │
│          (One Database Per Service)                         │
│          每个服务拥有独立的数据库实例                        │
│          适合：大型企业、高度隔离需求                       │
│                                                             │
│          Level 3: 每服务独立 Schema                         │
│          (One Schema Per Service)                          │
│          同一数据库实例，不同 Schema                        │
│          适合：中等规模团队，资源共享                       │
│                                                             │
│          Level 2: 多租户 Schema 分区 ⭐ 推荐               │
│          (Tenant Schema Partitioning)                      │
│          单数据库，租户隔离在 Schema 层                    │
│          适合：SaaS、当前架构                               │
│                                                             │
│          Level 1: 共享数据库表分区                          │
│          (Shared Database + Table Partitioning)            │
│          单数据库，租户通过 tenant_id + 分区隔离          │
│          适合：小规模团队，快速开发                         │
│                                                             │
│          Level 0: 单数据库单表                             │
│          (Single Database Single Table)                    │
│          所有服务共享单数据库所有表                        │
│          适合：MVP、原型验证                               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 二、四种主要架构方案

### 方案 A: 单数据库 + Schema 隔离 ⭐ 当前推荐

#### 架构图

```
                    PostgreSQL Instance
                    ┌─────────────────────────────────┐
                    │                                 │
    ┌───────────────┼─────────────┐                  │
    │               │             │                  │
┌───▼────┐    ┌────▼────┐   ┌───▼─────┐           ┌──▼──┐
│ Public │    │ iam_id_ │   │mdm_mat_ │   ...    │tenant│
│ Schema │    │ entity  │   │ service │           │_0000 │
└────────┘    └─────────┘   └─────────┘           └──────┘
    │               │               │                 │
    └───────────────┴───────────────┴─────────────────┘
                单数据库，多 Schema

每个租户独立 Schema:
- tenant_0001 (客户A)
- tenant_0002 (客户B)
- tenant_0003 (客户C)
...
```

#### 表结构示例

```sql
-- iam-access 租户 0001
CREATE SCHEMA IF NOT EXISTS tenant_0001;
SET search_path TO tenant_0001, public;

CREATE TABLE tenant_0001.roles (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- mdm-material 租户 0001
CREATE TABLE tenant_0001.materials (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    material_number VARCHAR(40) NOT NULL,
    name VARCHAR(200) NOT NULL,
    -- ...
);

-- iam-access 租户 0002
CREATE SCHEMA IF NOT EXISTS tenant_0002;
CREATE TABLE tenant_0002.roles (
    -- 相同结构
);
```

#### Rust 代码示例

```rust
impl PostgresMaterialRepository {
    // 动态选择 Schema
    fn schema_for_tenant(tenant_id: &TenantId) -> String {
        format!("tenant_{:04}", tenant_id.id())
    }

    async fn find_by_id(
        &self,
        id: &MaterialId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<Material>> {
        let schema = Self::schema_for_tenant(tenant_id);
        let query = format!(
            r#"SELECT id, tenant_id, material_number, ...
               FROM {}.materials
               WHERE id = $1 AND tenant_id = $2"#,
            schema
        );

        let result = sqlx::query_as::<_, Material>(&query)
            .bind(id)
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result)
    }
}
```

#### 优缺点

| 优点 | 缺点 |
|------|------|
| ✅ 租户数据完全隔离 | ❌ Schema 数量多（租户数 × 服务数） |
| ✅ 可以独立备份/恢复 | ❌ Schema 切换需要代码支持 |
| ✅ 可以按租户优化索引 | ❌ 跨租户查询困难 |
| ✅ 符合当前多租户架构 | ❌ Schema 管理复杂 |
| ✅ 迁移成本相对低 | - |

#### 适用场景

- SaaS 产品（多租户）
- 租户数量 < 500
- 单租户数据量 < 100GB
- 需要租户级别隔离

---

### 方案 B: 单数据库 + 表分区

#### 架构图

```
                    PostgreSQL Instance
                    ┌─────────────────────────────────┐
                    │         materials 表              │
                    │  ┌────────────────────────────────┐│
                    │  │  Partitions by tenant_id       ││
                    │  │  ┌────────┬────────┬──────────┐││
                    │  │  │tenant_1│tenant_2│tenant_3 │││
                    │  │  │partition│partition│partition│││
                    │  │  └────────┴────────┴──────────┘││
                    │  └────────────────────────────────┘│
                    └─────────────────────────────────┘
                    ┌─────────────────────────────────┐
                    │         roles 表                 │
                    │  ┌────────────────────────────────┐│
                    │  │  Partitions by tenant_id       ││
                    │  └────────────────────────────────┘│
                    └─────────────────────────────────┘
```

#### 表结构示例

```sql
-- 创建分区表
CREATE TABLE materials (
    id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    material_number VARCHAR(40) NOT NULL,
    name VARCHAR(200),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (id, tenant_id)
) PARTITION BY LIST (tenant_id);

-- 为每个租户创建分区
CREATE TABLE materials_tenant_0001
    PARTITION OF materials
    FOR VALUES IN ('00000000-0000-0001-0000-000000000001');

CREATE TABLE materials_tenant_0002
    PARTITION OF materials
    FOR VALUES IN ('00000000-0000-0002-0000-000000000002');

-- 按租户索引（可选）
CREATE INDEX idx_materials_tenant_0001
    ON materials (material_number)
    WHERE tenant_id = '00000000-0000-0001-0000-000000000001';
```

#### Rust 代码示例

```rust
impl PostgresMaterialRepository {
    async fn find_by_id(
        &self,
        id: &MaterialId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<Material>> {
        let query = r#"
            SELECT id, tenant_id, material_number, name
            FROM materials
            WHERE id = $1 AND tenant_id = $2
        "#;

        let result = sqlx::query_as::<_, Material>(query)
            .bind(id)
            .bind(tenant_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(result)
    }
}
```

#### 优缺点

| 优点 | 缺点 |
|------|------|
| ✅ 查询简单，无需切换 Schema | ❌ 所有租户数据在同一表 |
| ✅ 跨租户查询（仅管理员） | ❌ 需要手动管理分区 |
| ✅ 备份操作简单 | ❌ 分区裁剪优化有限 |
| ✅ 代码改动小 | ❌ 单分区超限性能下降 |

#### 适用场景

- 租户数据量适中（< 50GB/租户）
- 租户数量 < 100
- 需要跨租户分析
- 快速迭代

---

### 方案 C: 一库多 Schema（服务隔离）

#### 架构图

```
                    PostgreSQL Instance
                    ┌─────────────────────────────────┐
                    │                                 │
    ┌────────────────┼────────────────┐              │
    │                │                │              │
┌───▼──────┐   ┌────▼──────┐  ┌─────▼──────┐       │
│ iam-     │   │ mdm-      │  │ scm-       │       │
│ access   │   │ material  │  │ warehouse  │       │
│ schema   │   │ schema    │  │ schema     │       │
│          │   │           │  │            │       │
│ +roles   │   │ +materials│  │ +inventory │       │
│ +perms   │   │ +groups   │  │ +movement  │       │
│ +users   │   │ +types    │  │ +bin       │       │
└──────────┘   └───────────┘  └────────────┘       │
                    └─────────────────────────────────┘
```

#### 表结构示例

```sql
-- iam-access 服务专用 Schema
CREATE SCHEMA iam_access;

CREATE TABLE iam_access.roles (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL
);

-- mdm-material 服务专用 Schema
CREATE SCHEMA mdm_material;

CREATE TABLE mdm_material.materials (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    material_number VARCHAR(40) NOT NULL
);
```

#### 优缺点

| 优点 | 缺点 |
|------|------|
| ✅ 服务边界清晰 | ❌ 跨服务查询需要 JOIN |
| ✅ 独立迁移 | ❌ 数据库连接数增加 |
| ✅ 可以独立优化索引 | ❌ 事务跨 Service 困难 |
| ✅ 符合 DDD 原则 | ❌ 租户隔离较弱 |

#### 适用场景

- 微服务边界清晰
- 跨服务事务少
- 每个服务数据量 < 50GB
- 中等规模团队

---

### 方案 D: 每服务独立数据库实例

#### 架构图

```
    ┌─────────────────┐
    │ PostgreSQL 1    │
    │ iam-access     │
    │ iam-identity   │
    └─────────────────┘
           │
    ┌─────────────────┐
    │ PostgreSQL 2    │
    │ mdm-material    │
    │ mdm-bp          │
    └─────────────────┘
           │
    ┌─────────────────┐
    │ PostgreSQL 3    │
    │ scm-warehouse   │
    │ pm-purchase     │
    └─────────────────┘
           │
          ...
```

#### 优缺点

| 优点 | 缺点 |
|------|------|
| ✅ 完全隔离 | ❌ 运维成本高 |
| ✅ 独立扩缩容 | ❌ 跨服务查询困难 |
| ✅ 故障隔离最好 | ❌ 数据一致性复杂 |
| ✅ 符合微服务最佳实践 | ❌ 大量数据库实例 |

#### 适用场景

- 大型团队（> 50 人）
- 高可用要求
- 服务数据量大（> 100GB）
- 有专门 DBA

---

## 三、针对 Cuba ERP 的推荐方案

### 3.1 当前项目特点分析

```
项目规模：
  - 46 个微服务规划（当前 4 个有代码）
  - 多租户 SaaS 架构
  - DDD + CQRS + Event Sourcing

技术栈：
  - PostgreSQL (主数据)
  - ClickHouse (分析数据)
  - Redis (缓存)
  - Kafka (事件)

当前架构：
  - 单 PostgreSQL 实例
  - 读写分离支持
  - TenantId 在所有实体中
```

### 3.2 建议实施方案：混合方案

```
┌─────────────────────────────────────────────────────────────┐
│                   Cuba ERP 混合数据库架构                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  第一阶段：单库多 Schema (当前 ~ 6 个月)                    │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━       │
│                                                             │
│    PostgreSQL Instance                                      │
│    ┌─────────────────────────────────┐                    │
│    │                                 │                    │
│    │  ┌──────────┬──────────┬──────┐│                    │
│    │  │ Schema:  │ Schema:  │ ...  ││                    │
│    │  │ iam      │ mdm      │ service schemas          ││
│    │  │          │          │      ││                    │
│    │  │ +roles   │ +materials│     ││                    │
│    │  │ +perms   │ +groups   │     ││                    │
│    │  │ (按      │ (按      │     ││                    │
│    │  │ tenant_  │ tenant_  │     ││                    │
│    │  │ id 索引) │ id 索引) │     ││                    │
│    │  └──────────┴──────────┴──────┘│                    │
│    └─────────────────────────────────┘                    │
│                                                             │
│  策略：                                                     │
│    - 单数据库，按服务 Schema 隔离                          │
│    - 所有表保留 tenant_id 字段                             │
│    - 通过 tenant_id + 索引实现性能隔离                     │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  第二阶段：读写分离 + Redis (6-12 个月)                     │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━       │
│                                                             │
│    Primary DB        ──复制──>      Read Replicas          │
│    (写操作)                         (读操作)              │
│        │                              │                   │
│        v                              v                   │
│   [Pool: 50]                    [Pool: 200]               │
│                                                             │
│  策略：                                                     │
│    - 启用 PostgreSQL 流复制                               │
│    - 读写分离：写 -> Primary，读 -> Replicas               │
│    - Redis 缓存热点数据                                   │
│    - ReadWritePool 管理                                    │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  第三阶段：数据库分片 (12-24 个月，视规模而定)            │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━       │
│                                                             │
│    按 Tenant 分片:                                          │
│                                                             │
│    Shard 1: Tenant 001-010    Shard 2: Tenant 011-020      │
│    ┌─────────────────┐     ┌─────────────────┐           │
│    │ PostgreSQL      │     │ PostgreSQL      │           │
│    │ (Primary + RW)  │     │ (Primary + RW)  │           │
│    └─────────────────┘     └─────────────────┘           │
│                                                             │
│  策略：                                                     │
│    - 按租户 ID 范围分片                                    │
│    - 每个 Shard 独立主从                                   │
│    - 使用 Citus 或(pg_shardman))                          │
│    - 应用层路由：TenantID -> Shard                         │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  第四阶段：服务独立数据库 (24+ 个月，按需)                 │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━       │
│                                                             │
│    核心服务独立：                                          │
│                                                             │
│    ┌──────────────┐ ┌──────────────┐ ┌──────────────┐     │
│    │ iam-*        │ │ mdm-*        │ │ scm-*        │     │
│    │ 独立 PG      │ │ 独立 PG      │ │ 独立 PG      │     │
│    └──────────────┘ └──────────────┘ └──────────────┘     │
│                                                             │
│    轻量服务共享：                                          │
│                                                             │
│    ┌──────────────────────────────────────────┐            │
│    │  Shared DB: hr-*, fi-*, pm-*            │            │
│    └──────────────────────────────────────────┘            │
│                                                             │
│  策略：                                                     │
│    - 高频/核心服务独立 DB                                  │
│    - 低频/轻量服务共享 DB                                  │
│    - 通过 Event Sourcing 保证最终一致性                   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 3.3 具体实施建议

#### 第一阶段：当前实施（立即执行）

**数据库配置：**
```yaml
# docker-compose.yml 或 .env
DATABASE_URL=postgres://postgres:password@localhost:5432/erp
DATABASE_MAX_CONNECTIONS=100
```

**Schema 策略：**
```sql
-- 公共 Schema（共享表，如租户配置）
CREATE SCHEMA public;

-- IAM 服务 Schema
CREATE SCHEMA iam_access;
CREATE SCHEMA iam_identity;

-- MDM 服务 Schema
CREATE SCHEMA mdm_material;
CREATE SCHEMA mdm_bp;

-- SCM 服务 Schema（预留）
CREATE SCHEMA scm_warehouse;
CREATE SCHEMA scm_inventory;

-- ... 其他服务 Schema
```

**表的 tenant_id 索引：**
```sql
CREATE TABLE mdm_material.materials (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    material_number VARCHAR(40) NOT NULL,
    name VARCHAR(200),
    created_at TIMESTAMPTZ DEFAULT NOW(),

    -- 多个关键索引
    CONSTRAINT uk_material_tenant_number
        UNIQUE (tenant_id, material_number)
);

-- 租户 ID 作为所有查询的第一个条件
CREATE INDEX idx_materials_tenant_id
    ON mdm_material.materials (tenant_id);

CREATE INDEX idx_materials_tenant_number
    ON mdm_material.materials (tenant_id, material_number);

CREATE INDEX idx_materials_tenant_created
    ON mdm_material.materials (tenant_id, created_at);
```

**Rust 仓储层面：**
```rust
impl PostgresMaterialRepository {
    async fn find_by_id(
        &self,
        id: &MaterialId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<Material>> {
        // SQLx 会自动使用 tenant_id 索引
        let result = sqlx::query_as::<_, Material>(r#"
            SELECT id, tenant_id, material_number, name, ...
            FROM mdm_material.materials
            WHERE id = $1 AND tenant_id = $2
        "#)
        .bind(id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }
}
```

#### 第二阶段：读写分离（3-6 个月后）

**配置示例：**
```yaml
# .env (开发环境可能不需要，生产环境推荐)
DATABASE_URL=postgres://primary:password@primary-db:5432/erp
DATABASE_READ_URL=postgres://replica:password@replica-db:5432/erp
DATABASE_MAX_CONNECTIONS=50
DATABASE_READ_MAX_CONNECTIONS=100
```

**Bootstrap 已支持：** （已实现）
```rust
// bootstrap/src/infrastructure.rs
let rw_pool = ReadWritePool::new(write_pool, read_pool);

// Repository 使用读库
impl PostgresMaterialRepository {
    async fn find_by_id(&self, ...) -> AppResult<Option<Material>> {
        // 读操作使用 read_pool
        self.rw_pool.read_pool()
            .map(|pool| pool.fetch_one(...))
    }

    async fn save(&self, ...) -> AppResult<()> {
        // 写操作使用 write_pool
        self.rw_pool.write_pool().execute(...)
    }
}
```

#### 第三阶段：数据库分片（12-24 个月后视规模而定）

**分片策略选择：**

| 策略 | 描述 | 优势 | 劣势 | 适用场景 |
|------|------|------|------|----------|
| **租户范围分片** | Tenant ID 范围 | 简单，可预知扩容 | 可能数据倾斜 | 租户数 < 1000 |
| **租户 Hash 分片** | Tenant ID Hash | 数据均匀分布 | Reshard 复杂 | 租户数 > 1000 |
| **功能分片** | 业务模块 | 符合业务边界 | 跨分片 Join 困难 | 核心服务独立 |

**推荐：租户范围分片 + Citus**

```sql
-- 使用 Citus 创建分布式表
SELECT create_distributed_table('mdm_material.materials', 'tenant_id');

-- Citus 自动按 tenant_id 分片到不同节点
-- Shard 1: Tenant 000001-000100
-- Shard 2: Tenant 000101-000200
-- ...
```

**应用层路由：**
```rust
struct ShardRouter {
    shards: Vec<ShardInfo>,
}

impl ShardRouter {
    fn route(&self, tenant_id: &TenantId) -> &ShardInfo {
        // 根据 tenant_id 选择正确的数据库连接池
        let shard_index = (tenant_id.id() % self.shards.len() as u64) as usize;
        &self.shards[shard_index]
    }
}

impl PostgresMaterialRepository {
    async fn find_by_id(
        &self,
        id: &MaterialId,
        tenant_id: &TenantId,
    ) -> AppResult<Option<Material>> {
        let shard = self.router.route(tenant_id);
        let pool = shard.pool();

        // 查询时自动路由到正确的分片
        sqlx::query_as::<_, Material>("...")
            .bind(id)
            .bind(tenant_id)
            .fetch_optional(pool)
            .await
    }
}
```

#### 第四阶段：服务独立数据库（可选，24+ 个月后）

**决定是否分离的判断标准：**

```
决策树：
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

服务是否需要独立数据库？

 YES, 如果满足以下条件之一：

  □ 单服务数据量 > 100GB
  □ 单服务 QPS > 10000
  □ 服务需要独立升级或降级
  □ 服务需要数据库级别的故障隔离
  □ 服务有不同的备份/恢复 SLA

 示例：
  - iam-access (高频认证，需要独立优化)
  - mdm-material (大量物料数据)
  - scm-warehouse (实时库存更新)

 NO, 否则共享数据库：

  □ 单服务数据量 < 50GB
  □ 单服务 QPS < 1000
  □ 服务间有 join 需求
  □ 简化运维

 示例：
  - hr-* (低频查询)
  - sys-* (轻量服务)
  - fi-coa (科目表小)
```

---

## 四、性能优化建议

### 4.1 索引策略

```sql
-- 每个表都有标准的租户索引
CREATE INDEX ON table_name (tenant_id);

-- 常用查询的组合索引
CREATE INDEX ON table_name (tenant_id, created_at);
CREATE INDEX ON table_name (tenant_id, status);

-- 唯一约束保护租户数据
ALTER TABLE table_name
    ADD CONSTRAINT uk_tenant_business_key
    UNIQUE (tenant_id, business_key);

-- 部分索引（针对常用条件）
CREATE INDEX ON table_name (tenant_id, status)
    WHERE status IN ('ACTIVE', 'PENDING');
```

### 4.2 查询优化

```rust
// ✅ 正确：tenant_id 作为第一个条件
sqlx::query_as!(Material, r#"
    SELECT * FROM mdm_material.materials
    WHERE tenant_id = $1 AND material_number = $2
"#, tenant_id, material_number)

// ❌ 错误：未使用 tenant_id
sqlx::query_as!(Material, r#"
    SELECT * FROM mdm_material.materials
    WHERE material_number = $1
"#, material_number)  // 会导致全表扫描
```

### 4.3 连接池配置

```yaml
# 生产环境推荐
DATABASE_MAX_CONNECTIONS=50  # 每个服务
DATABASE_MIN_CONNECTIONS=10
DATABASE_READ_MAX_CONNECTIONS=100  # 从库

# 连接池公式： connections = (core_count * 2) + effective_spindle_count
# PostgreSQL 通常是：核心数 * 2
```

---

## 五、监控与维护

### 5.1 监控指标

```promql
# Prometheus 查询示例

# 数据库连接数
pg_stat_database_numbackends{datname="erp"}

# 租户级别的查询延迟
histogram_quantile(0.99,
  rate(db_query_duration_seconds_bucket{tenant_id=$tenant}[5m])
)

# 表大小监控
pg_stat_user_tables{relname="materials"}
SELECT pg_size_pretty(pg_total_relation_size('mdm_material.materials'));

# 索引使用率
SELECT schemaname, relname,
       idx_scan, idx_tup_read,
       pg_size_pretty(pg_relation_size(indexrelid)) AS index_size
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;
```

### 5.2 维护任务

```bash
# 定期 VACUUM 和 ANALYZE
0 2 * * * psql -U postgres -d erp -c "VACUUM ANALYZE;"

# 租户数据归档（保留最近 3 个月）
0 3 1 * * psql -U postgres -d erp -c "
  DELETE FROM mdm_material.materials
  WHERE tenant_id = $1 AND created_at < NOW() - INTERVAL '3 months';
"

# 重建膨胀的索引
SELECT pg_reindex('mdm_material.idx_materials_tenant_id');
```

---

## 六、成本与风险分析

### 6.1 成本对比

| 方案 | 数据库实例数 | 存储成本 | 运维成本 | 年成本预估 |
|------|-------------|---------|---------|-----------|
| 单数据库 | 1 | $100/月 | 低 | $1,200 |
| 一库多 Schema | 1 | $100/月 | 低 | $1,200 |
| 读写分离 | 1+2 | $300/月 | 中 | $3,600 |
| 分片 (3 节点) | 3 | $300/月 | 高 | $3,600 |
| 全独立 (46 个 DB) | 46 | $4,600/月 | 极高 | $55,200 |
| **混合推荐** | 3-5 | $300-500/月 | 中 | $3,600-6,000 |

### 6.2 风险控制

```
技术风险：
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

风险 1: 单点故障
  缓解：PostgreSQL 流复制 + 自动故障转移 (Patroni)

风险 2: 数据倾斜
  缓解：监控租户数据量，主动 re-balance

风险 3: 迁移困难
  缓减：提前规划 schema，使用 ORM (sqlx)

风险 4: 性能瓶颈
  缓减：读写分离、Redis 缓存、连接池优化

风险 5: 事务一致性
  缓减：Saga 模式 + Event Sourcing
```

---

## 七、总结与行动计划

### 7.1 核心建议

```
短期 (0-6 个月):
  ✅ 使用方案 A: 单数据库 + Schema 隔离
  ✅ 所有表保留 tenant_id
  ✅ 建立 tenant_id 索引
  ✅ 使用ReadWritePool 准备读写分离

中期 (6-18 个月):
  ✅ 启用读写分离（生产环境）
  ✅ Redis 缓存热点数据
  ✅ 监控租户数据增长
  ✅ 规划分片策略

长期 (18-24 个月+):
  ⚠️ 根据实际情况决定是否分片
  ⚠️ 考虑核心服务独立数据库
  ⚠️ 使用 Citus 或 pg_shardman
```

### 7.2 快速决策树

```
当前状态评估：
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Q1: 当前有多少租户？
  < 50   → 方案 A（单库 Schema）
  50-200 → 方案 A + 监控
  > 200 → 考虑方案 B（表分区）

Q2: 单租户数据量？
  < 10GB → 方案 A 足够
  10-50GB → 方案 A + 分区
  > 50GB → 考虑分片

Q3: 预期 QPS？
  < 1000 → 方案 A 为主
  1000-5000 → 方案 A + 读写分离
  > 5000 → 分片
```

### 7.3 推荐实施路径

```
Phase 1 (当前): 单库多 Schema
  ──────────────────────────────
  ├── 创建 iam, mdm, scm 等 schemas
  ├── 所有表添加 tenant_id 字段
  ├── 建立标准的 tenant_id 索引
  └── 编写 Schema 迁移脚本

Phase 2 (3-6 个月): 读写分离
  ──────────────────────────────
  ├── 配置 PostgreSQL 流复制
  ├── 更新 .env 添加 read_url
  ├── 利用现有 ReadWritePool
  └── 优化读库连接池配置

Phase 3 (12+ 个月): 评估分片
  ──────────────────────────────
  ├── 分析租户数据分布
  ├── 评估跨库查询需求
  ├── 制定分片方案
  └── 按需实施
```

---

**结论：**

对于 Cuba ERP 项目，**不需要**一开始就为每个服务创建独立数据库。推荐采用**渐进式演进策略**：

1. **短期:** 单数据库 + Schema 隔离（服务级）
2. **中期:** 读写分离 + Redis 缓存
3. **长期:** 根据实际负载决定是否分片或独立

这种策略：
- ✅ 降低初期成本和复杂度
- ✅ 保留未来扩展的灵活性
- ✅ 符合当前架构设计
- ✅ 平衡性能与成本

**关键原则：先简化，后优化；先隔离，后扩展。**

---

**文档版本:** 1.0
**创建日期:** 2026-02-01
**适用项目:** Cuba ERP
**维护团队:** Infrastructure Team
