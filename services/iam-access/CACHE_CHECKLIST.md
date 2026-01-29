# 缓存改进实施检查清单

## 📋 实施前检查

### 环境准备

- [ ] Rust 版本 >= 1.70
- [ ] Redis 运行正常
- [ ] 有足够的应用内存（L1 缓存需要约 10-50MB）
- [ ] 已备份当前代码

### 依赖检查

- [x] `moka = { version = "0.12", features = ["future"] }` 已添加
- [x] `rand = "0.8"` 已添加
- [x] 代码编译通过 ✓
- [x] 测试通过 (9/9) ✓

## 🚀 实施步骤

### 第一步：代码集成（5 分钟）

#### 1.1 修改 main.rs

**位置**: `services/iam-access/src/main.rs` 第 76-79 行

**原代码**:
```rust
// 初始化缓存
use infrastructure::cache::AuthCache;
let redis_cache = infra.redis_cache();
let auth_cache = Arc::new(AuthCache::new(Arc::new(redis_cache)));
```

**新代码**:
```rust
// 初始化增强的缓存
use infrastructure::cache::{create_enhanced_cache, CacheStrategyConfig};

let redis_conn = infra.redis_connection_manager();
let cache_config = CacheStrategyConfig::default();
let auth_cache = create_enhanced_cache(redis_conn, cache_config);
```

- [ ] 已修改 main.rs
- [ ] 代码编译通过

#### 1.2 可选：添加缓存预热

**位置**: `services/iam-access/src/main.rs` 第 99 行之后（创建 gRPC 服务之后）

```rust
// 可选：启动缓存预热
if cache_config.enable_cache_warming {
    use infrastructure::cache::start_cache_warming;

    tokio::spawn({
        let auth_cache = auth_cache.clone();
        let policy_repo = policy_repo.clone();
        let role_repo = role_repo.clone();

        async move {
            // 获取需要预热的租户列表
            let tenant_ids = vec![
                // 添加你的租户 ID
            ];

            if let Err(e) = start_cache_warming(
                auth_cache,
                policy_repo,
                role_repo,
                tenant_ids,
            ).await {
                tracing::warn!("Cache warming failed: {}", e);
            } else {
                tracing::info!("Cache warming completed successfully");
            }
        }
    });
}
```

- [ ] 已添加缓存预热（可选）
- [ ] 代码编译通过

### 第二步：编译和测试

#### 2.1 编译检查

```bash
cd services/iam-access
cargo check
```

- [ ] 编译通过，无错误
- [ ] 无警告（或只有预期的警告）

#### 2.2 运行测试

```bash
cargo test --lib cache
```

预期输出：
```
test result: ok. 9 passed; 0 failed; 0 ignored
```

- [ ] 所有测试通过
- [ ] 无测试失败

#### 2.3 运行完整测试

```bash
cargo test
```

- [ ] 所有测试通过
- [ ] 无回归问题

### 第三步：本地验证

#### 3.1 启动服务

```bash
cargo run
```

预期日志：
```
INFO Enabling avalanche protection with jitter range: 30 seconds
INFO Enabling multi-layer cache (L1 max: 10000, TTL: 60s)
INFO IAM Access Service started
```

- [ ] 服务启动成功
- [ ] 看到缓存初始化日志
- [ ] 无错误日志

#### 3.2 功能测试

测试基本功能是否正常：

```bash
# 测试 gRPC 健康检查
grpcurl -plaintext localhost:50052 grpc.health.v1.Health/Check

# 测试 RBAC 服务
grpcurl -plaintext localhost:50052 list iam.access.v1.RbacService
```

- [ ] 健康检查通过
- [ ] gRPC 服务可访问
- [ ] 功能正常

#### 3.3 缓存验证

观察日志中的缓存命中情况：

```bash
# 设置日志级别为 DEBUG
export RUST_LOG=debug
cargo run
```

预期看到：
```
DEBUG Cache hit in L1
DEBUG Cache hit in L2
DEBUG Cache miss
```

- [ ] 看到 L1 缓存命中日志
- [ ] 看到 L2 缓存命中日志
- [ ] 缓存工作正常

### 第四步：故障测试

#### 4.1 测试 Redis 故障降级

1. 启动服务
2. 停止 Redis：`docker stop redis` 或 `redis-cli shutdown`
3. 观察服务是否继续运行
4. 检查日志是否有降级警告
5. 重启 Redis：`docker start redis`

