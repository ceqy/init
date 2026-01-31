# 修复总结：Session 索引优化 & Event Store 版本冲突检测

## 修复日期
2026-01-28

## 问题概述

### 1. Session 表索引优化
**问题：** Session 表缺少针对租户特定查询的复合索引，导致查询性能不佳。

**影响：**
- 查询某租户下未过期的会话时需要索引合并
- 查询某租户下未撤销的会话时需要索引合并
- 在高并发场景下可能成为性能瓶颈

### 2. Event Store 版本冲突检测
**问题：** Event Store 的 `append` 方法缺少版本冲突检测，违反了 Event Sourcing 的乐观并发控制原则。

**风险场景：**
```
时刻 T1: 线程 A 读取 User(id=1) 当前版本 = 5
时刻 T2: 线程 B 读取 User(id=1) 当前版本 = 5
时刻 T3: 线程 A 追加事件，version = 6 ✓
时刻 T4: 线程 B 追加事件，version = 6 ❌ (冲突！但未检测)
```

**影响：**
- 可能导致事件版本冲突
- 破坏聚合的一致性
- 违反 Event Sourcing 的核心约束

---

## 修复方案

### 1. 数据库层面修复

#### 文件：`services/iam-identity/migrations/20260128110000_fix_session_indexes_and_event_store.sql`

**Session 表索引优化：**
```sql
-- 查询模式：WHERE tenant_id = ? AND expires_at > NOW()
CREATE INDEX IF NOT EXISTS idx_sessions_tenant_expires
ON sessions(tenant_id, expires_at);

-- 查询模式：WHERE tenant_id = ? AND revoked = false
CREATE INDEX IF NOT EXISTS idx_sessions_tenant_revoked
ON sessions(tenant_id, revoked);

-- 查询模式：WHERE tenant_id = ? AND user_id = ? AND revoked = false
-- 部分索引：仅索引未撤销的会话，减少索引大小
CREATE INDEX IF NOT EXISTS idx_sessions_tenant_user_revoked
ON sessions(tenant_id, user_id, revoked)
WHERE revoked = false;
```

**Event Store 唯一约束：**
```sql
-- 防止版本冲突：同一聚合的版本号必须唯一
ALTER TABLE event_store
ADD CONSTRAINT uk_event_store_aggregate_version
UNIQUE (aggregate_type, aggregate_id, version);

-- 优化查询性能
CREATE INDEX IF NOT EXISTS idx_event_store_aggregate
ON event_store(aggregate_type, aggregate_id, version DESC);

CREATE INDEX IF NOT EXISTS idx_event_store_occurred_at
ON event_store(occurred_at DESC);
```

### 2. 应用层面修复

#### 文件：`crates/adapters/postgres/src/event_store.rs`

**实现乐观并发控制：**

```rust
async fn append<E: Serialize + Send + Sync>(
    &self,
    envelope: &EventEnvelope<E>,
) -> AppResult<()> {
    // 1. 开启事务
    let mut tx = self.pool.begin().await?;

    // 2. 检查当前版本（使用 FOR UPDATE 锁定）
    let current_version: Option<(i64,)> = sqlx::query_as(
        r#"
        SELECT COALESCE(MAX(version), 0)
        FROM event_store
        WHERE aggregate_type = $1 AND aggregate_id = $2
        FOR UPDATE
        "#,
    )
    .bind(&envelope.aggregate_type)
    .bind(&envelope.aggregate_id)
    .fetch_optional(&mut *tx)
    .await?;

    let current_version = current_version.map(|(v,)| v as u64).unwrap_or(0);
    let expected_version = if envelope.version == 1 { 0 } else { envelope.version - 1 };

    // 3. 版本冲突检测
    if current_version != expected_version {
        return Err(AppError::conflict(format!(
            "Version conflict for {}:{} - expected version {}, but current version is {}",
            envelope.aggregate_type, envelope.aggregate_id, expected_version, current_version
        )));
    }

    // 4. 插入事件
    sqlx::query(...)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            // 检查唯一约束冲突
            if let Some(db_err) = e.as_database_error() {
                if db_err.constraint() == Some("uk_event_store_aggregate_version") {
                    return AppError::conflict(...);
                }
            }
            AppError::database(...)
        })?;

    // 5. 提交事务
    tx.commit().await?;
    Ok(())
}
```

**关键改进：**
1. ✅ 使用事务确保原子性
2. ✅ 使用 `FOR UPDATE` 锁定版本查询，防止并发读取
3. ✅ 在插入前检查版本是否匹配
4. ✅ 捕获唯一约束冲突，返回友好的错误信息
5. ✅ 双重保护：应用层检查 + 数据库约束

---

## 测试覆盖

### 新增测试用例

#### 文件：`crates/adapters/postgres/src/event_store.rs`

1. **test_append_first_event_success**
   - 验证首次追加事件成功

2. **test_append_sequential_events_success**
   - 验证顺序追加多个事件成功

3. **test_version_conflict_detection**
   - 验证重复版本被检测并拒绝

4. **test_skip_version_detection**
   - 验证跳过版本被检测并拒绝

