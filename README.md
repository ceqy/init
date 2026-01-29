# Cuba ERP

基于 Rust 的企业资源规划系统。

## 项目结构

```
cuba-erp/
├── .github/                   # GitHub 配置
│   └── workflows/             # CI/CD 流水线
│       ├── ci.yml             # 持续集成 (Rust 1.93, 全分支)
│       └── cd.yml             # 持续部署 (k3s, Self-hosted Runner)
├── crates/                    # 共享库
│   ├── common/                # 通用类型和工具
│   ├── errors/                # 统一错误处理
│   ├── config/                # 配置加载 (含读写分离支持)
│   ├── telemetry/             # 可观测性 (OpenTelemetry)
│   ├── auth-core/             # 认证核心 (JWT/Token)
│   ├── ports/                 # 抽象 trait 层 (Cache, EventStore)
│   ├── domain-core/           # 跨 context 领域核心
│   ├── cqrs-core/             # CQRS 核心
│   ├── event-core/            # 事件核心 (Event Sourcing)
│   └── adapters/              # 基础设施适配器
│       ├── postgres/          # 数据库 (含 ReadWritePool)
│       ├── redis/             # 缓存与分布式锁
│       ├── clickhouse/        # 分析数据库
│       ├── kafka/             # 消息队列
│       └── email/             # 邮件发送
├── proto/                     # Protocol Buffers 定义
├── services/                  # 微服务
│   ├── iam-identity/          # 身份服务 (认证、用户、OAuth、WebAuthn)
│   └── iam-access/            # 权限服务 (RBAC、Policy、授权)
├── gateway/                   # API 网关 (gRPC-Web, WebSocket)
├── bootstrap/                 # 统一启动骨架 (Infrastructure)
├── deploy/                    # 部署配置
│   ├── docker/                # Docker Compose 编排
│   ├── k3s/                   # Kubernetes 部署 Manifests
│   ├── envoy/                 # Envoy Sidecar 配置
│   ├── consul/                # Consul 服务发现
│   ├── prometheus/            # Prometheus 监控
│   └── grafana/               # Grafana 仪表盘
└── docs/                      # 文档
```

## 快速开始

### 前置条件

- Rust 1.93+
- Docker & Docker Compose
- just (命令运行器)
- k3s / kubectl (生产环境部署)

### 启动基础设施

```bash
just infra-up
```

### 构建项目

```bash
just build
```

### 运行开发服务器

```bash
just dev
```

## 技术栈

- **语言**: Rust 1.93
- **Web 框架**: Axum
- **gRPC**: Tonic
- **数据库**: PostgreSQL, ClickHouse
- **缓存**: Redis
- **消息队列**: Kafka
- **服务网格**: Envoy, Consul
- **容器化**: Docker, k3s (Kubernetes)
- **CI/CD**: GitHub Actions

## 架构

采用 DDD + CQRS + Event Sourcing 架构：

- **领域驱动设计 (DDD)**: 按业务领域划分服务
- **CQRS**: 命令查询职责分离
- **事件溯源**: 通过事件记录状态变更

详细架构说明请查看 [架构文档](docs/architecture/README.md)。

## 文档

完整文档请访问 [文档中心](docs/README.md)：

- [开发指南](docs/guides/development.md) - 开发规范和流程
- [架构文档](docs/architecture/README.md) - 系统架构设计
- [API 文档](docs/api/README.md) - API 接口说明
- [安全指南](docs/guides/security.md) - 安全最佳实践
- [多租户指南](docs/guides/multi-tenancy.md) - 多租户架构
- [CI/CD 指南](deploy/CICD_SETUP_GUIDE.md) - 自动化部署配置
- [Envoy 部署](deploy/ENVOY_DEPLOYMENT_GUIDE.md) - 服务网格部署

## 许可证

Apache License 2.0
