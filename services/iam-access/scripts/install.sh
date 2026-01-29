#!/bin/bash
# 数据库连接池配置 - 一键安装脚本
# 用于快速设置完整的连接池配置环境

set -e

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo ""
echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                                                            ║${NC}"
echo -e "${CYAN}║        数据库连接池配置 - 一键安装                        ║${NC}"
echo -e "${CYAN}║        Database Connection Pool - Quick Setup             ║${NC}"
echo -e "${CYAN}║                                                            ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# 检查当前目录
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}错误: 请在 iam-access 服务目录下运行此脚本${NC}"
    exit 1
fi

# 显示安装选项
echo -e "${BLUE}请选择安装模式:${NC}"
echo ""
echo "  1) 完整安装 (配置 + Docker + 监控 + 工具)"
echo "  2) 仅配置文件"
echo "  3) 仅 Docker 环境"
echo "  4) 仅监控配置"
echo "  5) 仅工具脚本"
echo ""
read -p "请输入选项 (1-5): " INSTALL_MODE

case $INSTALL_MODE in
    1)
        INSTALL_CONFIG=true
        INSTALL_DOCKER=true
        INSTALL_MONITORING=true
        INSTALL_SCRIPTS=true
        ;;
    2)
        INSTALL_CONFIG=true
        ;;
    3)
        INSTALL_DOCKER=true
        ;;
    4)
        INSTALL_MONITORING=true
        ;;
    5)
        INSTALL_SCRIPTS=true
        ;;
    *)
        echo -e "${RED}无效的选项${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${BLUE}开始安装...${NC}"
echo ""

# 1. 安装配置文件
if [ "$INSTALL_CONFIG" = true ]; then
    echo -e "${CYAN}[1/4] 安装配置文件...${NC}"

    if [ -d "config" ]; then
        echo -e "${YELLOW}⚠${NC}  配置目录已存在"
        read -p "是否覆盖? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "跳过配置文件安装"
        else
            echo -e "${GREEN}✓${NC} 配置文件已存在，无需重新创建"
        fi
    else
        echo -e "${GREEN}✓${NC} 配置文件已存在"
    fi
    echo ""
fi

# 2. 安装 Docker 环境
if [ "$INSTALL_DOCKER" = true ]; then
    echo -e "${CYAN}[2/4] 安装 Docker 环境...${NC}"

    # 检查 Docker
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}✗${NC} Docker 未安装，请先安装 Docker"
        echo "   访问: https://docs.docker.com/get-docker/"
    else
        echo -e "${GREEN}✓${NC} Docker 已安装"

        if [ -f "docker-compose.pool-test.yml" ]; then
            echo -e "${GREEN}✓${NC} Docker Compose 配置已存在"

            read -p "是否启动 Docker 环境? (y/N) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                echo "启动 Docker 服务..."
                docker-compose -f docker-compose.pool-test.yml up -d
                echo "等待服务就绪..."
                sleep 10
                echo -e "${GREEN}✓${NC} Docker 环境已启动"

                echo ""
                echo "服务访问地址:"
                echo "  PostgreSQL 主库: localhost:5432"
                echo "  PostgreSQL 从库: localhost:5433"
                echo "  Redis: localhost:6379"
                echo "  pgAdmin: http://localhost:5050"
                echo "  Prometheus: http://localhost:9090"
                echo "  Grafana: http://localhost:3000"
            fi
        fi
    fi
    echo ""
fi

# 3. 安装监控配置
if [ "$INSTALL_MONITORING" = true ]; then
    echo -e "${CYAN}[3/4] 安装监控配置...${NC}"

    if [ -d "monitoring" ]; then
        echo -e "${GREEN}✓${NC} 监控配置已存在"
    else
        echo -e "${YELLOW}⚠${NC}  监控配置不存在"
    fi
    echo ""
fi

