# API 文档

## 概述

ERP 使用 gRPC 作为服务间通信协议，所有 API 定义使用 Protocol Buffers (proto3)。

## 文档结构

- [IAM 身份服务](./iam/README.md) - 认证、用户管理、OAuth2
- [通用类型](./common.md) - 跨服务共享的消息类型

## gRPC 服务列表

| 服务 | 端口 | 描述 |
|------|------|------|
| IAM Identity | 50051 | 身份认证和用户管理 |
| API Gateway | 8080 | HTTP/REST 网关 |

## 健康检查端点

每个 gRPC 服务在 **端口 + 1000** 上提供 HTTP 健康检查：

| 端点 | 说明 |
|------|------|
| `GET /health` | 存活检查（liveness） |
| `GET /ready` | 就绪检查（readiness） |
| `GET /metrics` | Prometheus metrics |

示例：
```bash
# IAM Identity 服务健康检查
curl http://localhost:51051/health
curl http://localhost:51051/ready
curl http://localhost:51051/metrics
```

## 认证方式

所有 API 调用需要在 gRPC metadata 中携带 JWT token：

```
Authorization: Bearer <access_token>
```

## 错误处理

gRPC 使用标准状态码：

| 状态码 | 说明 |
|--------|------|
| OK (0) | 成功 |
| INVALID_ARGUMENT (3) | 参数错误 |
| UNAUTHENTICATED (16) | 未认证 |
| PERMISSION_DENIED (7) | 权限不足 |
| NOT_FOUND (5) | 资源不存在 |
| ALREADY_EXISTS (6) | 资源已存在 |
| INTERNAL (13) | 内部错误 |

## 生成客户端代码

### Go

```bash
protoc --go_out=. --go-grpc_out=. proto/**/*.proto
```

### Python

```bash
python -m grpc_tools.protoc -I proto --python_out=. --grpc_python_out=. proto/**/*.proto
```

### JavaScript/TypeScript

```bash
protoc --js_out=import_style=commonjs:. --grpc-web_out=import_style=typescript,mode=grpcwebtext:. proto/**/*.proto
```

## 开发工具

### grpcurl - gRPC 命令行工具

```bash
# 列出服务
grpcurl -plaintext localhost:50051 list

# 列出方法
grpcurl -plaintext localhost:50051 list cuba.iam.auth.AuthService

# 调用方法
grpcurl -plaintext -d '{"username":"admin","password":"password","tenant_id":"default"}' \
  localhost:50051 cuba.iam.auth.AuthService/Login
```

### BloomRPC - gRPC GUI 客户端

下载：https://github.com/bloomrpc/bloomrpc

## 相关资源

- [Protocol Buffers 文档](https://protobuf.dev/)
- [gRPC 文档](https://grpc.io/docs/)
- [Buf 工具](https://buf.build/)
