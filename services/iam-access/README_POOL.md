# IAM Access Service - 数据库连接池配置

## 概述

本目录包含 IAM Access 服务的数据库连接池配置和相关工具。已完成高并发优化和读写分离支持。

## 📁 目录结构

```
services/iam-access/
├── config/                          # 配置文件
│   ├── default.toml                 # 默认配置（20 连接）
│   ├── development.toml             # 开发环境（30 连接）
│   └── production.toml              # 生产环境（100 连接 + 读写分离）
├── scripts/                         # 工具脚本
│   ├── monitor_connections.sql     # 数据库连接监控 SQL
│   └── stress_test_pool.sh         # 连接池压力测试脚本
├── DATABASE_POOL_CONFIG.md         # 详细配置文档
└── POOL_OPTIMIZATION_SUMMARY.md    # 优化总结
```

## 🚀 快速开始

### 1. 配置环境

根据运行环境选择配置文件：

```bash
# 开发环境
export APP_ENV=development

# 生产环境
export APP_ENV=production
```

### 2. 启动服务

```bash
cd services/iam-access
cargo run
```

服务会自动加载对应环境的配置文件。

### 3. 验证配置

运行验证脚本检查配置是否正确：

```bash
./scripts/verify_pool_config.sh
```

## 📊 配置说明

### 开发环境

```toml
[database]
max_connections = 30
```

- 适合本地开发和测试
- 支持适度并发

### 生产环境

```toml
[database]
url = "postgres://postgres:postgres@postgres-primary:5432/cuba"
max_connections = 100

# 可选: 读写分离
read_url = "postgres://postgres:postgres@postgres-replica:5432/cuba"
read_max_connections = 150
```

- 支持高并发（100+ 并发）
- 可选读写分离配置
- 自动连接生命周期管理

## 🔧 工具使用

### 验证配置

```bash
./scripts/verify_pool_config.sh
```

检查项：
- ✅ 配置文件完整性
- ✅ 连接池实现
- ✅ 读写分离支持
- ✅ 监控指标
- ✅ 编译状态

### 压力测试

```bash
# 使用默认参数（50 并发，60 秒）
./scripts/stress_test_pool.sh

# 自定义参数
CONCURRENT_CONNECTIONS=100 TEST_DURATION=120 ./scripts/stress_test_pool.sh
```

测试内容：
- 并发连接测试
- 查询性能测试
- 错误率统计
- 连接池使用率分析

### 数据库监控

使用 `psql` 执行监控查询：

```bash
psql -h localhost -U postgres -d cuba -f scripts/monitor_connections.sql
```

监控内容：
- 当前连接状态
- 连接池使用率
- 长时间运行的查询
- 空闲连接统计
- 性能指标

## 📈 监控指标

服务自动暴露以下 Prometheus 指标：

### 写连接池
- `postgres_pool_size{pool="write"}` - 连接池大小
- `postgres_pool_idle{pool="write"}` - 空闲连接数
- `postgres_pool_active{pool="write"}` - 活跃连接数
- `postgres_pool_utilization{pool="write"}` - 使用率 (%)

### 读连接池（如果启用）
- `postgres_pool_size{pool="read"}` - 连接池大小
- `postgres_pool_idle{pool="read"}` - 空闲连接数
- `postgres_pool_active{pool="read"}` - 活跃连接数
- `postgres_pool_utilization{pool="read"}` - 使用率 (%)

## 🎯 性能调优

### 连接数计算公式

```
max_connections = (CPU 核心数 × 2) + 有效磁盘数
```

### 应用规模建议

| 应用规模 | 并发量 | 写连接数 | 读连接数 |
|---------|--------|---------|---------|
| 小型 | < 1000 | 20 | - |
| 中型 | 1000-10000 | 50 | 100 |
| 大型 | > 10000 | 100 | 200 |

### 告警阈值

- **连接池使用率 > 80%**: 考虑增加连接数
- **空闲连接数 = 0**: 连接池可能耗尽
- **空闲连接 > 50%**: 考虑减少连接数

## 📚 文档

- [DATABASE_POOL_CONFIG.md](DATABASE_POOL_CONFIG.md) - 详细配置文档
  - 配置参数说明
  - 读写分离配置
  - 监控指标详解
  - 性能调优建议
  - 故障处理指南

- [POOL_OPTIMIZATION_SUMMARY.md](POOL_OPTIMIZATION_SUMMARY.md) - 优化总结
  - 已完成的优化
  - 性能提升对比
  - 使用示例
  - 验证结果

## 🔍 故障排查

### 连接池耗尽

**症状**:
```
Failed to acquire connection: timeout
```

**解决方案**:
1. 检查连接池使用率: `postgres_pool_utilization`
2. 增加 `max_connections`
3. 检查是否有连接泄漏
4. 优化慢查询

### 读库连接失败

**行为**:
- 自动降级到主库
- 记录警告日志

**解决方案**:
1. 检查读库状态
2. 验证 `read_url` 配置
3. 确认网络连通性

### 性能下降

**检查项**:
1. 连接池使用率是否过高
2. 是否有长时间运行的查询
3. 数据库 CPU/内存使用情况
4. 是否需要启用读写分离

## 🛠️ 开发指南

### 使用读写分离

```rust
use cuba_bootstrap::Infrastructure;

// 获取基础设施
let infra = Infrastructure::from_config(config).await?;

// 使用读写分离
if let Some(rw_pool) = infra.read_write_pool() {
    // 写操作
    let write_pool = rw_pool.write_pool();
    sqlx::query("INSERT INTO users (...) VALUES (...)")
        .execute(write_pool)
        .await?;

    // 读操作（自动使用从库）
    let read_pool = rw_pool.read_pool();
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(read_pool)
        .await?;
}
```

### 监控连接池状态

```rust
// 获取连接池状态
let status = infra.postgres_pool_status();

println!("写连接池: {}/{} (活跃/总数)",
    status.write_active, status.write_size);

if status.read_size > 0 {
    println!("读连接池: {}/{} (活跃/总数)",
        status.read_active, status.read_size);
}
```

## 📝 相关链接

- [PostgreSQL 连接池最佳实践](https://wiki.postgresql.org/wiki/Number_Of_Database_Connections)
- [SQLx 文档](https://docs.rs/sqlx/latest/sqlx/)
- [ERP 架构文档](../../.vscode/docs/architecture.md)

## 🤝 贡献

如果发现配置问题或有优化建议，请提交 Issue 或 Pull Request。

## 📄 许可证

MIT License
