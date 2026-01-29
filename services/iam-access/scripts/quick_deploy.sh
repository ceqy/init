#!/bin/bash
# 数据库连接池配置快速部署脚本

set -e

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "=========================================="
echo "数据库连接池配置 - 快速部署"
echo "=========================================="
echo ""

# 检查当前目录
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}错误: 请在 iam-access 服务目录下运行此脚本${NC}"
    exit 1
fi

# 1. 检查配置文件
echo -e "${BLUE}步骤 1/6: 检查配置文件${NC}"
if [ -d "config" ] && [ -f "config/default.toml" ]; then
    echo -e "${GREEN}✓${NC} 配置文件已存在"
else
    echo -e "${YELLOW}⚠${NC} 配置文件不存在，将创建默认配置"
    mkdir -p config
    # 这里可以添加创建配置文件的逻辑
fi
echo ""

# 2. 验证配置
echo -e "${BLUE}步骤 2/6: 验证配置${NC}"
if [ -x "../../scripts/verify_pool_config.sh" ]; then
    ../../scripts/verify_pool_config.sh
else
    echo -e "${YELLOW}⚠${NC} 验证脚本不存在，跳过验证"
fi
echo ""

# 3. 编译检查
echo -e "${BLUE}步骤 3/6: 编译检查${NC}"
if cargo check --quiet 2>/dev/null; then
    echo -e "${GREEN}✓${NC} 编译成功"
else
    echo -e "${RED}✗${NC} 编译失败，请检查代码"
    exit 1
fi
echo ""

# 4. 启动测试环境（可选）
echo -e "${BLUE}步骤 4/6: 启动测试环境${NC}"
read -p "是否启动 Docker 测试环境？(y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -f "docker-compose.pool-test.yml" ]; then
        echo "启动 Docker 服务..."
        docker-compose -f docker-compose.pool-test.yml up -d
        echo "等待服务就绪..."
        sleep 10
        echo -e "${GREEN}✓${NC} 测试环境已启动"
    else
        echo -e "${YELLOW}⚠${NC} docker-compose.pool-test.yml 不存在"
    fi
else
    echo "跳过测试环境启动"
fi
echo ""

# 5. 运行测试
echo -e "${BLUE}步骤 5/6: 运行测试${NC}"
read -p "是否运行压力测试？(y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -x "scripts/stress_test_pool.sh" ]; then
        echo "运行压力测试（30 并发，30 秒）..."
        CONCURRENT_CONNECTIONS=30 TEST_DURATION=30 scripts/stress_test_pool.sh
    else
        echo -e "${YELLOW}⚠${NC} 压力测试脚本不存在"
    fi
else
    echo "跳过压力测试"
fi
echo ""

# 6. 部署检查清单
echo -e "${BLUE}步骤 6/6: 部署检查清单${NC}"
echo ""
echo "请确认以下项目："
echo ""
echo "配置："
echo "  [ ] 连接池大小已根据服务器规格设置"
echo "  [ ] 超时参数已配置"
echo "  [ ] 读写分离已配置（如需要）"
echo ""
echo "监控："
echo "  [ ] Prometheus 已部署"
echo "  [ ] Grafana 仪表板已配置"
echo "  [ ] 告警规则已设置"
echo ""
echo "测试："
echo "  [ ] 压力测试已通过"
echo "  [ ] 健康检查正常"
echo "  [ ] 性能指标符合预期"
echo ""
echo "文档："
echo "  [ ] 配置文档已更新"
echo "  [ ] 运维手册已准备"
echo "  [ ] 团队已培训"
echo ""

# 7. 生成部署报告
echo "=========================================="
echo "部署信息"
echo "=========================================="
echo ""
echo "服务: iam-access"
echo "部署时间: $(date '+%Y-%m-%d %H:%M:%S')"
echo "环境: ${APP_ENV:-development}"
echo ""

if [ -f "config/${APP_ENV:-development}.toml" ]; then
    MAX_CONN=$(grep "max_connections" "config/${APP_ENV:-development}.toml" | awk '{print $3}')
    echo "连接池配置:"
    echo "  最大连接数: $MAX_CONN"

    if grep -q "read_url" "config/${APP_ENV:-development}.toml" 2>/dev/null; then
        READ_CONN=$(grep "read_max_connections" "config/${APP_ENV:-development}.toml" | awk '{print $3}')
        echo "  读库连接数: $READ_CONN"
        echo "  读写分离: 已启用"
    else
        echo "  读写分离: 未启用"
    fi
fi
echo ""

# 8. 下一步
echo "=========================================="
echo "下一步操作"
echo "=========================================="
echo ""
echo "1. 启动服务:"
echo "   cargo run"
echo ""
echo "2. 监控连接池:"
echo "   ./scripts/health_check_pool.sh"
echo ""
echo "3. 查看监控:"
echo "   Grafana: http://localhost:3000"
echo "   Prometheus: http://localhost:9090"
echo ""
echo "4. 查看文档:"
echo "   cat DATABASE_POOL_CONFIG.md"
echo "   cat POOL_SETUP_GUIDE.md"
echo "   cat POOL_FAQ.md"
echo ""

echo -e "${GREEN}部署准备完成！${NC}"
