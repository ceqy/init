# 数据库连接池配置 - 常见问题解答 (FAQ)

## 基础问题

### Q1: 为什么需要连接池？

**A:** 连接池的主要优势：

1. **性能提升**: 避免频繁创建/销毁数据库连接的开销
2. **资源管理**: 限制并发连接数，防止数据库过载
3. **连接复用**: 多个请求共享连接，提高资源利用率
4. **故障恢复**: 自动检测和重建失效的连接

### Q2: 如何确定合适的连接池大小？

**A:** 使用以下公式作为起点：

```
max_connections = (CPU 核心数 × 2) + 有效磁盘数
```

然后根据实际负载调整：

- **监控使用率**: 保持在 60-80% 之间
- **压力测试**: 逐步增加并发，找到最佳值
- **考虑其他服务**: 确保数据库总连接数不超过限制

**示例**:
- 开发环境: 20-30 连接
- 小型应用: 30-50 连接
- 中型应用: 50-100 连接
- 大型应用: 100-200 连接

### Q3: 读写分离什么时候需要？

**A:** 考虑启用读写分离的场景：

- ✅ 读操作占比 > 70%
- ✅ 主库 CPU 使用率 > 60%
- ✅ 需要提高读性能
- ✅ 需要提高可用性（主库故障时读操作仍可用）

**不需要的场景**:
- ❌ 读写比例接近 1:1
- ❌ 数据一致性要求极高（不能容忍复制延迟）
- ❌ 小型应用，单库足够

### Q4: 连接池配置后需要重启服务吗？

**A:** 是的，连接池配置在服务启动时加载，修改后需要重启服务才能生效。

**最佳实践**:
1. 在测试环境验证配置
2. 使用滚动更新避免服务中断
3. 监控重启后的连接池状态

## 配置问题

### Q5: 如何配置多个数据库连接池？

**A:** 当前实现支持主从两个连接池。如果需要连接多个数据库：

```rust
// 为不同数据库创建独立的连接池
let pool1 = create_pool(&config1).await?;
let pool2 = create_pool(&config2).await?;

// 或使用读写分离
let rw_pool = ReadWritePool::new(write_pool, Some(read_pool));
```

### Q6: 连接超时参数如何设置？

**A:** 推荐配置：

```rust
PostgresConfig::new(url)
    .with_connect_timeout(Duration::from_secs(30))    // 连接超时
    .with_acquire_timeout(Duration::from_secs(30))    // 获取连接超时
    .with_idle_timeout(Duration::from_secs(600))      // 空闲超时（10分钟）
    .with_max_lifetime(Duration::from_secs(1800))     // 最大生命周期（30分钟）
```

**调整建议**:
- **网络不稳定**: 增加 `connect_timeout`
- **高并发**: 减少 `acquire_timeout` 快速失败
- **连接泄漏**: 减少 `idle_timeout` 和 `max_lifetime`

### Q7: min_connections 应该设置多少？

**A:** 推荐设置：

- **开发环境**: 1-2 个（节省资源）
- **生产环境**: max_connections 的 10-20%

**原因**:
- 保持最小连接数可以减少冷启动延迟
- 但过多会浪费资源

```rust
.with_min_connections(10)  // 对于 max_connections = 100
```

## 性能问题

### Q8: 连接池使用率一直很高怎么办？

**A:** 诊断步骤：

1. **检查是否有慢查询**:
```sql
SELECT pid, query, NOW() - query_start as duration
FROM pg_stat_activity
WHERE state = 'active' AND NOW() - query_start > interval '5 seconds'
ORDER BY duration DESC;
```

2. **检查是否有连接泄漏**:
```sql
SELECT state, COUNT(*) FROM pg_stat_activity GROUP BY state;
```

3. **解决方案**:
   - 优化慢查询
   - 增加连接池大小
   - 启用读写分离
   - 添加缓存层

### Q9: 为什么会出现 "connection timeout" 错误？

