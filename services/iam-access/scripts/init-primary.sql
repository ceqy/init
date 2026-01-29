#!/bin/bash
# PostgreSQL 主库初始化脚本

set -e

echo "Initializing PostgreSQL primary database..."

# 创建复制用户
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    -- 创建复制用户
    CREATE USER replicator WITH REPLICATION ENCRYPTED PASSWORD 'replicator_password';

    -- 授予复制权限
    GRANT CONNECT ON DATABASE $POSTGRES_DB TO replicator;
EOSQL

# 配置 pg_hba.conf 允许复制连接
cat >> "$PGDATA/pg_hba.conf" <<EOF

# Replication connections
host    replication     replicator      0.0.0.0/0               md5
EOF

echo "Primary database initialized successfully"
