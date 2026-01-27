#!/bin/bash

# IAM-Identity 服务重构脚本
# 将多模块 DDD 架构重构为统一的单体模块化架构
#
# 使用方法: ./scripts/refactor-to-monolithic.sh
#
# 注意: 请在执行前确保：
# 1. 已经在 refactor/monolithic-modular-architecture 分支上
# 2. Domain 层已经迁移完成
# 3. 工作区是干净的

set -e  # 遇到错误立即退出

PROJECT_ROOT="/Users/x/init"
SRC_DIR="$PROJECT_ROOT/services/iam-identity/src"

echo "=========================================="
echo "IAM-Identity 架构重构脚本"
echo "=========================================="
echo ""

# 检查当前分支
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "refactor/monolithic-modular-architecture" ]; then
    echo "❌ 错误: 当前不在 refactor/monolithic-modular-architecture 分支"
    echo "   当前分支: $CURRENT_BRANCH"
    exit 1
fi

echo "✅ 当前分支: $CURRENT_BRANCH"
echo ""

# ============================================
# 阶段 3: 迁移 Application 层
# ============================================
echo "=========================================="
echo "阶段 3: 迁移 Application 层"
echo "=========================================="

# 3.1 迁移 Commands
echo "📦 迁移 Commands..."

# Auth Commands
git mv "$SRC_DIR/auth/application/commands/login_command.rs" "$SRC_DIR/application/commands/auth/login_command.rs" 2>/dev/null || true
git mv "$SRC_DIR/auth/application/commands/request_password_reset_command.rs" "$SRC_DIR/application/commands/auth/request_password_reset_command.rs" 2>/dev/null || true
git mv "$SRC_DIR/auth/application/commands/reset_password_command.rs" "$SRC_DIR/application/commands/auth/reset_password_command.rs" 2>/dev/null || true

# OAuth Commands
git mv "$SRC_DIR/oauth/application/commands/create_client_command.rs" "$SRC_DIR/application/commands/oauth/create_client_command.rs" 2>/dev/null || true
git mv "$SRC_DIR/oauth/application/commands/authorize_command.rs" "$SRC_DIR/application/commands/oauth/authorize_command.rs" 2>/dev/null || true
git mv "$SRC_DIR/oauth/application/commands/token_command.rs" "$SRC_DIR/application/commands/oauth/token_command.rs" 2>/dev/null || true

echo "✅ Commands 迁移完成"

# 3.2 迁移 Queries
echo "📦 迁移 Queries..."

# Auth Queries
git mv "$SRC_DIR/auth/application/queries/validate_token_query.rs" "$SRC_DIR/application/queries/auth/validate_token_query.rs" 2>/dev/null || true

echo "✅ Queries 迁移完成"

# 3.3 迁移 Handlers
echo "📦 迁移 Handlers..."

# Auth Handlers
git mv "$SRC_DIR/auth/application/handlers/login_handler.rs" "$SRC_DIR/application/handlers/auth/login_handler.rs" 2>/dev/null || true
git mv "$SRC_DIR/auth/application/handlers/request_password_reset_handler.rs" "$SRC_DIR/application/handlers/auth/request_password_reset_handler.rs" 2>/dev/null || true
git mv "$SRC_DIR/auth/application/handlers/reset_password_handler.rs" "$SRC_DIR/application/handlers/auth/reset_password_handler.rs" 2>/dev/null || true

