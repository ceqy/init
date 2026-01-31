#!/bin/bash

# 更新导入路径的脚本
# 这个脚本会批量更新所有文件中的 use 语句

set -e

PROJECT_ROOT="/Users/x/init"
SRC_DIR="$PROJECT_ROOT/services/iam-identity/src"

echo "=========================================="
echo "更新导入路径"
echo "=========================================="
echo ""

echo "📝 更新所有 Rust 文件中的导入路径..."

# 查找所有 .rs 文件并更新导入路径
find "$SRC_DIR" -name "*.rs" -type f | while read -r file; do
    # 跳过 proto 生成的文件
    if [[ "$file" == *"/iam."* ]]; then
        continue
    fi

    # 备份原文件
    cp "$file" "$file.bak"

    # 更新导入路径
    sed -i '' \
        -e 's|use crate::auth::domain::entities::|use crate::domain::auth::|g' \
        -e 's|use crate::auth::domain::repositories::|use crate::domain::repositories::auth::|g' \
        -e 's|use crate::auth::domain::services::|use crate::domain::services::auth::|g' \
        -e 's|use crate::auth::domain::events::|use crate::domain::events::|g' \
        -e 's|use crate::auth::application::commands::|use crate::application::commands::auth::|g' \
        -e 's|use crate::auth::application::queries::|use crate::application::queries::auth::|g' \
        -e 's|use crate::auth::application::handlers::|use crate::application::handlers::auth::|g' \
        -e 's|use crate::auth::application::dto::|use crate::application::dto::auth::|g' \
        -e 's|use crate::auth::infrastructure::persistence::|use crate::infrastructure::persistence::auth::|g' \
        -e 's|use crate::auth::infrastructure::cache::|use crate::infrastructure::cache::|g' \
        -e 's|use crate::user::domain::entities::|use crate::domain::user::|g' \
        -e 's|use crate::user::domain::events::|use crate::domain::events::|g' \
        -e 's|use crate::user::application::dto::|use crate::application::dto::user::|g' \
        -e 's|use crate::oauth::domain::entities::|use crate::domain::oauth::|g' \
        -e 's|use crate::oauth::domain::repositories::|use crate::domain::repositories::oauth::|g' \
        -e 's|use crate::oauth::domain::services::|use crate::domain::services::oauth::|g' \
        -e 's|use crate::oauth::application::commands::|use crate::application::commands::oauth::|g' \
        -e 's|use crate::oauth::application::handlers::|use crate::application::handlers::oauth::|g' \
        -e 's|use crate::oauth::application::dto::|use crate::application::dto::oauth::|g' \
        -e 's|use crate::oauth::infrastructure::persistence::|use crate::infrastructure::persistence::oauth::|g' \
        -e 's|use crate::shared::domain::entities::|use crate::domain::user::|g' \
        -e 's|use crate::shared::domain::value_objects::|use crate::domain::value_objects::|g' \
        -e 's|use crate::shared::domain::repositories::|use crate::domain::repositories::user::|g' \
        -e 's|use crate::shared::domain::services::|use crate::domain::services::user::|g' \
        -e 's|use crate::shared::application::handlers::|use crate::application::handlers::user::|g' \
        -e 's|use crate::shared::infrastructure::persistence::|use crate::infrastructure::persistence::user::|g' \
        -e 's|use crate::shared::infrastructure::middleware::|use crate::infrastructure::middleware::|g' \
        "$file"

    # 如果文件没有变化，恢复备份
    if diff -q "$file" "$file.bak" > /dev/null 2>&1; then
        mv "$file.bak" "$file"
    else
        rm "$file.bak"
        echo "  ✓ 更新: $(basename "$file")"
    fi
done

echo ""
echo "✅ 导入路径更新完成"
echo ""

# 更新 lib.rs
echo "📝 更新 lib.rs..."

cat > "$SRC_DIR/lib.rs" << 'EOF'
//! IAM Identity Service Library
//!
//! 统一的单体模块化架构：
//! - `domain`: 领域层（实体、值对象、仓储接口、领域服务、事件）
//! - `application`: 应用层（命令、查询、处理器、DTO）
//! - `infrastructure`: 基础设施层（持久化、缓存、外部服务）
//! - `api`: API 层（gRPC 服务）

pub mod api;
pub mod application;
pub mod config;
pub mod domain;
pub mod error;
pub mod infrastructure;
EOF

echo "✅ lib.rs 更新完成"
echo ""

# 更新 main.rs 的导入
echo "📝 更新 main.rs 导入..."

# 这部分需要手动调整，因为 main.rs 的结构比较复杂
echo "⚠️  注意: main.rs 需要手动更新导入路径"
echo "   主要更新:"
echo "   - use auth::... -> use crate::domain::..., use crate::application::..."
echo "   - use user::... -> use crate::domain::..., use crate::application::..."
echo "   - use oauth::... -> use crate::domain::..., use crate::application::..."
echo "   - use shared::... -> use crate::domain::..."
echo ""

echo "=========================================="
echo "✅ 导入路径更新完成！"
echo "=========================================="
echo ""
