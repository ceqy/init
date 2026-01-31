#!/bin/bash
# 数据库恢复脚本

set -e

# 配置
BACKUP_DIR="/backups/postgres"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-erp}"
DB_USER="${DB_USER:-postgres}"
DB_PASSWORD="${DB_PASSWORD:-postgres}"

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1"
}

# 检查参数
if [ $# -eq 0 ]; then
    echo "用法: $0 <backup_file>"
    echo ""
    echo "可用的备份文件:"
    ls -lh "$BACKUP_DIR"/erp_*.sql.gz | awk '{print "  " $9 " (" $5 ", " $6 " " $7 ")"}'
    exit 1
fi

BACKUP_FILE="$1"

# 检查备份文件是否存在
if [ ! -f "$BACKUP_FILE" ]; then
    error "备份文件不存在: $BACKUP_FILE"
    exit 1
fi

log "准备恢复数据库: $DB_NAME"
log "备份文件: $BACKUP_FILE"

# 确认操作
read -p "⚠️  此操作将覆盖现有数据库，是否继续？(yes/no): " CONFIRM
if [ "$CONFIRM" != "yes" ]; then
    log "操作已取消"
    exit 0
fi

# 创建数据库备份（恢复前）
log "创建当前数据库快照..."
SNAPSHOT_FILE="$BACKUP_DIR/pre_restore_$(date +%Y%m%d_%H%M%S).sql.gz"
export PGPASSWORD="$DB_PASSWORD"
pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" | gzip > "$SNAPSHOT_FILE"
log "快照已保存: $SNAPSHOT_FILE"

# 断开所有连接
log "断开所有数据库连接..."
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c \
    "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '$DB_NAME' AND pid <> pg_backend_pid();" \
    2>/dev/null || true

# 删除并重建数据库
log "重建数据库..."
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "DROP DATABASE IF EXISTS $DB_NAME;"
psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "CREATE DATABASE $DB_NAME;"

# 恢复数据
log "开始恢复数据..."
if gunzip -c "$BACKUP_FILE" | psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" > /dev/null 2>&1; then
    log "✅ 数据库恢复成功"

    # 验证恢复
    log "验证数据库..."
    TABLE_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c \
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';")
    log "恢复的表数量: $TABLE_COUNT"

    # 发送通知
    if [ -n "$WEBHOOK_URL" ]; then
        curl -X POST "$WEBHOOK_URL" \
            -H "Content-Type: application/json" \
            -d "{\"text\":\"✅ 数据库恢复成功: $DB_NAME\"}" \
            2>/dev/null || true
    fi
else
    error "数据库恢复失败"

    # 尝试回滚
    log "尝试回滚到恢复前状态..."
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "DROP DATABASE IF EXISTS $DB_NAME;"
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d postgres -c "CREATE DATABASE $DB_NAME;"
    gunzip -c "$SNAPSHOT_FILE" | psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" > /dev/null 2>&1

    error "已回滚到恢复前状态"
    exit 1
fi

unset PGPASSWORD
