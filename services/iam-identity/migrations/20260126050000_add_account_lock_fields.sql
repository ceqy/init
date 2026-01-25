-- 添加账户锁定相关字段

-- 为 users 表添加锁定字段
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS locked_until TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS lock_reason VARCHAR(500),
    ADD COLUMN IF NOT EXISTS failed_login_count INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS last_failed_login_at TIMESTAMPTZ;

-- 添加索引以优化查询
CREATE INDEX IF NOT EXISTS idx_users_locked_until ON users(locked_until) WHERE locked_until IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_users_failed_login_count ON users(failed_login_count) WHERE failed_login_count > 0;

-- 添加注释
COMMENT ON COLUMN users.locked_until IS '账户锁定截止时间，NULL表示未锁定';
COMMENT ON COLUMN users.lock_reason IS '锁定原因';
COMMENT ON COLUMN users.failed_login_count IS '连续登录失败次数';
COMMENT ON COLUMN users.last_failed_login_at IS '最后一次登录失败时间';

-- 创建自动解锁函数
CREATE OR REPLACE FUNCTION auto_unlock_expired_accounts()
RETURNS INTEGER AS $$
DECLARE
    unlocked_count INTEGER;
BEGIN
    UPDATE users
    SET locked_until = NULL,
        lock_reason = NULL,
        failed_login_count = 0
    WHERE locked_until IS NOT NULL
      AND locked_until < NOW();
    
    GET DIAGNOSTICS unlocked_count = ROW_COUNT;
    
    RETURN unlocked_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION auto_unlock_expired_accounts() IS '自动解锁过期的账户锁定';
