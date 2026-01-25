# 依赖管理规范修复

## 问题描述

在 `services/iam-identity/Cargo.toml` 中，2FA 相关依赖直接指定了版本号，不符合项目的 workspace 依赖管理规范。

### 问题代码
```toml
# 2FA 相关依赖
totp-rs = "5.5"
data-encoding = "2.5"
rand = "0.8"
urlencoding = "2.1"
```

## 解决方案

### 1. 更新根 Cargo.toml

在 `Cargo.toml` 的 `[workspace.dependencies]` 中添加 2FA 依赖：

```toml
# Auth
jsonwebtoken = { version = "9.3" }
argon2 = { version = "0.5" }
sha2 = { version = "0.10" }

# 2FA
totp-rs = { version = "5.5" }
data-encoding = { version = "2.5" }
rand = { version = "0.8" }
urlencoding = { version = "2.1" }
```

### 2. 更新 services/iam-identity/Cargo.toml

使用 workspace 依赖引用：

```toml
# 2FA 相关依赖
totp-rs = { workspace = true }
data-encoding = { workspace = true }
rand = { workspace = true }
urlencoding = { workspace = true }
```

## 优势

### 统一版本管理
- 所有依赖版本在根 `Cargo.toml` 中统一管理
- 避免不同服务使用不同版本导致的冲突
- 便于批量升级依赖版本

### 符合 Workspace 规范
- 遵循 Rust workspace 最佳实践
- 与项目其他依赖管理方式一致
- 提高代码可维护性

### 减少重复
- 版本号只需在一处定义
- 新服务可以直接引用 workspace 依赖
- 减少配置错误的可能性

## 验证

编译检查通过：
```bash
cargo check -p iam-identity
# ✅ 编译成功，只有未使用代码的警告
```

## 相关文件

- `Cargo.toml` - 根 workspace 配置
- `services/iam-identity/Cargo.toml` - IAM Identity 服务配置

## 最佳实践

### 添加新依赖的步骤

1. **在根 Cargo.toml 添加依赖**
   ```toml
   [workspace.dependencies]
   new-crate = { version = "1.0" }
   ```

2. **在服务中引用**
   ```toml
   [dependencies]
   new-crate = { workspace = true }
   ```

3. **如需特定 features**
   ```toml
   [dependencies]
   new-crate = { workspace = true, features = ["extra"] }
   ```

### 不应该做的

❌ **直接在服务中指定版本**
```toml
new-crate = "1.0"  # 错误！
```

✅ **使用 workspace 引用**
```toml
new-crate = { workspace = true }  # 正确！
```

## 总结

依赖管理规范已修复，所有 2FA 相关依赖现在通过 workspace 统一管理，符合项目规范。

---

**修复日期**: 2026-01-26  
**修复人员**: Kiro AI  
**状态**: ✅ 已完成
