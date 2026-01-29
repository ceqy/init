# 生产环境 TLS/mTLS 配置指南

## 📋 概述

本指南介绍如何为 Envoy + Consul 架构启用 TLS 加密，实现：
- 服务间通信加密（mTLS）
- Consul 通信加密
- 证书管理和自动轮换

## 🔐 证书生成

### 1. 生成 CA 证书

```bash
#!/bin/bash
# scripts/generate-certs.sh

set -e

CERT_DIR="deploy/certs"
mkdir -p "$CERT_DIR"

# 生成 CA 私钥
openssl genrsa -out "$CERT_DIR/ca-key.pem" 4096

# 生成 CA 证书
openssl req -new -x509 -days 3650 -key "$CERT_DIR/ca-key.pem" \
  -out "$CERT_DIR/ca-cert.pem" \
  -subj "/C=CN/ST=Beijing/L=Beijing/O=Cuba ERP/OU=IT/CN=Cuba CA"

echo "✓ CA 证书生成完成"
```

### 2. 生成服务证书

```bash
# 为每个服务生成证书
generate_service_cert() {
  SERVICE=$1

  # 生成私钥
  openssl genrsa -out "$CERT_DIR/${SERVICE}-key.pem" 2048

  # 生成 CSR
  openssl req -new -key "$CERT_DIR/${SERVICE}-key.pem" \
    -out "$CERT_DIR/${SERVICE}.csr" \
    -subj "/C=CN/ST=Beijing/L=Beijing/O=Cuba ERP/OU=IT/CN=${SERVICE}"

  # 签发证书
  openssl x509 -req -days 365 \
    -in "$CERT_DIR/${SERVICE}.csr" \
    -CA "$CERT_DIR/ca-cert.pem" \
    -CAkey "$CERT_DIR/ca-key.pem" \
    -CAcreateserial \
    -out "$CERT_DIR/${SERVICE}-cert.pem" \
    -extfile <(printf "subjectAltName=DNS:${SERVICE},DNS:localhost,IP:127.0.0.1")

  # 清理 CSR
  rm "$CERT_DIR/${SERVICE}.csr"

  echo "✓ ${SERVICE} 证书生成完成"
}

# 生成各服务证书
generate_service_cert "gateway-envoy"
generate_service_cert "iam-access-envoy"
generate_service_cert "consul"
```

### 3. 使用 cert-manager（Kubernetes 环境）

```yaml
# deploy/k8s/cert-manager/issuer.yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: cuba-ca-issuer
spec:
  ca:
    secretName: cuba-ca-secret
---
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: gateway-envoy-cert
  namespace: cuba
spec:
  secretName: gateway-envoy-tls
  issuerRef:
    name: cuba-ca-issuer
    kind: ClusterIssuer
  dnsNames:
    - gateway-envoy
    - gateway-envoy.cuba.svc.cluster.local
  duration: 2160h # 90 天
  renewBefore: 360h # 提前 15 天续期
```

## 🔧 Envoy TLS 配置

### Gateway Envoy（出站 mTLS）

```yaml
# deploy/envoy/gateway-envoy-tls.yaml
static_resources:
  clusters:
    - name: iam_cluster
      type: STRICT_DNS
      lb_policy: ROUND_ROBIN

      # 启用 TLS
      transport_socket:
        name: envoy.transport_sockets.tls
        typed_config:
          "@type": type.googleapis.com/envoy.extensions.transport_sockets.tls.v3.UpstreamTlsContext
          common_tls_context:
            # 客户端证书（mTLS）
            tls_certificates:
              - certificate_chain:
                  filename: /etc/envoy/certs/gateway-envoy-cert.pem
                private_key:
                  filename: /etc/envoy/certs/gateway-envoy-key.pem

            # 验证服务端证书
            validation_context:
              trusted_ca:
                filename: /etc/envoy/certs/ca-cert.pem
              match_subject_alt_names:
                - exact: "iam-access-envoy"

          # SNI 配置
          sni: iam-access-envoy

      load_assignment:
        cluster_name: iam_cluster
        endpoints:
          - lb_endpoints:
              - endpoint:
                  address:
                    socket_address:
                      address: iam-access-envoy
                      port_value: 50051
```

### IAM Envoy（入站 mTLS）

