#!/bin/bash

# WebAuthn 功能测试脚本

set -e

echo "=========================================="
echo "WebAuthn 功能测试"
echo "=========================================="

# 配置
BASE_URL="http://localhost:50051"
USER_ID="test-user-id"
USERNAME="testuser"
TENANT_ID="00000000-0000-0000-0000-000000000001"

echo ""
echo "1. 开始 WebAuthn 注册"
echo "----------------------------------------"
grpcurl -plaintext -d '{
  "user_id": "'$USER_ID'",
  "credential_name": "YubiKey 5"
}' localhost:50051 cuba.iam.auth.AuthService/StartWebAuthnRegistration

echo ""
echo "2. 列出用户的 WebAuthn 凭证"
echo "----------------------------------------"
grpcurl -plaintext -d '{
  "user_id": "'$USER_ID'"
}' localhost:50051 cuba.iam.auth.AuthService/ListWebAuthnCredentials

echo ""
echo "3. 开始 WebAuthn 认证"
echo "----------------------------------------"
grpcurl -plaintext -d '{
  "username": "'$USERNAME'",
  "tenant_id": "'$TENANT_ID'"
}' localhost:50051 cuba.iam.auth.AuthService/StartWebAuthnAuthentication

echo ""
echo "=========================================="
echo "测试完成"
echo "=========================================="
echo ""
echo "注意："
echo "- 完整的 WebAuthn 流程需要浏览器支持"
echo "- 此脚本仅测试服务端 API 可用性"
echo "- 实际注册和认证需要前端配合"
