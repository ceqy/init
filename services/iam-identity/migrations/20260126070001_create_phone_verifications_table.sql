-- 创建手机验证表

CREATE TABLE IF NOT EXISTS phone_verifications (
    -- 主键
    id UUID PRIMARY KEY,
    
    -- 用户信息
    user_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    phone VARCHAR(20) NOT NULL,
    
    -- 验证码
    code VARCHAR(6) NOT NULL,
    
    -- 状态
    status VARCHAR(20) NOT NULL DEFAULT 'Pending',  -- Pending, Verified, Expired
    
    -- 时间
    expires_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_phone_verifications_user_id ON phone_verifications(user_id);
CREATE INDEX idx_phone_verifications_tenant_id ON phone_verifications(tenant_id);
CREATE INDEX idx_phone_verifications_phone ON phone_verifications(phone);
CREATE INDEX idx_phone_verifications_status ON phone_verifications(status);
CREATE INDEX idx_phone_verifications_created_at ON phone_verifications(created_at DESC);
CREATE INDEX idx_phone_verifications_expires_at ON phone_verifications(expires_at);

-- 复合索引
CREATE INDEX idx_phone_verifications_user_tenant ON phone_verifications(user_id, tenant_id, created_at DESC);

-- 外键约束
ALTER TABLE phone_verifications
    ADD CONSTRAINT fk_phone_verifications_user
    FOREIGN KEY (user_id)
    REFERENCES users(id)
    ON DELETE CASCADE;

-- 注释
COMMENT ON TABLE phone_verifications IS '手机验证表';
COMMENT ON COLUMN phone_verifications.id IS '验证ID';
COMMENT ON COLUMN phone_verifications.user_id IS '用户ID';
COMMENT ON COLUMN phone_verifications.tenant_id IS '租户ID';
COMMENT ON COLUMN phone_verifications.phone IS '手机号码';
COMMENT ON COLUMN phone_verifications.code IS '验证码（6位数字）';
COMMENT ON COLUMN phone_verifications.status IS '状态';
COMMENT ON COLUMN phone_verifications.expires_at IS '过期时间';
COMMENT ON COLUMN phone_verifications.verified_at IS '验证时间';
COMMENT ON COLUMN phone_verifications.created_at IS '创建时间';

-- 创建自动清理函数
CREATE OR REPLACE FUNCTION cleanup_expired_phone_verifications()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM phone_verifications
    WHERE expires_at < NOW() - INTERVAL '24 hours';
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_expired_phone_verifications() IS '清理过期的手机验证记录';

-- 启用 RLS
ALTER TABLE phone_verifications ENABLE ROW LEVEL SECURITY;

-- 创建 RLS 策略
CREATE POLICY phone_verifications_tenant_isolation ON phone_verifications
    USING (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

CREATE POLICY phone_verifications_insert_policy ON phone_verifications
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true)::uuid);

COMMENT ON POLICY phone_verifications_tenant_isolation ON phone_verifications IS '手机验证表租户隔离策略';

-- 为 users 表添加手机验证字段
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS phone_verified BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS phone_verified_at TIMESTAMPTZ;

COMMENT ON COLUMN users.phone_verified IS '手机是否已验证';
COMMENT ON COLUMN users.phone_verified_at IS '手机验证时间';