```yaml
# deploy/envoy/iam-envoy-tls.yaml
static_resources:
  listeners:
    - name: iam_inbound
      address:
        socket_address:
          address: 0.0.0.0
          port_value: 50051

      filter_chains:
        - filters:
            - name: envoy.filters.network.http_connection_manager
              typed_config:
                "@type": type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager
                stat_prefix: iam_inbound
                codec_type: AUTO
                route_config:
                  name: local_route
                  virtual_hosts:
                    - name: iam_backend
                      domains: ["*"]
                      routes:
                        - match:
                            prefix: "/"
                          route:
                            cluster: iam_local
                http_filters:
                  - name: envoy.filters.http.router
                    typed_config:
                      "@type": type.googleapis.com/envoy.extensions.filters.http.router.v3.Router

          # TLS 配置
          transport_socket:
            name: envoy.transport_sockets.tls
            typed_config:
              "@type": type.googleapis.com/envoy.extensions.transport_sockets.tls.v3.DownstreamTlsContext
              common_tls_context:
                # 服务端证书
                tls_certificates:
                  - certificate_chain:
                      filename: /etc/envoy/certs/iam-access-envoy-cert.pem
                    private_key:
                      filename: /etc/envoy/certs/iam-access-envoy-key.pem

                # 验证客户端证书（mTLS）
                validation_context:
                  trusted_ca:
                    filename: /etc/envoy/certs/ca-cert.pem

              # 要求客户端证书
              require_client_certificate: true
```

## 🏛️ Consul TLS 配置

### Consul 服务器配置

```json
{
  "datacenter": "dc1",
  "data_dir": "/consul/data",
  "log_level": "INFO",
  "server": true,
  "bootstrap_expect": 1,
  "ui": true,
  "client_addr": "0.0.0.0",
  "bind_addr": "0.0.0.0",

  "ports": {
    "http": -1,
    "https": 8501,
    "grpc": -1,
    "grpc_tls": 8503,
    "dns": 8600
  },

  "tls": {
    "defaults": {
      "ca_file": "/consul/config/certs/ca-cert.pem",
      "cert_file": "/consul/config/certs/consul-cert.pem",
      "key_file": "/consul/config/certs/consul-key.pem",
      "verify_incoming": true,
      "verify_outgoing": true,
      "verify_server_hostname": true
    },
    "internal_rpc": {
      "verify_server_hostname": true
    }
  },

  "connect": {
    "enabled": true,
    "ca_provider": "consul",
    "ca_config": {
      "leaf_cert_ttl": "72h",
      "rotation_period": "2160h"
    }
  }
}
```

## 🐳 Docker Compose TLS 配置

```yaml
# deploy/docker/docker-compose.envoy-tls.yml
version: '3.9'

services:
  # Gateway Envoy with TLS
  gateway-envoy:
    image: envoyproxy/envoy:v1.29-latest
    container_name: cuba-gateway-envoy
    command: ["-c", "/etc/envoy/envoy.yaml", "--log-level", "info"]
    ports:
      - "50053:50051"
      - "9901:9901"
    volumes:
      - ../envoy/gateway-envoy-tls.yaml:/etc/envoy/envoy.yaml:ro
      - ../certs:/etc/envoy/certs:ro  # 挂载证书
    environment:
      - ENVOY_UID=0
    networks:
      - cuba-network

  # IAM Envoy with TLS
  iam-access-envoy:
    image: envoyproxy/envoy:v1.29-latest
    container_name: cuba-iam-access-envoy
    command: ["-c", "/etc/envoy/envoy.yaml", "--log-level", "info"]
    ports:
      - "50051:50051"
      - "9902:9902"
    volumes:
      - ../envoy/iam-envoy-tls.yaml:/etc/envoy/envoy.yaml:ro
      - ../certs:/etc/envoy/certs:ro  # 挂载证书
    environment:
      - ENVOY_UID=0
    networks:
      - cuba-network

  # Consul with TLS
  consul:
    image: hashicorp/consul:1.18
    container_name: cuba-consul
    command: agent -server -ui -bootstrap-expect=1 -config-file=/consul/config/consul-tls-config.json
    ports:
      - "8501:8501"  # HTTPS
      - "8503:8503"  # gRPC TLS
      - "8600:8600/udp"
    volumes:
      - consul_data:/consul/data
      - ../consul/consul-tls-config.json:/consul/config/consul-tls-config.json:ro
      - ../certs:/consul/config/certs:ro  # 挂载证书
    networks:
      - cuba-network

networks:
  cuba-network:
    driver: bridge

volumes:
  consul_data:
```

