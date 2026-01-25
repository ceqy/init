-- 为新创建的表添加 tenant_id 字段

-- webauthn_credentials 表
ALTER TABLE webauthn_credentials ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_webauthn_credentials_tenant_id ON webauthn_credentials(tenant_id);

-- backup_codes 表
ALTER TABLE backup_codes ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_backup_codes_tenant_id ON backup_codes(tenant_id);

-- login_logs 表
ALTER TABLE login_logs ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_login_logs_tenant_id ON login_logs(tenant_id);

-- email_verifications 表
ALTER TABLE email_verifications ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_email_verifications_tenant_id ON email_verifications(tenant_id);

-- phone_verifications 表
ALTER TABLE phone_verifications ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_phone_verifications_tenant_id ON phone_verifications(tenant_id);

-- oauth_clients 表
ALTER TABLE oauth_clients ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_oauth_clients_tenant_id ON oauth_clients(tenant_id);

-- authorization_codes 表
ALTER TABLE authorization_codes ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_authorization_codes_tenant_id ON authorization_codes(tenant_id);

-- access_tokens 表
ALTER TABLE access_tokens ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_access_tokens_tenant_id ON access_tokens(tenant_id);

-- refresh_tokens 表
ALTER TABLE refresh_tokens ADD COLUMN IF NOT EXISTS tenant_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
CREATE INDEX IF NOT EXISTS idx_refresh_tokens_tenant_id ON refresh_tokens(tenant_id);
