-- 更新 WebAuthn 凭证表结构，使用 passkey_data 替代单独字段
-- 这简化了与 webauthn-rs 0.5.x 的集成

-- 添加新字段
ALTER TABLE webauthn_credentials
    ADD COLUMN IF NOT EXISTS passkey_data BYTEA,
    ADD COLUMN IF NOT EXISTS tenant_id UUID;

-- 更新 tenant_id（从 users 表获取）
UPDATE webauthn_credentials wc
SET tenant_id = u.tenant_id
FROM users u
WHERE wc.user_id = u.id AND wc.tenant_id IS NULL;

-- 设置 tenant_id 为 NOT NULL
ALTER TABLE webauthn_credentials
    ALTER COLUMN tenant_id SET NOT NULL;

-- 添加外键约束
ALTER TABLE webauthn_credentials
    ADD CONSTRAINT fk_webauthn_credentials_tenant
        FOREIGN KEY (tenant_id)
        REFERENCES tenants(id)
        ON DELETE CASCADE;

-- 添加索引
CREATE INDEX IF NOT EXISTS idx_webauthn_credentials_tenant_id ON webauthn_credentials(tenant_id);

-- 注释
COMMENT ON COLUMN webauthn_credentials.passkey_data IS '序列化的 Passkey 数据（JSON）';
COMMENT ON COLUMN webauthn_credentials.tenant_id IS '租户 ID';

-- 注意：旧字段（credential_id, public_key, counter, backup_eligible, backup_state）
-- 暂时保留以支持数据迁移，可以在确认数据迁移完成后删除