预期行为：
- 服务不中断
- 看到 "L2 cache error" 警告
- 自动降级到 L1
- Redis 恢复后自动恢复

- [ ] Redis 故障时服务继续运行
- [ ] 看到降级日志
- [ ] Redis 恢复后正常

#### 4.2 测试缓存雪崩防护

1. 清空 Redis：`redis-cli FLUSHALL`
2. 发起大量并发请求
3. 观察数据库 QPS

预期行为：
- 数据库 QPS 不会突然飙升
- Singleflight 合并请求
- 系统稳定

- [ ] 缓存清空后系统稳定
- [ ] 数据库压力可控

### 第五步：性能验证

#### 5.1 基准测试

使用 `wrk` 或 `hey` 进行压力测试：

```bash
# 安装 hey
go install github.com/rakyll/hey@latest

# 压力测试
hey -n 10000 -c 100 http://localhost:50052/health
```

观察指标：
- 响应时间 P50, P95, P99
- 吞吐量 (QPS)
- 错误率

- [ ] 响应时间符合预期
- [ ] 吞吐量提升
- [ ] 错误率低

#### 5.2 缓存命中率

观察一段时间后的缓存命中率：

```bash
# 查看 Redis 统计
redis-cli INFO stats | grep keyspace
```

预期：
- L1 命中率 > 70%
- L2 命中率 > 10%
- 总命中率 > 90%

- [ ] 缓存命中率符合预期

### 第六步：监控接入（可选）

#### 6.1 添加监控指标

在代码中添加 Prometheus 指标：

```rust
// 在缓存操作中
metrics::counter!("cache_l1_hits_total", 1);
metrics::counter!("cache_l2_hits_total", 1);
metrics::counter!("cache_misses_total", 1);
metrics::counter!("cache_fallback_total", 1);
```

- [ ] 已添加监控指标
- [ ] Prometheus 可以抓取指标

#### 6.2 配置告警

配置 Prometheus 告警规则：

```yaml
groups:
  - name: cache_alerts
    rules:
      - alert: CacheHighFallbackRate
        expr: rate(cache_fallback_total[5m]) > 10
        annotations:
          summary: "Redis 可能故障"

      - alert: CacheLowHitRate
        expr: rate(cache_l1_hits_total[5m]) / rate(cache_requests_total[5m]) < 0.7
        annotations:
          summary: "缓存命中率过低"
```

- [ ] 已配置告警规则
- [ ] 告警测试通过

## 📊 验收标准

### 功能验收

- [ ] 所有原有功能正常
- [ ] 缓存读写正常
- [ ] 缓存失效正常
- [ ] 无功能回归

### 性能验收

- [ ] 响应时间 P99 < 10ms（L1 命中）
- [ ] 响应时间 P99 < 50ms（L2 命中）
- [ ] 缓存命中率 > 90%
- [ ] 数据库 QPS 降低 > 50%

### 可用性验收

- [ ] Redis 故障时服务可用
- [ ] 降级自动触发
- [ ] 恢复自动完成
- [ ] 无数据丢失

### 稳定性验收

- [ ] 缓存雪崩场景稳定
- [ ] 缓存击穿场景稳定
- [ ] 高并发场景稳定
- [ ] 长时间运行稳定

## 🔄 回滚方案

如果遇到问题，可以快速回滚：

### 回滚步骤

1. 恢复 main.rs 到原来的代码：

```rust
// 回滚到原实现
use infrastructure::cache::AuthCache;
let redis_cache = infra.redis_cache();
let auth_cache = Arc::new(AuthCache::new(Arc::new(redis_cache)));
```

2. 重新编译和部署：

```bash
cargo build --release
# 重启服务
```

3. 验证功能正常

- [ ] 已准备回滚方案
- [ ] 回滚步骤已测试

## 📝 部署清单

### 测试环境部署

- [ ] 代码已合并到测试分支
- [ ] 测试环境部署完成
- [ ] 功能测试通过
- [ ] 性能测试通过
- [ ] 稳定性测试通过（运行 24 小时）

### 预生产环境部署

- [ ] 代码已合并到预生产分支
- [ ] 预生产环境部署完成
- [ ] 灰度测试通过（10% 流量）
- [ ] 监控指标正常
- [ ] 无异常告警

### 生产环境部署

