#!/bin/bash
# 安装 Git hooks 以防止提交敏感信息

set -e

echo "🔧 设置 Git hooks..."

# 创建 .git/hooks 目录（如果不存在）
mkdir -p .git/hooks

# 复制 pre-commit hook
if [ -f ".githooks/pre-commit" ]; then
    cp .githooks/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo "✅ Pre-commit hook 已安装"
else
    echo "❌ 未找到 .githooks/pre-commit"
    exit 1
fi

echo ""
echo "🎉 Git hooks 设置完成！"
echo ""
echo "Pre-commit hook 将检查："
echo "  - 私有 IP 地址"
echo "  - UUID 凭证"
echo "  - 硬编码的密码/密钥"
echo "  - API 密钥"
echo ""
echo "如需绕过检查（不推荐）："
echo "  git commit --no-verify"