# 4. 安装工具脚本
if [ "$INSTALL_SCRIPTS" = true ]; then
    echo -e "${CYAN}[4/4] 安装工具脚本...${NC}"

    if [ -d "scripts" ]; then
        echo -e "${GREEN}✓${NC} 工具脚本已存在"

        # 设置执行权限
        chmod +x scripts/*.sh 2>/dev/null || true
        echo -e "${GREEN}✓${NC} 脚本执行权限已设置"
    else
        echo -e "${YELLOW}⚠${NC}  工具脚本目录不存在"
    fi
    echo ""
fi

# 5. 验证安装
echo -e "${CYAN}验证安装...${NC}"
echo ""

# 检查配置文件
if [ -f "config/default.toml" ]; then
    echo -e "${GREEN}✓${NC} 配置文件: config/default.toml"
else
    echo -e "${RED}✗${NC} 配置文件: config/default.toml"
fi

if [ -f "config/development.toml" ]; then
    echo -e "${GREEN}✓${NC} 配置文件: config/development.toml"
else
    echo -e "${RED}✗${NC} 配置文件: config/development.toml"
fi

if [ -f "config/production.toml" ]; then
    echo -e "${GREEN}✓${NC} 配置文件: config/production.toml"
else
    echo -e "${RED}✗${NC} 配置文件: config/production.toml"
fi

# 检查脚本
if [ -x "scripts/verify_pool_config.sh" ]; then
    echo -e "${GREEN}✓${NC} 工具脚本: verify_pool_config.sh"
else
    echo -e "${RED}✗${NC} 工具脚本: verify_pool_config.sh"
fi

if [ -x "scripts/stress_test_pool.sh" ]; then
    echo -e "${GREEN}✓${NC} 工具脚本: stress_test_pool.sh"
else
    echo -e "${RED}✗${NC} 工具脚本: stress_test_pool.sh"
fi

if [ -x "scripts/health_check_pool.sh" ]; then
    echo -e "${GREEN}✓${NC} 工具脚本: health_check_pool.sh"
else
    echo -e "${RED}✗${NC} 工具脚本: health_check_pool.sh"
fi

# 检查文档
if [ -f "DATABASE_POOL_CONFIG.md" ]; then
    echo -e "${GREEN}✓${NC} 文档: DATABASE_POOL_CONFIG.md"
else
    echo -e "${RED}✗${NC} 文档: DATABASE_POOL_CONFIG.md"
fi

if [ -f "INDEX.md" ]; then
    echo -e "${GREEN}✓${NC} 文档: INDEX.md"
else
    echo -e "${RED}✗${NC} 文档: INDEX.md"
fi

echo ""

# 6. 运行验证脚本
if [ -x "../../scripts/verify_pool_config.sh" ]; then
    echo -e "${CYAN}运行配置验证...${NC}"
    echo ""
    ../../scripts/verify_pool_config.sh
fi

# 7. 显示下一步
echo ""
echo -e "${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                    安装完成！                              ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}下一步操作:${NC}"
echo ""
echo "1. 查看快速入门:"
echo "   ${CYAN}cat README_POOL.md${NC}"
echo ""
echo "2. 查看完整文档索引:"
echo "   ${CYAN}cat INDEX.md${NC}"
echo ""
echo "3. 启动服务:"
echo "   ${CYAN}export APP_ENV=development${NC}"
echo "   ${CYAN}cargo run${NC}"
echo ""
echo "4. 运行压力测试:"
echo "   ${CYAN}./scripts/stress_test_pool.sh${NC}"
echo ""
echo "5. 健康检查:"
echo "   ${CYAN}./scripts/health_check_pool.sh${NC}"
echo ""
echo "6. 访问监控 (如果启动了 Docker):"
echo "   Grafana:    ${CYAN}http://localhost:3000${NC} (admin/admin)"
echo "   Prometheus: ${CYAN}http://localhost:9090${NC}"
echo "   pgAdmin:    ${CYAN}http://localhost:5050${NC} (admin@cuba-erp.local/admin)"
echo ""
echo -e "${GREEN}祝您使用愉快！${NC}"
echo ""
