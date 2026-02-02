# Cuba ERP 生产级数据库架构一步到位方案

## 架构目标

- ✅ **一步到位**：生产级配置，支持未来扩展
- ✅ **多租户分层**：支持普通 / VIP / 企业级差异化管理
- ✅ **水平扩展**：支持 1000+ 租户，无限扩展
- ✅ **高可用**：99.99% SLA，自动故障转移
- ✅ **VIP 隔离**：VIP 租户独立资源池
- ✅ **智能路由**：自动按租户等级路由
- ✅ **成本优化**：根据租户等级分配资源

---

## 一、整体架构设计

### 1.1 三层租户架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Cuba ERP 生产级数据库架构                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                  应用层 (Rust 微服务)                       │   │
│  │  ┌─────────────────────────────────────────────────────┐   │   │
│  │  │         Tenant Router Service (租户路由服务)          │   │   │
│  │  │  - 租户等级识别                                       │   │   │
│  │  │  - 动态路由至对应资源池                               │   │   │
│  │  │  - 连接池管理                                         │   │   │
│  │  └─────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                        │                          │
│                                        ▼                          │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                  租户等级分层路由                           │   │   │
│  │  ┌──────────┬──────────┬──────────────────────────────┐    │   │
│  │  │ Enterprise│   VIP    │  Standard (Shared)          │    │   │
│  │  │  Tier     │  Tier    │  Tier                       │    │   │
│  │  │ (独享)    │ (半独享) │                             │    │   │
│  │  │           │          │                             │    │   │
│  │  │ 专用资源  │ 专用资源  │ 共享资源池                  │    │   │
│  │  └────┬──────┴────┬─────┴──────────┬───────────────────┘    │   │
│  └───────┼────────────┼───────────────┼──────────────────────┘   │
│          │            │               │                         │
│          ▼            ▼               ▼                         │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐      │
│  │ Enterprise   │ │   VIP Pool   │ │   Shared Pool         │      │
│  │ Cluster      │ │  (N+M Pool)  │ │   (Massive Pool)    │      │
│  │              │ │              │ │                      │      │
│  │  PG-Shard-1  │ │ PG-VIP-1     │ │ PG-Shard-1           │      │
│  │  PG-Shard-2  │ │ PG-VIP-2     │ │ PG-Shard-2           │      │
│  │  ...         │ │ ...          │ │ ...                  │      │
│  │              │ │              │ │                      │      │
│  │ 专用读库     ││ 专用读库     ││ 共享读库组            │      │
│  └──────────────┘ └──────────────┘ └──────────────────────┘      │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                辅助基础设施                                │   │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │   │
│  │  │ Redis   │  │  Kafka  │  │ClickHouse│ │Consul   │       │   │
│  │  │ Cache   │  │ Events  │  │ Analytics│ │Service  │       │   │
│  │  │ (分层)  │  │         │  │ (分层)   │ │Discovery│       │   │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘       │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.2 租户等级定义

```
租户等级分级策略
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Enterprise Tier (企业级)
  ──────────────────────────────────────────────────────
  目标客户：
    □ 大型企业（>5000 员工）
    □ 政府机构
    □ 银行/金融
    □ 跨国公司

  SLA 承诺：
    □ 99.99% Uptime (~43分钟/年宕机)
    □ 数据持久性 99.999999%
    □ 恢复时间目标 RTO < 5 分钟
    □ 恢复点目标 RPO < 1 分钟

  资源隔离：
    □ 独立 PostgreSQL 集群（3+ 节点）
    □ 独立 Redis 集群
    □ 独立 Kafka 实例（或独立 Topic 分区）
    □ 独立 ClickHouse 集群
    □ 专用网络隔离
    □ 专用监控和告警

  性能保证：
    □ 专用 CPU 核心（如 32 核）
    □ 专用内存（如 256GB）
    □ 专用存储 SSD (NVMe)
    □ 无共享连接池
    □ 无性能竞争

  安全性：
    □ VPC 隔离
    □ 静态数据加密（TDE）
    □ 传输加密（TLS 1.3）
    □ 专用审计日志
    □ 定期安全扫描
    □ 访问控制 (IAM/RBAC)

  定价：
    □ 专属定价（$数万/月起）
    □ 包含专属支持（7x24）
    □ 包含专属客户经理


VIP Tier (VIP 级)
  ──────────────────────────────────────────────────────
  目标客户：
    □ 中大型企业（500-5000 员工）
    □ 高增长公司
    □ 电商/金融科技

  SLA 承诺：
    □ 99.95% Uptime (~4 小时/年宕机)
    □ 数据持久性 99.99999%
    □ RTO < 15 分钟
    □ RPO < 5 分钟

  资源隔离：
    □ 半独立 PostgreSQL 资源池（VIP Pool）
    □ 共享 Redis 但 VIP 专用分片
    □ 共享 Kafka 但 VIP 高优先级
    □ 分析数据隔离（专用 ClickHouse 分片）

  性能优化：
    □ VIP 专用连接池（无竞争）
    □ CPU 份额保障（最低 20%）
    □ 内存保障（最低 64GB）
    □ 查询优先级提升
    □ 索引优化（专职 DBA）

  安全性：
    □ 租户级数据隔离
    □ VPC 网段隔离
    □ 加密存储
    □ 高级审计

  定价：
    □ $5000-20000/月
    □ 包含优先支持（5x8）
    □ 包含架构评审


Standard Tier (标准级)
  ──────────────────────────────────────────────────────
  目标客户：
    □ 中小企业（<500 员工）
    □ 初创公司
    □ 个人开发者

  SLA 承诺：
    □ 99.9% Uptime (~8.7 小时/年宕机)
    □ 数据持久性 99.999%
    □ RTO < 60 分钟
    □ RPO < 15 分钟

  资源共享：
    □ 共享 PostgreSQL 集群（大规模分片）
    □ 共享 Redis 集群（多租户）
    □ 共享 Kafka 集群
    □ 共享 ClickHouse 集群

  性能：
    □ 共享连接池
    □ 按需分配资源
    □ 突发处理能力
    □ 自动扩缩容阈值

  安全性：
    □ tenant_id 逻辑隔离
    □ 标准 TLS 加密
    □ 基础审计

  定价：
    □ $100-500/月
    □ 社区支持
    □ 在线文档


Free Tier (免费级，可选)
  ──────────────────────────────────────────────────────
  目标：产品试用、开发者测试
  限制：
    □ 10 用户
    □ 100MB 数据
    □ 1000 API 调用/天
    □ 无 SLA 保证
```

---

## 二、数据库架构详细设计

### 2.1 Enterprise Tier：专属集群

