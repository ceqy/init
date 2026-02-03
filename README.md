# Init ERP

基于 Rust 的企业资源规划系统，采用微服务架构和 Vault 密钥管理。

## 🚀 项目特点

- ✅ **微服务架构**: 基于 DDD + CQRS + Event Sourcing
- 🔐 **安全优先**: 集成 HashiCorp Vault 统一密钥管理
- 📊 **可观测性**: OpenTelemetry + Prometheus + Grafana
- 🎯 **类型安全**: Rust 1.93 + 严格的类型系统
- 🔄 **事件驱动**: Kafka 消息队列 + Event Sourcing
- 🌐 **多租户**: 完整的租户隔离和数据安全

## 📁 项目结构

```
init/
├── .cargo/                    # Cargo 配置
├── bootstrap/                 # 统一启动骨架
│   └── src/                   # Infrastructure 初始化
├── crates/                    # 共享库
│   ├── adapters/              # 基础设施适配器
│   │   ├── clickhouse/        # ClickHouse 适配器
│   │   ├── email/             # 邮件发送适配器
│   │   ├── kafka/             # Kafka 消息队列适配器
│   │   ├── postgres/          # PostgreSQL 适配器
│   │   ├── redis/             # Redis 缓存适配器
│   │   └── vault/             # 🆕 Vault 密钥管理适配器
│   ├── auth-core/             # 认证核心 (JWT/Token)
│   ├── common/                # 通用类型和工具
│   ├── config/                # 配置加载
│   ├── cqrs-core/             # CQRS 核心
│   ├── domain-core/           # 领域核心
│   ├── errors/                # 统一错误处理
│   ├── event-core/            # 事件核心
│   ├── ports/                 # 抽象 trait 层
│   └── telemetry/             # 可观测性
├── gateway/                   # API 网关
│   └── src/                   # gRPC-Web, WebSocket, 路由
├── proto/                     # Protocol Buffers 定义
│   ├── common/                # 公共消息定义
│   ├── iam/                   # 身份和访问管理
│   ├── am/                    # 资产管理
│   ├── cs/                    # 客户服务
│   ├── fi/                    # 财务管理
│   ├── hr/                    # 人力资源
│   ├── mdm/                   # 主数据管理
│   ├── mf/                    # 制造管理
│   ├── org/                   # 组织管理
│   ├── pm/                    # 采购管理
│   ├── rd/                    # 研发管理
│   ├── sc/                    # 供应链管理
│   ├── sd/                    # 销售管理
│   └── sys/                   # 系统管理
└── services/                  # 微服务 (46个服务)
    ├── iam-identity/          # 身份服务
    ├── iam-access/            # 权限服务
    ├── am-ah/                 # 资产层级
    ├── am-eh/                 # 设备历史
    ├── am-gs/                 # 通用服务
    ├── am-pm/                 # 预防性维护
    ├── cs-cb/                 # 客户行为
    ├── cs-fd/                 # 反馈
    ├── cs-wc/                 # 工单中心
    ├── fi-ap/                 # 应付账款
    ├── fi-ar/                 # 应收账款
    ├── fi-co/                 # 成本核算
    ├── fi-coa/                # 会计科目表
    ├── fi-gl/                 # 总账
    ├── fi-tr/                 # 资金管理
    ├── hr-ex/                 # 费用报销
    ├── hr-ta/                 # 考勤管理
    ├── mdm-bp/                # 业务伙伴
    ├── mdm-material/          # 物料主数据
    ├── mf-eng/                # 工程管理
    ├── mf-kb/                 # 看板
    ├── mf-om/                 # 订单管理
    ├── mf-pp/                 # 生产计划
    ├── mf-qi/                 # 质量检验
    ├── mf-sf/                 # 车间管理
    ├── org-enterprise/        # 企业组织
    ├── pm-ct/                 # 合同管理
    ├── pm-iv/                 # 库存管理
    ├── pm-po/                 # 采购订单
    ├── pm-sa/                 # 供应商评估
    ├── pm-se/                 # 供应商启用
    ├── pm-vs/                 # 供应商选择
    ├── rd-pl/                 # 产品生命周期
    ├── rd-ps/                 # 项目服务
    ├── sc-bt/                 # 批次追踪
    ├── sc-df/                 # 需求预测
    ├── sc-im/                 # 库存管理
    ├── sc-tp/                 # 运输计划
    ├── sc-vs/                 # 供应商选择
    ├── sc-wm/                 # 仓库管理
    ├── sd-an/                 # 分析
    ├── sd-pe/                 # 绩效评估
    ├── sd-rr/                 # 报告和记录
    ├── sd-so/                 # 销售订单
    ├── sys-core/              # 系统核心
    └── sys-notify/            # 通知服务
```

