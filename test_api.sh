#!/bin/bash
# IAM Identity 服务接口测试命令

# 服务地址
HOST="localhost:50051"
TENANT_ID="00000000-0000-0000-0000-000000000001"

echo "=== IAM Identity 服务接口测试 ==="
echo ""

# 1. 注册用户
echo "1. 注册用户"
echo "grpcurl -plaintext -d '{
  \"tenant_id\": \"'$TENANT_ID'\",
  \"username\": \"testuser\",
  \"email\": \"test@example.com\",
  \"password\": \"Test123456\"
}' $HOST cuba.iam.auth.AuthService/Register"
echo ""

# 2. 登录（成功）
echo "2. 登录（成功）"
echo "grpcurl -plaintext -d '{
  \"tenant_id\": \"'$TENANT_ID'\",
  \"username\": \"testuser\",
  \"password\": \"Test123456\",
  \"ip_address\": \"127.0.0.1\",
  \"device_info\": \"Mozilla/5.0\"
}' $HOST cuba.iam.auth.AuthService/Login"
echo ""

# 3. 登录（失败 - 测试暴力破解）
echo "3. 测试暴力破解（连续5次失败）"
for i in {1..5}; do
  echo "  尝试 $i:"
  echo "  grpcurl -plaintext -d '{
    \"tenant_id\": \"'$TENANT_ID'\",
    \"username\": \"testuser\",
    \"password\": \"WrongPassword\",
    \"ip_address\": \"127.0.0.1\"
  }' $HOST cuba.iam.auth.AuthService/Login"
done
echo ""

# 4. 第6次尝试（应该被锁定）
echo "4. 第6次尝试（应该返回账户锁定错误）"
echo "grpcurl -plaintext -d '{
  \"tenant_id\": \"'$TENANT_ID'\",
  \"username\": \"testuser\",
  \"password\": \"Test123456\",
  \"ip_address\": \"127.0.0.1\"
}' $HOST cuba.iam.auth.AuthService/Login"
echo ""

# 5. 刷新令牌
echo "5. 刷新令牌"
echo "grpcurl -plaintext -d '{
  \"refresh_token\": \"<从登录响应获取>\"
}' $HOST cuba.iam.auth.AuthService/RefreshToken"
echo ""

# 6. 登出
echo "6. 登出"
echo "grpcurl -plaintext -H 'authorization: Bearer <access_token>' -d '{
  \"session_id\": \"<从登录响应获取>\"
}' $HOST cuba.iam.auth.AuthService/Logout"
echo ""

# 7. 获取用户信息
echo "7. 获取用户信息"
echo "grpcurl -plaintext -H 'authorization: Bearer <access_token>' -d '{
  \"user_id\": \"<user_id>\"
}' $HOST cuba.iam.identity.UserService/GetUser"
echo ""

# 8. 更新用户信息
echo "8. 更新用户信息"
echo "grpcurl -plaintext -H 'authorization: Bearer <access_token>' -d '{
  \"user_id\": \"<user_id>\",
  \"display_name\": \"Test User\",
  \"phone\": \"+86 13800138000\"
}' $HOST cuba.iam.identity.UserService/UpdateUser"
echo ""

# 9. 修改密码
echo "9. 修改密码"
echo "grpcurl -plaintext -H 'authorization: Bearer <access_token>' -d '{
  \"old_password\": \"Test123456\",
  \"new_password\": \"NewPass123456\"
}' $HOST cuba.iam.auth.AuthService/ChangePassword"
echo ""

# 10. 请求密码重置
echo "10. 请求密码重置"
echo "grpcurl -plaintext -d '{
  \"tenant_id\": \"'$TENANT_ID'\",
  \"email\": \"test@example.com\"
}' $HOST cuba.iam.auth.AuthService/RequestPasswordReset"
echo ""

# 11. 重置密码
echo "11. 重置密码"
echo "grpcurl -plaintext -d '{
  \"token\": \"<从邮件获取>\",
  \"new_password\": \"ResetPass123456\"
}' $HOST cuba.iam.auth.AuthService/ResetPassword"
echo ""

# 12. 启用2FA
echo "12. 启用2FA"
echo "grpcurl -plaintext -H 'authorization: Bearer <access_token>' -d '{}' \
  $HOST cuba.iam.auth.AuthService/Enable2FA"
echo ""

# 13. 验证2FA
echo "13. 验证2FA"
echo "grpcurl -plaintext -H 'authorization: Bearer <access_token>' -d '{
  \"code\": \"123456\"
}' $HOST cuba.iam.auth.AuthService/Verify2FA"
echo ""