```
┌─────────────────────────────────────────────────────────────┐
│              Enterprise 集群架构（每租户）                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  租户: enterprise_001 (某大型银行)                          │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │            PostgreSQL Patroni 集群                    │ │
│  │                                                       │ │
│  │   Primary (Leader)        Read Replicas              │ │
│  │   ┌────────────┐      ┌─────┐  ┌─────┐  ┌─────┐     │ │
│  │   │ PG-Master  │◄────┤ R1  │  │ R2  │  │ R3  │     │ │
│  │   │            │      │     │  │     │  │     │     │ │
│  │   │ 16 vCPU    │      │8vCPU│  │8vCPU│  │8vCPU│     │ │
│  │   │ 128GB RAM  │      │64GB │  │64GB │  │64GB │     │ │
│  │   │ 2TB SSD    │      │2TB  │  │2TB  │  │2TB  │     │ │
│  │   └────────────┘      └─────┘  └─────┘  └─────┘     │ │
│  │        │                │                          │   │ │
│  │        │                └─ 只读操作 (读写分离)       │   │ │
│  │        └──── 读写操作                               │   │ │
│  │                                                       │ │
│  │   自动故障转移 (Patroni + etcd)                      │ │
│  │   自动备份 (WAL 归档 + pgBackRest)                   │ │
│  │   只读延迟复制 (流复制)                               │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │            Redis Sentinel 集群                        │ │
│  │   ┌──────────┐  ┌──────────┐  ┌──────────┐          │ │
│  │   │ Redis-M  │  │ Redis-S1 │  │ Redis-S2 │          │ │
│  │   │ 8 vCPU   │  │ 4 vCPU   │  │ 4 vCPU   │          │ │
│  │   │ 64GB RAM │  │ 32GB RAM │  │ 32GB RAM │          │ │
│  │   └──────────┘  └──────────┘  └──────────┘          │ │
│  │                                                       │ │
│  │   高可用 + 持久化 (RDB + AOF)                         │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │            ClickHouse 集群                            │ │
│  │   ┌──────────────┬──────────────┬──────────────┐      │ │
│  │   │ CH-Replica-1 │ CH-Replica-2 │ CH-Replica-3 │      │ │
│  │   │   8 vCPU     │   8 vCPU     │   8 vCPU     │      │ │
│  │   │   64GB       │   64GB       │   64GB       │      │ │
│  │   │   10TB       │   10TB       │   10TB       │      │ │
│  │   └──────────────┴──────────────┴──────────────┘      │ │
│  │                                                       │ │
│  │   分布式表引擎 (ReplicatedMergeTree)                  │ │
│  │   ZooKeeper 协调                                      │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │            Kafka 集群（可选）                          │ │
│  │   ┌──────────┐  ┌──────────┐  ┌──────────┐          │ │
│  │   │ Kafka-1  │  │ Kafka-2  │  │ Kafka-3  │          │ │
│  │   └──────────┘  └──────────┘  └──────────┘          │ │
│  │                                                       │ │
│  │   专用 Topic 分区                                     │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘

配置示例：
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Enterprise 租户配置
[tenant.enterprise_001]
tier = "enterprise"

[tenant.enterprise_001.postgresql]
cluster_type = "dedicated"
primary.host = "pg-ent-001-master.internal"
read_replicas = [
    "pg-ent-001-replica-1.internal",
    "pg-ent-001-replica-2.internal",
    "pg-ent-001-replica-3.internal",
]
pool_size = 200
read_pool_size = 400

[tenant.enterprise_001.redis]
cluster_type = "dedicated"
hosts = [
    "redis-ent-001-master.internal",
    "redis-ent-001-slave-1.internal",
    "redis-ent-001-slave-2.internal",
]
max_memory = "64gb"

[tenant.enterprise_001.clickhouse]
cluster_type = "dedicated"
hosts = [
    "ch-ent-001-1.internal",
    "ch-ent-001-2.internal",
    "ch-ent-001-3.internal",
]
```

### 2.2 VIP Tier：VIP 资源池

```
┌─────────────────────────────────────────────────────────────┐
│                  VIP Pool 集群架构（共享专用）               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  VIP 租户: vip_001, vip_002, ..., vip_100                  │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │         PostgreSQL VIP Pool (Citus 集群)             │ │
│  │                                                       │ │
│  │  Coordinator Nodes (路由节点)                          │ │
│  │  ┌─────────┐  ┌─────────┐                             │ │
│  │  │ Coord 1 │  │ Coord 2 │                             │ │
│  │  │ 8 vCPU  │  │ 8 vCPU  │                             │ │
│  │  └────┬────┘  └────┬────┘                             │ │
│  │       │            │                                  │ │
│  │       └────────────┘                                  │ │
│  │               │                                       │ │
│  │       ┌───────▼────────┐                             │ │
│  │       │  负载均衡器      │                             │ │
│  │       └────────────────┘                             │ │
│  │               │                                       │ │
│  │  Worker Nodes (数据节点，按租户分片)                  │ │
│  │  ┌───────────────────────────────────────────────┐   │ │
│  │  │ Shard 1: VIP 001-025            │   │
│  │  │ ┌──────────┐ ┌────────────────────────────┐   │   │ │
│  │  │ │ VIP-001  │ │ 专属连接池 (200 connections)│   │   │ │
│  │  │ │ VIP-002  │ │ 专属 CPU 配额 (30%)           │   │   │ │
│  │  │ │ ...      │ │ 专属内存 (128GB)             │   │   │ │
│  │  │ └──────────┘ └────────────────────────────┘   │   │ │
│  │  │ Primary   ┌─► Read Replicas (x3)             │   │ │
│  │  └───────────────────────────────────────────────┘   │ │
│  │                                                       │ │
│  │  ┌───────────────────────────────────────────────┐   │ │
│  │  │ Shard 2: VIP 026-050                            │   │ │
│  │  │ ... (相同结构)                                   │   │ │
│  │  └───────────────────────────────────────────────┘   │ │
│  │                                                       │ │
│  │  ┌───────────────────────────────────────────────┐   │ │
│  │  │ Shard 3: VIP 051-075                            │   │ │
│  │  └───────────────────────────────────────────────┘   │ │
│  │                                                       │ │
│  │  ┌───────────────────────────────────────────────┐   │ │
│  │  │ Shard 4: VIP 076-100                            │   │ │
│  │  └───────────────────────────────────────────────┘   │ │
│  │                                                       │ │
│  │   Citus: 按租户 ID 范围分片                          │ │
│  │   自动路由: tenant_id -> shard                      │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │            Redis VIP Pool（集群模式）                 │ │
│  │                                                       │ │
│  │  ┌─────────┬─────────┬─────────┬─────────┐          │ │
│  │  │ Master  │ Slave 1 │ Slave 2 │ Slave 3 │          │ │
│  │  └────┬────┴────┬────┴────┬────┴────┬────┘          │ │
│  │       │         │         │         │               │ │
│  │  ┌────▼─────┬───▼───┬────▼────┬─────▼────┐         │ │
│  │  │Slot 001  │Slot...│Slot 8192│       │         │ │
│  │  │VIP-001   │       │VIP-100  │  ...   │         │ │
│  │  │(专属分片)│(共享) │(专属分片)│  ...   │         │ │
│  │  └──────────┴────────┴──────────┴────────┘         │ │
│  │                                                       │ │
│  │   VIP 租户使用专用分片，避免竞争                     │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘

配置示例：
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# VIP 租户配置
[tier.vip]
pool_type = "shared_dedicated"
max_tenants = 100
default_shards = 4
auto_shard = true

[tenant.vip_001]
tier = "vip"
shard_id = 1

[tenant.vip_001.postgresql]
coordinator_hosts = ["pg-vip-coord-1", "pg-vip-coord-2"]
worker_shard = 1
pool_size = 200
cpu_quota = "30%"
memory_quota = "128gb"

[tenant.vip_001.redis]
cluster_hosts = [
    "redis-vip-master.internal",
    "redis-vip-slave-1.internal",
    "redis-vip-slave-2.internal",
    "redis-vip-slave-3.internal",
]
dedicated_slots = 100
max_memory = "8gb"
```

