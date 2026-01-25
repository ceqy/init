#!/bin/bash

# 2FA 功能测试脚本
# 使用 grpcurl 测试完整的 2FA 流程

set -e

GRPC_HOST="localhost:50051"
USER_GRPC_HOST="localhost:50052"
TENANT_ID="00000000-0000-0000-0000-000000000001"

echo "=========================================="
echo "2FA 功能测试"
echo "=========================================="
echo ""

# 检查 grpcurl 是否安装
if ! command -v grpcurl &> /dev/null; then
    echo "❌ grpcurl 未安装"
    echo "请安装: brew install grpcurl"
    exit 1
fi

echo "✅ grpcurl 已安装"
echo ""

# 步骤 1: 创建测试用户
echo "步骤 1: 创建测试用户"
echo "----------------------------"

CREATE_USER_RESPONSE=$(grpcurl -plaintext \
  -d "{
    \"username\": \"testuser2fa\",
    \"email\": \"test2fa@example.com\",
    \"password\": \"Test123456!\",
    \"tenant_id\": \"$TENANT_ID\"
  }" \
  $USER_GRPC_HOST cuba.iam.user.UserService/CreateUser 2>&1)

if echo "$CREATE_USER_RESPONSE" | grep -q "error"; then
    echo "⚠️  用户可能已存在，继续测试..."
else
    echo "✅ 用户创建成功"
fi

USER_ID=$(echo "$CREATE_USER_RESPONSE" | grep -o '"id": "[^"]*"' | head -1 | cut -d'"' -f4)
echo "用户 ID: $USER_ID"
echo ""

# 步骤 2: 登录获取用户 ID（如果创建失败）
if [ -z "$USER_ID" ]; then
    echo "步骤 2: 登录获取用户信息"
    echo "----------------------------"
    
    LOGIN_RESPONSE=$(grpcurl -plaintext \
      -d "{
        \"username\": \"testuser2fa\",
        \"password\": \"Test123456!\",
        \"tenant_id\": \"$TENANT_ID\"
      }" \
      $GRPC_HOST cuba.iam.auth.AuthService/Login)
    
    USER_ID=$(echo "$LOGIN_RESPONSE" | grep -o '"id": "[^"]*"' | head -1 | cut -d'"' -f4)
    echo "✅ 登录成功"
    echo "用户 ID: $USER_ID"
    echo ""
fi

# 步骤 3: 启用 2FA（第一步 - 获取 QR 码）
echo "步骤 3: 启用 2FA - 获取 QR 码"
echo "----------------------------"

ENABLE_2FA_STEP1=$(grpcurl -plaintext \
  -d "{
    \"user_id\": \"$USER_ID\",
    \"method\": \"totp\",
    \"verification_code\": \"\"
  }" \
  $GRPC_HOST cuba.iam.auth.AuthService/Enable2FA)

SECRET=$(echo "$ENABLE_2FA_STEP1" | grep -o '"secret": "[^"]*"' | cut -d'"' -f4)
QR_URL=$(echo "$ENABLE_2FA_STEP1" | grep -o '"qrCodeUrl": "[^"]*"' | cut -d'"' -f4)

echo "✅ QR 码生成成功"
echo "Secret: $SECRET"
echo "QR Code URL: $QR_URL"
echo ""
echo "📱 请使用 Google Authenticator 扫描以下 URL 生成的 QR 码："
echo "$QR_URL"
echo ""

# 生成 QR 码（如果安装了 qrencode）
if command -v qrencode &> /dev/null; then
    echo "生成 QR 码到终端："
    echo "$QR_URL" | qrencode -t ANSIUTF8
    echo ""
fi

# 步骤 4: 等待用户输入验证码
echo "步骤 4: 验证并启用 2FA"
echo "----------------------------"
echo "请从 Google Authenticator 获取 6 位验证码："
read -p "验证码: " TOTP_CODE

ENABLE_2FA_STEP2=$(grpcurl -plaintext \
  -d "{
    \"user_id\": \"$USER_ID\",
    \"method\": \"totp\",
    \"verification_code\": \"$TOTP_CODE\"
  }" \
  $GRPC_HOST cuba.iam.auth.AuthService/Enable2FA)

if echo "$ENABLE_2FA_STEP2" | grep -q '"enabled": true'; then
    echo "✅ 2FA 启用成功！"
    
    # 提取备份码
    BACKUP_CODES=$(echo "$ENABLE_2FA_STEP2" | grep -o '"backupCodes": \[[^]]*\]' | sed 's/"backupCodes": //')
    echo ""
    echo "🔑 备份码（请妥善保存）："
    echo "$BACKUP_CODES" | jq -r '.[]' 2>/dev/null || echo "$BACKUP_CODES"
    echo ""
