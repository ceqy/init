# 灾难恢复手册（Disaster Recovery Playbook）

## 📋 概述

本手册定义了 Cuba ERP 系统的灾难恢复策略、流程和步骤。

### RTO/RPO 目标

| 服务 | RTO (恢复时间目标) | RPO (恢复点目标) | 优先级 |
|------|-------------------|-----------------|--------|
| Gateway API | 15 分钟 | 5 分钟 | P0 |
| IAM 服务 | 15 分钟 | 5 分钟 | P0 |
| PostgreSQL | 30 分钟 | 15 分钟 | P0 |
| Redis | 15 分钟 | 30 分钟 | P1 |
| Kafka | 1 小时 | 1 小时 | P1 |
| ClickHouse | 2 小时 | 4 小时 | P2 |
| Consul | 30 分钟 | 1 小时 | P1 |

---

## 🚨 灾难场景与应对

### 场景 1：单个服务实例故障

**症状：**
- Consul 健康检查失败
- Envoy 自动剔除不健康实例
- 部分请求失败

**恢复步骤：**

```bash
# 1. 确认故障实例
curl http://localhost:8500/v1/health/service/iam-access | jq '.[] | select(.Checks[].Status != "passing")'

# 2. 查看容器日志
docker logs cuba-iam-access --tail 100

# 3. 重启故障实例
docker-compose -f deploy/docker/docker-compose.envoy.yml restart iam-access

# 4. 验证恢复
curl http://localhost:8500/v1/health/service/iam-access
curl http://localhost:9901/clusters | grep iam_cluster
```

**预计恢复时间：** 2-5 分钟

---

### 场景 2：数据库完全故障

**症状：**
- 所有数据库连接失败
- 应用服务报错
- Prometheus 告警触发

**恢复步骤：**

#### 2.1 主数据库故障（有备份）

```bash
# 1. 停止所有依赖数据库的服务
docker-compose -f deploy/docker/docker-compose.envoy.yml stop gateway iam-access

# 2. 停止故障数据库
docker-compose -f deploy/docker/docker-compose.envoy.yml stop postgres

# 3. 清理数据目录（谨慎操作）
docker volume rm cuba_postgres_data

# 4. 重新创建数据库容器
docker-compose -f deploy/docker/docker-compose.envoy.yml up -d postgres

# 5. 等待数据库就绪
until docker exec cuba-postgres pg_isready -U postgres; do sleep 1; done

# 6. 恢复最新备份
LATEST_BACKUP=$(ls -t /backups/postgres/cuba_*.sql.gz | head -1)
./scripts/restore-database.sh "$LATEST_BACKUP"

# 7. 验证数据完整性
docker exec cuba-postgres psql -U postgres -d cuba -c "SELECT COUNT(*) FROM users;"

# 8. 重启应用服务
docker-compose -f deploy/docker/docker-compose.envoy.yml up -d gateway iam-access

# 9. 验证服务恢复
curl http://localhost:8080/health
```

**预计恢复时间：** 20-30 分钟（取决于数据库大小）

#### 2.2 主数据库故障（有从库）

```bash
# 1. 提升从库为主库
docker exec cuba-postgres-slave pg_ctl promote

# 2. 更新应用配置指向新主库
export DATABASE_URL="postgresql://postgres:postgres@postgres-slave:5432/cuba"

# 3. 重启应用服务
docker-compose -f deploy/docker/docker-compose.envoy.yml restart gateway iam-access

# 4. 修复原主库并配置为从库
# （详细步骤见 PostgreSQL 主从切换文档）
```

**预计恢复时间：** 5-10 分钟

---

### 场景 3：Consul 集群故障

**症状：**
- Consul UI 无法访问
- 服务发现失败
- Envoy 无法获取服务列表

**恢复步骤：**

#### 3.1 单节点故障（集群仍有 Quorum）

```bash
# 1. 确认集群状态
curl http://localhost:8500/v1/status/leader

# 2. 移除故障节点
consul force-leave <node-name>

# 3. 启动新节点加入集群
docker-compose -f deploy/docker/docker-compose.envoy.yml up -d consul-node-new

# 4. 验证集群健康
curl http://localhost:8500/v1/status/peers
```

**预计恢复时间：** 5-10 分钟

#### 3.2 集群完全故障（丢失 Quorum）

```bash
# 1. 停止所有 Consul 节点
docker-compose -f deploy/docker/docker-compose.envoy.yml stop consul

# 2. 从备份恢复 Consul 数据
tar -xzf /backups/consul/consul_data_latest.tar.gz -C /var/lib/consul/

# 3. 以 bootstrap 模式启动第一个节点
docker-compose -f deploy/docker/docker-compose.envoy.yml up -d consul-1

# 4. 等待 Leader 选举
sleep 10

# 5. 启动其他节点
docker-compose -f deploy/docker/docker-compose.envoy.yml up -d consul-2 consul-3

# 6. 验证集群状态
curl http://localhost:8500/v1/status/peers

# 7. 重新注册所有服务
./scripts/register-services.sh
```

**预计恢复时间：** 15-30 分钟

---

### 场景 4：整个数据中心故障

**症状：**
- 所有服务不可用
- 网络完全中断
- 物理设施故障

**恢复步骤：**

#### 4.1 切换到备用数据中心