### 2.3 Standard Tier：大规模共享集群

```
┌─────────────────────────────────────────────────────────────┐
│              Standard Pool 集群架构（大规模共享）             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Standard 租户: std_001, std_002, ..., std_10000+          │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │     PostgreSQL Massive Scale Cluster (Citus)          │ │
│  │                                                       │ │
│  │  Coordinator Cluster (HA)                             │ │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐               │ │
│  │  │ Coord 1 │  │ Coord 2 │  │ Coord 3 │               │ │
│  │  │ 16 vCPU │  │ 16 vCPU │  │ 16 vCPU │               │ │
│  │  └────┬────┘  └────┬────┘  └────┬────┘               │ │
│  │       └────────────┼─────────────┘                   │ │
│  │                    │                                  │ │
│  │         ┌──────────▼──────────┐                      │ │
│  │         │   负载均衡器        │                      │ │
│  │         │  (HAProxy/Envoy)   │                      │ │
│  │         └──────────┬──────────┘                      │ │
│  │                    │                                  │ │
│  │  Worker Cluster (Sharded & Distributed)              │ │
│  │                                                       │ │
│  │  ┌────────────┬────────────┬────────────┐             │ │
│  │  │ Shard 1-10 │ Shard 11-20│ Shard ...  │  ...       │ │
│  │  │           │            │            │             │ │
│  │  │ 1000 租户 │ 1000 租户   │ ...        │             │ │
│  │  │ std_001-  │ std_1001-  │            │             │ │
│  │  │ std_1000  │ std_2000   │            │             │ │
│  │  │           │            │            │             │ │
│  │  │ 32 vCPU   │ 32 vCPU    │ ...        │             │ │
│  │  │ 256GB RAM │ 256GB RAM  │ ...        │             │ │
│  │  │ 10TB SSD  │ 10TB SSD   │ ...        │             │ │
│  │  └───────────┴────────────┴────────────┘             │ │
│  │                                                       │ │
│  │  ┌───────────────────────────────────────────────┐   │ │
│  │  │  Shared Connection Pool                       │   │ │
│  │  │  ┌─────┬─────┬─────┬─────┬─────┬─────┐      │   │ │
│  │  │  │  C1 │  C2 │  C3 │  C4 │  C5 │ ... │      │   │ │
│  │  │  └─────┴─────┴─────┴─────┴─────┴─────┘      │   │ │
│  │  │  Total: 10,000 connections                  │   │ │
│  │  │  Per-Tenant: 动态分配 (1-10)                 │   │ │
│  │  └───────────────────────────────────────────────┘   │ │
│  │                                                       │ │
│  │  自动分片 (Sharding):                                  │ │
│  │  - 按 tenant_id Hash 分片                             │ │
│  │  - 自动 Rebalance (迁移时间段)                         │ │
│  │  - 水平扩展：动态添加 Shard                            │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │     Redis Cluster (分布式，多主)                      │ │
│  │                                                       │ │
│  │  ┌─────────┬─────────┬─────────┬─────────┬─────┐   │ │
│  │  │ Node 1  │ Node 2  │ Node 3  │ Node 4  │ ... │   │ │
│  │  │ 8192    │ 8192    │ 8192    │ 8192    │     │   │ │
│  │  │ Slots   │ Slots   │ Slots   │ Slots   │ ... │   │ │
│  │  │ std_*   │ std_*   │ std_*   │ std_*   │     │   │ │
│  │  └─────────┴─────────┴─────────┴─────────┴─────┘   │ │
│  │                                                       │ │
│  │  Hash Slot 分片:                                      │ │
│  │  - CRC16(key) % 16384                                │ │
│  │  - 按租户 ID 前缀分片: {tenant_id}:{key}           │ │
│  │  - 自动迁移和故障转移                                 │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │     ClickHouse 集群（分析数据）                        │ │
│  │                                                       │ │
│  │  ┌─────────────┬─────────────┬─────────────┐         │ │
│  │  │ Replicated  │ Replicated  │ Replicated  │   ...   │ │
│  │  │ Shard 1     │ Shard 2     │ Shard 3     │         │ │
│  │  └─────────────┴─────────────┴─────────────┘         │ │
│  │  (10TB per shard, 3 replicas)                          │ │
│  │                                                       │ │
│  │  按租户 ID 分片的分析表：                             │ │
│  │  - 标准表：普通查询                                   │ │
│  │  - 分布式表：跨 Shard 查询                            │ │
│  │  - 物化视图：预聚合                                   │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
└─────────────────────────────────────────────────────────────┘

配置示例：
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Standard Tier 配置
[tier.standard]
pool_type = "shared"
max_tenants = 10000
auto_scale = true
min_shards = 10
max_shards = 50

[tenant.std_12345]
tier = "standard"
expected_data_size = "50gb"
expected_qps = 100

# 动态调优
[tenant.std_12345.auto_scaling]
scale_up_qps_threshold = 500
scale_down_qps_threshold = 50
scale_up_data_threshold = "100gb"
rebalance_window = "02:00-06:00"  # 迁移时间段
```

---

## 三、租户路由架构

### 3.1 Tenant Router Service

