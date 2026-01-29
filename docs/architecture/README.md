# 架构文档

本目录包含 Cuba ERP 系统的架构设计文档。

## 文档列表

- [architecture.md](architecture.md) - 系统整体架构概览
- [architecture-detailed.md](architecture-detailed.md) - 详细架构设计说明

## 架构概览

Cuba ERP 采用以下架构模式：

### 核心架构
- **领域驱动设计 (DDD)**: 按业务领域划分服务边界
- **CQRS**: 命令查询职责分离，优化读写性能
- **事件溯源**: 通过事件记录状态变更历史

### 技术架构
- **微服务架构**: 服务独立部署和扩展
- **六边形架构**: 端口适配器模式，隔离业务逻辑
- **多租户架构**: 支持 SaaS 模式的租户隔离

### 基础设施
- **API 网关**: 统一入口，路由和认证
- **消息队列**: Kafka 实现异步通信
- **数据存储**: PostgreSQL (写) + ClickHouse (读)
- **缓存层**: Redis 提升性能

## 相关文档

- [开发指南](../guides/development.md)
- [多租户指南](../guides/multi-tenancy.md)
- [API 文档](../api/README.md)

## 架构决策记录 (ADR)

重要的架构决策应记录在此目录，包括：
- 决策背景
- 考虑的方案
- 最终决策
- 决策后果
