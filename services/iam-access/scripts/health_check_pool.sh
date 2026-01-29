#!/bin/bash
# 数据库连接池健康检查脚本
# 用于生产环境的定期健康检查

set -e

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置参数
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-cuba}"
DB_USER="${DB_USER:-postgres}"
DB_PASSWORD="${DB_PASSWORD:-postgres}"

# 阈值配置
POOL_UTILIZATION_WARNING=70
POOL_UTILIZATION_CRITICAL=85
IDLE_CONNECTION_WARNING=50
LONG_RUNNING_QUERY_THRESHOLD=30  # 秒
IDLE_IN_TRANSACTION_THRESHOLD=5   # 秒

export PGPASSWORD="$DB_PASSWORD"

echo "=========================================="
echo "数据库连接池健康检查"
echo "=========================================="
echo ""
echo "检查时间: $(date '+%Y-%m-%d %H:%M:%S')"
echo "数据库: $DB_HOST:$DB_PORT/$DB_NAME"
echo ""

# 健康状态
HEALTH_STATUS="healthy"
WARNINGS=0
ERRORS=0

# 1. 检查数据库连接
echo -e "${BLUE}1. 数据库连接检查${NC}"
if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT 1" >/dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} 数据库连接正常"
else
    echo -e "${RED}✗${NC} 数据库连接失败"
    HEALTH_STATUS="unhealthy"
    ((ERRORS++))
    exit 1
fi
echo ""

# 2. 检查连接池使用率
echo -e "${BLUE}2. 连接池使用率检查${NC}"
POOL_STATS=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -A -F',' <<EOF
SELECT
    COUNT(*) as total,
    COUNT(*) FILTER (WHERE state = 'active') as active,
    COUNT(*) FILTER (WHERE state = 'idle') as idle,
    (SELECT setting::int FROM pg_settings WHERE name = 'max_connections') as max_conn
FROM pg_stat_activity
WHERE datname = '$DB_NAME';
EOF
)

TOTAL_CONN=$(echo "$POOL_STATS" | cut -d',' -f1)
ACTIVE_CONN=$(echo "$POOL_STATS" | cut -d',' -f2)
IDLE_CONN=$(echo "$POOL_STATS" | cut -d',' -f3)
MAX_CONN=$(echo "$POOL_STATS" | cut -d',' -f4)

UTILIZATION=$(awk "BEGIN {printf \"%.0f\", ($TOTAL_CONN / $MAX_CONN) * 100}")

echo "  总连接数: $TOTAL_CONN / $MAX_CONN"
echo "  活跃连接: $ACTIVE_CONN"
echo "  空闲连接: $IDLE_CONN"
echo "  使用率: ${UTILIZATION}%"

if [ "$UTILIZATION" -ge "$POOL_UTILIZATION_CRITICAL" ]; then
    echo -e "${RED}✗${NC} 连接池使用率过高 (>= ${POOL_UTILIZATION_CRITICAL}%)"
    HEALTH_STATUS="critical"
    ((ERRORS++))
elif [ "$UTILIZATION" -ge "$POOL_UTILIZATION_WARNING" ]; then
    echo -e "${YELLOW}⚠${NC} 连接池使用率警告 (>= ${POOL_UTILIZATION_WARNING}%)"
    HEALTH_STATUS="warning"
    ((WARNINGS++))
else
    echo -e "${GREEN}✓${NC} 连接池使用率正常"
fi
echo ""

# 3. 检查空闲连接比例
echo -e "${BLUE}3. 空闲连接检查${NC}"
if [ "$TOTAL_CONN" -gt 0 ]; then
    IDLE_PERCENT=$(awk "BEGIN {printf \"%.0f\", ($IDLE_CONN / $TOTAL_CONN) * 100}")
    echo "  空闲连接比例: ${IDLE_PERCENT}%"

    if [ "$IDLE_PERCENT" -ge "$IDLE_CONNECTION_WARNING" ]; then
        echo -e "${YELLOW}⚠${NC} 空闲连接过多，考虑减少连接池大小"
        ((WARNINGS++))
    else
        echo -e "${GREEN}✓${NC} 空闲连接比例正常"
    fi
fi
echo ""

# 4. 检查长时间运行的查询
echo -e "${BLUE}4. 长时间运行查询检查${NC}"
LONG_QUERIES=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "
SELECT COUNT(*)
FROM pg_stat_activity
WHERE datname = '$DB_NAME'
    AND state = 'active'
    AND NOW() - query_start > interval '$LONG_RUNNING_QUERY_THRESHOLD seconds'
")

if [ "$LONG_QUERIES" -gt 0 ]; then
    echo -e "${YELLOW}⚠${NC} 发现 $LONG_QUERIES 个长时间运行的查询 (> ${LONG_RUNNING_QUERY_THRESHOLD}s)"
    ((WARNINGS++))

    # 显示详细信息
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" <<EOF
SELECT
    pid,
    usename,
    EXTRACT(EPOCH FROM (NOW() - query_start))::int as duration_seconds,
    state,
    LEFT(query, 80) as query_preview
