# 数据库连接池配置 - 完整使用指南

## 🎯 目标

本指南帮助您完成 IAM Access 服务的数据库连接池配置，包括：

1. ✅ 基础配置
2. ✅ 读写分离配置
3. ✅ 性能测试
4. ✅ 监控部署
5. ✅ 故障排查

## 📋 前置要求

- Docker 和 Docker Compose
- PostgreSQL 客户端 (psql)
- Rust 工具链
- 基本的 SQL 知识

## 🚀 快速开始

### 步骤 1: 启动测试环境

使用 Docker Compose 启动完整的测试环境（包括主从数据库、Redis、监控）：

```bash
cd services/iam-access
docker-compose -f docker-compose.pool-test.yml up -d
```

等待所有服务启动：

```bash
docker-compose -f docker-compose.pool-test.yml ps
```

### 步骤 2: 验证配置

运行验证脚本：

```bash
./scripts/verify_pool_config.sh
```

预期输出：所有检查项显示 ✓

### 步骤 3: 启动服务

```bash
# 开发环境
export APP_ENV=development
cargo run

# 或使用生产配置
export APP_ENV=production
cargo run
```

### 步骤 4: 运行压力测试

```bash
# 基础测试（50 并发，60 秒）
./scripts/stress_test_pool.sh

# 高并发测试（100 并发，120 秒）
CONCURRENT_CONNECTIONS=100 TEST_DURATION=120 ./scripts/stress_test_pool.sh
```

### 步骤 5: 查看监控

访问监控界面：

- **Grafana**: http://localhost:3000 (admin/admin)
- **Prometheus**: http://localhost:9090
- **pgAdmin**: http://localhost:5050 (admin@erp.local/admin)

## 📊 配置详解

### 基础配置（单数据库）

**适用场景**: 开发环境、小型应用

```toml
# config/development.toml
[database]
url = "postgres://postgres:postgres@localhost:5432/cuba"
max_connections = 30
```

**特点**:
- 简单易用
- 适合单机部署
- 成本低

### 读写分离配置（主从架构）

**适用场景**: 生产环境、高并发应用

```toml
# config/production.toml
[database]
# 主库（写操作）
url = "postgres://postgres:postgres@postgres-primary:5432/cuba"
max_connections = 100

# 从库（读操作）
read_url = "postgres://postgres:postgres@postgres-replica:5432/cuba"
read_max_connections = 150
```

**特点**:
- 读写分离，性能更好
- 主库故障时读操作仍可用
- 可扩展多个读副本

**代码使用**:

```rust
// 获取读写分离连接池
if let Some(rw_pool) = infra.read_write_pool() {
    // 写操作使用主库
    let write_pool = rw_pool.write_pool();
    sqlx::query("INSERT INTO users (...) VALUES (...)")
        .execute(write_pool)
        .await?;

    // 读操作使用从库
    let read_pool = rw_pool.read_pool();
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(read_pool)
        .await?;
}
```

## 🔧 高级配置

### 连接池参数调优

在代码层面，`PostgresConfig` 支持以下参数：

```rust
let config = PostgresConfig::new(database_url)
    .with_max_connections(100)           // 最大连接数
    .with_min_connections(10)            // 最小连接数
    .with_connect_timeout(Duration::from_secs(30))  // 连接超时
    .with_idle_timeout(Duration::from_secs(600))    // 空闲超时（10分钟）
    .with_max_lifetime(Duration::from_secs(1800))   // 最大生命周期（30分钟）
    .with_acquire_timeout(Duration::from_secs(30)); // 获取连接超时
```

### 连接数计算

**推荐公式**:
```
max_connections = (CPU 核心数 × 2) + 有效磁盘数
```

**示例**:
- 8 核 CPU + SSD: 8 × 2 + 1 = 17，建议设置 20-30
- 16 核 CPU + SSD: 16 × 2 + 1 = 33，建议设置 50-100
- 32 核 CPU + SSD: 32 × 2 + 1 = 65，建议设置 100-150