else
    echo "❌ 2FA 启用失败"
    echo "$ENABLE_2FA_STEP2"
    exit 1
fi

# 步骤 5: 测试登录（需要 2FA）
echo "步骤 5: 测试登录（需要 2FA）"
echo "----------------------------"

LOGIN_WITH_2FA=$(grpcurl -plaintext \
  -d "{
    \"username\": \"testuser2fa\",
    \"password\": \"Test123456!\",
    \"tenant_id\": \"$TENANT_ID\"
  }" \
  $GRPC_HOST cuba.iam.auth.AuthService/Login)

if echo "$LOGIN_WITH_2FA" | grep -q '"require2fa": true'; then
    echo "✅ 登录成功，需要 2FA 验证"
    SESSION_ID=$(echo "$LOGIN_WITH_2FA" | grep -o '"sessionId": "[^"]*"' | cut -d'"' -f4)
    echo "临时会话 ID: $SESSION_ID"
    echo ""
else
    echo "❌ 登录失败或未要求 2FA"
    echo "$LOGIN_WITH_2FA"
    exit 1
fi

# 步骤 6: 验证 2FA（使用 TOTP 码）
echo "步骤 6: 验证 2FA（使用 TOTP 码）"
echo "----------------------------"
echo "请从 Google Authenticator 获取新的 6 位验证码："
read -p "验证码: " TOTP_CODE_2

VERIFY_2FA=$(grpcurl -plaintext \
  -d "{
    \"user_id\": \"$USER_ID\",
    \"code\": \"$TOTP_CODE_2\"
  }" \
  $GRPC_HOST cuba.iam.auth.AuthService/Verify2FA)

if echo "$VERIFY_2FA" | grep -q '"success": true'; then
    echo "✅ 2FA 验证成功！"
    ACCESS_TOKEN=$(echo "$VERIFY_2FA" | grep -o '"accessToken": "[^"]*"' | cut -d'"' -f4)
    echo "访问令牌: ${ACCESS_TOKEN:0:50}..."
    echo ""
else
    echo "❌ 2FA 验证失败"
    echo "$VERIFY_2FA"
    exit 1
fi

# 步骤 7: 测试备份码验证（可选）
echo "步骤 7: 测试备份码验证（可选）"
echo "----------------------------"
read -p "是否测试备份码验证？(y/n): " TEST_BACKUP

if [ "$TEST_BACKUP" = "y" ]; then
    # 先登录
    grpcurl -plaintext \
      -d "{
        \"username\": \"testuser2fa\",
        \"password\": \"Test123456!\",
        \"tenant_id\": \"$TENANT_ID\"
      }" \
      $GRPC_HOST cuba.iam.auth.AuthService/Login > /dev/null
    
    echo "请输入一个备份码（8 位数字）："
    read -p "备份码: " BACKUP_CODE
    
    VERIFY_BACKUP=$(grpcurl -plaintext \
      -d "{
        \"user_id\": \"$USER_ID\",
        \"code\": \"$BACKUP_CODE\"
      }" \
      $GRPC_HOST cuba.iam.auth.AuthService/Verify2FA)
    
    if echo "$VERIFY_BACKUP" | grep -q '"success": true'; then
        echo "✅ 备份码验证成功！"
        echo "⚠️  注意：此备份码已被使用，不能再次使用"
        echo ""
    else
        echo "❌ 备份码验证失败"
        echo "$VERIFY_BACKUP"
    fi
fi

# 步骤 8: 禁用 2FA（可选）
echo "步骤 8: 禁用 2FA（可选）"
echo "----------------------------"
read -p "是否禁用 2FA？(y/n): " DISABLE_2FA

if [ "$DISABLE_2FA" = "y" ]; then
    DISABLE_RESPONSE=$(grpcurl -plaintext \
      -d "{
        \"user_id\": \"$USER_ID\",
        \"password\": \"Test123456!\"
      }" \
      $GRPC_HOST cuba.iam.auth.AuthService/Disable2FA)
    
    if echo "$DISABLE_RESPONSE" | grep -q '"success": true'; then
        echo "✅ 2FA 已禁用"
    else
        echo "❌ 禁用 2FA 失败"
        echo "$DISABLE_RESPONSE"
    fi
fi

echo ""
echo "=========================================="
echo "✅ 2FA 功能测试完成！"
echo "=========================================="