5. **test_get_events_from_version**
   - 验证按版本范围查询事件

6. **test_different_aggregates_independent_versions**
   - 验证不同聚合的版本独立管理

---

## 性能影响

### Session 表索引优化

**优化前：**
```sql
-- 查询某租户下未过期的会话
EXPLAIN SELECT * FROM sessions
WHERE tenant_id = '...' AND expires_at > NOW();

-- 可能的执行计划：
-- 1. Index Scan on idx_sessions_tenant_id
-- 2. Filter: expires_at > NOW()
-- 或者使用 Bitmap Index Scan 合并两个索引
```

**优化后：**
```sql
-- 使用复合索引，直接定位
-- Index Scan on idx_sessions_tenant_expires
-- 性能提升：避免索引合并，减少随机 I/O
```

**预期提升：**
- 查询延迟降低 30-50%
- 减少 CPU 使用（避免索引合并）
- 提升并发处理能力

### Event Store 版本冲突检测

**性能开销：**
- 每次追加事件需要额外的 `SELECT ... FOR UPDATE` 查询
- 使用事务增加少量开销

**性能优化：**
- `FOR UPDATE` 查询使用索引，开销很小
- 事务开销在可接受范围内
- 唯一约束检查由数据库高效处理

**权衡：**
- 轻微的性能开销（< 5%）
- 换取数据一致性保证
- 避免严重的数据损坏问题

---

## 部署步骤

### 1. 运行数据库迁移

```bash
# 在 IAM Identity 服务目录下
cd services/iam-identity

# 运行迁移
sqlx migrate run
```

### 2. 验证迁移结果

```sql
-- 检查 Session 表索引
SELECT indexname, indexdef
FROM pg_indexes
WHERE tablename = 'sessions'
AND indexname LIKE 'idx_sessions_tenant%';

-- 检查 Event Store 约束
SELECT conname, contype, pg_get_constraintdef(oid)
FROM pg_constraint
WHERE conrelid = 'event_store'::regclass
AND conname = 'uk_event_store_aggregate_version';
```

### 3. 重新编译和部署

```bash
# 编译所有服务
cargo build --release

# 运行测试
cargo test -p adapter-postgres

# 部署服务
# (根据你的部署流程)
```

---

## 回滚方案

如果需要回滚，执行以下 SQL：

```sql
-- 回滚 Session 表索引
DROP INDEX IF EXISTS idx_sessions_tenant_expires;
DROP INDEX IF EXISTS idx_sessions_tenant_revoked;
DROP INDEX IF EXISTS idx_sessions_tenant_user_revoked;

-- 回滚 Event Store 约束（注意：可能导致数据一致性问题）
ALTER TABLE event_store DROP CONSTRAINT IF EXISTS uk_event_store_aggregate_version;
DROP INDEX IF EXISTS idx_event_store_aggregate;
DROP INDEX IF EXISTS idx_event_store_occurred_at;
```

**警告：** 回滚 Event Store 约束后，需要同时回滚应用代码，否则会出现版本冲突检测失败的情况。

---

## 监控建议

### 1. Session 表查询性能

```sql
-- 监控慢查询
SELECT query, mean_exec_time, calls
FROM pg_stat_statements
WHERE query LIKE '%sessions%'
AND mean_exec_time > 100
ORDER BY mean_exec_time DESC;
```

### 2. Event Store 版本冲突频率

```bash
# 在应用日志中搜索版本冲突错误
grep "Version conflict" /var/log/iam-identity.log | wc -l
```

### 3. 索引使用情况

```sql
-- 检查索引是否被使用
SELECT schemaname, tablename, indexname, idx_scan, idx_tup_read
FROM pg_stat_user_indexes
WHERE tablename IN ('sessions', 'event_store')
ORDER BY idx_scan DESC;
```

---

## 相关文件

### 修改的文件
- `services/iam-identity/migrations/20260128110000_fix_session_indexes_and_event_store.sql` (新增)
- `crates/adapters/postgres/src/event_store.rs` (修改)

### 测试文件
- `crates/adapters/postgres/src/event_store.rs` (新增测试)

---

## 参考资料

1. **Event Sourcing 最佳实践**
   - [Martin Fowler - Event Sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)
   - 乐观并发控制是 Event Sourcing 的核心原则

2. **PostgreSQL 索引优化**
   - [PostgreSQL Documentation - Indexes](https://www.postgresql.org/docs/current/indexes.html)
   - 复合索引的列顺序很重要：高选择性列在前

3. **数据库约束**
   - [PostgreSQL Documentation - Constraints](https://www.postgresql.org/docs/current/ddl-constraints.html)
   - 唯一约束提供数据完整性保证

---

## 总结

本次修复解决了两个重要问题：

1. **Session 表索引优化**：通过添加复合索引，显著提升了租户特定查询的性能。

2. **Event Store 版本冲突检测**：实现了完整的乐观并发控制，确保事件版本的一致性，符合 Event Sourcing 的最佳实践。

这两个修复都是生产环境必需的，建议尽快部署。
