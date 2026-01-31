# 架构文档

## 概述

ERP 采用微服务架构，基于 DDD（领域驱动设计）、CQRS（命令查询职责分离）和事件溯源模式。

## 分层架构

每个服务采用六边形架构（端口与适配器）：

```
┌─────────────────────────────────────────────────────────────┐
│                        API Layer                             │
│                    (gRPC / REST)                            │
├─────────────────────────────────────────────────────────────┤
│                    Application Layer                         │
│              (Commands / Queries / Handlers)                │
├─────────────────────────────────────────────────────────────┤
│                      Domain Layer                            │
│        (Entities / Value Objects / Events / Services)       │
├─────────────────────────────────────────────────────────────┤
│                   Infrastructure Layer                       │
│            (Repositories / External Services)               │
└─────────────────────────────────────────────────────────────┘
```

## 核心组件

### Ports (抽象层)

定义所有基础设施的抽象接口：

- `Repository<T, ID>` - 数据访问抽象
- `CachePort` - 缓存抽象
- `EventPublisher` - 事件发布抽象
- `OutboxPort` - Outbox 模式抽象

### Adapters (实现层)

具体的基础设施实现：

- `adapter-postgres` - PostgreSQL 实现
- `adapter-redis` - Redis 实现
- `adapter-kafka` - Kafka 实现
- `adapter-clickhouse` - ClickHouse 实现

### Bootstrap (启动骨架)

统一的服务启动逻辑：

- 配置加载
- Tracing 初始化
- gRPC Server 启动
- Graceful Shutdown

## CQRS 模式

```
┌──────────┐     ┌─────────────┐     ┌──────────────┐
│ Command  │────▶│   Handler   │────▶│  Write Model │
└──────────┘     └─────────────┘     └──────────────┘
                                            │
                                            ▼
                                     ┌──────────────┐
                                     │    Events    │
                                     └──────────────┘
                                            │
                                            ▼
┌──────────┐     ┌─────────────┐     ┌──────────────┐
│  Query   │────▶│   Handler   │────▶│  Read Model  │
└──────────┘     └─────────────┘     └──────────────┘
```

## 事件流

```
Service A                    Kafka                    Service B
    │                          │                          │
    │  ──── Publish Event ────▶│                          │
    │                          │──── Consume Event ──────▶│
    │                          │                          │
```

## 数据存储策略

| 数据类型 | 存储 | 用途 |
|---------|------|------|
| 业务数据 | PostgreSQL | OLTP |
| 分析数据 | ClickHouse | OLAP |
| 缓存 | Redis | 热点数据 |
| 事件 | Kafka | 事件流 |
