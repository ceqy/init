#!/bin/bash
# Envoy + Consul 架构快速启动脚本

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Cuba ERP - Envoy + Consul 架构部署${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# 检查 Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}错误: Docker 未安装${NC}"
    exit 1
fi

# 检查 Docker Compose
if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}错误: Docker Compose 未安装${NC}"
    exit 1
fi

# 检查 .env 文件
if [ ! -f "../../.env" ]; then
    echo -e "${YELLOW}警告: .env 文件不存在，正在创建...${NC}"
    cp ../../.env.example ../../.env

    # 生成 JWT_SECRET
    JWT_SECRET=$(openssl rand -base64 32)
    echo "JWT_SECRET=${JWT_SECRET}" >> ../../.env
    echo -e "${GREEN}✓ 已生成 JWT_SECRET${NC}"
fi

# 进入 docker 目录
cd "$(dirname "$0")"

echo -e "${YELLOW}步骤 1/5: 停止旧容器...${NC}"
docker-compose -f docker-compose.envoy.yml down 2>/dev/null || true

echo -e "${YELLOW}步骤 2/5: 拉取镜像...${NC}"
docker-compose -f docker-compose.envoy.yml pull

echo -e "${YELLOW}步骤 3/5: 启动基础设施服务...${NC}"
docker-compose -f docker-compose.envoy.yml up -d consul postgres redis kafka clickhouse

echo -e "${YELLOW}等待基础设施服务就绪...${NC}"
sleep 10

# 检查 Consul 健康状态
echo -n "等待 Consul 启动..."
for i in {1..30}; do
    if curl -s http://localhost:8500/v1/status/leader | grep -q ":"; then
        echo -e " ${GREEN}✓${NC}"
        break
    fi
    echo -n "."
    sleep 1
done

# 检查 PostgreSQL 健康状态
echo -n "等待 PostgreSQL 启动..."
for i in {1..30}; do
    if docker exec cuba-postgres pg_isready -U postgres &>/dev/null; then
        echo -e " ${GREEN}✓${NC}"
        break
    fi
    echo -n "."
    sleep 1
done

echo -e "${YELLOW}步骤 4/5: 启动应用服务...${NC}"
docker-compose -f docker-compose.envoy.yml up -d iam-access iam-access-envoy gateway gateway-envoy

echo -e "${YELLOW}等待应用服务就绪...${NC}"
sleep 5

# 注册服务到 Consul
echo -e "${YELLOW}步骤 5/5: 注册服务到 Consul...${NC}"
docker-compose -f docker-compose.envoy.yml up consul-registrator

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}部署完成！${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# 验证部署
echo -e "${YELLOW}验证部署状态...${NC}"
echo ""

# 检查 Consul 服务
echo -n "Consul UI: "
if curl -s http://localhost:8500/v1/status/leader &>/dev/null; then
    echo -e "${GREEN}✓ http://localhost:8500${NC}"
else
    echo -e "${RED}✗ 不可用${NC}"
fi

# 检查 Gateway Envoy
echo -n "Gateway Envoy Admin: "
if curl -s http://localhost:9901/ready &>/dev/null; then
    echo -e "${GREEN}✓ http://localhost:9901${NC}"
else
    echo -e "${RED}✗ 不可用${NC}"
fi

# 检查 IAM Envoy
echo -n "IAM Envoy Admin: "
if curl -s http://localhost:9902/ready &>/dev/null; then
    echo -e "${GREEN}✓ http://localhost:9902${NC}"
else
    echo -e "${RED}✗ 不可用${NC}"
fi

# 检查 Prometheus
echo -n "Prometheus: "
if curl -s http://localhost:9090/-/healthy &>/dev/null; then
    echo -e "${GREEN}✓ http://localhost:9090${NC}"
else
    echo -e "${YELLOW}○ 未启动（可选）${NC}"
fi

# 检查 Grafana
echo -n "Grafana: "
if curl -s http://localhost:3001/api/health &>/dev/null; then
    echo -e "${GREEN}✓ http://localhost:3001 (admin/admin)${NC}"
else
    echo -e "${YELLOW}○ 未启动（可选）${NC}"
fi

echo ""
echo -e "${YELLOW}查看服务注册状态:${NC}"
sleep 2
SERVICES=$(curl -s http://localhost:8500/v1/catalog/services | jq -r 'keys[]' 2>/dev/null || echo "")
if [ -n "$SERVICES" ]; then
    echo "$SERVICES" | while read service; do
        HEALTH=$(curl -s "http://localhost:8500/v1/health/service/${service}" | jq -r '.[0].Checks[].Status' 2>/dev/null | head -1)
        if [ "$HEALTH" = "passing" ]; then
            echo -e "  ${GREEN}✓${NC} $service"
        else
            echo -e "  ${RED}✗${NC} $service"
        fi
    done
else
    echo -e "  ${YELLOW}无法获取服务列表${NC}"
fi

echo ""
echo -e "${YELLOW}常用命令:${NC}"
echo "  查看日志:     docker-compose -f docker-compose.envoy.yml logs -f [service]"
echo "  停止服务:     docker-compose -f docker-compose.envoy.yml down"
echo "  重启服务:     docker-compose -f docker-compose.envoy.yml restart [service]"
echo "  查看状态:     docker-compose -f docker-compose.envoy.yml ps"
echo ""
echo -e "${YELLOW}测试 API:${NC}"
echo "  健康检查:     curl http://localhost:8080/health"
echo "  登录测试:     curl -X POST http://localhost:8080/api/auth/login \\"
echo "                  -H 'Content-Type: application/json' \\"
echo "                  -d '{\"email\":\"admin@example.com\",\"password\":\"password123\"}'"
echo ""
echo -e "${GREEN}部署成功！请查看 deploy/ENVOY_DEPLOYMENT_GUIDE.md 了解更多信息。${NC}"
