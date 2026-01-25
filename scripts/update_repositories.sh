#!/bin/bash
# 批量更新 Repository 实现添加 tenant_id 支持

echo "🔄 开始批量更新 Repository 实现..."

# 需要更新的文件列表
files=(
    "services/iam-identity/src/auth/infrastructure/persistence/postgres_password_reset_repository.rs"
    "services/iam-identity/src/auth/infrastructure/persistence/postgres_backup_code_repository.rs"
    "services/iam-identity/src/auth/infrastructure/persistence/postgres_webauthn_credential_repository.rs"
    "services/iam-identity/src/auth/infrastructure/persistence/postgres_login_log_repository.rs"
    "services/iam-identity/src/shared/infrastructure/persistence/postgres_email_verification_repository.rs"
    "services/iam-identity/src/shared/infrastructure/persistence/postgres_phone_verification_repository.rs"
    "services/iam-identity/src/oauth/infrastructure/persistence/postgres_oauth_client_repository.rs"
    "services/iam-identity/src/oauth/infrastructure/persistence/postgres_authorization_code_repository.rs"
    "services/iam-identity/src/oauth/infrastructure/persistence/postgres_access_token_repository.rs"
    "services/iam-identity/src/oauth/infrastructure/persistence/postgres_refresh_token_repository.rs"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo "✅ 找到: $file"
        # 这里需要手动更新每个文件
        # 主要是在 SQL 查询中添加 tenant_id 条件
    else
        echo "❌ 未找到: $file"
    fi
done

echo ""
echo "📝 更新步骤："
echo "1. 在所有 SELECT 查询的 WHERE 子句中添加: AND tenant_id = \$N"
echo "2. 在所有 INSERT 查询中添加 tenant_id 字段"
echo "3. 在所有 UPDATE/DELETE 查询的 WHERE 子句中添加: AND tenant_id = \$N"
echo "4. 在 Row 结构中添加: tenant_id: Uuid"
echo "5. 在转换函数中添加: tenant_id: TenantId::from_uuid(row.tenant_id)"
echo ""
echo "💡 提示：参考 postgres_session_repository.rs 和 postgres_user_repository.rs 的实现"
