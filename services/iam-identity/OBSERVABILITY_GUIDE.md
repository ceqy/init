# IAM Identity 可观测性指南

## 概述

IAM Identity 服务提供完整的可观测性功能，包括：
- **Metrics（Prometheus）**: 业务指标和系统指标
- **Tracing（OpenTelemetry）**: 分布式追踪
- **Logging**: 结构化日志
- **Health Checks**: 健康检查端点

## Metrics（指标）

### 访问 Metrics

Metrics 通过 HTTP 端点暴露（Prometheus 格式）：

```bash
# 默认端口：gRPC 端口 + 1000
# 如果 gRPC 端口是 50051，则 Metrics 端口是 51051
curl http://localhost:51051/metrics
```

### 业务指标

#### 1. 认证指标

**登录尝试**:
```prometheus
# 总登录尝试次数
iam_login_attempts_total{success="true|false", method="password|webauthn", has_2fa="true|false"}

# 登录成功次数
iam_login_success_total{success="true", method="password", has_2fa="false"}

# 登录失败次数
iam_login_failure_total{success="false", method="password", has_2fa="false"}

# 登录成功率（最近 1 小时）
iam_login_success_rate_percent
```

**示例查询**:
```promql
# 登录成功率
rate(iam_login_success_total[5m]) / rate(iam_login_attempts_total[5m]) * 100

# 登录失败率
rate(iam_login_failure_total[5m]) / rate(iam_login_attempts_total[5m]) * 100
```

#### 2. 2FA 指标

**2FA 验证**:
```prometheus
# 2FA 验证总数
iam_2fa_verifications_total{success="true|false", method="totp|backup_code"}

# 2FA 成功次数
iam_2fa_success_total{success="true", method="totp"}

# 2FA 失败次数
iam_2fa_failure_total{success="false", method="totp"}

# 启用 2FA 的用户数
iam_2fa_enabled_users

# 2FA 使用率
iam_2fa_usage_rate_percent
```

**示例查询**:
```promql
# 2FA 使用率
iam_2fa_enabled_users / iam_active_users * 100

# 2FA 验证成功率
rate(iam_2fa_success_total[5m]) / rate(iam_2fa_verifications_total[5m]) * 100
```

#### 3. 会话指标

```prometheus
# 会话创建总数
iam_sessions_created_total

# 会话撤销总数
iam_sessions_revoked_total{reason="logout|expired|admin"}

# 当前活跃会话数
iam_active_sessions

# 过期会话清理数
iam_sessions_expired_total
```

#### 4. 账户安全指标

```prometheus
# 账户锁定次数
iam_account_locked_total{reason="brute_force|admin"}

# 账户解锁次数
iam_account_unlocked_total

# 可疑登录检测次数
iam_suspicious_login_detected_total{reason="new_location|new_device|unusual_time"}
```

#### 5. 密码重置指标

```prometheus
# 密码重置请求次数
iam_password_reset_requested_total

# 密码重置完成次数
iam_password_reset_completed_total{success="true|false"}

# 过期令牌清理数
iam_password_reset_tokens_expired_total
```

#### 6. WebAuthn 指标

```prometheus
# WebAuthn 注册次数
iam_webauthn_registrations_total{success="true|false"}

# WebAuthn 认证次数
iam_webauthn_authentications_total{success="true|false"}

# WebAuthn 凭证总数
iam_webauthn_credentials
```

#### 7. OAuth2 指标

```prometheus
# OAuth2 授权次数
iam_oauth_authorizations_total{grant_type="authorization_code|client_credentials", success="true|false"}

# Token 颁发次数
iam_oauth_tokens_issued_total{token_type="access_token|refresh_token"}

# Token 撤销次数
iam_oauth_tokens_revoked_total{token_type="access_token|refresh_token"}

# 活跃 OAuth Client 数量
iam_oauth_active_clients
```

#### 8. 用户指标

```prometheus
# 用户注册次数
iam_users_registered_total

# 用户激活次数
iam_users_activated_total

# 用户停用次数
iam_users_deactivated_total

# 总用户数
iam_total_users

# 活跃用户数
iam_active_users
```

#### 9. 租户指标

```prometheus
# 租户创建次数
iam_tenants_created_total

# 总租户数
iam_total_tenants

# 活跃租户数
iam_active_tenants
```

### 性能指标

#### API 响应时间

```prometheus
# API 请求总数
iam_api_requests_total{service="auth", method="Login", status="ok|error"}

# API 请求响应时间（毫秒）
iam_api_request_duration_ms{service="auth", method="Login", status="ok"}
```

**示例查询**:
```promql
# P95 响应时间
histogram_quantile(0.95, rate(iam_api_request_duration_ms_bucket[5m]))

# P99 响应时间
histogram_quantile(0.99, rate(iam_api_request_duration_ms_bucket[5m]))

# 平均响应时间
rate(iam_api_request_duration_ms_sum[5m]) / rate(iam_api_request_duration_ms_count[5m])
```