**A:** 常见原因：

1. **连接池耗尽**: 所有连接都在使用中
   - 解决: 增加 `max_connections`

2. **慢查询**: 连接被长时间占用
   - 解决: 优化查询，添加索引

3. **连接泄漏**: 连接未正确释放
   - 解决: 检查代码，确保使用 RAII 模式

4. **数据库过载**: 数据库响应慢
   - 解决: 优化数据库性能，增加资源

**诊断命令**:
```bash
./scripts/health_check_pool.sh
```

### Q10: 读写分离后读延迟怎么办？

**A:** 处理复制延迟的策略：

1. **监控延迟**:
```sql
SELECT
    application_name,
    pg_wal_lsn_diff(pg_current_wal_lsn(), replay_lsn) / 1024 / 1024 as lag_mb
FROM pg_stat_replication;
```

2. **应用层处理**:
```rust
// 写后立即读，使用主库
let write_pool = rw_pool.write_pool();
sqlx::query("INSERT INTO users ...").execute(write_pool).await?;
let user = sqlx::query_as("SELECT * FROM users WHERE id = ?")
    .fetch_one(write_pool)  // 使用主库读取
    .await?;

// 普通读取，使用从库
let read_pool = rw_pool.read_pool();
let users = sqlx::query_as("SELECT * FROM users")
    .fetch_all(read_pool)
    .await?;
```

3. **配置同步复制**（如果延迟不可接受）:
```sql
ALTER SYSTEM SET synchronous_commit = 'on';
ALTER SYSTEM SET synchronous_standby_names = 'replica1';
```

## 监控问题

### Q11: 如何监控连接池状态？

**A:** 多种监控方式：

1. **Prometheus 指标**:
```promql
postgres_pool_utilization{pool="write"}
postgres_pool_active{pool="write"}
postgres_pool_idle{pool="write"}
```

2. **健康检查脚本**:
```bash
./scripts/health_check_pool.sh
```

3. **数据库查询**:
```bash
psql -f scripts/monitor_connections.sql
```

4. **应用日志**:
```rust
let status = infra.postgres_pool_status();
tracing::info!(
    write_active = status.write_active,
    write_size = status.write_size,
    "Pool status"
);
```

### Q12: 应该设置哪些告警？

**A:** 推荐告警规则：

1. **连接池使用率 > 80%** (警告)
2. **连接池使用率 > 90%** (严重)
3. **空闲连接数 = 0** (严重)
4. **长时间运行查询 > 30s** (警告)
5. **空闲事务 > 5s** (严重)
6. **复制延迟 > 10MB** (警告)

**Prometheus 告警示例**:
```yaml
- alert: HighPoolUtilization
  expr: postgres_pool_utilization > 80
  for: 5m
  labels:
    severity: warning
```

## 故障排查

### Q13: 如何排查连接泄漏？

**A:** 步骤：

1. **查看长时间空闲的连接**:
```sql
SELECT pid, usename, state, NOW() - state_change as idle_duration
FROM pg_stat_activity
WHERE state = 'idle' AND NOW() - state_change > interval '10 minutes'
ORDER BY idle_duration DESC;
```

2. **检查应用代码**:
```rust
// ❌ 错误：手动管理连接
let conn = pool.acquire().await?;
// ... 如果这里出错，连接不会释放

// ✅ 正确：使用 RAII
{
    let conn = pool.acquire().await?;
    // 连接会在作用域结束时自动释放
}
```

3. **配置自动清理**:
```rust
.with_idle_timeout(Duration::from_secs(600))
.with_max_lifetime(Duration::from_secs(1800))
```

### Q14: 数据库连接数达到上限怎么办？

**A:** 紧急处理：

1. **查看当前连接**:
```sql
SELECT COUNT(*) FROM pg_stat_activity;
```

2. **终止空闲连接**（谨慎使用）:
```sql
SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE state = 'idle' AND NOW() - state_change > interval '10 minutes';
```

