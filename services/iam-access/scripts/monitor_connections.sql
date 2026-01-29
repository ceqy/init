-- 数据库连接池监控查询
-- 用于检查 PostgreSQL 数据库端的连接状态

-- 1. 查看当前所有连接
SELECT
    pid,
    usename,
    application_name,
    client_addr,
    state,
    query_start,
    state_change,
    wait_event_type,
    wait_event,
    query
FROM pg_stat_activity
WHERE datname = current_database()
ORDER BY state_change DESC;

-- 2. 统计连接状态
SELECT
    state,
    COUNT(*) as count
FROM pg_stat_activity
WHERE datname = current_database()
GROUP BY state
ORDER BY count DESC;

-- 3. 查看空闲连接
SELECT
    pid,
    usename,
    application_name,
    client_addr,
    state,
    state_change,
    NOW() - state_change as idle_duration
FROM pg_stat_activity
WHERE datname = current_database()
    AND state = 'idle'
ORDER BY idle_duration DESC;

-- 4. 查看活跃查询
SELECT
    pid,
    usename,
    application_name,
    client_addr,
    NOW() - query_start as duration,
    state,
    query
FROM pg_stat_activity
WHERE datname = current_database()
    AND state = 'active'
ORDER BY duration DESC;

-- 5. 查看长时间运行的查询（超过 30 秒）
SELECT
    pid,
    usename,
    application_name,
    NOW() - query_start as duration,
    state,
    LEFT(query, 100) as query_preview
FROM pg_stat_activity
WHERE datname = current_database()
    AND state = 'active'
    AND NOW() - query_start > interval '30 seconds'
ORDER BY duration DESC;

-- 6. 查看数据库连接限制
SELECT
    setting as max_connections,
    unit
FROM pg_settings
WHERE name = 'max_connections';

-- 7. 查看当前连接数占比
SELECT
    (SELECT COUNT(*) FROM pg_stat_activity WHERE datname = current_database()) as current_connections,
    (SELECT setting::int FROM pg_settings WHERE name = 'max_connections') as max_connections,
    ROUND(
        (SELECT COUNT(*) FROM pg_stat_activity WHERE datname = current_database())::numeric /
        (SELECT setting::int FROM pg_settings WHERE name = 'max_connections')::numeric * 100,
        2
    ) as usage_percentage;

-- 8. 按应用名称统计连接
SELECT
    application_name,
    state,
    COUNT(*) as count
FROM pg_stat_activity
WHERE datname = current_database()
GROUP BY application_name, state
ORDER BY count DESC;

-- 9. 查看等待事件
SELECT
    wait_event_type,
    wait_event,
    COUNT(*) as count
FROM pg_stat_activity
WHERE datname = current_database()
    AND wait_event IS NOT NULL
GROUP BY wait_event_type, wait_event
ORDER BY count DESC;

-- 10. 终止空闲超过 10 分钟的连接（谨慎使用）
-- SELECT pg_terminate_backend(pid)
-- FROM pg_stat_activity
-- WHERE datname = current_database()
--     AND state = 'idle'
--     AND NOW() - state_change > interval '10 minutes';

-- 11. 查看连接池配置建议
WITH connection_stats AS (
    SELECT
        COUNT(*) as total_connections,
        COUNT(*) FILTER (WHERE state = 'active') as active_connections,
        COUNT(*) FILTER (WHERE state = 'idle') as idle_connections,
        COUNT(*) FILTER (WHERE state = 'idle in transaction') as idle_in_transaction
    FROM pg_stat_activity
    WHERE datname = current_database()
)
SELECT
    total_connections,
    active_connections,
    idle_connections,
    idle_in_transaction,
    CASE
        WHEN active_connections::float / NULLIF(total_connections, 0) > 0.8
        THEN '建议增加连接池大小'
        WHEN idle_connections::float / NULLIF(total_connections, 0) > 0.5
        THEN '建议减少连接池大小或调整 idle_timeout'
        WHEN idle_in_transaction > 0
        THEN '警告: 存在空闲事务，可能导致锁等待'
        ELSE '连接池配置正常'
    END as recommendation
FROM connection_stats;

-- 12. 查看数据库性能指标
SELECT
    numbackends as current_connections,
    xact_commit as transactions_committed,
    xact_rollback as transactions_rolled_back,
    blks_read as blocks_read,
    blks_hit as blocks_hit,
    ROUND(
        blks_hit::numeric / NULLIF(blks_hit + blks_read, 0) * 100,
        2
    ) as cache_hit_ratio,
    tup_returned as rows_returned,
    tup_fetched as rows_fetched,
    tup_inserted as rows_inserted,
    tup_updated as rows_updated,
    tup_deleted as rows_deleted
FROM pg_stat_database
WHERE datname = current_database();