#### 数据库查询时间

```prometheus
# 数据库查询总数
iam_db_queries_total{operation="select|insert|update|delete", table="users", success="true|false"}

# 数据库查询时间（毫秒）
iam_db_query_duration_ms{operation="select", table="users", success="true"}
```

**示例查询**:
```promql
# 慢查询（> 100ms）
iam_db_query_duration_ms > 100

# 查询错误率
rate(iam_db_queries_total{success="false"}[5m]) / rate(iam_db_queries_total[5m]) * 100
```

### 系统指标

#### PostgreSQL 连接池

```prometheus
# 连接池大小
postgres_pool_size{pool="main"}

# 空闲连接数
postgres_pool_idle{pool="main"}

# 活跃连接数
postgres_pool_active{pool="main"}
```

#### Redis 连接状态

```prometheus
# Redis 连接状态（1=connected, 0=disconnected）
redis_connection_status
```

## Tracing（追踪）

### 分布式追踪

服务使用 OpenTelemetry 进行分布式追踪，支持导出到 Jaeger。

#### Span 类型

1. **认证 Span** (`auth`):
   - `operation`: 操作类型（login, logout, verify_2fa）
   - `user_id`: 用户 ID
   - `tenant_id`: 租户 ID
   - `success`: 是否成功
   - `error`: 错误信息（如果失败）

2. **用户操作 Span** (`user`):
   - `operation`: 操作类型（create, update, delete, activate）
   - `user_id`: 用户 ID
   - `tenant_id`: 租户 ID
   - `success`: 是否成功

3. **会话操作 Span** (`session`):
   - `operation`: 操作类型（create, revoke, refresh）
   - `session_id`: 会话 ID
   - `user_id`: 用户 ID
   - `tenant_id`: 租户 ID

4. **OAuth2 操作 Span** (`oauth`):
   - `operation`: 操作类型（authorize, token, revoke）
   - `client_id`: Client ID
   - `grant_type`: 授权类型
   - `scope`: 权限范围
   - `success`: 是否成功

5. **数据库操作 Span** (`db`):
   - `operation`: 操作类型（select, insert, update, delete）
   - `table`: 表名
   - `duration_ms`: 执行时间
   - `success`: 是否成功

6. **缓存操作 Span** (`cache`):
   - `operation`: 操作类型（get, set, delete）
   - `key`: 缓存键
   - `hit`: 是否命中

### 使用示例

```rust
use crate::infrastructure::observability::tracing::*;

// 创建认证 Span
let span = auth_span("login");
let _enter = span.enter();

// 记录字段
span.record("user_id", &user_id.to_string());
span.record("tenant_id", &tenant_id.to_string());
span.record("success", &true);

// 记录日志
log_auth_success(&user_id.to_string(), &tenant_id.to_string(), "password");
```

## Logging（日志）

### 结构化日志

服务使用结构化日志，所有日志都包含上下文信息。

#### 日志级别

- **ERROR**: 错误和异常
- **WARN**: 警告和可疑活动
- **INFO**: 重要业务事件
- **DEBUG**: 调试信息
- **TRACE**: 详细追踪信息

#### 日志格式

**开发环境**（人类可读）:
```
2026-01-26T10:30:45.123Z INFO auth: Authentication successful user_id="123" tenant_id="456" method="password"
```

**生产环境**（JSON 格式）:
```json
{
  "timestamp": "2026-01-26T10:30:45.123Z",
  "level": "INFO",
  "target": "auth",
  "message": "Authentication successful",
  "user_id": "123",
  "tenant_id": "456",
  "method": "password"
}
```

#### 重要日志事件

1. **认证事件**:
   - 登录成功/失败
   - 2FA 验证成功/失败
   - 账户锁定
   - 可疑登录检测

2. **安全事件**:
   - 密码重置
   - 权限变更
   - 会话撤销
   - OAuth2 授权

3. **错误事件**:
   - 数据库错误
   - 缓存错误
   - 外部服务错误

### 日志级别管理

通过环境变量 `RUST_LOG` 控制日志级别：

```bash
# 全局 INFO 级别
export RUST_LOG=info

# 特定模块 DEBUG 级别
export RUST_LOG=iam_identity=debug,sqlx=warn

# 生产环境推荐配置
export RUST_LOG=iam_identity=info,sqlx=warn,tower_http=info
```

## Health Checks（健康检查）

### 健康检查端点

服务提供 HTTP 健康检查端点（端口 = gRPC 端口 + 1000）：

#### 1. Liveness 检查

**端点**: `GET /health`

**用途**: Kubernetes liveness probe

**响应**:
```json
{
  "status": "healthy",
  "checks": []
}
```

**状态码**: 始终返回 `200 OK`

#### 2. Readiness 检查

**端点**: `GET /ready`

**用途**: Kubernetes readiness probe

**响应**（健康）:
```json
{
  "status": "healthy",
  "checks": [
    {
      "name": "postgres",
      "status": "healthy"
    },
    {
      "name": "redis",
      "status": "healthy"
    }
  ]
}
```