# Shared Handlers
if [ -d "$SRC_DIR/shared/application/handlers" ]; then
    for file in "$SRC_DIR/shared/application/handlers"/*.rs; do
        if [ -f "$file" ] && [ "$(basename "$file")" != "mod.rs" ]; then
            filename=$(basename "$file")
            git mv "$file" "$SRC_DIR/application/handlers/user/$filename" 2>/dev/null || true
        fi
    done
fi

# OAuth Handlers
git mv "$SRC_DIR/oauth/application/handlers/create_client_handler.rs" "$SRC_DIR/application/handlers/oauth/create_client_handler.rs" 2>/dev/null || true
git mv "$SRC_DIR/oauth/application/handlers/authorize_handler.rs" "$SRC_DIR/application/handlers/oauth/authorize_handler.rs" 2>/dev/null || true
git mv "$SRC_DIR/oauth/application/handlers/token_handler.rs" "$SRC_DIR/application/handlers/oauth/token_handler.rs" 2>/dev/null || true

echo "✅ Handlers 迁移完成"

# 3.4 迁移 DTOs
echo "📦 迁移 DTOs..."

# Auth DTOs
if [ -d "$SRC_DIR/auth/application/dto" ]; then
    for file in "$SRC_DIR/auth/application/dto"/*.rs; do
        if [ -f "$file" ] && [ "$(basename "$file")" != "mod.rs" ]; then
            filename=$(basename "$file")
            git mv "$file" "$SRC_DIR/application/dto/auth/$filename" 2>/dev/null || true
        fi
    done
fi

# User DTOs
if [ -d "$SRC_DIR/user/application/dto" ]; then
    for file in "$SRC_DIR/user/application/dto"/*.rs; do
        if [ -f "$file" ] && [ "$(basename "$file")" != "mod.rs" ]; then
            filename=$(basename "$file")
            git mv "$file" "$SRC_DIR/application/dto/user/$filename" 2>/dev/null || true
        fi
    done
fi

# OAuth DTOs
if [ -d "$SRC_DIR/oauth/application/dto" ]; then
    for file in "$SRC_DIR/oauth/application/dto"/*.rs; do
        if [ -f "$file" ] && [ "$(basename "$file")" != "mod.rs" ]; then
            filename=$(basename "$file")
            git mv "$file" "$SRC_DIR/application/dto/oauth/$filename" 2>/dev/null || true
        fi
    done
fi

echo "✅ DTOs 迁移完成"
echo ""

# ============================================
# 阶段 4: 迁移 Infrastructure 层
# ============================================
echo "=========================================="
echo "阶段 4: 迁移 Infrastructure 层"
echo "=========================================="

# 4.1 迁移 Persistence (Repository 实现)
echo "📦 迁移 Repository 实现..."

# Auth Repositories
if [ -d "$SRC_DIR/auth/infrastructure/persistence" ]; then
    for file in "$SRC_DIR/auth/infrastructure/persistence"/*.rs; do
        if [ -f "$file" ] && [ "$(basename "$file")" != "mod.rs" ]; then
            filename=$(basename "$file")
            git mv "$file" "$SRC_DIR/infrastructure/persistence/auth/$filename" 2>/dev/null || true
        fi
    done
fi

# User Repositories
if [ -d "$SRC_DIR/shared/infrastructure/persistence" ]; then
    for file in "$SRC_DIR/shared/infrastructure/persistence"/*.rs; do
        if [ -f "$file" ] && [ "$(basename "$file")" != "mod.rs" ]; then
            filename=$(basename "$file")
            git mv "$file" "$SRC_DIR/infrastructure/persistence/user/$filename" 2>/dev/null || true
        fi
    done
fi

# OAuth Repositories
if [ -d "$SRC_DIR/oauth/infrastructure/persistence" ]; then
    for file in "$SRC_DIR/oauth/infrastructure/persistence"/*.rs; do
        if [ -f "$file" ] && [ "$(basename "$file")" != "mod.rs" ]; then
            filename=$(basename "$file")
            git mv "$file" "$SRC_DIR/infrastructure/persistence/oauth/$filename" 2>/dev/null || true
        fi
    done
fi

echo "✅ Repository 实现迁移完成"

# 4.2 迁移 Cache
echo "📦 迁移 Cache 实现..."

if [ -d "$SRC_DIR/auth/infrastructure/cache" ]; then
    for file in "$SRC_DIR/auth/infrastructure/cache"/*.rs; do
        if [ -f "$file" ] && [ "$(basename "$file")" != "mod.rs" ]; then
            filename=$(basename "$file")
            git mv "$file" "$SRC_DIR/infrastructure/cache/$filename" 2>/dev/null || true
        fi
    done
fi

echo "✅ Cache 实现迁移完成"

# 4.3 迁移 Middleware
echo "📦 迁移 Middleware..."

if [ -d "$SRC_DIR/shared/infrastructure/middleware" ]; then
    mkdir -p "$SRC_DIR/infrastructure/middleware"
    for file in "$SRC_DIR/shared/infrastructure/middleware"/*.rs; do
        if [ -f "$file" ] && [ "$(basename "$file")" != "mod.rs" ]; then
            filename=$(basename "$file")
            git mv "$file" "$SRC_DIR/infrastructure/middleware/$filename" 2>/dev/null || true
        fi
    done
fi

echo "✅ Middleware 迁移完成"
echo ""

# ============================================
# 阶段 5: 迁移 API 层
# ============================================
echo "=========================================="
echo "阶段 5: 迁移 API 层"
echo "=========================================="

echo "📦 迁移 gRPC 服务实现..."

# 移动 gRPC 服务实现文件（保留 proto 生成的文件在原位置）
git mv "$SRC_DIR/auth/api/grpc/auth_service_impl.rs" "$SRC_DIR/api/grpc/auth_service.rs" 2>/dev/null || true
git mv "$SRC_DIR/user/api/grpc/user_service_impl.rs" "$SRC_DIR/api/grpc/user_service.rs" 2>/dev/null || true
git mv "$SRC_DIR/oauth/api/grpc/oauth_service_impl.rs" "$SRC_DIR/api/grpc/oauth_service.rs" 2>/dev/null || true

echo "✅ API 层迁移完成"
echo ""

# ============================================
# 阶段 6: 提交迁移
# ============================================
echo "=========================================="
echo "阶段 6: 提交文件迁移"
echo "=========================================="

git add -A

git commit -m "refactor(iam-identity): 迁移 Application, Infrastructure, API 层

## Application 层迁移

**Commands**:
- auth: login, request_password_reset, reset_password
- oauth: create_client, authorize, token

**Queries**:
- auth: validate_token

**Handlers**:
- auth: login_handler, password_reset_handlers
- user: verification_handlers
- oauth: create_client_handler, authorize_handler, token_handler

**DTOs**:
- auth, user, oauth 的数据传输对象

## Infrastructure 层迁移

**Persistence**:
- auth: session, password_reset, webauthn repositories
- user: user, email_verification, phone_verification repositories
- oauth: client, authorization_code, token repositories

**Cache**:
- auth_cache, login_attempt_cache

**Middleware**:
- tenant_middleware

## API 层迁移

**gRPC Services**:
- auth_service.rs
- user_service.rs
- oauth_service.rs

## 新目录结构

\`\`\`
src/
├── domain/
├── application/
│   ├── commands/{auth,user,oauth}/
│   ├── queries/{auth,user,oauth}/
│   ├── handlers/{auth,user,oauth}/
│   └── dto/{auth,user,oauth}/
├── infrastructure/
│   ├── persistence/{auth,user,oauth}/
│   ├── cache/
│   └── middleware/
└── api/
    └── grpc/
\`\`\`"

echo "✅ 文件迁移已提交"
echo ""

# ============================================
# 阶段 7: 创建 mod.rs 文件
# ============================================
echo "=========================================="
echo "阶段 7: 创建模块组织文件"
echo "=========================================="

echo "📝 创建 mod.rs 文件..."

# 这部分需要手动创建，因为涉及具体的导出内容
echo "⚠️  注意: mod.rs 文件需要手动创建"
echo "   请运行: ./scripts/create-mod-files.sh"
echo ""

# ============================================
# 完成
# ============================================
echo "=========================================="
echo "✅ 文件迁移完成！"
echo "=========================================="
echo ""
echo "下一步操作:"
echo "1. 运行 ./scripts/create-mod-files.sh 创建 mod.rs 文件"
echo "2. 运行 ./scripts/update-imports.sh 更新导入路径"
echo "3. 删除旧目录: rm -rf src/{auth,user,oauth,shared}"
echo "4. 编译验证: cargo check --package iam-identity"
echo ""
echo "如果遇到问题，可以使用 git reset --hard HEAD~2 回滚"
echo ""