```rust
//! 租户路由服务
//!
//! 负责根据租户等级和 ID 路由到正确的数据库资源池

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::PgPool;

/// 租户等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TenantTier {
    Enterprise = 3,
    Vip = 2,
    Standard = 1,
    Free = 0,
}

impl TenantTier {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "enterprise" => Some(Self::Enterprise),
            "vip" => Some(Self::Vip),
            "standard" => Some(Self::Standard),
            "free" => Some(Self::Free),
            _ => None,
        }
    }
}

/// 租户配置
#[derive(Debug, Clone)]
pub struct TenantConfig {
    pub tenant_id: String,
    pub tier: TenantTier,
    pub cluster_endpoint: ClusterEndpoint,
    pub pool_config: PoolConfig,
    pub limits: TenantLimits,
}

/// 集群端点配置
#[derive(Debug, Clone)]
pub enum ClusterEndpoint {
    /// 专属集群（Enterprise）
    Dedicated {
        primary: String,
        replicas: Vec<String>,
    },
    /// VIP 资源池（VIP）
    VipPool {
        coordinators: Vec<String>,
        shard_id: u32,
    },
    /// 共享池（Standard）
    SharedPool {
        coordinators: Vec<String>,
    },
}

/// 连接池配置
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub cpu_quota: Option<f64>,  // CPU 份额百分比
    pub memory_quota: Option<String>,  // 如 "128gb"
}

/// 租户资源限制
#[derive(Debug, Clone)]
pub struct TenantLimits {
    pub max_qps: u64,
    pub max_storage_gb: u64,
    pub max_connections: u32,
    pub query_timeout_ms: u64,
}

/// 租户路由器
pub struct TenantRouter {
    /// 租户配置缓存
    tenants: Arc<RwLock<HashMap<String, TenantConfig>>>,
    /// Enterprise 专属连接池（每个租户独立）
    enterprise_pools: Arc<RwLock<HashMap<String, PgPool>>>,
    /// VIP 共享连接池（按 Shard 分组）
    vip_pools_by_shard: Arc<RwLock<HashMap<u32, PgPool>>>,
    /// Standard 共享连接池（全局共享）
    standard_pool: PgPool,
    /// Redis 连接（按 Tier 分组）
    redis_pools: HashMap<TenantTier, RedisPool>,
}

impl TenantRouter {
    /// 创建租户路由器
    pub async fn new(
        standard_pool: PgPool,
        redis_standard: RedisPool,
        redis_vip: RedisPool,
    ) -> Self {
        Self {
            tenants: Arc::new(RwLock::new(HashMap::new())),
            enterprise_pools: Arc::new(RwLock::new(HashMap::new())),
            vip_pools_by_shard: Arc::new(RwLock::new(HashMap::new())),
            standard_pool,
            redis_pools: {
                let mut pools = HashMap::new();
                pools.insert(TenantTier::Standard, redis_standard);
                pools.insert(TenantTier::Vip, redis_vip);
                pools
            },
        }
    }

    /// 加载租户配置（从数据库或配置中心）
    pub async fn load_tenant_config(&self, tenant_id: &str) -> AppResult<TenantConfig> {
        // 1. 从 Redis 读取缓存
        if let Some(cached) = self.get_cached_config(tenant_id).await? {
            return Ok(cached);
        }

        // 2. 从数据库加载
        let config = self.load_config_from_db(tenant_id).await?;

        // 3. 缓存到 Redis（TTL 1 小时）
        self.cache_config(tenant_id, &config).await?;

        // 4. 初始化连接池（如需要）
        self.init_connection_pool(&config).await?;

        Ok(config)
    }

    /// 获取数据库连接池
    pub async fn get_postgres_pool(
        &self,
        tenant_id: &str,
    ) -> AppResult<PgPool> {
        let config = self.load_tenant_config(tenant_id).await?;

        match config.cluster_endpoint {
            ClusterEndpoint::Dedicated { primary, .. } => {
                // Enterprise: 返回专属连接池
                let pools = self.enterprise_pools.read().await;
                pools.get(tenant_id)
                    .cloned()
                    .ok_or_else(|| AppError::internal(format!("Enterprise pool not found for {}", tenant_id)))
            }
            ClusterEndpoint::VipPool { shard_id, .. } => {
                // VIP: 返回该 Shard 的共享连接池
                let pools = self.vip_pools_by_shard.read().await;
                pools.get(&shard_id)
                    .cloned()
                    .ok_or_else(|| AppError::internal(format!("VIP pool not found for shard {}", shard_id)))
            }
            ClusterEndpoint::SharedPool { .. } => {
                // Standard: 返回全局共享连接池
                Ok(self.standard_pool.clone())
            }
        }
    }

    /// 获取 Redis 连接池
    pub async fn get_redis_pool(
        &self,
        tenant_id: &str,
    ) -> AppResult<RedisPool> {
        let config = self.load_tenant_config(tenant_id).await?;
        self.redis_pools.get(&config.tier)
            .cloned()
            .ok_or_else(|| AppError::internal("Redis pool not found for tier"))
    }

    /// 动态路由查询
    pub async fn route_query<F, R>(
        &self,
        tenant_id: &str,
        query: F,
    ) -> AppResult<R>
    where
        F: FnOnce(PgPool) -> futures::future::BoxFuture<'static, AppResult<R>>,
    {
        let pool = self.get_postgres_pool(tenant_id).await?;
        query(pool).await
    }

    /// 初始化连接池
    async fn init_connection_pool(&self, config: &TenantConfig) -> AppResult<()> {
        match &config.cluster_endpoint {
            ClusterEndpoint::Dedicated { primary, .. } => {
                // 初始化 Enterprise 专属连接池
                let pool = create_pool(&PostgresConfig::new(primary)
                    .with_pool(config.pool_config.min_connections, config.pool_config.max_connections)
                ).await?;

                let mut pools = self.enterprise_pools.write().await;
                pools.insert(config.tenant_id.clone(), pool);
            }
            ClusterEndpoint::VipPool { shard_id, .. } => {
                // 初始化 VIP Shard 连接池（如果不存在）
                let mut pools = self.vip_pools_by_shard.write().await;
                if !pools.contains_key(shard_id) {
                    let pool = create_pool(&PostgresConfig::new(&format!("pg-vip-shard-{}", shard_id))
                        .with_pool(100, 400)  // VIP 默认配置
                    ).await?;
                    pools.insert(*shard_id, pool);
                }
            }
            ClusterEndpoint::SharedPool { .. } => {
                // Standard 使用全局连接池，无需初始化
            }
        }
        Ok(())
    }

    /// 从数据库加载配置
    async fn load_config_from_db(&self, tenant_id: &str) -> AppResult<TenantConfig> {
        sqlx::query_as::<_, TenantConfigRecord>(
            r#"
            SELECT tenant_id, tier, cluster_endpoint, pool_config, limits
            FROM tenant_registry
            WHERE tenant_id = $1 AND active = true
            "#
        )
        .bind(tenant_id)
        .fetch_one(&self.standard_pool)  // 配置存在标准库
        .await
        .map_err(|e| AppError::database(format!("Load tenant config failed: {}", e)))?;

        // 转换为 Domain 对象...
        todo!("Implement conversion")
    }
}

/// 租户仓储实现示例
pub struct PostgresMaterialRepository {
    router: Arc<TenantRouter>,
}

impl PostgresMaterialRepository {
    pub fn new(router: Arc<TenantRouter>) -> Self {
        Self { router }
    }

    async fn find_by_id(
        &self,
        tenant_id: &str,
        material_id: &MaterialId,
    ) -> AppResult<Option<Material>> {
        self.router.route_query(tenant_id, |pool| {
            let tenant_id = tenant_id.to_string();
            let material_id = material_id.clone();

            Box::pin(async move {
                sqlx::query_as::<_, Material>(r#"
                    SELECT id, tenant_id, material_number, name, ...
                    FROM mdm_material.materials
                    WHERE id = $1 AND tenant_id = $2
                "#)
                .bind(&material_id)
                .bind(&tenant_id)
                .fetch_optional(&pool)
                .await
                .map_err(|e| AppError::database(e.to_string()))
            })
        }).await
    }
}
```