3. **增加数据库连接限制**:
```sql
ALTER SYSTEM SET max_connections = 200;
SELECT pg_reload_conf();
```

4. **长期解决**:
   - 优化应用连接池配置
   - 增加服务器资源
   - 实施读写分离

### Q15: 如何测试连接池配置是否合理？

**A:** 测试流程：

1. **基准测试**:
```bash
CONCURRENT_CONNECTIONS=30 TEST_DURATION=60 ./scripts/stress_test_pool.sh
```

2. **压力测试**:
```bash
CONCURRENT_CONNECTIONS=100 TEST_DURATION=120 ./scripts/stress_test_pool.sh
```

3. **持久性测试**:
```bash
CONCURRENT_CONNECTIONS=50 TEST_DURATION=3600 ./scripts/stress_test_pool.sh
```

4. **评估指标**:
   - 成功率 > 99%
   - 平均响应时间 < 100ms
   - 连接池使用率 60-80%
   - 无连接泄漏

## 最佳实践

### Q16: 生产环境部署前的检查清单？

**A:** 部署前检查：

- [ ] 连接池大小根据服务器规格设置
- [ ] 超时参数已配置
- [ ] 读写分离已配置（如需要）
- [ ] 监控和告警已部署
- [ ] 压力测试已通过
- [ ] 健康检查脚本已配置
- [ ] 数据库 max_connections 足够
- [ ] 备份和恢复流程已测试
- [ ] 文档已更新
- [ ] 团队已培训

### Q17: 如何进行连接池配置的 A/B 测试？

**A:** 测试方法：

1. **准备两套配置**:
```toml
# 配置 A: 当前配置
max_connections = 50

# 配置 B: 新配置
max_connections = 100
```

2. **分流测试**:
   - 50% 流量使用配置 A
   - 50% 流量使用配置 B

3. **对比指标**:
   - 响应时间
   - 错误率
   - 资源使用
   - 成本

4. **选择最优配置**

### Q18: 如何优化连接池性能？

**A:** 优化策略：

1. **代码层面**:
   - 使用连接池而不是直接连接
   - 及时释放连接
   - 避免长事务
   - 使用批量操作

2. **配置层面**:
   - 合理设置连接数
   - 配置超时参数
   - 启用读写分离
   - 使用连接池预热

3. **数据库层面**:
   - 优化慢查询
   - 添加索引
   - 调整数据库参数
   - 增加资源

4. **架构层面**:
   - 添加缓存层
   - 使用消息队列
   - 实施微服务拆分
   - 数据库分片

## 高级话题

### Q19: 如何实现连接池的动态调整？

**A:** 当前实现不支持动态调整，需要重启服务。未来可以考虑：

1. **热重载配置**
2. **自动扩缩容**
3. **基于负载的动态调整**

### Q20: 如何处理多租户场景的连接池？

**A:** 策略：

1. **共享连接池**（推荐）:
   - 所有租户共享一个连接池
   - 使用 Row-Level Security (RLS)
   - 简单高效

2. **租户隔离连接池**:
   - 每个租户独立连接池
   - 更好的隔离性
   - 资源开销大

3. **混合模式**:
   - VIP 租户独立连接池
   - 普通租户共享连接池

## 参考资源

- [DATABASE_POOL_CONFIG.md](DATABASE_POOL_CONFIG.md) - 详细配置文档
- [POOL_SETUP_GUIDE.md](POOL_SETUP_GUIDE.md) - 完整设置指南
- [PostgreSQL 官方文档](https://www.postgresql.org/docs/)
- [SQLx 文档](https://docs.rs/sqlx/latest/sqlx/)

## 获取帮助

如果本 FAQ 没有解答您的问题：

1. 查看详细文档
2. 运行诊断脚本
3. 检查日志
4. 提交 Issue
5. 联系团队

---

**最后更新**: 2026-01-29
**版本**: 1.0.0
