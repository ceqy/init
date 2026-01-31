#!/bin/bash
# 数据库自动备份脚本

set -e

# 配置
BACKUP_DIR="/backups/postgres"
RETENTION_DAYS=30
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="erp_${DATE}.sql"
COMPRESSED_FILE="${BACKUP_FILE}.gz"

# 数据库连接信息
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

# 日志函数
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1"
}

# 创建备份目录
mkdir -p "$BACKUP_DIR"

log "开始备份数据库: $DB_NAME"

# 执行备份
export PGPASSWORD="$DB_PASSWORD"

if pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" \
    --format=plain \
    --no-owner \
    --no-acl \
    --verbose \
    > "$BACKUP_DIR/$BACKUP_FILE" 2>&1; then

    log "数据库备份完成: $BACKUP_FILE"

    # 压缩备份文件
    log "压缩备份文件..."
    gzip "$BACKUP_DIR/$BACKUP_FILE"

    # 计算文件大小
    SIZE=$(du -h "$BACKUP_DIR/$COMPRESSED_FILE" | cut -f1)
    log "备份文件大小: $SIZE"

    # 验证备份文件
    log "验证备份文件完整性..."
    if gunzip -t "$BACKUP_DIR/$COMPRESSED_FILE" 2>/dev/null; then
        log "备份文件验证通过"
    else
        error "备份文件验证失败"
        exit 1
    fi

    # 清理旧备份
    log "清理 ${RETENTION_DAYS} 天前的备份..."
    find "$BACKUP_DIR" -name "erp_*.sql.gz" -mtime +${RETENTION_DAYS} -delete

    # 统计备份文件数量
    BACKUP_COUNT=$(find "$BACKUP_DIR" -name "erp_*.sql.gz" | wc -l)
    log "当前保留备份数量: $BACKUP_COUNT"

    # 上传到远程存储（可选）
    if [ -n "$S3_BUCKET" ]; then
        log "上传备份到 S3: $S3_BUCKET"
        aws s3 cp "$BACKUP_DIR/$COMPRESSED_FILE" "s3://$S3_BUCKET/backups/postgres/" || warn "S3 上传失败"
    fi

    # 发送通知（可选）
    if [ -n "$WEBHOOK_URL" ]; then
        curl -X POST "$WEBHOOK_URL" \
            -H "Content-Type: application/json" \
            -d "{\"text\":\"✅ 数据库备份成功: $COMPRESSED_FILE ($SIZE)\"}" \
            2>/dev/null || warn "通知发送失败"
    fi

    log "备份任务完成"

else
    error "数据库备份失败"

    # 发送失败通知
    if [ -n "$WEBHOOK_URL" ]; then
        curl -X POST "$WEBHOOK_URL" \
            -H "Content-Type: application/json" \
            -d "{\"text\":\"❌ 数据库备份失败: $DB_NAME\"}" \
            2>/dev/null
    fi

    exit 1
fi

# 清理环境变量
unset PGPASSWORD