# 14. 查询登录日志
echo "14. 查询登录日志"
echo "grpcurl -plaintext -H 'authorization: Bearer <access_token>' -d '{
  \"user_id\": \"<user_id>\",
  \"page\": 1,
  \"page_size\": 10
}' $HOST cuba.iam.auth.AuthService/GetLoginLogs"
echo ""

# 15. 查询可疑登录
echo "15. 查询可疑登录"
echo "grpcurl -plaintext -H 'authorization: Bearer <access_token>' -d '{
  \"days\": 7,
  \"limit\": 10
}' $HOST cuba.iam.auth.AuthService/GetSuspiciousLogins"
echo ""

# 16. 列出用户会话
echo "16. 列出用户会话"
echo "grpcurl -plaintext -H 'authorization: Bearer <access_token>' -d '{}' \
  $HOST cuba.iam.auth.AuthService/ListSessions"
echo ""

# 17. 撤销会话
echo "17. 撤销会话"
echo "grpcurl -plaintext -H 'authorization: Bearer <access_token>' -d '{
  \"session_id\": \"<session_id>\"
}' $HOST cuba.iam.auth.AuthService/RevokeSession"
echo ""

# 18. 撤销所有会话
echo "18. 撤销所有会话"
echo "grpcurl -plaintext -H 'authorization: Bearer <access_token>' -d '{}' \
  $HOST cuba.iam.auth.AuthService/RevokeAllSessions"
echo ""

echo "=== 多租户测试 ==="
echo ""

# 19. 创建租户（需要管理员权限）
echo "19. 创建租户"
echo "grpcurl -plaintext -H 'authorization: Bearer <admin_token>' -d '{
  \"name\": \"tenant2\",
  \"display_name\": \"Tenant 2\"
}' $HOST cuba.iam.tenant.TenantService/CreateTenant"
echo ""

# 20. 使用不同租户登录
echo "20. 使用不同租户登录"
echo "grpcurl -plaintext -d '{
  \"tenant_id\": \"<tenant2_id>\",
  \"username\": \"user2\",
  \"password\": \"Pass123456\"
}' $HOST cuba.iam.auth.AuthService/Login"
echo ""

# 21. 验证租户隔离（应该看不到其他租户的数据）
echo "21. 验证租户隔离"
echo "grpcurl -plaintext -H 'authorization: Bearer <tenant2_token>' -d '{
  \"page\": 1,
  \"page_size\": 10
}' $HOST cuba.iam.identity.UserService/ListUsers"
echo ""

echo "=== 健康检查 ==="
echo ""

# 22. 健康检查
echo "22. 健康检查（HTTP）"
echo "curl http://localhost:51051/health"
echo ""

# 23. 就绪检查
echo "23. 就绪检查（HTTP）"
echo "curl http://localhost:51051/ready"
echo ""

# 24. Metrics
echo "24. Metrics（HTTP）"
echo "curl http://localhost:51051/metrics"
echo ""

echo "=== 完整测试流程示例 ==="
echo ""
cat << 'EXAMPLE'
# 完整测试流程
# 1. 注册
REGISTER_RESP=$(grpcurl -plaintext -d '{
  "tenant_id": "00000000-0000-0000-0000-000000000001",
  "username": "demo",
  "email": "demo@example.com",
  "password": "Demo123456"
}' localhost:50051 cuba.iam.auth.AuthService/Register)

# 2. 登录
LOGIN_RESP=$(grpcurl -plaintext -d '{
  "tenant_id": "00000000-0000-0000-0000-000000000001",
  "username": "demo",
  "password": "Demo123456",
  "ip_address": "127.0.0.1"
}' localhost:50051 cuba.iam.auth.AuthService/Login)

# 3. 提取 token
ACCESS_TOKEN=$(echo $LOGIN_RESP | jq -r '.tokens.access_token')

# 4. 使用 token 访问受保护接口
grpcurl -plaintext -H "authorization: Bearer $ACCESS_TOKEN" -d '{}' \
  localhost:50051 cuba.iam.identity.UserService/GetCurrentUser
EXAMPLE

echo ""
echo "=== 注意事项 ==="
echo "1. 需要安装 grpcurl: brew install grpcurl"
echo "2. 需要安装 jq: brew install jq"
echo "3. 确保服务已启动: just dev"
echo "4. 默认租户ID: 00000000-0000-0000-0000-000000000001"
echo "5. 健康检查端口 = gRPC端口 + 1000"
