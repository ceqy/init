-- 创建 WebAuthn 凭证表
CREATE TABLE IF NOT EXISTS webauthn_credentials (
    -- 主键
    id UUID PRIMARY KEY,
    
    -- 用户关联
    user_id UUID NOT NULL,
    
    -- WebAuthn 凭证数据
    credential_id BYTEA NOT NULL UNIQUE,  -- 凭证 ID（二进制）
    public_key BYTEA NOT NULL,            -- 公钥（二进制）
    counter BIGINT NOT NULL DEFAULT 0,    -- 签名计数器
    
    -- 凭证元数据
    name VARCHAR(255) NOT NULL,           -- 凭证名称（如 "YubiKey 5"）
    aaguid UUID,                          -- 认证器 AAGUID
    transports TEXT[],                    -- 传输方式（usb, nfc, ble, internal）
    
    -- 备份状态
    backup_eligible BOOLEAN NOT NULL DEFAULT false,  -- 是否可备份
    backup_state BOOLEAN NOT NULL DEFAULT false,     -- 是否已备份
    
    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    
    -- 外键约束
    CONSTRAINT fk_webauthn_credentials_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE
);

-- 索引
CREATE INDEX idx_webauthn_credentials_user_id ON webauthn_credentials(user_id);
CREATE INDEX idx_webauthn_credentials_credential_id ON webauthn_credentials(credential_id);
CREATE INDEX idx_webauthn_credentials_created_at ON webauthn_credentials(created_at DESC);

-- 注释
COMMENT ON TABLE webauthn_credentials IS 'WebAuthn 凭证表';
COMMENT ON COLUMN webauthn_credentials.credential_id IS '凭证 ID（二进制）';
COMMENT ON COLUMN webauthn_credentials.public_key IS '公钥（二进制）';
COMMENT ON COLUMN webauthn_credentials.counter IS '签名计数器（防重放攻击）';
COMMENT ON COLUMN webauthn_credentials.name IS '凭证名称';
COMMENT ON COLUMN webauthn_credentials.aaguid IS '认证器 AAGUID';
COMMENT ON COLUMN webauthn_credentials.transports IS '传输方式';
COMMENT ON COLUMN webauthn_credentials.backup_eligible IS '是否可备份';
COMMENT ON COLUMN webauthn_credentials.backup_state IS '是否已备份';
