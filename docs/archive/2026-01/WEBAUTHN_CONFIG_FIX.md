# WebAuthn 配置修复

## 问题描述

启动服务时出现错误：
```
ERROR webauthn_rs: rp_id is not an effective_domain of rp_origin
Error: Internal("Failed to create WebAuthn service: Internal error: Failed to create WebAuthn builder: The configuration was invalid")
```

## 根本原因

WebAuthn 要求 `rp_id` 必须是 `rp_origin` 的有效域名。

**错误配置**：
- `rp_id`: `127.0.0.1`（IP 地址）
- `rp_origin`: `https://127.0.0.1`

IP 地址不是有效的域名，导致 WebAuthn 验证失败。

## 解决方案

### 1. 添加 WebAuthn 配置到 config

**文件**: `crates/config/src/lib.rs`

```rust
/// WebAuthn 配置
#[derive(Debug, Clone, Deserialize)]
pub struct WebAuthnConfig {
    pub rp_id: String,
    pub rp_origin: String,
}

/// 应用配置
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    // ... 其他字段
    pub webauthn: WebAuthnConfig,
}
```

### 2. 更新配置文件

#### default.toml
```toml
[webauthn]
rp_id = "localhost"
rp_origin = "http://localhost:3000"
```

#### development.toml
```toml
[webauthn]
rp_id = "localhost"
rp_origin = "http://localhost:3000"
```

#### production.toml
```toml
[webauthn]
rp_id = "erp.com"
rp_origin = "https://erp.com"
```

### 3. 更新 main.rs

**文件**: `services/iam-identity/src/main.rs`

```rust
// 组装 WebAuthn 服务
let rp_id = config.webauthn.rp_id.clone();
let rp_origin = config.webauthn.rp_origin
    .parse()
    .map_err(|e| cuba_errors::AppError::internal(format!("Invalid RP origin: {}", e)))?;

let webauthn_service = Arc::new(
    WebAuthnService::new(rp_id, rp_origin, webauthn_credential_repo)
        .map_err(|e| cuba_errors::AppError::internal(format!("Failed to create WebAuthn service: {}", e)))?,
);
```

## WebAuthn 配置说明

### rp_id（Relying Party ID）
- 必须是有效的域名（不能是 IP 地址）
- 开发环境：使用 `localhost`
- 生产环境：使用实际域名（如 `erp.com`）

### rp_origin（Relying Party Origin）
- 完整的 URL（包括协议）
- 开发环境：`http://localhost:3000`（前端地址）
- 生产环境：`https://erp.com`

### 域名验证规则
WebAuthn 要求 `rp_id` 必须是 `rp_origin` 的有效域名：
- ✅ `rp_id: localhost`, `rp_origin: http://localhost:3000`
- ✅ `rp_id: erp.com`, `rp_origin: https://erp.com`
- ✅ `rp_id: auth.erp.com`, `rp_origin: https://auth.erp.com`
- ❌ `rp_id: 127.0.0.1`, `rp_origin: https://127.0.0.1`（IP 地址）
- ❌ `rp_id: localhost`, `rp_origin: https://example.com`（域名不匹配）

## 测试验证

启动服务后应该不再出现 WebAuthn 配置错误：

```bash
just iam
```

预期输出：
```
✓ WebAuthn service initialized successfully
✓ Server listening on 127.0.0.1:50051
```

## 生产环境配置

部署到生产环境时，需要更新 `production.toml`：

```toml
[webauthn]
rp_id = "your-domain.com"
rp_origin = "https://your-domain.com"
```

确保：
1. 使用 HTTPS（WebAuthn 要求）
2. 域名与实际部署域名一致
3. 前端应用部署在相同域名下

## 相关文件

- `crates/config/src/lib.rs` - 配置结构定义
- `services/iam-identity/config/default.toml` - 默认配置
- `services/iam-identity/config/development.toml` - 开发环境配置
- `services/iam-identity/config/production.toml` - 生产环境配置
- `services/iam-identity/src/main.rs` - WebAuthn 服务初始化
