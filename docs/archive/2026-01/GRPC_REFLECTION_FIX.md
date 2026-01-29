# gRPC 反射 API 修复

## 问题

使用 `grpcurl` 测试时出现错误：
```
Error: server does not support the reflection API
```

## 原因

gRPC 服务默认不启用反射 API。反射 API 允许客户端（如 grpcurl、Postman）动态发现服务定义，无需提前知道 proto 文件。

## 解决方案

### 1. 添加依赖

**文件**: `services/iam-identity/Cargo.toml`

```toml
[dependencies]
tonic-reflection = "0.12"
```

### 2. 更新 build.rs

**文件**: `services/iam-identity/build.rs`

为每个 proto 文件生成文件描述符集：

```rust
fn main() {
    // 编译 auth.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path("src/auth/api/grpc/auth_descriptor.bin")
        .out_dir("src/auth/api/grpc")
        .compile_protos(&["../../proto/iam/auth.proto"], &["../../proto"])
        .expect("Failed to compile auth.proto");

    // 编译 user.proto
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path("src/user/api/grpc/user_descriptor.bin")
        .out_dir("src/user/api/grpc")
        .compile_protos(&["../../proto/iam/user.proto"], &["../../proto"])
        .expect("Failed to compile user.proto");
}
```

### 3. 导出文件描述符

**文件**: `services/iam-identity/src/auth/api/grpc/mod.rs`

```rust
pub mod proto {
    tonic::include_proto!("cuba.iam.auth");
    
    pub const FILE_DESCRIPTOR_SET: &[u8] = 
        include_bytes!("auth_descriptor.bin");
}
```

**文件**: `services/iam-identity/src/user/api/grpc/mod.rs`

```rust
pub mod proto {
    tonic::include_proto!("cuba.iam.user");
    
    pub const FILE_DESCRIPTOR_SET: &[u8] = 
        include_bytes!("user_descriptor.bin");
}
```

### 4. 注册反射服务

**文件**: `services/iam-identity/src/main.rs`

```rust
use tonic_reflection::server::Builder as ReflectionBuilder;

// 在 main 函数中
let reflection_service = ReflectionBuilder::configure()
    .register_encoded_file_descriptor_set(auth::api::grpc::proto::FILE_DESCRIPTOR_SET)
    .register_encoded_file_descriptor_set(user::api::grpc::proto::FILE_DESCRIPTOR_SET)
    .build()
    .map_err(|e| cuba_errors::AppError::internal(format!("Failed to build reflection service: {}", e)))?;

server
    .add_service(AuthServiceServer::new(auth_service))
    .add_service(UserServiceServer::new(user_service))
    .add_service(reflection_service)  // 添加反射服务
    .serve_with_shutdown(addr, cuba_bootstrap::shutdown_signal())
    .await
```

## 重新构建

```bash
# 清理旧的构建产物
cargo clean -p iam-identity

# 重新构建（会生成 descriptor.bin 文件）
cargo build -p iam-identity

# 启动服务
just iam
```

## 验证

### 1. 列出所有服务
```bash
grpcurl -plaintext localhost:50051 list
```

预期输出：
```
cuba.iam.auth.AuthService
cuba.iam.user.UserService
grpc.reflection.v1alpha.ServerReflection
```

### 2. 列出服务方法
```bash
grpcurl -plaintext localhost:50051 list cuba.iam.auth.AuthService
```

预期输出：
```
cuba.iam.auth.AuthService.Login
cuba.iam.auth.AuthService.Logout
cuba.iam.auth.AuthService.RefreshToken
cuba.iam.auth.AuthService.ValidateToken
cuba.iam.auth.AuthService.RequestPasswordReset
cuba.iam.auth.AuthService.ResetPassword
cuba.iam.auth.AuthService.EnableTOTP
cuba.iam.auth.AuthService.VerifyTOTP
cuba.iam.auth.AuthService.DisableTOTP
cuba.iam.auth.AuthService.GenerateBackupCodes
cuba.iam.auth.AuthService.VerifyBackupCode
cuba.iam.auth.AuthService.RegisterWebAuthn
cuba.iam.auth.AuthService.AuthenticateWebAuthn
```

### 3. 查看方法详情
```bash
grpcurl -plaintext localhost:50051 describe cuba.iam.auth.AuthService.Login
```

### 4. 测试登录接口
```bash
grpcurl -plaintext \
  -d '{"username":"test","password":"pass","ip_address":"127.0.0.1","user_agent":"grpcurl"}' \
  -H 'tenant-id: tenant-123' \
  localhost:50051 \
  cuba.iam.auth.AuthService/Login
```

## 反射 API 的好处

1. **动态发现**: 客户端无需 proto 文件即可发现服务
2. **开发调试**: 使用 grpcurl、Postman 等工具测试
3. **文档生成**: 自动生成 API 文档
4. **客户端生成**: 动态生成客户端代码

## 生产环境注意事项

在生产环境中，可以选择禁用反射 API 以提高安全性：

```rust
#[cfg(debug_assertions)]
let reflection_service = ReflectionBuilder::configure()
    .register_encoded_file_descriptor_set(auth::api::grpc::proto::FILE_DESCRIPTOR_SET)
    .register_encoded_file_descriptor_set(user::api::grpc::proto::FILE_DESCRIPTOR_SET)
    .build()?;

let mut server_builder = server
    .add_service(AuthServiceServer::new(auth_service))
    .add_service(UserServiceServer::new(user_service));

#[cfg(debug_assertions)]
{
    server_builder = server_builder.add_service(reflection_service);
}

server_builder
    .serve_with_shutdown(addr, cuba_bootstrap::shutdown_signal())
    .await?;
```

这样反射服务只在 debug 模式下启用。

## 相关文档

- [tonic-reflection 文档](https://docs.rs/tonic-reflection/)
- [gRPC 反射协议](https://github.com/grpc/grpc/blob/master/doc/server-reflection.md)
- [grpcurl 使用指南](https://github.com/fullstorydev/grpcurl)
