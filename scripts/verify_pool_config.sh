#!/bin/bash
# 数据库连接池性能测试脚本

set -e

echo "=========================================="
echo "数据库连接池配置验证"
echo "=========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 检查配置文件
echo "1. 检查配置文件..."
CONFIG_DIR="services/iam-access/config"

if [ -f "$CONFIG_DIR/default.toml" ]; then
    echo -e "${GREEN}✓${NC} default.toml 存在"
    DEFAULT_CONN=$(grep "max_connections" "$CONFIG_DIR/default.toml" | awk '{print $3}')
    echo "  - 默认连接数: $DEFAULT_CONN"
else
    echo -e "${RED}✗${NC} default.toml 不存在"
fi

if [ -f "$CONFIG_DIR/development.toml" ]; then
    echo -e "${GREEN}✓${NC} development.toml 存在"
    DEV_CONN=$(grep "max_connections" "$CONFIG_DIR/development.toml" | awk '{print $3}')
    echo "  - 开发环境连接数: $DEV_CONN"
else
    echo -e "${RED}✗${NC} development.toml 不存在"
fi

if [ -f "$CONFIG_DIR/production.toml" ]; then
    echo -e "${GREEN}✓${NC} production.toml 存在"
    PROD_CONN=$(grep "max_connections" "$CONFIG_DIR/production.toml" | awk '{print $3}')
    echo "  - 生产环境连接数: $PROD_CONN"

    # 检查是否配置了读写分离
    if grep -q "read_url" "$CONFIG_DIR/production.toml"; then
        echo -e "${GREEN}✓${NC} 已配置读写分离"
        READ_CONN=$(grep "read_max_connections" "$CONFIG_DIR/production.toml" | awk '{print $3}')
        echo "  - 读库连接数: $READ_CONN"
    else
        echo -e "${YELLOW}!${NC} 未配置读写分离（可选）"
    fi
else
    echo -e "${RED}✗${NC} production.toml 不存在"
fi

echo ""

# 检查代码实现
echo "2. 检查连接池实现..."

# 检查 PostgresConfig
if grep -q "max_lifetime" "crates/adapters/postgres/src/connection.rs"; then
    echo -e "${GREEN}✓${NC} PostgresConfig 支持 max_lifetime"
else
    echo -e "${RED}✗${NC} PostgresConfig 缺少 max_lifetime"
fi

if grep -q "acquire_timeout" "crates/adapters/postgres/src/connection.rs"; then
    echo -e "${GREEN}✓${NC} PostgresConfig 支持 acquire_timeout"
else
    echo -e "${RED}✗${NC} PostgresConfig 缺少 acquire_timeout"
fi

# 检查读写分离支持
if grep -q "ReadWritePool" "crates/adapters/postgres/src/connection.rs"; then
    echo -e "${GREEN}✓${NC} 支持读写分离 (ReadWritePool)"
else
    echo -e "${RED}✗${NC} 缺少读写分离支持"
fi

echo ""

# 检查配置层
echo "3. 检查配置层..."

if grep -q "read_url" "crates/config/src/lib.rs"; then
    echo -e "${GREEN}✓${NC} DatabaseConfig 支持 read_url"
else
    echo -e "${RED}✗${NC} DatabaseConfig 缺少 read_url"
fi

if grep -q "read_max_connections" "crates/config/src/lib.rs"; then
    echo -e "${GREEN}✓${NC} DatabaseConfig 支持 read_max_connections"
else
    echo -e "${RED}✗${NC} DatabaseConfig 缺少 read_max_connections"
fi

echo ""

# 检查 Bootstrap 集成
echo "4. 检查 Bootstrap 集成..."

if grep -q "read_write_pool" "bootstrap/src/infrastructure.rs"; then
    echo -e "${GREEN}✓${NC} Infrastructure 支持 read_write_pool()"
else
    echo -e "${RED}✗${NC} Infrastructure 缺少 read_write_pool()"
fi

if grep -q "ReadWritePool::new" "bootstrap/src/infrastructure.rs"; then
    echo -e "${GREEN}✓${NC} Bootstrap 初始化读写分离连接池"
else
    echo -e "${RED}✗${NC} Bootstrap 未初始化读写分离连接池"
fi

echo ""

# 检查监控指标
echo "5. 检查监控指标..."

if grep -q 'pool.*write' "bootstrap/src/metrics.rs"; then
    echo -e "${GREEN}✓${NC} 支持写连接池指标"
else
    echo -e "${RED}✗${NC} 缺少写连接池指标"
fi

if grep -q 'pool.*read' "bootstrap/src/metrics.rs"; then
    echo -e "${GREEN}✓${NC} 支持读连接池指标"
else
    echo -e "${RED}✗${NC} 缺少读连接池指标"
fi

if grep -q "postgres_pool_utilization" "bootstrap/src/metrics.rs"; then
    echo -e "${GREEN}✓${NC} 支持连接池使用率指标"
else
    echo -e "${RED}✗${NC} 缺少连接池使用率指标"
fi

echo ""

# 编译检查
echo "6. 编译检查..."
if cargo check -p iam-access --quiet 2>/dev/null; then
    echo -e "${GREEN}✓${NC} iam-access 编译成功"
else
    echo -e "${RED}✗${NC} iam-access 编译失败"
    echo "运行 'cargo check -p iam-access' 查看详细错误"
fi

echo ""

# 性能建议
echo "=========================================="
echo "性能调优建议"
echo "=========================================="
echo ""

echo "根据您的应用规模选择合适的连接池配置："
echo ""
echo "小型应用 (< 1000 并发):"
echo "  max_connections = 20"
echo ""
echo "中型应用 (1000-10000 并发):"
echo "  max_connections = 50"
echo "  read_max_connections = 100"
echo ""
echo "大型应用 (> 10000 并发):"
echo "  max_connections = 100"
echo "  read_max_connections = 200"
echo ""

echo "连接数计算公式："
echo "  max_connections = (CPU 核心数 × 2) + 有效磁盘数"
echo ""

# 系统信息
if [[ "$OSTYPE" == "darwin"* ]]; then
    CPU_CORES=$(sysctl -n hw.ncpu)
    echo "当前系统 CPU 核心数: $CPU_CORES"
    SUGGESTED=$((CPU_CORES * 2 + 1))
    echo "建议连接数: $SUGGESTED - $((SUGGESTED * 2))"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    CPU_CORES=$(nproc)
    echo "当前系统 CPU 核心数: $CPU_CORES"
    SUGGESTED=$((CPU_CORES * 2 + 1))
    echo "建议连接数: $SUGGESTED - $((SUGGESTED * 2))"
fi

echo ""
echo "=========================================="
echo "验证完成"
echo "=========================================="
echo ""
echo "查看详细配置说明: services/iam-access/DATABASE_POOL_CONFIG.md"
