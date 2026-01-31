# Envoy + Consul 服务发现与高可用架构 - 部署指南

## 📋 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                        客户端请求                            │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
                  ┌───────────────┐
                  │   Gateway     │ (Axum HTTP Server)
                  │   :8080       │
                  └───────┬───────┘
                          │ localhost:50051
                          ▼
                  ┌───────────────┐
                  │ Gateway Envoy │ (Sidecar)
                  │   :50053      │
                  │   Admin:9901  │
                  └───────┬───────┘
                          │
                          │ 服务发现、负载均衡、熔断
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
        ▼                 ▼                 ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐
│ IAM Envoy 1   │ │ IAM Envoy 2   │ │ IAM Envoy 3   │
│   :50051      │ │   :50051      │ │   :50051      │
│ Admin:9902    │ │ Admin:9903    │ │ Admin:9904    │
└───────┬───────┘ └───────┬───────┘ └───────┬───────┘
        │                 │                 │
        ▼                 ▼                 ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐
│ IAM Service 1 │ │ IAM Service 2 │ │ IAM Service 3 │
│   :50052      │ │   :50052      │ │   :50052      │
└───────────────┘ └───────────────┘ └───────────────┘
        │                 │                 │
        └─────────────────┼─────────────────┘
                          │
                          ▼
                  ┌───────────────┐
                  │    Consul     │ (服务注册中心)
                  │   :8500 UI    │
                  │   :8502 gRPC  │
                  └───────────────┘
```

## 🚀 快速开始

### 1. 环境准备

确保已安装：
- Docker 20.10+
- Docker Compose 2.0+

### 2. 配置环境变量

```bash
# 复制环境变量模板
cp .env.example .env

# 编辑 .env 文件，设置 JWT_SECRET（至少 32 字符）
echo "JWT_SECRET=$(openssl rand -base64 32)" >> .env
```

### 3. 启动服务

```bash
# 启动完整的 Envoy + Consul 架构
cd deploy/docker
docker-compose -f docker-compose.envoy.yml up -d

# 查看服务状态
docker-compose -f docker-compose.envoy.yml ps

# 查看日志
docker-compose -f docker-compose.envoy.yml logs -f gateway-envoy
docker-compose -f docker-compose.envoy.yml logs -f iam-access-envoy
```

### 4. 验证部署

#### 检查 Consul 服务注册
```bash
# 访问 Consul UI
open http://localhost:8500

# 或使用 API 查询
curl http://localhost:8500/v1/catalog/services
curl http://localhost:8500/v1/health/service/iam-access
```

#### 检查 Envoy 状态
```bash
# Gateway Envoy Admin
curl http://localhost:9901/stats
curl http://localhost:9901/clusters

# IAM Envoy Admin
curl http://localhost:9902/stats
curl http://localhost:9902/clusters
```

#### 测试 API 请求
```bash
# 健康检查
curl http://localhost:8080/health

# 登录（通过 Gateway -> Gateway Envoy -> IAM Envoy -> IAM Service）
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "password": "password123"
  }'
```

## 📊 监控与可观测性

### Prometheus 指标

访问 Prometheus UI：
```bash
open http://localhost:9090
```

查询示例：
```promql
# Envoy 请求成功率
rate(envoy_cluster_upstream_rq_completed{envoy_cluster_name="iam_cluster"}[5m])

# Envoy 请求延迟 P99
histogram_quantile(0.99, rate(envoy_cluster_upstream_rq_time_bucket[5m]))

# 熔断器打开次数
envoy_cluster_circuit_breakers_default_cx_open

# 健康检查失败次数
envoy_cluster_health_check_failure
```

### Grafana 仪表盘

访问 Grafana：
```bash
open http://localhost:3001
# 用户名: admin
# 密码: admin
```

预置仪表盘：
- Envoy Global Dashboard
- Envoy Cluster Dashboard
- Service Mesh Overview

### Envoy Admin 接口

**Gateway Envoy (9901):**
```bash
# 查看集群状态
curl http://localhost:9901/clusters

