#!/bin/bash
# 手动运行的安全检查脚本

echo "🔍 扫描仓库中的敏感信息..."
echo ""

# 检查是否安装了必要工具
if ! command -v rg &> /dev/null; then
    echo "⚠️  未找到 ripgrep。安装: brew install ripgrep"
    echo "   回退到 grep..."
    GREP_CMD="grep -r"
else
    GREP_CMD="rg"
fi

FOUND=0

# 检查私有 IP 地址
echo "检查私有 IP 地址..."
if $GREP_CMD -E "10\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}:[0-9]+" \
    --glob '!.env.local' \
    --glob '!target/*' \
    --glob '!.git/*' \
    . 2>/dev/null; then
    FOUND=1
fi

# 检查 UUID 格式凭证
echo "检查 UUID 格式凭证..."
if $GREP_CMD -E "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}" \
    --glob '!.env.local' \
    --glob '!target/*' \
    --glob '!.git/*' \
    --glob '!Cargo.lock' \
    . 2>/dev/null; then
    FOUND=1
fi

# 检查密码模式
echo "检查硬编码密码..."
if $GREP_CMD -iE "(password|secret|token)\s*[:=]\s*['\"][^'\"]{8,}['\"]" \
    --glob '!.env.local' \
    --glob '!target/*' \
    --glob '!.git/*' \
    --glob '!*.md' \
    . 2>/dev/null; then
    FOUND=1
fi

echo ""
if [ $FOUND -eq 1 ]; then
    echo "❌ 发现潜在敏感信息！"
    echo ""
    echo "需要采取的行动："
    echo "  1. 检查上述发现"
    echo "  2. 将真实值替换为占位符"
    echo "  3. 将凭证移至 .env.local"
    exit 1
else
    echo "✅ 未发现明显的敏感信息"
    exit 0
fi