## 🔄 证书轮换策略

### 自动轮换脚本

```bash
#!/bin/bash
# scripts/rotate-certs.sh

set -e

CERT_DIR="deploy/certs"
BACKUP_DIR="deploy/certs/backup/$(date +%Y%m%d_%H%M%S)"

# 备份旧证书
mkdir -p "$BACKUP_DIR"
cp "$CERT_DIR"/*.pem "$BACKUP_DIR/"

# 生成新证书
./scripts/generate-certs.sh

# 重启服务（滚动更新）
docker-compose -f deploy/docker/docker-compose.envoy-tls.yml restart gateway-envoy
sleep 5
docker-compose -f deploy/docker/docker-compose.envoy-tls.yml restart iam-access-envoy
sleep 5
docker-compose -f deploy/docker/docker-compose.envoy-tls.yml restart consul

echo "✓ 证书轮换完成"
```

### Cron 定时任务

```bash
# 每 60 天自动轮换证书
0 2 1 */2 * /path/to/scripts/rotate-certs.sh >> /var/log/cert-rotation.log 2>&1
```

## ✅ 验证 TLS 配置

### 1. 验证证书有效性

```bash
# 检查证书信息
openssl x509 -in deploy/certs/gateway-envoy-cert.pem -text -noout

# 验证证书链
openssl verify -CAfile deploy/certs/ca-cert.pem deploy/certs/gateway-envoy-cert.pem
```

### 2. 测试 mTLS 连接

```bash
# 使用 curl 测试（需要客户端证书）
curl --cacert deploy/certs/ca-cert.pem \
     --cert deploy/certs/gateway-envoy-cert.pem \
     --key deploy/certs/gateway-envoy-key.pem \
     https://localhost:50051/health

# 使用 grpcurl 测试
grpcurl -cacert deploy/certs/ca-cert.pem \
        -cert deploy/certs/gateway-envoy-cert.pem \
        -key deploy/certs/gateway-envoy-key.pem \
        localhost:50051 list
```

### 3. 检查 Envoy TLS 统计

```bash
# 查看 TLS 握手统计
curl http://localhost:9901/stats | grep ssl

# 查看证书过期时间
curl http://localhost:9901/certs
```

## 🚨 故障排查

### 常见问题

1. **证书验证失败**
   ```
   错误: SSL routines:tls_process_server_certificate:certificate verify failed

   解决: 检查 CA 证书是否正确配置
   ```

2. **SNI 不匹配**
   ```
   错误: SSL routines:tls_process_server_certificate:Hostname mismatch

   解决: 确保证书 SAN 包含正确的主机名
   ```

3. **证书过期**
   ```bash
   # 检查证书有效期
   openssl x509 -in cert.pem -noout -dates

   # 自动续期
   ./scripts/rotate-certs.sh
   ```

## 📚 最佳实践

1. **证书管理**
   - 使用短期证书（90 天）
   - 自动化证书轮换
   - 定期备份私钥

2. **密钥安全**
   - 私钥权限设置为 600
   - 使用硬件安全模块（HSM）存储 CA 私钥
   - 定期审计证书使用

3. **监控告警**
   - 监控证书过期时间
   - 告警提前 30 天通知
   - 记录所有 TLS 握手失败

## 🎯 生产环境检查清单

- [ ] 所有服务间通信启用 TLS
- [ ] 启用 mTLS 双向认证
- [ ] 配置证书自动轮换
- [ ] 设置证书过期告警
- [ ] 备份 CA 私钥到安全位置
- [ ] 测试证书轮换流程
- [ ] 文档化证书管理流程
- [ ] 配置 TLS 监控指标

## 📖 参考资料

- [Envoy TLS 文档](https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/security/ssl)
- [Consul TLS 配置](https://www.consul.io/docs/security/encryption)
- [OpenSSL 证书管理](https://www.openssl.org/docs/man1.1.1/man1/openssl-x509.html)