FROM pg_stat_activity
WHERE datname = '$DB_NAME'
    AND state = 'active'
    AND NOW() - query_start > interval '$LONG_RUNNING_QUERY_THRESHOLD seconds'
ORDER BY query_start;
EOF
else
    echo -e "${GREEN}✓${NC} 没有长时间运行的查询"
fi
echo ""

# 5. 检查空闲事务
echo -e "${BLUE}5. 空闲事务检查${NC}"
IDLE_IN_TX=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "
SELECT COUNT(*)
FROM pg_stat_activity
WHERE datname = '$DB_NAME'
    AND state = 'idle in transaction'
    AND NOW() - state_change > interval '$IDLE_IN_TRANSACTION_THRESHOLD seconds'
")

if [ "$IDLE_IN_TX" -gt 0 ]; then
    echo -e "${RED}✗${NC} 发现 $IDLE_IN_TX 个空闲事务 (> ${IDLE_IN_TRANSACTION_THRESHOLD}s)"
    HEALTH_STATUS="critical"
    ((ERRORS++))

    # 显示详细信息
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" <<EOF
SELECT
    pid,
    usename,
    application_name,
    EXTRACT(EPOCH FROM (NOW() - state_change))::int as idle_seconds,
    LEFT(query, 80) as last_query
FROM pg_stat_activity
WHERE datname = '$DB_NAME'
    AND state = 'idle in transaction'
    AND NOW() - state_change > interval '$IDLE_IN_TRANSACTION_THRESHOLD seconds'
ORDER BY state_change;
EOF
else
    echo -e "${GREEN}✓${NC} 没有空闲事务"
fi
echo ""

# 6. 检查等待事件
echo -e "${BLUE}6. 等待事件检查${NC}"
WAITING_QUERIES=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "
SELECT COUNT(*)
FROM pg_stat_activity
WHERE datname = '$DB_NAME'
    AND wait_event IS NOT NULL
    AND state = 'active'
")

if [ "$WAITING_QUERIES" -gt 0 ]; then
    echo -e "${YELLOW}⚠${NC} 发现 $WAITING_QUERIES 个等待中的查询"
    ((WARNINGS++))

    # 显示等待事件统计
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" <<EOF
SELECT
    wait_event_type,
    wait_event,
    COUNT(*) as count
FROM pg_stat_activity
WHERE datname = '$DB_NAME'
    AND wait_event IS NOT NULL
    AND state = 'active'
GROUP BY wait_event_type, wait_event
ORDER BY count DESC;
EOF
else
    echo -e "${GREEN}✓${NC} 没有等待中的查询"
fi
echo ""

# 7. 检查数据库性能指标
echo -e "${BLUE}7. 数据库性能指标${NC}"
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" <<EOF
SELECT
    numbackends as connections,
    xact_commit as commits,
    xact_rollback as rollbacks,
    ROUND(blks_hit::numeric / NULLIF(blks_hit + blks_read, 0) * 100, 2) as cache_hit_ratio,
    tup_returned as rows_returned,
    tup_fetched as rows_fetched
FROM pg_stat_database
WHERE datname = '$DB_NAME';
EOF
echo ""

# 8. 检查复制状态（如果有）
echo -e "${BLUE}8. 复制状态检查${NC}"
REPLICATION_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT COUNT(*) FROM pg_stat_replication")

if [ "$REPLICATION_COUNT" -gt 0 ]; then
    echo "  发现 $REPLICATION_COUNT 个复制连接"
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" <<EOF
SELECT
    application_name,
    client_addr,
    state,
    sync_state,
    ROUND(pg_wal_lsn_diff(pg_current_wal_lsn(), replay_lsn) / 1024 / 1024, 2) as lag_mb
FROM pg_stat_replication;
EOF
    echo -e "${GREEN}✓${NC} 复制状态正常"
else
    echo "  未配置复制"
fi
echo ""

# 9. 生成健康报告
echo "=========================================="
echo "健康检查总结"
echo "=========================================="
echo ""
echo "状态: $HEALTH_STATUS"
echo "警告数: $WARNINGS"
echo "错误数: $ERRORS"
echo ""

# 10. 建议
echo -e "${BLUE}建议:${NC}"

if [ "$UTILIZATION" -ge "$POOL_UTILIZATION_CRITICAL" ]; then
    echo "  • 立即增加连接池大小或优化查询性能"
fi

if [ "$IDLE_PERCENT" -ge "$IDLE_CONNECTION_WARNING" ]; then
    echo "  • 考虑减少 max_connections 或调整 idle_timeout"
fi

if [ "$LONG_QUERIES" -gt 0 ]; then
    echo "  • 优化长时间运行的查询"
fi

if [ "$IDLE_IN_TX" -gt 0 ]; then
    echo "  • 检查应用代码，确保事务正确提交或回滚"
fi

if [ "$WARNINGS" -eq 0 ] && [ "$ERRORS" -eq 0 ]; then
    echo "  • 连接池配置良好，继续保持"
fi

echo ""

# 退出码
if [ "$HEALTH_STATUS" = "critical" ]; then
    exit 2
elif [ "$HEALTH_STATUS" = "warning" ]; then
    exit 1
else
    exit 0
fi
