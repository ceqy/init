# 数据库连接池配置优化

## 概述

已完成数据库连接池配置优化，支持高并发场景和读写分离架构。

## 配置文件位置

- `services/iam-access/config/default.toml` - 默认配置
- `services/iam-access/config/development.toml` - 开发环境配置
- `services/iam-access/config/production.toml` - 生产环境配置

## 连接池配置参数

### 基础配置

```toml
[database]
url = "postgres://user:password@host:port/database"
max_connections = 100  # 最大连接数
```

### 高级配置（代码层面）

在 `PostgresConfig` 中支持以下参数：

- **max_connections**: 最大连接数（默认: 开发 10，生产 50）
- **min_connections**: 最小连接数（默认: 1）
- **connect_timeout**: 连接超时（默认: 30 秒）
- **idle_timeout**: 空闲连接超时（默认: 600 秒 / 10 分钟）
- **max_lifetime**: 连接最大生命周期（默认: 1800 秒 / 30 分钟）
- **acquire_timeout**: 获取连接超时（默认: 30 秒）

## 环境配置建议

### 开发环境 (development.toml)

```toml
[database]
max_connections = 30
```

- 适合本地开发和测试
- 支持适度并发

### 生产环境 (production.toml)

```toml
[database]
url = "postgres://postgres:postgres@postgres-primary:5432/cuba"
max_connections = 100
```

**连接数计算公式**:
```
max_connections = (CPU 核心数 × 2) + 有效磁盘数
```

例如：
- 16 核 CPU + SSD: 16 × 2 + 1 = 33，建议设置 50-100
- 32 核 CPU + SSD: 32 × 2 + 1 = 65，建议设置 100-150

## 读写分离配置

### 启用读写分离

在生产环境配置中添加读库配置：

```toml
[database]
# 主库（写操作）
url = "postgres://postgres:postgres@postgres-primary:5432/cuba"
max_connections = 100

# 从库（读操作）- 可选
read_url = "postgres://postgres:postgres@postgres-replica:5432/cuba"
read_max_connections = 150
```

### 读写分离优势

1. **性能优化**: 读操作分流到从库，减轻主库压力
2. **高可用**: 主库故障时，读操作仍可继续
3. **扩展性**: 可添加多个读副本应对高并发读场景

### 使用方式

```rust
// 获取读写分离连接池
if let Some(rw_pool) = infra.read_write_pool() {
    // 写操作使用主库
    let write_pool = rw_pool.write_pool();

    // 读操作使用从库（如果配置了）
    let read_pool = rw_pool.read_pool();
}
```

## 监控指标

系统自动收集以下连接池指标：

### 写连接池指标
- `postgres_pool_size{pool="write"}`: 写连接池大小
- `postgres_pool_idle{pool="write"}`: 写连接池空闲连接数
- `postgres_pool_active{pool="write"}`: 写连接池活跃连接数
- `postgres_pool_utilization{pool="write"}`: 写连接池使用率 (%)

### 读连接池指标（如果启用读写分离）
- `postgres_pool_size{pool="read"}`: 读连接池大小
- `postgres_pool_idle{pool="read"}`: 读连接池空闲连接数
- `postgres_pool_active{pool="read"}`: 读连接池活跃连接数
- `postgres_pool_utilization{pool="read"}`: 读连接池使用率 (%)

## 性能调优建议

### 1. 连接数设置

**过少的连接数**:
- 症状: 请求排队等待，响应时间增加
- 解决: 增加 `max_connections`

**过多的连接数**:
- 症状: 数据库 CPU 使用率高，上下文切换频繁
- 解决: 减少 `max_connections`

### 2. 超时设置

**connect_timeout**:
- 建议: 30 秒（默认）
- 场景: 数据库启动或网络不稳定时

**idle_timeout**:
- 建议: 600 秒（10 分钟）
- 场景: 释放长时间空闲的连接

**max_lifetime**:
- 建议: 1800 秒（30 分钟）
- 场景: 防止连接泄漏，定期刷新连接

**acquire_timeout**:
- 建议: 30 秒
- 场景: 获取连接超时，避免请求无限等待

### 3. 读写分离策略

**何时启用读写分离**:
- 读操作占比 > 70%
- 主库 CPU 使用率 > 60%
- 需要提高读性能和可用性

**读库连接数建议**:
- 通常设置为主库的 1.5-2 倍
- 例如: 主库 100，读库 150-200

## 故障处理

### 连接池耗尽

**症状**:
```
Failed to acquire connection: timeout
```

**解决方案**:
1. 增加 `max_connections`
2. 检查是否有连接泄漏（未正确释放）
3. 优化慢查询，减少连接占用时间

### 读库连接失败

**行为**:
- 系统自动降级，使用主库处理读请求
- 记录警告日志

**解决方案**:
1. 检查读库状态
2. 验证读库连接配置
3. 确认网络连通性

## 配置示例

### 小型应用（< 1000 并发）

```toml
[database]
max_connections = 20
```

### 中型应用（1000-10000 并发）

```toml
[database]
max_connections = 50
read_max_connections = 100
```

### 大型应用（> 10000 并发）

```toml
[database]
max_connections = 100
read_max_connections = 200
```

## 相关文件

- `crates/adapters/postgres/src/connection.rs` - 连接池实现
- `crates/config/src/lib.rs` - 配置定义
- `bootstrap/src/infrastructure.rs` - 基础设施初始化
- `bootstrap/src/metrics.rs` - 监控指标收集

## 注意事项

1. **数据库限制**: 确保数据库的 `max_connections` 设置大于所有服务的连接数总和
2. **资源限制**: 考虑服务器内存和 CPU 资源，避免过度配置
3. **监控告警**: 设置连接池使用率告警（建议阈值: 80%）
4. **渐进调整**: 生产环境调整连接数时，建议逐步增加并观察效果