### 3.2 租户配置表

```sql
-- 租户注册表（存在于标准库）
CREATE TABLE tenant_registry (
    tenant_id UUID PRIMARY KEY,
    tenant_code VARCHAR(50) UNIQUE NOT NULL,
    tenant_name VARCHAR(200) NOT NULL,
    tier VARCHAR(20) NOT NULL CHECK (tier IN ('enterprise', 'vip', 'standard', 'free')),

    -- 集群配置
    cluster_endpoint_type VARCHAR(20) NOT NULL,  -- 'dedicated', 'vip_pool', 'shared_pool'
    primary_endpoint VARCHAR(255),              -- Enterprise 主库地址
    replica_endpoints TEXT[],                    -- Enterprise 从库地址列表
    shard_id INTEGER,                            -- VIP/Standard Shard ID
    coordinator_endpoints TEXT[],                -- Citus Coordinator 地址

    -- 连接池配置
    max_connections INTEGER DEFAULT 10,
    min_connections INTEGER DEFAULT 1,
    cpu_quota_percentage FLOAT,                  -- CPU 份额（20 表示 20%）
    memory_quota_gb FLOAT,                       -- 内存配额（GB）

    -- 资源限制
    max_qps INTEGER,
    max_storage_gb INTEGER,
    query_timeout_ms INTEGER DEFAULT 30000,

    -- 状态
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,

    -- 审计
    created_by VARCHAR(100),
    updated_by VARCHAR(100),
);

-- 索引
CREATE INDEX idx_tenant_registry_active ON tenant_registry(active);
CREATE INDEX idx_tenant_registry_tier ON tenant_registry(tier);
CREATE INDEX idx_tenant_registry_shard ON tenant_registry(shard_id) WHERE shard_id IS NOT NULL;

-- 租户使用情况统计
CREATE TABLE tenant_metrics_daily (
    tenant_id UUID NOT NULL,
    metric_date DATE NOT NULL,

    -- 数据库指标
    db_query_count BIGINT,
    db_avg_latency_ms FLOAT,
    db_errors BIGINT,

    -- 存储指标
    storage_used_gb FLOAT,
    storage_growth_gb FLOAT,

    -- QPS 指标
    api_qps BIGINT,
    api_errors BIGINT,

    PRIMARY KEY (tenant_id, metric_date),
    FOREIGN KEY (tenant_id) REFERENCES tenant_registry(tenant_id)
);

CREATE INDEX idx_tenant_metrics_date ON tenant_metrics_daily(metric_date DESC);
```

---

## 四、自动扩缩容架构

### 4.1 水平扩容策略

```
┌─────────────────────────────────────────────────────────────┐
│                  自动扩缩容架构                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  监控层                                                     │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Prometheus + Grafana + Alertmanager                 │   │
│  │  - 实时监控租户 QPS、延迟、存储增长                  │   │
│  │  - 按租户等级设置不同的阈值                          │   │
│  └────────────┬────────────────────────────────────────┘   │
│               │                                            │
│               ▼                                            │
│  决策层                                                     │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Autoscaler Service (自动扩缩容决策)                 │   │
│  │                                                       │ │
│  │  决策规则：                                            │ │
│  │  ┌─────────────────────────────────────────────┐   │   │
│  │  │ Enterprise: 超阈即扩，无延迟                   │   │   │
│  │  │                                             │   │   │
│  │  │ VIP: 超阈 5 分钟内决定是否扩容                │   │   │
│  │  │                                             │   │   │
│  │  │ Standard: 批量扩容（低谷时段），合并处理       │   │   │
│  │  │                                             │   │   │
│  │  │ Free: 不自动扩容，需审核                       │   │   │
│  │  └─────────────────────────────────────────────┘   │   │
│  └────────────┬────────────────────────────────────────┘   │
│               │                                            │
│               ▼                                            │
│  执行层                                                     │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Orchestrator Service (协调器 - Kubernetes/Ansible) │ │
│  │                                                       │ │
│  │  扩容任务：                                            │ │
│  │  1. 创建新的 PostgreSQL Shard                        │ │
│  │  2. 数据迁移（在线迁移，零停机）                     │ │
│  │  3. 更新 Citus Coordinator 元数据                     │ │
│  │  4. 验证迁移完成                                     │ │
│  │  5. 更新租户路由配置                                 │ │
│  │  6. 缩容清理旧 Shard                                 │ │
│  └────────────┬────────────────────────────────────────┘   │
│               │                                            │
│               ▼                                            │
│  基础设施层                                                 │
│  ┌───────────────────┬───────────────────┬───────┐       │
│  │  Cloud Provider   │   Kubernetes     │ Ansible│       │
│  │  (AWS/Azure/GCP)  │   (K8s)          │       │       │
│  │                   │                   │       │       │
│  │  - RDS/GCP Cloud  │   - Pods         │ - VMs  │       │
│  │  - EBS disks      │   - PVCs         │       │       │
│  │                   │   - Services     │       │       │
│  └───────────────────┴───────────────────┴───────┘       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 扩容决策算法

```rust
//! 自动扩缩容决策引擎

use std::time::Duration;

/// 扩容决策
#[derive(Debug, Clone)]
pub struct ScalingDecision {
    pub should_scale: bool,
    pub direction: ScalingDirection,
    pub reason: String,
    pub target_shards: Option<u32>,
    pub estimated_cost_change: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum ScalingDirection {
    Up,    // 扩容：增加 Shard
    Down,  // 缩容：减少 Shard
}

/// 扩缩容决策引擎
pub struct AutoscalerEngine {
    metrics_store: MetricsStore,
    tenant_registry: TenantRegistry,
}

impl AutoscalerEngine {
    /// 评估单个租户是否需要扩缩容
    pub async fn evaluate_tenant(
        &self,
        tenant_id: &str,
    ) -> AppResult<ScalingDecision> {
        let tier = self.tenant_registry.get_tier(tenant_id).await?.tier;
        let metrics = self.metrics_store.get_metrics(tenant_id).await?;

        match tier {
            TenantTier::Enterprise => {
                // Enterprise: 立即响应
                self.evaluate_immediate(tenant_id, &metrics).await
            }
            TenantTier::Vip => {
                // VIP: 5 分钟内决定
                self.evaluate_delayed(tenant_id, &metrics, Duration::from_secs(300)).await
            }
            TenantTier::Standard => {
                // Standard: 批量处理，低谷时段扩容
                self.evaluate_batch(tenant_id, &metrics).await
            }
            TenantTier::Free => {
                // Free: 不自动扩容
                Ok(ScalingDecision {
                    should_scale: false,
                    direction: ScalingDirection::Up,
                    reason: "Free tier requires manual approval".to_string(),
                    target_shards: None,
                    estimated_cost_change: 0.0,
                })
            }
        }
    }

