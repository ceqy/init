#!/bin/bash
# 数据库连接池压力测试脚本
# 用于验证连接池配置在高并发场景下的表现

set -e

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "=========================================="
echo "数据库连接池压力测试"
echo "=========================================="
echo ""

# 检查依赖
command -v psql >/dev/null 2>&1 || {
    echo -e "${RED}错误: 需要安装 PostgreSQL 客户端 (psql)${NC}"
    exit 1
}

# 配置参数
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-cuba}"
DB_USER="${DB_USER:-postgres}"
DB_PASSWORD="${DB_PASSWORD:-postgres}"

# 测试参数
CONCURRENT_CONNECTIONS="${CONCURRENT_CONNECTIONS:-50}"
TEST_DURATION="${TEST_DURATION:-60}"
QUERY_INTERVAL="${QUERY_INTERVAL:-0.1}"

echo -e "${BLUE}测试配置:${NC}"
echo "  数据库: $DB_HOST:$DB_PORT/$DB_NAME"
echo "  并发连接数: $CONCURRENT_CONNECTIONS"
echo "  测试时长: ${TEST_DURATION}秒"
echo "  查询间隔: ${QUERY_INTERVAL}秒"
echo ""

# 检查数据库连接
echo -e "${BLUE}1. 检查数据库连接...${NC}"
export PGPASSWORD="$DB_PASSWORD"
if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT 1" >/dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} 数据库连接成功"
else
    echo -e "${RED}✗${NC} 数据库连接失败"
    exit 1
fi
echo ""

# 查看当前连接数
echo -e "${BLUE}2. 查看当前连接状态...${NC}"
CURRENT_CONNECTIONS=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM pg_stat_activity WHERE datname = '$DB_NAME'")
MAX_CONNECTIONS=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT setting FROM pg_settings WHERE name = 'max_connections'")

echo "  当前连接数: $CURRENT_CONNECTIONS"
echo "  最大连接数: $MAX_CONNECTIONS"
echo ""

# 创建测试表（如果不存在）
echo -e "${BLUE}3. 准备测试环境...${NC}"
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" <<EOF >/dev/null 2>&1
CREATE TABLE IF NOT EXISTS pool_test (
    id SERIAL PRIMARY KEY,
    test_data TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);
EOF
echo -e "${GREEN}✓${NC} 测试表准备完成"
echo ""

# 创建测试函数
test_query() {
    local worker_id=$1
    local start_time=$(date +%s)
    local end_time=$((start_time + TEST_DURATION))
    local query_count=0
    local error_count=0

    while [ $(date +%s) -lt $end_time ]; do
        if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT COUNT(*) FROM pool_test" >/dev/null 2>&1; then
            ((query_count++))
        else
            ((error_count++))
        fi
        sleep "$QUERY_INTERVAL"
    done

    echo "$worker_id,$query_count,$error_count"
}

export -f test_query
export DB_HOST DB_PORT DB_NAME DB_USER DB_PASSWORD TEST_DURATION QUERY_INTERVAL PGPASSWORD

# 开始压力测试
echo -e "${BLUE}4. 开始压力测试...${NC}"
echo "  启动 $CONCURRENT_CONNECTIONS 个并发连接"
echo "  测试时长: ${TEST_DURATION}秒"
echo ""

# 创建临时目录存储结果
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# 启动并发测试
for i in $(seq 1 $CONCURRENT_CONNECTIONS); do
    test_query $i > "$TEMP_DIR/worker_$i.txt" &
done

# 显示进度
for i in $(seq 1 $TEST_DURATION); do
    echo -ne "\r  进度: [$i/$TEST_DURATION] "
    # 显示当前连接数
    CURRENT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM pg_stat_activity WHERE datname = '$DB_NAME'" 2>/dev/null || echo "N/A")
    echo -ne "当前连接: $CURRENT"
    sleep 1
done
echo ""

# 等待所有测试完成
wait

echo ""
echo -e "${GREEN}✓${NC} 压力测试完成"
echo ""

# 统计结果
echo -e "${BLUE}5. 测试结果统计...${NC}"
total_queries=0
total_errors=0

for i in $(seq 1 $CONCURRENT_CONNECTIONS); do
    if [ -f "$TEMP_DIR/worker_$i.txt" ]; then
        result=$(cat "$TEMP_DIR/worker_$i.txt")
        worker_id=$(echo "$result" | cut -d',' -f1)
        queries=$(echo "$result" | cut -d',' -f2)
        errors=$(echo "$result" | cut -d',' -f3)
        total_queries=$((total_queries + queries))
        total_errors=$((total_errors + errors))
    fi
done

echo "  总查询数: $total_queries"
echo "  总错误数: $total_errors"
echo "  成功率: $(awk "BEGIN {printf \"%.2f\", ($total_queries - $total_errors) / $total_queries * 100}")%"
echo "  平均 QPS: $(awk "BEGIN {printf \"%.2f\", $total_queries / $TEST_DURATION}")"
echo ""

# 查看最终连接状态
echo -e "${BLUE}6. 最终连接状态...${NC}"
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" <<EOF
SELECT
    state,
    COUNT(*) as count
FROM pg_stat_activity
WHERE datname = '$DB_NAME'
GROUP BY state
ORDER BY count DESC;
EOF
echo ""

# 性能建议
echo -e "${BLUE}7. 性能建议...${NC}"

if [ $total_errors -gt 0 ]; then
    echo -e "${RED}⚠${NC}  检测到 $total_errors 个错误"
    echo "     建议: 增加连接池大小或优化查询性能"
fi

# 检查连接池使用率
FINAL_CONNECTIONS=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM pg_stat_activity WHERE datname = '$DB_NAME'")
USAGE_PERCENT=$(awk "BEGIN {printf \"%.0f\", $FINAL_CONNECTIONS / $MAX_CONNECTIONS * 100}")

if [ $USAGE_PERCENT -gt 80 ]; then
    echo -e "${YELLOW}⚠${NC}  连接池使用率: ${USAGE_PERCENT}%"
    echo "     建议: 连接池使用率较高，考虑增加 max_connections"
elif [ $USAGE_PERCENT -lt 30 ]; then
    echo -e "${GREEN}✓${NC}  连接池使用率: ${USAGE_PERCENT}%"
    echo "     建议: 连接池使用率正常，可以考虑减少 max_connections 以节省资源"
else
    echo -e "${GREEN}✓${NC}  连接池使用率: ${USAGE_PERCENT}%"
    echo "     建议: 连接池配置合理"
fi

echo ""

# 清理测试数据
echo -e "${BLUE}8. 清理测试数据...${NC}"
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "DROP TABLE IF EXISTS pool_test" >/dev/null 2>&1
echo -e "${GREEN}✓${NC} 清理完成"
echo ""

echo "=========================================="
echo "测试完成"
echo "=========================================="
echo ""
echo "详细监控查询请参考: services/iam-access/scripts/monitor_connections.sql"