# 查看统计信息
curl http://localhost:9901/stats/prometheus

# 查看配置
curl http://localhost:9901/config_dump

# 查看日志级别
curl http://localhost:9901/logging
```

**IAM Envoy (9902):**
```bash
# 查看监听器
curl http://localhost:9902/listeners

# 查看路由配置
curl http://localhost:9902/config_dump?resource=routes
```

## 🔧 配置说明

### Envoy 关键配置

#### 1. 负载均衡策略
```yaml
# deploy/envoy/gateway-envoy.yaml
clusters:
  - name: iam_cluster
    lb_policy: ROUND_ROBIN  # 可选: LEAST_REQUEST, RANDOM, RING_HASH
```

#### 2. 熔断器配置
```yaml
circuit_breakers:
  thresholds:
    - max_connections: 1000      # 最大连接数
      max_pending_requests: 1000 # 最大等待请求数
      max_requests: 1000         # 最大并发请求数
      max_retries: 3             # 最大重试次数
```

#### 3. 异常检测（自动剔除不健康实例）
```yaml
outlier_detection:
  consecutive_5xx: 5           # 连续 5 次 5xx 错误触发剔除
  interval: 30s                # 检测间隔
  base_ejection_time: 30s      # 剔除时长
  max_ejection_percent: 50     # 最多剔除 50% 实例
```

#### 4. 重试策略
```yaml
retry_policy:
  retry_on: "5xx,reset,connect-failure,refused-stream"
  num_retries: 3
  per_try_timeout: 10s
```

### Consul 服务注册

服务自动注册配置位于 `deploy/consul/services/`：

```json
{
  "service": {
    "name": "iam-access",
    "port": 50051,
    "checks": [
      {
        "grpc": "iam-access-envoy:50051/grpc.health.v1.Health",
        "interval": "10s"
      }
    ]
  }
}
```

## 🎯 高级功能

### 1. 灰度发布（金丝雀部署）

修改 `gateway-envoy.yaml`：

```yaml
routes:
  - match: { prefix: "/" }
    route:
      weighted_clusters:
        clusters:
          - name: iam_cluster_stable
            weight: 90
          - name: iam_cluster_canary
            weight: 10
```

### 2. 基于 Header 的路由

```yaml
routes:
  - match:
      prefix: "/"
      headers:
        - name: "X-Canary"
          exact_match: "true"
    route:
      cluster: iam_cluster_canary
  - match:
      prefix: "/"
    route:
      cluster: iam_cluster_stable
```

### 3. 速率限制

在 `iam-envoy.yaml` 中启用：

```yaml
http_filters:
  - name: envoy.filters.http.local_ratelimit
    typed_config:
      "@type": type.googleapis.com/envoy.extensions.filters.http.local_ratelimit.v3.LocalRateLimit
      stat_prefix: http_local_rate_limiter
      token_bucket:
        max_tokens: 1000
        tokens_per_fill: 1000
        fill_interval: 1s
```

### 4. 动态服务发现（Consul xDS）

取消注释 `gateway-envoy.yaml` 中的 `dynamic_resources` 部分：

```yaml
dynamic_resources:
  cds_config:
    api_config_source:
      api_type: GRPC
      grpc_services:
        - envoy_grpc:
            cluster_name: consul_cluster
```

## 🧪 测试场景

### 场景 1：测试负载均衡

```bash
# 启动 3 个 IAM 实例
docker-compose -f docker-compose.envoy.yml up -d --scale iam-access=3

# 发送多个请求，观察负载分布
for i in {1..10}; do
  curl -s http://localhost:8080/api/auth/health | jq .
done

# 查看 Envoy 统计
curl http://localhost:9901/clusters | grep iam_cluster
```

### 场景 2：测试熔断

```bash
# 停止所有 IAM 实例
docker-compose -f docker-compose.envoy.yml stop iam-access

# 发送请求，观察熔断行为
curl -v http://localhost:8080/api/auth/health