    /// Enterprise: 立即扩容评估
    async fn evaluate_immediate(
        &self,
        tenant_id: &str,
        metrics: &TenantMetrics,
    ) -> AppResult<ScalingDecision> {
        let config = self.tenant_registry.get_config(tenant_id).await?;

        // 检查 QPS 阈值
        if metrics.avg_qps > config.limits.max_qps * 2 {
            return Ok(ScalingDecision {
                should_scale: true,
                direction: ScalingDirection::Up,
                reason: format!(
                    "QPS {:.0} exceeds threshold {} by 2x",
                    metrics.avg_qps,
                    config.limits.max_qps
                ),
                target_shards: Some(config.current_shards + 1),
                estimated_cost_change: 2_000.0,  // 约 $2000/月
            });
        }

        // 检查存储
        if metrics.storage_gb > config.limits.max_storage_gb * 0.9 {
            return Ok(ScalingDecision {
                should_scale: true,
                direction: ScalingDirection::Up,
                reason: format!(
                    "Storage {:.1} GB at 90% of limit {} GB",
                    metrics.storage_gb,
                    config.limits.max_storage_gb
                ),
                target_shards: Some(config.current_shards + 1),
                estimated_cost_change: 2_000.0,
            });
        }

        // 检查延迟
        if metrics.p99_latency_ms > 5000.0 {
            return Ok(ScalingDecision {
                should_scale: true,
                direction: ScalingDirection::Up,
                reason: format!(
                    "P99 latency {:.0} ms exceeds threshold 5000 ms",
                    metrics.p99_latency_ms
                ),
                target_shards: Some(config.current_shards + 1),
                estimated_cost_change: 2_000.0,
            });
        }

        Ok(ScalingDecision {
            should_scale: false,
            direction: ScalingDirection::Up,
            reason: "All metrics within limits".to_string(),
            target_shards: None,
            estimated_cost_change: 0.0,
        })
    }

    /// VIP: 延迟扩容评估
    async fn evaluate_delayed(
        &self,
        tenant_id: &str,
        metrics: &TenantMetrics,
        window: Duration,
    ) -> AppResult<ScalingDecision> {
        // 获取过去时间窗口的指标
        let historical = self.metrics_store
            .get_historical_metrics(tenant_id, window)
            .await?;

        // 持续超阈才扩容
        let sustained_breach = historical.iter()
            .all(|m| m.avg_qps > m.max_qps * 1.5);

        if sustained_breach {
            Ok(ScalingDecision {
                should_scale: true,
                direction: ScalingDirection::Up,
                reason: format!(
                    "Sustained QPS breach over {:?}", window
                ),
                target_shards: Some(4),  // VIP 总是扩到 4 个 Shard
                estimated_cost_change: 1_000.0,
            })
        } else {
            Ok(ScalingDecision {
                should_scale: false,
                direction: ScalingDirection::Up,
                reason: "No sustained breach".to_string(),
                target_shards: None,
                estimated_cost_change: 0.0,
            })
        }
    }

