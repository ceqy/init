-- 创建登录日志表

CREATE TABLE IF NOT EXISTS login_logs (
    -- 主键
    id UUID PRIMARY KEY,
    
    -- 用户信息
    user_id UUID,  -- 可能为空（用户名不存在时）
    tenant_id UUID NOT NULL,
    username VARCHAR(255) NOT NULL,
    
    -- 网络信息
    ip_address VARCHAR(45) NOT NULL,  -- 支持 IPv6
    user_agent TEXT NOT NULL,
    
    -- 设备信息
    device_type VARCHAR(50),
    os VARCHAR(50),
    os_version VARCHAR(50),
    browser VARCHAR(50),
    browser_version VARCHAR(50),
    is_mobile BOOLEAN NOT NULL DEFAULT FALSE,
    device_fingerprint VARCHAR(255),
    
    -- 登录结果
    result VARCHAR(20) NOT NULL,  -- Success, Failed
    failure_reason VARCHAR(100),
    
    -- 地理位置
    country VARCHAR(100),
    city VARCHAR(100),
    
    -- 可疑登录标记
    is_suspicious BOOLEAN NOT NULL DEFAULT FALSE,
    suspicious_reason TEXT,
    
    -- 时间戳
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_login_logs_user_id ON login_logs(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_login_logs_tenant_id ON login_logs(tenant_id);
CREATE INDEX idx_login_logs_username ON login_logs(username);
CREATE INDEX idx_login_logs_ip_address ON login_logs(ip_address);
CREATE INDEX idx_login_logs_result ON login_logs(result);
CREATE INDEX idx_login_logs_created_at ON login_logs(created_at DESC);
CREATE INDEX idx_login_logs_suspicious ON login_logs(is_suspicious) WHERE is_suspicious = TRUE;
CREATE INDEX idx_login_logs_device_fingerprint ON login_logs(device_fingerprint) WHERE device_fingerprint IS NOT NULL;

-- 复合索引
CREATE INDEX idx_login_logs_user_tenant_time ON login_logs(user_id, tenant_id, created_at DESC) WHERE user_id IS NOT NULL;
CREATE INDEX idx_login_logs_tenant_time ON login_logs(tenant_id, created_at DESC);

-- 外键约束
ALTER TABLE login_logs
    ADD CONSTRAINT fk_login_logs_user
    FOREIGN KEY (user_id)
    REFERENCES users(id)
    ON DELETE SET NULL;

-- 分区表（按月分区，提高查询性能）
-- 注意：这需要 PostgreSQL 10+
-- CREATE TABLE login_logs_y2026m01 PARTITION OF login_logs
--     FOR VALUES FROM ('2026-01-01') TO ('2026-02-01');

-- 注释
COMMENT ON TABLE login_logs IS '登录日志表';
COMMENT ON COLUMN login_logs.id IS '日志ID';
COMMENT ON COLUMN login_logs.user_id IS '用户ID（可能为空）';
COMMENT ON COLUMN login_logs.tenant_id IS '租户ID';
COMMENT ON COLUMN login_logs.username IS '登录用户名';
COMMENT ON COLUMN login_logs.ip_address IS 'IP地址';
COMMENT ON COLUMN login_logs.user_agent IS 'User-Agent';
COMMENT ON COLUMN login_logs.device_fingerprint IS '设备指纹';
COMMENT ON COLUMN login_logs.result IS '登录结果';
COMMENT ON COLUMN login_logs.failure_reason IS '失败原因';
COMMENT ON COLUMN login_logs.is_suspicious IS '是否可疑登录';
COMMENT ON COLUMN login_logs.suspicious_reason IS '可疑原因';
COMMENT ON COLUMN login_logs.created_at IS '创建时间';

-- 创建自动清理函数（清理90天前的日志）
CREATE OR REPLACE FUNCTION cleanup_old_login_logs()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM login_logs
    WHERE created_at < NOW() - INTERVAL '90 days';
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_old_login_logs() IS '清理90天前的登录日志';

-- 启用 RLS
ALTER TABLE login_logs ENABLE ROW LEVEL SECURITY;

-- 创建 RLS 策略
CREATE POLICY login_logs_tenant_isolation ON login_logs
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY login_logs_insert_policy ON login_logs
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

COMMENT ON POLICY login_logs_tenant_isolation ON login_logs IS '登录日志表租户隔离策略';