**响应**（不健康）:
```json
{
  "status": "unhealthy",
  "checks": [
    {
      "name": "postgres",
      "status": "healthy"
    },
    {
      "name": "redis",
      "status": "unhealthy",
      "message": "Connection refused"
    }
  ]
}
```

**状态码**: 
- `200 OK`: 所有依赖健康
- `503 Service Unavailable`: 至少一个依赖不健康

### Kubernetes 配置示例

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: iam-identity
spec:
  containers:
  - name: iam-identity
    image: iam-identity:latest
    ports:
    - containerPort: 50051  # gRPC
    - containerPort: 51051  # Health checks
    livenessProbe:
      httpGet:
        path: /health
        port: 51051
      initialDelaySeconds: 10
      periodSeconds: 10
    readinessProbe:
      httpGet:
        path: /ready
        port: 51051
      initialDelaySeconds: 5
      periodSeconds: 5
```

## Grafana Dashboard

### 推荐的 Dashboard 面板

#### 1. 认证概览

- 登录成功率（时间序列）
- 登录尝试次数（计数器）
- 2FA 使用率（仪表盘）
- 账户锁定次数（时间序列）

#### 2. 性能监控

- API 响应时间（P50, P95, P99）
- 数据库查询时间（P50, P95, P99）
- 慢查询列表（表格）
- 错误率（时间序列）

#### 3. 用户活动

- 活跃用户数（仪表盘）
- 活跃会话数（仪表盘）
- 用户注册趋势（时间序列）
- 租户分布（饼图）

#### 4. 系统健康

- PostgreSQL 连接池状态
- Redis 连接状态
- 服务可用性
- 资源使用率

### Prometheus 告警规则

```yaml
groups:
- name: iam_identity_alerts
  rules:
  # 登录失败率过高
  - alert: HighLoginFailureRate
    expr: rate(iam_login_failure_total[5m]) / rate(iam_login_attempts_total[5m]) > 0.5
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High login failure rate"
      description: "Login failure rate is {{ $value | humanizePercentage }}"

  # API 响应时间过长
  - alert: SlowAPIResponse
    expr: histogram_quantile(0.95, rate(iam_api_request_duration_ms_bucket[5m])) > 1000
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Slow API response time"
      description: "P95 response time is {{ $value }}ms"

  # 数据库连接池耗尽
  - alert: DatabasePoolExhausted
    expr: postgres_pool_idle{pool="main"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Database connection pool exhausted"
      description: "No idle connections available"

  # Redis 连接断开
  - alert: RedisDisconnected
    expr: redis_connection_status == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Redis connection lost"
      description: "Redis is not connected"

  # 可疑登录激增
  - alert: SuspiciousLoginSpike
    expr: rate(iam_suspicious_login_detected_total[5m]) > 10
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Suspicious login spike detected"
      description: "{{ $value }} suspicious logins per second"
```

## 最佳实践

### 1. Metrics 采集

- 使用 Counter 记录事件总数
- 使用 Gauge 记录当前状态
- 使用 Histogram 记录分布（响应时间、查询时间）
- 添加有意义的标签（但不要过多）

### 2. Tracing

- 为每个重要操作创建 Span
- 记录关键字段（user_id, tenant_id, operation）
- 记录错误信息
- 避免在 Span 中记录敏感信息

### 3. Logging

- 使用结构化日志
- 包含上下文信息（user_id, tenant_id, request_id）
- 不记录敏感信息（密码、Token）
- 使用适当的日志级别

### 4. 性能优化

- 定期清理过期数据
- 监控慢查询并优化
- 使用缓存减少数据库压力
- 设置合理的连接池大小

## 故障排查

### 常见问题

#### 1. Metrics 不更新

**检查**:
```bash
# 检查 Metrics 端点是否可访问
curl http://localhost:51051/metrics

# 检查 Metrics 采集器是否运行
# 查看日志中是否有 "Metrics collected successfully"
```

#### 2. 健康检查失败

**检查**:
```bash
# 检查 readiness 端点
curl http://localhost:51051/ready

# 检查数据库连接
psql -h localhost -U postgres -d cuba -c "SELECT 1"

# 检查 Redis 连接
redis-cli ping
```

#### 3. 日志级别过高

**调整**:
```bash
# 临时调整（重启后失效）
export RUST_LOG=info

# 永久调整（修改配置文件）
# config/production.toml
[telemetry]
log_level = "info"
```

## 总结

IAM Identity 服务提供了完整的可观测性功能：

✅ **Metrics**: 30+ 业务指标和系统指标  
✅ **Tracing**: 分布式追踪和性能分析  
✅ **Logging**: 结构化日志和日志级别管理  
✅ **Health Checks**: Liveness 和 Readiness 检查  

通过这些功能，您可以：
- 实时监控服务健康状态
- 分析用户行为和业务趋势
- 快速定位和解决问题
- 优化系统性能