**注意**: 数据库的 `max_connections` 必须大于所有服务的连接数总和。

## 📈 性能测试

### 测试场景

#### 1. 基准测试

```bash
# 测试默认配置性能
CONCURRENT_CONNECTIONS=30 TEST_DURATION=60 ./scripts/stress_test_pool.sh
```

**预期结果**:
- 成功率 > 99%
- 平均 QPS > 100
- 错误数 = 0

#### 2. 压力测试

```bash
# 测试极限并发
CONCURRENT_CONNECTIONS=100 TEST_DURATION=120 ./scripts/stress_test_pool.sh
```

**观察指标**:
- 连接池使用率
- 查询响应时间
- 错误率

#### 3. 持久性测试

```bash
# 长时间运行测试
CONCURRENT_CONNECTIONS=50 TEST_DURATION=3600 ./scripts/stress_test_pool.sh
```

**检查项**:
- 连接泄漏
- 内存使用
- 性能稳定性

### 性能基准

| 配置 | 并发数 | QPS | 响应时间 | 成功率 |
|-----|-------|-----|---------|--------|
| 开发环境 (30 连接) | 30 | 150-200 | < 50ms | > 99% |
| 生产环境 (100 连接) | 100 | 500-800 | < 100ms | > 99.9% |
| 读写分离 (100+150) | 200 | 1000-1500 | < 80ms | > 99.9% |

## 📊 监控和告警

### Grafana 仪表板

访问 http://localhost:3000，创建仪表板监控以下指标：

#### 连接池指标

```promql
# 写连接池使用率
postgres_pool_utilization{pool="write"}

# 读连接池使用率
postgres_pool_utilization{pool="read"}

# 活跃连接数
postgres_pool_active{pool="write"}
postgres_pool_active{pool="read"}

# 空闲连接数
postgres_pool_idle{pool="write"}
postgres_pool_idle{pool="read"}
```

#### 告警规则

```yaml
groups:
  - name: database_pool
    interval: 30s
    rules:
      # 连接池使用率告警
      - alert: HighPoolUtilization
        expr: postgres_pool_utilization > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "连接池使用率过高"
          description: "{{ $labels.pool }} 连接池使用率 {{ $value }}%"

      # 连接池耗尽告警
      - alert: PoolExhausted
        expr: postgres_pool_idle == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "连接池耗尽"
          description: "{{ $labels.pool }} 连接池所有连接都在使用中"

      # 连接池过度配置告警
      - alert: PoolOverProvisioned
        expr: postgres_pool_utilization < 20
        for: 30m
        labels:
          severity: info
        annotations:
          summary: "连接池可能过度配置"
          description: "{{ $labels.pool }} 连接池使用率持续低于 20%"
```

### 数据库监控

使用提供的 SQL 脚本监控数据库：

```bash
# 查看连接状态
psql -h localhost -U postgres -d cuba -f scripts/monitor_connections.sql

# 或使用 watch 实时监控
watch -n 5 'psql -h localhost -U postgres -d cuba -c "SELECT state, COUNT(*) FROM pg_stat_activity WHERE datname = '\''cuba'\'' GROUP BY state"'
```

## 🔍 故障排查

### 问题 1: 连接池耗尽

**症状**:
```
Error: Failed to acquire connection from pool: timeout
```

**诊断**:
```bash
# 检查连接池使用率
curl http://localhost:9090/api/v1/query?query=postgres_pool_utilization

# 查看数据库连接
psql -h localhost -U postgres -d cuba -c "SELECT COUNT(*) FROM pg_stat_activity WHERE datname = 'cuba'"
```

**解决方案**:
1. 增加 `max_connections`
2. 检查慢查询: `SELECT * FROM pg_stat_activity WHERE state = 'active' AND NOW() - query_start > interval '30 seconds'`
3. 检查连接泄漏: 查看长时间空闲的连接
4. 优化查询性能

### 问题 2: 读库连接失败

**症状**:
```
WARN: Failed to create read replica pool, using primary for reads
```