```bash
# 1. 更新 DNS 指向备用数据中心
# （通过 DNS 提供商控制台操作）

# 2. 在备用数据中心启动服务
ssh backup-dc-server
cd /opt/cuba-erp
./deploy/docker/start-envoy.sh

# 3. 从远程备份恢复数据
aws s3 sync s3://cuba-backups/latest/ /backups/
./scripts/restore-all.sh

# 4. 验证服务可用性
curl https://api.cuba-erp.com/health

# 5. 通知用户服务已恢复
./scripts/send-notification.sh "服务已切换到备用数据中心"
```

**预计恢复时间：** 1-2 小时

---

## 🔄 定期演练

### 月度演练（每月第一个周六）

**演练内容：**
1. 数据库备份恢复测试
2. 单个服务故障恢复
3. 告警系统测试

**演练步骤：**
```bash
# 1. 创建测试环境
./scripts/create-test-env.sh

# 2. 模拟故障
docker stop cuba-iam-access

# 3. 执行恢复流程
./scripts/recover-service.sh iam-access

# 4. 验证恢复
./scripts/verify-recovery.sh

# 5. 记录演练结果
./scripts/log-drill-result.sh
```

### 季度演练（每季度最后一个周六）

**演练内容：**
1. 完整数据中心故障切换
2. 数据库主从切换
3. Consul 集群重建
4. 全量数据恢复

---

## 📊 恢复流程决策树

```
故障发生
    │
    ├─ 单个服务实例？
    │   └─ 是 → 重启实例 → 验证恢复
    │
    ├─ 数据库故障？
    │   ├─ 有从库？
    │   │   └─ 是 → 主从切换 → 验证恢复
    │   └─ 否 → 从备份恢复 → 验证恢复
    │
    ├─ Consul 故障？
    │   ├─ 有 Quorum？
    │   │   └─ 是 → 移除故障节点 → 添加新节点
    │   └─ 否 → 重建集群 → 重新注册服务
    │
    └─ 数据中心故障？
        └─ 切换到备用 DC → 恢复数据 → 更新 DNS
```

---

## 📝 恢复检查清单

### 恢复后验证

- [ ] 所有服务健康检查通过
- [ ] Consul 服务注册正常
- [ ] Envoy 集群状态正常
- [ ] 数据库连接正常
- [ ] API 响应正常
- [ ] 监控指标正常
- [ ] 告警系统正常
- [ ] 日志收集正常

### 数据完整性验证

```sql
-- 检查关键表记录数
SELECT 'users' AS table_name, COUNT(*) AS count FROM users
UNION ALL
SELECT 'sessions', COUNT(*) FROM sessions
UNION ALL
SELECT 'audit_logs', COUNT(*) FROM audit_logs;

-- 检查最新数据时间戳
SELECT MAX(created_at) AS latest_record FROM audit_logs;

-- 检查数据一致性
SELECT COUNT(*) FROM users WHERE email IS NULL;  -- 应该为 0
```

---

## 🔐 备份策略

### 自动备份

```bash
# Cron 配置
# /etc/crontab

# 每天凌晨 2 点备份数据库
0 2 * * * /opt/cuba-erp/scripts/backup-database.sh

# 每天凌晨 3 点备份 Consul 数据
0 3 * * * /opt/cuba-erp/scripts/backup-consul.sh

# 每周日凌晨 4 点备份配置文件
0 4 * * 0 /opt/cuba-erp/scripts/backup-configs.sh

# 每天凌晨 5 点上传备份到远程存储
0 5 * * * /opt/cuba-erp/scripts/upload-backups.sh
```

### 备份验证

```bash
#!/bin/bash
# scripts/verify-backup.sh

# 验证最新备份文件
LATEST_BACKUP=$(ls -t /backups/postgres/cuba_*.sql.gz | head -1)

# 检查文件完整性
if gunzip -t "$LATEST_BACKUP"; then
    echo "✅ 备份文件完整"
else
    echo "❌ 备份文件损坏"
    exit 1
fi

# 测试恢复到临时数据库
createdb cuba_test
gunzip -c "$LATEST_BACKUP" | psql -d cuba_test

# 验证数据
psql -d cuba_test -c "SELECT COUNT(*) FROM users;"

# 清理
dropdb cuba_test
```

---

## 📞 应急联系流程

### 故障等级

| 等级 | 定义 | 响应时间 | 通知范围 |
|------|------|---------|---------|
| P0 | 核心服务完全不可用 | 立即 | 所有人 |
| P1 | 核心服务部分不可用 | 15 分钟 | 运维团队 + 管理层 |
| P2 | 非核心服务不可用 | 1 小时 | 运维团队 |
| P3 | 性能下降 | 4 小时 | 运维团队 |

### 通知模板

```bash
# P0 故障通知
Subject: 🚨 [P0] Cuba ERP 核心服务故障

故障时间: 2024-01-29 14:30:00
影响范围: 所有用户无法登录
故障原因: 数据库主库宕机
当前状态: 正在切换到从库
预计恢复: 15 分钟

负责人: 张三 (13800138000)
```

---

## 📚 相关文档

- [生产环境检查清单](PRODUCTION_CHECKLIST.md)
- [备份恢复脚本](../scripts/backup-database.sh)
- [监控告警配置](../prometheus/alerts/)
- [PostgreSQL 主从配置](POSTGRES_REPLICATION.md)
