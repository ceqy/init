# Cuba ERP

基于 Rust 的企业资源规划系统。

## 项目结构

```
cuba-erp/
├── crates/                    # 共享库
│   ├── common/                # 通用类型和工具
│   ├── errors/                # 统一错误处理
│   ├── config/                # 配置加载
│   ├── telemetry/             # 可观测性
│   ├── auth-core/             # 认证核心
│   ├── ports/                 # 抽象 trait 层
│   ├── domain-core/           # 跨 context 领域核心
│   ├── cqrs-core/             # CQRS 核心
│   ├── event-core/            # 事件核心
│   └── adapters/              # 基础设施适配器
│       ├── postgres/
│       ├── redis/
│       ├── clickhouse/
│       └── kafka/
├── proto/                     # Protocol Buffers 定义
├── services/                  # 微服务
│   └── iam-identity/          # 身份服务（认证、用户、OAuth）
├── gateway/                   # API 网关
├── bootstrap/                 # 统一启动骨架
├── deploy/                    # 部署配置
└── docs/                      # 文档
```

## 快速开始

### 前置条件

- Rust 1.75+
- Docker & Docker Compose
- just (命令运行器)

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

- **语言**: Rust
- **Web 框架**: Axum
- **gRPC**: Tonic
- **数据库**: PostgreSQL, ClickHouse
- **缓存**: Redis
- **消息队列**: Kafka
- **容器化**: Docker, Kubernetes

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

## 许可证

Apache License 2.0
