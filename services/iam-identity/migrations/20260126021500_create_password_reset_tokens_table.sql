-- 创建密码重置令牌表
CREATE TABLE IF NOT EXISTS password_reset_tokens (
    -- 主键
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 用户 ID
    user_id UUID NOT NULL,
    
    -- 令牌哈希（SHA256）
    token_hash VARCHAR(64) NOT NULL,
    
    -- 过期时间
    expires_at TIMESTAMPTZ NOT NULL,
    
    -- 是否已使用
    used BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- 使用时间
    used_at TIMESTAMPTZ,
    
    -- 创建时间
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 外键约束
    CONSTRAINT fk_password_reset_tokens_user_id 
        FOREIGN KEY (user_id) 
        REFERENCES users(id) 
        ON DELETE CASCADE
);

-- 索引
CREATE INDEX idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
CREATE INDEX idx_password_reset_tokens_token_hash ON password_reset_tokens(token_hash);
CREATE INDEX idx_password_reset_tokens_expires_at ON password_reset_tokens(expires_at);
CREATE INDEX idx_password_reset_tokens_used ON password_reset_tokens(user_id, used);

-- 注释
COMMENT ON TABLE password_reset_tokens IS '密码重置令牌表';
COMMENT ON COLUMN password_reset_tokens.id IS '令牌 ID';
COMMENT ON COLUMN password_reset_tokens.user_id IS '用户 ID';
COMMENT ON COLUMN password_reset_tokens.token_hash IS '令牌哈希（SHA256）';
COMMENT ON COLUMN password_reset_tokens.expires_at IS '过期时间';
COMMENT ON COLUMN password_reset_tokens.used IS '是否已使用';
COMMENT ON COLUMN password_reset_tokens.used_at IS '使用时间';
COMMENT ON COLUMN password_reset_tokens.created_at IS '创建时间';
