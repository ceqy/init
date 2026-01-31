# ERP ERP 项目启动检查清单

## 前置条件检查

### 1. 环境依赖
```bash
# 检查 Rust 版本
rustc --version  # 需要 1.70+

# 检查 Docker
docker --version
docker compose version

# 检查 PostgreSQL 客户端（可选）
psql --version

# 检查 Redis 客户端（可选）
redis-cli --version
```

### 2. 启动基础设施
```bash
# 启动 PostgreSQL、Redis、Kafka 等
just infra-up

# 验证服务状态
docker compose -f deploy/docker/docker-compose.yml ps

# 预期输出：
# - postgres: Up (5432)
# - redis: Up (6379)
# - kafka: Up (9092)
# - zookeeper: Up (2181)
```

### 3. 数据库迁移
```bash
# 运行 IAM Identity 服务的数据库迁移
just migrate iam-identity

# 验证迁移成功
psql postgres://postgres:postgres@localhost:5432/cuba -c "\dt"

# 预期看到的表：
# - users
# - sessions
# - backup_codes
# - password_reset_tokens
# - webauthn_credentials
# - tenants
# - login_logs
# - email_verifications
# - phone_verifications
# - oauth_clients
# - authorization_codes
# - access_tokens
# - refresh_tokens
```

## 启动服务

### 方式 1：分别启动（推荐开发）

#### 终端 1：启动 IAM Identity 服务
```bash
just iam

# 预期输出：
# ✓ PostgreSQL connected
# ✓ Redis connected
# ✓ WebAuthn service initialized
# ✓ Server listening on 127.0.0.1:50051
# ✓ Health check server on 127.0.0.1:51051
```

#### 终端 2：启动 Gateway（如果需要）
```bash
just dev

# 预期输出：
# ✓ Gateway listening on 0.0.0.0:8080
```

### 方式 2：查看所有可用命令
```bash
just --list

# 或
just
```

## 服务端口

| 服务 | 端口 | 协议 | 说明 |
|------|------|------|------|
| IAM Identity | 50051 | gRPC | 身份认证服务 |
| IAM Health | 51051 | HTTP | 健康检查 (/health, /ready, /metrics) |
| Gateway | 8080 | HTTP | API 网关 |
| PostgreSQL | 5432 | TCP | 数据库 |
| Redis | 6379 | TCP | 缓存 |
| Kafka | 9092 | TCP | 消息队列 |

## 健康检查

### IAM Identity 服务
```bash
# 存活检查
curl http://localhost:51051/health
# 预期：200 OK

# 就绪检查
curl http://localhost:51051/ready
# 预期：200 OK（所有依赖正常）

# Prometheus Metrics
curl http://localhost:51051/metrics
# 预期：返回 metrics 数据
```

### 数据库连接
```bash
# 测试 PostgreSQL
psql postgres://postgres:postgres@localhost:5432/cuba -c "SELECT 1"

# 测试 Redis
redis-cli ping
# 预期：PONG
```

## 常见问题排查

### 问题 1：WebAuthn 配置错误
```
ERROR: rp_id is not an effective_domain of rp_origin
```

**解决方案**：检查 `services/iam-identity/config/development.toml`
```toml
[webauthn]
rp_id = "localhost"
rp_origin = "http://localhost:3000"
```

### 问题 2：数据库连接失败
```
ERROR: Failed to connect to PostgreSQL
```

**解决方案**：
```bash
# 检查 PostgreSQL 是否运行
docker compose -f deploy/docker/docker-compose.yml ps postgres

# 重启 PostgreSQL
docker compose -f deploy/docker/docker-compose.yml restart postgres
```

### 问题 3：Redis 连接失败
```
ERROR: Failed to connect to Redis
```

**解决方案**：
```bash
# 检查 Redis 是否运行
docker compose -f deploy/docker/docker-compose.yml ps redis

# 重启 Redis
docker compose -f deploy/docker/docker-compose.yml restart redis
```

### 问题 4：端口被占用
```
ERROR: Address already in use (os error 48)
```

**解决方案**：
```bash
# 查找占用端口的进程
lsof -i :50051

# 杀死进程
kill -9 <PID>
```

### 问题 5：OpenSSL 编译错误（macOS）
```
ERROR: Could not find directory of OpenSSL installation
```

**解决方案**：
```bash
# 安装 OpenSSL
brew install openssl

# 设置环境变量
export OPENSSL_DIR=$(brew --prefix openssl)
```

## 开发工作流

### 1. 代码检查
```bash
# 格式化代码
just fmt

# 运行 Clippy
just lint

# 类型检查
just check
```

### 2. 运行测试
```bash
# 运行所有测试
just test

# 运行特定服务的测试
cargo test -p iam-identity

# 运行集成测试
cargo test -p iam-identity --test integration

# 运行单元测试
cargo test -p iam-identity --lib
```

### 3. 构建项目
```bash
# Debug 构建
just build

# Release 构建
just build-release
```

## Just 命令速查

| 命令 | 说明 |
|------|------|
| `just` | 显示所有可用命令 |
| `just iam` | 启动 IAM Identity 服务 |
| `just dev` | 启动 Gateway |
| `just infra-up` | 启动基础设施（Docker） |
| `just infra-down` | 停止基础设施 |
| `just migrate <service>` | 运行数据库迁移 |
| `just build` | 构建所有服务 |
| `just test` | 运行所有测试 |
| `just check` | 代码检查 |
| `just fmt` | 格式化代码 |
| `just lint` | Clippy 检查 |
| `just clean` | 清理构建产物 |

## 完整启动流程

```bash
# 1. 启动基础设施
just infra-up

# 2. 等待服务就绪（约 10 秒）
sleep 10

# 3. 运行数据库迁移
just migrate iam-identity

# 4. 启动 IAM Identity 服务
just iam

# 在另一个终端：
# 5. 启动 Gateway（可选）
just dev
```

## 停止服务

```bash
# 停止 Rust 服务
# Ctrl+C 在运行服务的终端

# 停止基础设施
just infra-down

# 清理所有 Docker 资源（谨慎使用）
docker compose -f deploy/docker/docker-compose.yml down -v
```

## 日志级别调整

编辑 `services/iam-identity/config/development.toml`：

```toml
[telemetry]
log_level = "debug"  # trace, debug, info, warn, error
```

- `trace`: 最详细（包括所有 SQL 查询）
- `debug`: 调试信息（包括连接池状态）
- `info`: 一般信息（推荐生产环境）
- `warn`: 警告信息
- `error`: 仅错误信息

## 监控和调试

### 查看 Metrics
```bash
# Prometheus 格式的 metrics
curl http://localhost:51051/metrics

# 关键指标：
# - postgres_pool_size
# - postgres_pool_idle
# - postgres_pool_active
# - redis_connection_status
# - grpc_requests_total
# - db_queries_total
```

### 查看日志
```bash
# 实时查看日志
just iam 2>&1 | tee iam.log

# 过滤特定级别
just iam 2>&1 | grep ERROR
```

## 下一步

- 阅读 [开发指南](docs/guides/development.md)
- 查看 [API 文档](docs/api/README.md)
- 了解 [多租户实现](docs/guides/multi-tenancy.md)
- 学习 [安全最佳实践](docs/guides/security.md)