**诊断**:
```bash
# 检查读库状态
docker-compose -f docker-compose.pool-test.yml ps postgres-replica

# 测试读库连接
psql -h localhost -p 5433 -U postgres -d cuba -c "SELECT 1"

# 检查复制状态
psql -h localhost -U postgres -d cuba -c "SELECT * FROM pg_stat_replication"
```

**解决方案**:
1. 确认读库服务运行正常
2. 检查 `read_url` 配置
3. 验证网络连通性
4. 查看读库日志: `docker-compose logs postgres-replica`

### 问题 3: 性能下降

**症状**:
- 响应时间增加
- QPS 下降
- 连接等待时间长

**诊断**:
```bash
# 检查连接池状态
./scripts/verify_pool_config.sh

# 查看慢查询
psql -h localhost -U postgres -d cuba -f scripts/monitor_connections.sql

# 检查数据库性能
psql -h localhost -U postgres -d cuba -c "SELECT * FROM pg_stat_database WHERE datname = 'cuba'"
```

**解决方案**:
1. 分析慢查询并优化
2. 增加连接池大小
3. 启用读写分离
4. 检查数据库资源（CPU、内存、磁盘 I/O）
5. 考虑添加索引

### 问题 4: 连接泄漏

**症状**:
- 连接数持续增长
- 空闲连接过多
- 最终导致连接池耗尽

**诊断**:
```sql
-- 查看长时间空闲的连接
SELECT
    pid,
    usename,
    application_name,
    state,
    NOW() - state_change as idle_duration
FROM pg_stat_activity
WHERE datname = 'cuba'
    AND state = 'idle'
    AND NOW() - state_change > interval '10 minutes'
ORDER BY idle_duration DESC;
```

**解决方案**:
1. 检查代码中是否正确释放连接
2. 调整 `idle_timeout` 参数
3. 设置 `max_lifetime` 强制回收长期连接
4. 使用连接池的自动清理功能

## 📚 最佳实践

### 1. 连接池配置

- ✅ 根据应用规模选择合适的连接数
- ✅ 设置合理的超时参数
- ✅ 启用连接生命周期管理
- ✅ 生产环境考虑读写分离

### 2. 代码实践

- ✅ 使用连接池而不是直接连接
- ✅ 及时释放连接（使用 RAII）
- ✅ 避免长时间持有连接
- ✅ 使用事务时注意连接占用时间

### 3. 监控和告警

- ✅ 监控连接池使用率
- ✅ 设置告警阈值（建议 80%）
- ✅ 定期检查慢查询
- ✅ 监控数据库性能指标

### 4. 性能优化

- ✅ 定期进行压力测试
- ✅ 根据监控数据调整配置
- ✅ 优化慢查询
- ✅ 考虑使用缓存减少数据库压力

## 🎓 学习资源

- [PostgreSQL 官方文档](https://www.postgresql.org/docs/)
- [SQLx 文档](https://docs.rs/sqlx/latest/sqlx/)
- [连接池最佳实践](https://wiki.postgresql.org/wiki/Number_Of_Database_Connections)
- [DATABASE_POOL_CONFIG.md](DATABASE_POOL_CONFIG.md) - 详细配置文档

## 📝 检查清单

部署前检查：

- [ ] 配置文件已创建并验证
- [ ] 连接数根据服务器规格设置
- [ ] 超时参数已配置
- [ ] 读写分离已配置（如需要）
- [ ] 监控已部署
- [ ] 告警规则已设置
- [ ] 压力测试已通过
- [ ] 文档已更新

## 🤝 获取帮助

如果遇到问题：

1. 查看 [DATABASE_POOL_CONFIG.md](DATABASE_POOL_CONFIG.md)
2. 运行 `./scripts/verify_pool_config.sh` 检查配置
3. 查看服务日志
4. 检查数据库连接状态
5. 提交 Issue 或联系团队

---

**最后更新**: 2026-01-29
**版本**: 1.0.0
