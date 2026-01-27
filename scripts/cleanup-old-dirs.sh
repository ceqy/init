#!/bin/bash

# 清理旧目录和文件的脚本

set -e

PROJECT_ROOT="/Users/x/init"
SRC_DIR="$PROJECT_ROOT/services/iam-identity/src"

echo "=========================================="
echo "清理旧目录结构"
echo "=========================================="
echo ""

echo "⚠️  警告: 此操作将删除以下目录:"
echo "  - $SRC_DIR/auth"
echo "  - $SRC_DIR/user"
echo "  - $SRC_DIR/oauth"
echo "  - $SRC_DIR/shared"
echo ""
read -p "确认删除? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "❌ 取消操作"
    exit 1
fi

echo ""
echo "🗑️  删除旧目录..."

# 删除旧的模块目录
rm -rf "$SRC_DIR/auth"
rm -rf "$SRC_DIR/user"
rm -rf "$SRC_DIR/oauth"
rm -rf "$SRC_DIR/shared"

echo "✅ 旧目录已删除"
echo ""

# 提交删除
git add -A
git commit -m "refactor(iam-identity): 删除旧的模块目录

删除已迁移的旧目录:
- auth/
- user/
- oauth/
- shared/

所有代码已迁移到新的统一结构:
- domain/
- application/
- infrastructure/
- api/"

echo "✅ 删除已提交"
echo ""

echo "=========================================="
echo "✅ 清理完成！"
echo "=========================================="
echo ""
echo "最终目录结构:"
echo ""
tree -L 2 -d "$SRC_DIR" 2>/dev/null || find "$SRC_DIR" -type d -maxdepth 2 | sort
echo ""
