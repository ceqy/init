-- 添加 2FA 支持
-- 创建时间: 2026-01-26

-- 备份码表已经在 users 表中有 two_factor_enabled 和 two_factor_secret 字段
-- 这里只需要创建备份码表

-- 创建备份码表
CREATE TABLE IF NOT EXISTS backup_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code_hash VARCHAR(255) NOT NULL,
    used BOOLEAN DEFAULT FALSE,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_backup_codes_user_id ON backup_codes(user_id);
CREATE INDEX IF NOT EXISTS idx_backup_codes_used ON backup_codes(user_id, used);

-- 添加注释
COMMENT ON TABLE backup_codes IS '双因素认证备份码';
COMMENT ON COLUMN backup_codes.id IS '备份码 ID';
COMMENT ON COLUMN backup_codes.user_id IS '用户 ID';
COMMENT ON COLUMN backup_codes.code_hash IS '备份码的 SHA256 哈希值';
COMMENT ON COLUMN backup_codes.used IS '是否已使用';
COMMENT ON COLUMN backup_codes.used_at IS '使用时间';
COMMENT ON COLUMN backup_codes.created_at IS '创建时间';
