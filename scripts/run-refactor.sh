#!/bin/bash

# IAM-Identity 架构重构主脚本
# 协调执行所有重构步骤

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=========================================="
echo "IAM-Identity 架构重构"
echo "从多模块 DDD 到单体模块化架构"
echo "=========================================="
echo ""

# 检查是否在正确的分支
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "refactor/monolithic-modular-architecture" ]; then
    echo "❌ 错误: 当前不在 refactor/monolithic-modular-architecture 分支"
    echo "   当前分支: $CURRENT_BRANCH"
    exit 1
fi

echo "✅ 当前分支: $CURRENT_BRANCH"
echo ""

# 检查工作区状态
if [ -n "$(git status --porcelain)" ]; then
    echo "⚠️  警告: 工作区有未提交的更改"
    git status --short
    echo ""
    read -p "是否继续? (yes/no): " confirm
    if [ "$confirm" != "yes" ]; then
        echo "❌ 取消操作"
        exit 1
    fi
fi

echo "=========================================="
echo "重构步骤概览"
echo "=========================================="
echo ""
echo "步骤 1: 迁移 Application, Infrastructure, API 层"
echo "步骤 2: 创建所有 mod.rs 文件"
echo "步骤 3: 更新导入路径"
echo "步骤 4: 清理旧目录"
echo "步骤 5: 编译验证"
echo ""
read -p "开始执行? (yes/no): " start
if [ "$start" != "yes" ]; then
    echo "❌ 取消操作"
    exit 1
fi

echo ""
echo "=========================================="
echo "步骤 1: 迁移文件"
echo "=========================================="
bash "$SCRIPT_DIR/refactor-to-monolithic.sh"

echo ""
echo "=========================================="
echo "步骤 2: 创建 mod.rs 文件"
echo "=========================================="
bash "$SCRIPT_DIR/create-mod-files.sh"

# 提交 mod.rs 文件
git add -A
git commit -m "refactor(iam-identity): 创建统一架构的 mod.rs 文件

为新的统一架构创建所有必要的模块组织文件:

**Domain 层**:
- domain/auth/mod.rs
- domain/user/mod.rs
- domain/oauth/mod.rs
- domain/repositories/{auth,user,oauth}/mod.rs
- domain/services/{auth,user,oauth}/mod.rs
- domain/value_objects/mod.rs
- domain/events/mod.rs

**Application 层**:
- application/commands/{auth,user,oauth}/mod.rs
- application/queries/{auth,user,oauth}/mod.rs
- application/handlers/{auth,user,oauth}/mod.rs
- application/dto/{auth,user,oauth}/mod.rs

**Infrastructure 层**:
- infrastructure/persistence/{auth,user,oauth}/mod.rs
- infrastructure/cache/mod.rs
- infrastructure/external/mod.rs

**API 层**:
- api/grpc/mod.rs (包含 proto 模块引用)"

echo ""
echo "=========================================="
echo "步骤 3: 更新导入路径"
echo "=========================================="
bash "$SCRIPT_DIR/update-imports.sh"

# 提交导入路径更新
git add -A
git commit -m "refactor(iam-identity): 更新所有导入路径到新架构

批量更新所有文件中的 use 语句:

**旧路径** -> **新路径**:
- auth::domain::entities -> domain::auth
- auth::domain::repositories -> domain::repositories::auth
- auth::application::commands -> application::commands::auth
- user::domain::entities -> domain::user
- oauth::domain::entities -> domain::oauth
- shared::domain::value_objects -> domain::value_objects
- shared::domain::repositories -> domain::repositories::user

更新了 lib.rs 以反映新的模块结构。"

echo ""
echo "=========================================="
echo "步骤 4: 清理旧目录"
echo "=========================================="
bash "$SCRIPT_DIR/cleanup-old-dirs.sh"

echo ""
echo "=========================================="
echo "步骤 5: 编译验证"
echo "=========================================="
echo ""
echo "🔨 开始编译..."

cd /Users/x/init

if cargo check --package iam-identity 2>&1 | tee /tmp/refactor-compile.log; then
    echo ""
    echo "✅ 编译成功！"
else
    echo ""
    echo "❌ 编译失败"
    echo ""
    echo "错误日志已保存到: /tmp/refactor-compile.log"
    echo ""
    echo "常见问题:"
    echo "1. 导入路径未完全更新 - 检查 main.rs"
    echo "2. mod.rs 文件缺少导出 - 检查各层的 mod.rs"
    echo "3. Proto 文件路径错误 - 检查 api/grpc/mod.rs"
    echo ""
    echo "可以使用以下命令回滚:"
    echo "  git reset --hard HEAD~5"
    echo ""
    exit 1
fi

echo ""
echo "=========================================="
echo "✅ 重构完成！"
echo "=========================================="
echo ""
echo "📊 重构统计:"
git log --oneline HEAD~5..HEAD
echo ""
echo "📁 新目录结构:"
tree -L 2 -d services/iam-identity/src 2>/dev/null || find services/iam-identity/src -type d -maxdepth 2 | sort
echo ""
echo "🎉 IAM-Identity 已成功重构为单体模块化架构！"
echo ""
echo "下一步:"
echo "1. 运行测试: cargo test --package iam-identity"
echo "2. 启动服务: cargo run --package iam-identity"
echo "3. 验证 gRPC 端点: grpcurl -plaintext localhost:50051 list"
echo ""
