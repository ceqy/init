#!/bin/bash
# PostgreSQL 从库设置脚本

set -e

echo "Setting up PostgreSQL replica..."

# 等待主库就绪
until pg_isready -h postgres-primary -U postgres; do
  echo "Waiting for primary database..."
  sleep 2
done

# 停止 PostgreSQL
pg_ctl -D "$PGDATA" -m fast -w stop || true

# 清空数据目录
rm -rf "$PGDATA"/*

# 从主库创建基础备份
PGPASSWORD=replicator_password pg_basebackup \
  -h postgres-primary \
  -D "$PGDATA" \
  -U replicator \
  -v \
  -P \
  -X stream \
  -R

# 创建 standby.signal 文件（PostgreSQL 12+）
touch "$PGDATA/standby.signal"

# 配置复制参数
cat >> "$PGDATA/postgresql.auto.conf" <<EOF
primary_conninfo = 'host=postgres-primary port=5432 user=replicator password=replicator_password'
primary_slot_name = 'replica_slot'
EOF

echo "Replica setup completed"

# 启动 PostgreSQL
pg_ctl -D "$PGDATA" -w start