- [ ] 代码已合并到主分支
- [ ] 生产环境部署计划已制定
- [ ] 回滚方案已准备
- [ ] 监控和告警已配置
- [ ] 值班人员已通知

#### 灰度发布计划

1. **第一阶段**（10% 流量）
   - [ ] 部署到 10% 实例
   - [ ] 观察 1 小时
   - [ ] 监控指标正常
   - [ ] 无异常告警

2. **第二阶段**（50% 流量）
   - [ ] 部署到 50% 实例
   - [ ] 观察 2 小时
   - [ ] 监控指标正常
   - [ ] 无异常告警

3. **第三阶段**（100% 流量）
   - [ ] 部署到所有实例
   - [ ] 观察 24 小时
   - [ ] 监控指标正常
   - [ ] 无异常告警

## 📈 上线后观察

### 第一天

- [ ] 每小时检查监控指标
- [ ] 缓存命中率 > 90%
- [ ] 响应时间正常
- [ ] 错误率 < 0.1%
- [ ] 无异常告警

### 第一周

- [ ] 每天检查监控指标
- [ ] 性能指标稳定
- [ ] 无故障发生
- [ ] 用户反馈正常

### 第一个月

- [ ] 每周检查监控指标
- [ ] 长期稳定性验证
- [ ] 成本优化评估
- [ ] 经验总结文档

## 📚 文档清单

### 技术文档

- [x] `CACHE_IMPROVEMENT.md` - 详细设计文档
- [x] `CACHE_QUICKSTART.md` - 快速开始指南
- [x] `CACHE_ARCHITECTURE.md` - 架构图
- [x] `CACHE_IMPROVEMENT_SUMMARY.md` - 改进总结
- [x] `CACHE_CHECKLIST.md` - 本检查清单

### 代码文档

- [x] `src/infrastructure/cache/avalanche_protection.rs` - 雪崩防护
- [x] `src/infrastructure/cache/multi_layer.rs` - 多层缓存
- [x] `src/infrastructure/cache/bloom_filter.rs` - 布隆过滤器
- [x] `src/infrastructure/cache/cache_warmer.rs` - 缓存预热
- [x] `src/infrastructure/cache/strategy.rs` - 策略组合
- [x] `src/infrastructure/cache/integration_example.rs` - 集成示例
- [x] `src/infrastructure/cache/tests.rs` - 测试用例

### 运维文档

- [ ] 部署文档
- [ ] 监控配置文档
- [ ] 故障处理手册
- [ ] 性能调优指南

## ✅ 最终确认

### 代码质量

- [x] 代码编译通过
- [x] 所有测试通过
- [ ] 代码审查通过
- [ ] 无安全漏洞

### 功能完整性

- [x] 雪崩防护实现
- [x] 多层缓存实现
- [x] 布隆过滤器实现
- [x] 缓存预热实现
- [x] 配置管理实现

### 文档完整性

- [x] 设计文档完整
- [x] 使用文档完整
- [x] 测试文档完整
- [ ] 运维文档完整

### 准备就绪

- [ ] 所有检查项已完成
- [ ] 团队已培训
- [ ] 上线计划已制定
- [ ] 可以开始实施

## 🎯 成功指标

### 短期目标（1 周内）

- [ ] 缓存命中率 > 90%
- [ ] 响应时间降低 50%
- [ ] 数据库 QPS 降低 60%
- [ ] 无故障发生

### 中期目标（1 个月内）

- [ ] Redis 故障时服务可用性 > 99.9%
- [ ] 缓存雪崩场景零故障
- [ ] 成本降低 30%（数据库资源）
- [ ] 用户满意度提升

### 长期目标（3 个月内）

- [ ] 系统整体可用性 > 99.99%
- [ ] 性能持续优化
- [ ] 经验推广到其他服务
- [ ] 形成最佳实践

## 📞 支持联系

如有问题，请参考：

1. **文档**: 查看 `CACHE_QUICKSTART.md` 和 `CACHE_IMPROVEMENT.md`
2. **日志**: 设置 `RUST_LOG=debug` 查看详细日志
3. **测试**: 运行 `cargo test --lib cache` 验证功能
4. **监控**: 查看 Prometheus 指标和 Grafana 仪表板

---

**最后更新**: 2026-01-29
**版本**: 1.0.0
**状态**: ✅ 准备就绪
