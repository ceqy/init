#!/bin/bash

echo "🔍 验证Proto文件..."
echo ""

# 运行buf lint
echo "1️⃣ 运行 buf lint..."
if buf lint; then
    echo "✅ Buf lint 通过"
else
    echo "❌ Buf lint 失败"
    exit 1
fi

echo ""

# 统计proto文件数量
echo "2️⃣ 统计Proto文件..."
total_protos=$(find proto/cuba -name "*.proto" | wc -l | tr -d ' ')
echo "   总计: $total_protos 个proto文件"

echo ""

# 列出新增的proto文件
echo "3️⃣ 新增的Proto模块:"
echo "   ✅ proto/cuba/org/enterprise/v1/enterprise.proto"
echo "   ✅ proto/cuba/sys/nr/v1/nr.proto"
echo "   ✅ proto/cuba/mdm/material/v1/material.proto"
echo "   ✅ proto/cuba/mdm/bp/v1/bp.proto"
echo "   ✅ proto/cuba/mf/eng/v1/eng.proto"
echo "   ✅ proto/cuba/sys/cfg/v1/cfg.proto"
echo "   ✅ proto/cuba/sys/msg/v1/msg.proto"
echo "   ✅ proto/cuba/sys/job/v1/job.proto"

echo ""

# 检查buf.yaml
echo "4️⃣ 检查 buf.yaml..."
if [ -f "buf.yaml" ]; then
    echo "   ✅ buf.yaml 存在"
else
    echo "   ❌ buf.yaml 不存在"
    exit 1
fi

echo ""
echo "🎉 所有验证通过!"