# 查看熔断器状态
curl http://localhost:9901/stats | grep circuit_breakers
```

### 场景 3：测试健康检查

```bash
# 停止一个 IAM 实例
docker stop iam-access

# 等待 Consul 检测到不健康（约 30 秒）
watch -n 1 'curl -s http://localhost:8500/v1/health/service/iam-access | jq'

# 查看 Envoy 是否自动剔除
curl http://localhost:9901/clusters | grep iam_cluster
```

## 🐛 故障排查

### 问题 1：Envoy 无法连接到后端服务

```bash
# 检查 Envoy 日志
docker logs gateway-envoy

# 检查网络连通性
docker exec gateway-envoy ping iam-access-envoy

# 检查 DNS 解析
docker exec gateway-envoy nslookup iam-access-envoy
```

### 问题 2：Consul 服务注册失败

```bash
# 检查 Consul 日志
docker logs consul

# 手动注册服务
curl -X PUT -d @deploy/consul/services/iam-access.json \
  http://localhost:8500/v1/agent/service/register

# 查看注册的服务
curl http://localhost:8500/v1/catalog/services
```

### 问题 3：健康检查失败

```bash
# 检查 IAM 服务健康检查端点
curl http://localhost:8081/health

# 检查 Envoy 健康检查配置
curl http://localhost:9902/config_dump | jq '.configs[] | select(.["@type"] | contains("Cluster"))'
```

### 问题 4：请求超时

```bash
# 查看 Envoy 超时配置
curl http://localhost:9901/config_dump | grep timeout

# 增加超时时间（修改 gateway-envoy.yaml）
route:
  cluster: iam_cluster
  timeout: 60s  # 增加到 60 秒
```

## 📈 性能优化

### 1. 连接池优化

```yaml
# gateway-envoy.yaml
http2_protocol_options:
  initial_stream_window_size: 2097152  # 2MB
  initial_connection_window_size: 2097152  # 2MB
  max_concurrent_streams: 2000
```

### 2. 缓冲区优化

```yaml
per_connection_buffer_limit_bytes: 1048576  # 1MB
```

### 3. 健康检查优化

```yaml
health_checks:
  - timeout: 1s
    interval: 5s  # 减少检查频率以降低开销
    unhealthy_threshold: 3
    healthy_threshold: 2
```

## 🔐 安全加固

### 1. 启用 mTLS（双向 TLS）

```yaml
# 在 Envoy 配置中添加 TLS 上下文
transport_socket:
  name: envoy.transport_sockets.tls
  typed_config:
    "@type": type.googleapis.com/envoy.extensions.transport_sockets.tls.v3.UpstreamTlsContext
    common_tls_context:
      tls_certificates:
        - certificate_chain: { filename: "/etc/envoy/certs/cert.pem" }
          private_key: { filename: "/etc/envoy/certs/key.pem" }
```

### 2. 限制 Admin 接口访问

```yaml
admin:
  address:
    socket_address:
      address: 127.0.0.1  # 只监听本地
      port_value: 9901
```

## 📚 参考资料

- [Envoy 官方文档](https://www.envoyproxy.io/docs/envoy/latest/)
- [Consul 服务发现](https://www.consul.io/docs/discovery)
- [gRPC 健康检查协议](https://github.com/grpc/grpc/blob/master/doc/health-checking.md)
- [Prometheus Envoy 指标](https://www.envoyproxy.io/docs/envoy/latest/configuration/observability/statistics)

## 🆘 获取帮助

如遇问题，请检查：
1. Docker 容器日志：`docker-compose logs -f <service>`
2. Envoy Admin 接口：`http://localhost:9901`
3. Consul UI：`http://localhost:8500`
4. Prometheus 指标：`http://localhost:9090`

## 📝 下一步

- [ ] 配置生产环境的 Consul 集群（3-5 节点）
- [ ] 集成 OpenTelemetry 分布式追踪
- [ ] 配置 Grafana 告警规则
- [ ] 实施 mTLS 加密通信
- [ ] 配置 Envoy 访问日志到 ELK/Loki