## 🔐 密钥管理 (Vault)

项目使用 HashiCorp Vault 进行统一密钥管理：

### 快速开始

1. **复制配置模板**
   ```bash
   cp .env.example .env.local
   ```

2. **填写 Vault 凭证**
   ```bash
   vim .env.local
   # 填入 VAULT_ADDR, VAULT_ROLE_ID, VAULT_SECRET_ID
   ```

3. **运行服务**
   ```bash
   cargo run
   ```

### 环境配置文件

| 文件 | 用途 | 提交到 Git |
|------|------|-----------|
| `.env.local` | 本地开发（包含 Vault 凭证） | ❌ 不提交 |
| `.env.example` | 配置模板 | ✅ 提交 |

## 🚀 快速开始

### 前置条件

- **Rust**: 1.93+
- **Vault**: HashiCorp Vault (可选，用于密钥管理)



###  配置 Vault（推荐）

```bash
# 1. 启动 Vault (开发模式)
docker run -d --name vault \
  -p 10018:8200 \
  -e VAULT_DEV_ROOT_TOKEN_ID=root \
  hashicorp/vault:latest

# 2. 配置 AppRole 认证
# 参考: docs/guides/VAULT_MIGRATION_GUIDE.md

# 3. 存储密钥到 Vault
vault kv put secret/database/postgresql \
  host="10.0.0.10" \
  port="5432" \
  username="postgres" \
  password="your-password"
```

###  构建项目

```bash
# 或构建特定服务
cargo build -p iam-identity
```

### 测试
```
# 运行特定包的测试
cargo test -p adapter-vault
```

## 🛠️ 技术栈

### 核心技术

| 类别 | 技术 | 版本 |
|------|------|------|
| 语言 | Rust | 1.93+ |
| Web 框架 | Axum | 0.8 |
| gRPC | Tonic | 0.13 |
| 异步运行时 | Tokio | 1.0 |

### 数据存储

| 类别 | 技术 | 用途 |
|------|------|------|
| OLTP 数据库 | PostgreSQL | 16 | 主数据库 |
| OLAP 数据库 | ClickHouse | 分析数据库 |
| 缓存 | Redis | 7 | 缓存/会话 |
| 对象存储 | MinIO | 对象存储 |
| 搜索引擎 | Elasticsearch | 全文搜索 |

### 消息队列

| 技术 | 用途 |
|------|------|
| Kafka/AutoMQ | 事件流 |
| RabbitMQ | 消息队列 |

### 基础设施

| 类别 | 技术 | 用途 |
|------|------|------|
| 密钥管理 | HashiCorp Vault | 统一密钥管理 |
| 服务网格 | Envoy | 代理/负载均衡 |
| 服务发现 | Consul | 服务注册与发现 |

### 可观测性

| 类别 | 技术 | 用途 |
|------|------|------|
| 指标 | Prometheus | 指标采集 |
| 可视化 | Grafana | 仪表盘 |
| 追踪 | Jaeger | 分布式追踪 |
| 日志 | Loki + Promtail | 日志聚合 |
| 告警 | AlertManager | 告警管理 |


## 🏗️ 架构

### 架构模式

- **领域驱动设计 (DDD)**: 按业务领域划分服务
- **CQRS**: 命令查询职责分离
- **事件溯源 (Event Sourcing)**: 通过事件记录状态变更
- **微服务架构**: 独立部署、独立扩展

### 分层架构

```
┌─────────────────────────────────────┐
│         API Layer (gRPC)            │
├─────────────────────────────────────┤
│      Application Layer (CQRS)      │
├─────────────────────────────────────┤
│        Domain Layer (DDD)           │
├─────────────────────────────────────┤
│   Infrastructure Layer (Adapters)   │
└─────────────────────────────────────┘
```


## 🔒 安全

### 密钥管理

- ✅ 所有敏感数据存储在 Vault 中
- ✅ 使用 AppRole 认证
- ✅ 支持密钥轮换
- ✅ 审计日志记录

### 最佳实践

- 永远不要在代码中硬编码密码
- 永远不要提交 `.env.local` 到 Git
- 定期轮换 Vault 凭证
- 启用 Vault 审计日志
- 使用 HTTPS 连接 Vault（生产环境）


## 📄 许可证

Apache License 2.0

---

**注意**: 本项目正在积极开发中，API 可能会发生变化。
