#!/bin/bash
# IAM-Identity 架构重构脚本集

## 概述

这些脚本用于将 IAM-Identity 服务从**多模块 DDD 架构**重构为**统一的单体模块化架构**。

### 架构变更

**重构前**:
```
src/
├── auth/           # 独立的 Auth 模块（完整 DDD 分层）
│   ├── domain/
│   ├── application/
│   ├── infrastructure/
│   └── api/
├── user/           # 独立的 User 模块（完整 DDD 分层）
├── oauth/          # 独立的 OAuth 模块（完整 DDD 分层）
└── shared/         # 共享模块
```

**重构后**:
```
src/
├── domain/         # 统一的领域层
│   ├── auth/       # Auth 实体
│   ├── user/       # User 实体
│   ├── oauth/      # OAuth 实体
│   ├── repositories/
│   ├── services/
│   ├── value_objects/
│   └── events/
├── application/    # 统一的应用层
│   ├── commands/{auth,user,oauth}/
│   ├── queries/{auth,user,oauth}/
│   ├── handlers/{auth,user,oauth}/
│   └── dto/{auth,user,oauth}/
├── infrastructure/ # 统一的基础设施层
│   ├── persistence/{auth,user,oauth}/
│   ├── cache/
│   └── external/
└── api/           # 统一的 API 层
    └── grpc/
```

## 脚本说明

### 1. `run-refactor.sh` - 主脚本（推荐）

**用途**: 自动执行完整的重构流程

**执行步骤**:
1. 检查分支和工作区状态
2. 迁移 Application, Infrastructure, API 层文件
3. 创建所有 mod.rs 文件
4. 更新导入路径
5. 清理旧目录
6. 编译验证

**使用方法**:
```bash
./scripts/run-refactor.sh
```

**注意事项**:
- 必须在 `refactor/monolithic-modular-architecture` 分支上执行
- 会自动创建多个 git commit
- 如果编译失败，可以使用 `git reset --hard HEAD~5` 回滚

---

### 2. `refactor-to-monolithic.sh` - 文件迁移脚本

**用途**: 迁移 Application, Infrastructure, API 层的文件到新位置

**迁移内容**:
- Application 层: Commands, Queries, Handlers, DTOs
- Infrastructure 层: Repository 实现, Cache, Middleware
- API 层: gRPC 服务实现

**使用方法**:
```bash
./scripts/refactor-to-monolithic.sh
```

**输出**: 创建一个 git commit 包含所有文件移动

---

### 3. `create-mod-files.sh` - 模块组织脚本

**用途**: 为新架构创建所有必要的 mod.rs 文件

**创建的文件**:
- Domain 层: 12 个 mod.rs
- Application 层: 13 个 mod.rs
- Infrastructure 层: 5 个 mod.rs
- API 层: 2 个 mod.rs

**使用方法**:
```bash
./scripts/create-mod-files.sh
```

**注意**: 需要在文件迁移后执行

---

### 4. `update-imports.sh` - 导入路径更新脚本

**用途**: 批量更新所有 Rust 文件中的 use 语句

**更新规则**:
```rust
// 旧路径 -> 新路径
use crate::auth::domain::entities::Session
  -> use crate::domain::auth::Session

use crate::auth::application::commands::LoginCommand
  -> use crate::application::commands::auth::LoginCommand

use crate::shared::domain::value_objects::Email
  -> use crate::domain::value_objects::Email
```

**使用方法**:
```bash
./scripts/update-imports.sh
```

**注意**:
- 会自动跳过 proto 生成的文件
- main.rs 可能需要手动调整

---

### 5. `cleanup-old-dirs.sh` - 清理脚本

**用途**: 删除已迁移的旧目录

**删除的目录**:
- `src/auth/`
- `src/user/`
- `src/oauth/`
- `src/shared/`

**使用方法**:
```bash
./scripts/cleanup-old-dirs.sh
```

**警告**:
- 会要求确认
- 删除前确保所有文件已迁移

---

## 完整执行流程

### 方式 A: 一键执行（推荐）

```bash
# 1. 确保在正确的分支
git checkout refactor/monolithic-modular-architecture

# 2. 执行主脚本
./scripts/run-refactor.sh

# 3. 验证结果
cargo test --package iam-identity
cargo run --package iam-identity
```

### 方式 B: 分步执行

```bash
# 1. 确保在正确的分支
git checkout refactor/monolithic-modular-architecture

# 2. 迁移文件
./scripts/refactor-to-monolithic.sh

# 3. 创建 mod.rs
./scripts/create-mod-files.sh
git add -A && git commit -m "refactor: 创建 mod.rs 文件"

# 4. 更新导入
./scripts/update-imports.sh
git add -A && git commit -m "refactor: 更新导入路径"

# 5. 清理旧目录
./scripts/cleanup-old-dirs.sh

# 6. 编译验证
cargo check --package iam-identity
```

---

## 故障排除

### 问题 1: 编译错误 - 找不到模块

**原因**: mod.rs 文件缺少导出

**解决**:
```bash
# 检查对应的 mod.rs 文件
# 确保所有子模块都被正确导出
```

### 问题 2: 编译错误 - 导入路径错误

**原因**: 某些文件的导入路径未更新

**解决**:
```bash
# 手动检查错误文件
# 更新导入路径
# 特别注意 main.rs
```

### 问题 3: Proto 文件找不到

**原因**: api/grpc/mod.rs 中的 proto 路径错误

**解决**:
```bash
# 检查 src/api/grpc/mod.rs
# 确保 include! 路径正确指向旧位置的 proto 文件
```

### 问题 4: 想要回滚

**解决**:
```bash
# 回滚所有重构提交（假设有 5 个提交）
git reset --hard HEAD~5

# 或者回到特定提交
git reset --hard <commit-hash>
```

---

## 验证清单

重构完成后，请验证以下内容：

- [ ] 编译通过: `cargo check --package iam-identity`
- [ ] 测试通过: `cargo test --package iam-identity`
- [ ] 服务启动: `cargo run --package iam-identity`
- [ ] gRPC 端点可用: `grpcurl -plaintext localhost:50051 list`
- [ ] 所有三个服务都注册:
  - [ ] iam.auth.AuthService
  - [ ] iam.user.UserService
  - [ ] iam.oauth.OAuthService

---

## 重构收益

1. **简化架构**: 减少重复的 DDD 分层
2. **统一 CQRS**: 所有模块共享统一的 Command/Query/Handler
3. **减少代码**: 删除重复的 mod.rs 和导入
4. **更易维护**: 清晰的模块边界，统一的代码组织
5. **保留扩展性**: 如需拆分微服务，可基于领域边界拆分

---

## 注意事项

1. **Proto 文件位置**: Proto 生成的文件仍保留在原位置（auth/api/grpc, user/api/grpc, oauth/api/grpc），通过 api/grpc/mod.rs 引用

2. **Git 历史**: 使用 `git mv` 保留文件历史

3. **分步提交**: 每个阶段单独提交，便于回滚

4. **main.rs**: 可能需要手动调整导入路径

5. **测试**: 重构后务必运行完整测试

---

## 联系方式

如有问题，请查看:
- 重构计划文档: `/Users/x/.claude/plans/imperative-gliding-babbage.md`
- 编译日志: `/tmp/refactor-compile.log`