    /// Standard: 批量扩容评估
    async fn evaluate_batch(
        &self,
        tenant_id: &str,
        metrics: &TenantMetrics,
    ) -> AppResult<ScalingDecision> {
        // 只在低谷时段执行（凌晨 2-6 点）
        let now = Utc::now();
        let hour = now.hour();

        if !(2..=6).contains(&hour) {
            return Ok(ScalingDecision {
                should_scale: false,
                direction: ScalingDirection::Up,
                reason: "Not in scaling window (02:00-06:00 UTC)".to_string(),
                target_shards: None,
                estimated_cost_change: 0.0,
            });
        }

        // 收集所有需要扩容的租户
        let all_tenants = self.tenant_registry.list_tenants_by_tier(TenantTier::Standard).await?;
        let decisions = futures::future::join_all(
            all_tenants.iter()
                .map(|tid| self.evaluate_standard_tenant(tid))
        ).await;

        // 批量执行扩容（合并操作）
        let scale_up_count = decisions.iter().filter(|d| d.should_scale).count();

        if scale_up_count > 0 {
            Ok(ScalingDecision {
                should_scale: true,
                direction: ScalingDirection::Up,
                reason: format!(
                    "Batch scaling: {} tenants need scaling",
                    scale_up_count
                ),
                target_shards: Some(12),  // 批量新增 2 个 Shard（基于当前 10）
                estimated_cost_change: scale_up_count as f64 * 100.0,
            })
        } else {
            Ok(ScalingDecision {
                should_scale: false,
                direction: ScalingDirection::Up,
                reason: "No tenants need scaling in batch".to_string(),
                target_shards: None,
                estimated_cost_change: 0.0,
            })
        }
    }
}
```

---

## 五、完整实施方案

### 5.1 第一阶段：基础设施搭建（1-2 个月）

```
Week 1-2: 核心数据库部署
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ PostgreSQL 集群
  ├─ Citus Coordinator Cluster (3 节点，HA)
  ├─ Standard Shard Cluster (10+ 节点，可扩展)
  ├─ VIP Shard Cluster (4 节点，预留 100 VIP)
  └─ Enterprise 集群（按需创建）

✅ Redis 集群
  ├─ Standard Pool Cluster (分布式，6 节点)
  ├─ VIP Pool Cluster (独立 3 节点)
  └─ Enterprise 集群（按需创建）

✅ ClickHouse 集群
  ├─ 分析集群（Sharded，3 节点）
  ├─ 标准数据存储
  └─ VIP 数据独立分片

✅ Kafka 集群
  ├─ 跨 Tier 事件流
  ├─ 分区规划
  └─ 压缩策略

✅ 监控与告警
  ├─ Prometheus + Grafana
  ├─ 各层次监控
  └─ 租户级别指标


Week 3-4: 租户路由服务
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Tenant Router Service
  ├─ 按租户 ID 查询配置
  ├─ 动态路由到对应资源池
  ├─ 连接池管理（按 Tier）
  └─ 故障转移（自动切换）

✅ 租户管理 API
  ├─ 租户注册/升级/降级
  ├─ 租户配置查询
  ├─ 资源配额管理
  └─ 计费集成

✅ 配置中心（Consul/etcd）
  ├─ 租户配置存储
  ├─ 动态热加载
  └─ 缓存更新
```

### 5.2 第二阶段：服务集成（1-2 个月）

```
Week 5-6: 现有服务迁移
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ iam-access 租改
  ├─ TenantRouter 集成
  ├─ 代码改动最小化
  ├─ 测试验证
  └─ 灰度发布

✅ iam-identity 租改

✅ mdm-material 租改

✅ 其他已有服务租改


Week 7-8: 新服务按标准集成
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ 新服务模板
  ├─ TenantRouter 默认集成
  ├─ 租户 ID 传递链路
  ├─ Schema 定义标准
  └─ 测试套件

✅ 开发者指南
  ├─ 使用 TenantRouter
  ├─ 租户隔离最佳实践
  └─ 性能优化建议
```

### 5.3 第三阶段：自动化与优化（1-2 个月）

```
Week 9-10: 自动扩缩容
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Autoscaler Service
  ├─ 监控集成
  ├─ 决策引擎
  ├─ 执行器（K8s Operator）
  └─ 回滚机制

✅ 数据迁移自动化
  ├─ Shard Rebalance
  ├─ 在线迁移
  ├─ 零停机
  └─ 数据验证


Week 11-12: 性能优化与测试
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ 性能调优
  ├─ 连接池优化
  ├─ 查询优化
  ├─ 索引优化
  └─ Redis 缓存优化

✅ 压力测试
  ├─ Enterprise 层测试
  ├─ VIP 层测试
  ├─ Standard 层测试
  └─ 混合负载测试

✅ 灾难恢复演练
  ├─ 故障注入
  ├─ 自动切换
  ├─ 数据恢复
  └─ SLA 验证
```

---

## 六、成本优化策略

### 6.1 分层计费

```
┌─────────────────────────────────────────────────────────────┐
│                  租户计费策略                                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Enterprise 层                                              │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━        │
│  定价模式：                                                │
│  □ 固定月费 + 资源按需                                  │
│  □ 基础费用：$20,000/月                                   │
│  □ 包含核心资源（32 核，256GB，2TB）                       │
│  □ 超额按量计费：                                          │
│    - CPU: $10/核/月                                      │
│    - 内存: $1/GB/月                                      │
│    - 存储: $0.2/GB/月                                    │
│    - 连接数: $0.01/连接/月                               │
│                                                             │
│  示例计算：                                                │
│    Base:      $20,000                                     │
│    +4 vCPU:    $40                                        │
│    +128GB:    $128                                        │
│    +1TB:      $200                                        │
│    ──────────────────                                  │
│    Total:     $20,368/月                                 │
│                                                             │
│  VIP 层                                                     │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━        │
│  定价模式：                                                │
│  □ 套餐 + 突发计费                                    │
│  □ 套餐级别：                                             │
│    ┌─────────┬──────────┬──────────┬─────────────┐      │
│    │  套餐   │    QPS   │  数据量   │   月费      │      │
│    ├─────────┼──────────┼──────────┼─────────────┤      │
│    │   V1    │   500    │  100GB   │  $5,000     │      │
│    │   V2    │  2,000   │  500GB   │  $10,000    │      │
│    │   V3    │  5,000   │   1TB    │  $15,000    │      │
│    └─────────┴──────────┴──────────┴─────────────┘      │
│  □ 突发计费：超套餐 20% 内免费，超出按量                 │
│  □ 自动扩容：超套餐自动升级套餐                           │
│                                                             │
│  Standard 层                                              │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━        │
│  定价模式：                                                │
│  □ 按需计费（Pay-as-you-go）                              │
│  □ 单价：                                                │
│    - QPS:     $0.01/1000 QPS                            │
│    - 存储:    $0.5/GB/月                                │
│    - 连接:    $0.01/连接/月                             │
│                                                             │
│  示例计算（中等负载）：                                    │
│    QPS:      100 x 24h x 30d = 260M                      │
│    存储:    50GB                                         │
│    连接:    20                                           │
│    ───────────────────────────────────────────          │
│    月费:    $2,600 + $25 + $0.2 = $2,625/月            │
│                                                             │
│  容量预留折扣：                                            │
│  □ 预留 1 年：七折                                      │
│  □ 预留 3 年：五折                                      │
│                                                             │
│  Free 层（可选）                                           │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━        │
│  定价：免费                                                │
│  限制：                                                  │
│  □ 10 用户                                               │
│  □ 100MB 数据                                            │
│  □ 1,000 API 调用/天                                    │
│  □ 无 SLA 保证                                           │
│  目标：产品试用、开发者教育                               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 成本优化技术

```
┌─────────────────────────────────────────────────────────────┐
│                  成本优化技术                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. 数据压缩                                                │
│  ▲ TOAST 压缩                                              │
│    ALTER TABLE materials ALTER COLUMN description              │
│      SET STORAGE EXTERNAL;  -- 压缩文本字段                  │
│                                                             │
│  ▲ 时间序列压缩 (ClickHouse)                                │
│    SETTINGS compression_codec = 'ZSTD';                     │
│                                                             │
│  2. 冷热数据分离                                            │
│  ▴ 热数据（最近 3 个月）:  SSD                              │
│  ▴ 温数据（3-12 个月）:   HDD                              │
│  ▴ 冷数据（>12 个月）:   归档到 S3/Object Storage          │
│                                                             │
│  3. 连接池动态调整                                          │
│  ▴ 根据负载自动调整                                        │
│  ▴ 高峰期扩容，低峰期缩容                                  │
│  ▴ 预计节省 30%+ 连接数成本（大部分时间闲置）                │
│                                                             │
│  4. 查询成本优化                                            │
│  ▴ 限制复杂查询（深度 > 3 JOIN）                            │
│  ▴ 强制使用索引                                            │
│  ▴ 超时自动取消                                            │
│  ▴ 缓存结果（Redis）                                       │
│                                                             │
│  5. 实例自动化管理                                          │
│  ▴ 非工作时间自动降频/停机（开发环境）                      │
│  ▴ Spot Instance（预置实例）用于非关键负载                  │
│  ▴ 自动资源回收（空闲 1 个月）                             │
│                                                             │
│  6. 跨区域优化                                              │
│  ▴ 数据就近存储                                            │
│  ▴ 跨区域只读副本（用于分析）                              │
│  ▴ 延迟带宽成本权衡                                        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 七、监控与运维

### 7.1 租户级别监控

```promql
# 租户 QPS 监控
sum(rate(db_query_count{tenant_id="$tenant_id"}[5m]))

# 租户 P99 延迟
histogram_quantile(0.99,
  rate(db_query_duration_seconds_bucket{tenant_id="$tenant_id"}[5m])
)

# 租户存储使用
tenant_storage_used_gb{tenant_id="$tenant_id"}

# 租户连接数
tenant_active_connections{tenant_id="$tenant_id"}

# 按分级聚合
sum(rate(db_query_count{tier="enterprise"}[5m]))
sum(rate(db_query_count{tier="vip"}[5m]))
sum(rate(db_query_count{tier="standard"}[5m]))

# 警报规则
alert: TenantQPSBreach
  expr: sum(rate(db_query_count[5m])) by (tenant_id) > <max_qps>
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "Tenant {{ $labels.tier_id }} QPS breach"
    description: "QPS {{ $value }} exceeds threshold"

alert: TenantStorageNearLimit
  expr: tenant_storage_used_gb / tenant_storage_max_gb > 0.9
  for: 10m
  labels:
    severity: critical
  annotations:
    summary: "Tenant {{ $labels.tenant_id }} storage near limit"
```

### 7.2 告警规则分级

```
Enterprise 层告警（最高优先级）
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ P0 (立即处理)
  □ 数据库不可用 (>30s)
  □ 数据丢失风险
  □ 安全入侵检测
  □ SLA 威胁
  通知方式：电话、短信、PagerDuty、Slack #urgent

✅ P1 (30 分钟内处理)
  □ P99 延迟 > 5s
  □ QPS 超阈值 2x
  □ 错误率 > 5%
  通知方式：Slack、邮件、PagerDuty

✅ P2 (2 小时内处理)
  □ 存储使用 > 90%
  □ 连接池耗尽
  □ 复制延迟 > 1s
  通知方式：Slack、邮件


VIP 层告警（高优先级）
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ P0 (立即处理)
  □ 数据库不可用 (>1 分钟)
  □ 安全入侵
  通知方式：电话、Slack #urgent

✅ P1 (1 小时内处理)
  □ P99 延迟 > 10s
  □ QPS 超阈值 1.5x
  □ 错误率 > 10%
  通知方式：Slack、邮件

✅ P2 (4 小时内处理)
  □ 存储使用 > 95%
  □ 连接池使用 > 80%
  通知方式：邮件


Standard 层告警（中优先级）
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ P1 (4 小时内处理)
  □ P99 延迟 > 30s
  □ 错误率 > 20%
  通知方式：Slack、邮件

✅ P2（按批处理）
  □ 存储使用 > 99%（次日处理）
  □ 扩容决策（扩容窗口处理）
  通知方式：邮件


Free 层告警（低优先级）
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ P3（每周处理）
  □ 达到免费限制
  通知方式：邮件（鼓励升级）
```

---

## 八、安全与合规

### 8.1 分层安全策略

```
Enterprise 层安全
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ 网络隔离
  □ VPC 专用
  □ 安全组严格限制
  □ 私有端点
  □ 跨区域加密连接

✅ 数据加密
  □ 静态加密：TDE (Transparent Data Encryption)
  □ 传输加密：TLS 1.3
  □ 备份加密：AES-256
  □ 密钥管理：KMS，定期轮换

✅ 访问控制
  □ IAM/RBAC：细粒度权限
  □ 多因素认证 (MFA)
  □ 租户管理员权限
  □ 审计日志（所有操作）

✅ 合规性
  □ GDPR
  □ SOC2 Type II
  □ ISO 27001
  □ HIPAA（如需要）

✅ 数据保留
  □ 完整审计日志（7 年）
  □ 数据导出功能
  □ 删除请求处理（30 天内）


VIP 层安全
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ 数据隔离
  □ 逻辑隔离（tenant_id）
  □ 分片隔离
  □ 独立 Redis 分片

✅ 加密
  □ 静态加密
  □ 传输加密

✅ 访问控制
  □ RBAC
  □ 审计日志（1 年）


Standard 层安全
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ 基础安全
  □ 逻辑隔离
  □ TLS 传输加密
  □ 基础 RBAC
```

---

## 九、总结与行动清单

### 9.1 生产级架构优势

```
✅ 一步到位，未来无忧
  □ 支持任意规模（1-10000+ 租户）
  □ 分层管理（Enterprise / VIP / Standard / Free）
  □ 自动水平扩展
  □ 高可用（99.99% SLA）

✅ 成本优化
  □ 资源按需分配
  □ 分层计费
  □ 自动缩容节省成本
  □ 预留折扣

✅ VIP 隔离
  □ Enterprise: 独享集群
  □ VIP: 专用资源池
  □ Standard: 共享池

✅ 运维自动化
  □ 自动扩缩容
  □ 自动故障转移
  □ 自动备份恢复
  □ 自动监控告警
```

### 9.2 实施检查清单

```
第一阶段：基础设施（Week 1-4）
  □ PostgreSQL Citus 集群部署
  □ Redis 集群部署
  □ ClickHouse 集群部署
  □ Kafka 集群部署
  □ 监控告警配置
  □ 租户路由服务开发
  □ 租户管理 API 开发

第二阶段：服务集成（Week 5-8）
  □ 现有服务迁移到 TenantRouter
  □ 新服务模板（TenantRouter 集成）
  □ 数据库 Schema 迁移
  □ 测试验证
  □ 灰度发布

第三阶段：自动化（Week 9-12）
  □ Autoscaler Service 开发
  □ 数据迁移自动化
  □ 性能优化
  □ 压力测试
  □ 灾难恢复演练

持续优化
  □ 成本优化（按月评估）
  □ 性能优化（按周评估）
  □ 安全审计（按季度）
  □ 容量规划（按月）
```

### 9.3 关键决策点

```
决策 1: 是否需要 Enterprise 层？
  ■ 如果您是面向大型企业/政府/银行：是
  ■ 如果您是面向中小企业：暂不（可预留）

决策 2: VIP 层资源池大小？
  ■ 预估 VIP 客户数：100 名
  ■ 平均 VIP 数据量：50GB
  ■ VIP Shard 规划：4 个（每个 25 租户）
  ■ 总存储：4 x 500GB = 2TB

决策 3: Standard 层初始规模？
  ■ 预估 Standard 客户数：1000 名（首年）
  ■ 平均数据量：10GB
  ■ Shard 数：10 个（每个 100 租户）
  ■ 总存储：10 x 1TB = 10TB

决策 4: 是否需要独立 Kafka？
  ■ Enterprise: 是（独立实例）
  ■ VIP: 否（共享 Topic 分区）
  ■ Standard: 否

决策 5: 成本预算？
  ▪ Enterprise 客户: $20K-50K/月
  ▪ VIP 客户: $5K-15K/月
  ▪ Standard 客户: $100-5K/月
  ▪ Free 客户: $0

  按目标客户数计算月收入：
    - 10 Enterprise: $200K-500K
    - 50 VIP: $250K-750K
    - 500 Standard: $50K-250K
    ─────────────────────────────
    Total: $500K-1.5M/月
```

---

**结论：**

对于一步到位的生产级架构，推荐**分层多租户架构 + 智能路由**：

1. **Enterprise 层**: 专属集群（32 核，256GB，2TB，HA）
2. **VIP 层**: 专用资源池（4 Shard，128GB/租户）
3. **Standard 层**: 大规模共享池（10+ Shard，弹性扩展）
4. **Tenant Router**: 自动路由到对应资源池
5. **Autoscaler**: 自动扩缩容，按需优化

**关键特性：**
- ✅ VIP 租户真正隔离
- ✅ 自动水平扩展
- ✅ 99.99% SLA
- ✅ 分层计费
- ✅ 运维自动化

**实施周期：** 3-4 个月可完全投产

**成本：** 初期投入 $50K-100K/月，随客户数增长

---

**文档版本:** 2.0
**创建日期:** 2026-02-01
**适用项目:** Cuba ERP (生产级一步到位)
**维护团队:** Infrastructure Team
