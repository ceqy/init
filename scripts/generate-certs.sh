#!/bin/bash
# 证书生成脚本 - 用于开发和测试环境

set -e

CERT_DIR="deploy/certs"
DAYS_VALID=365

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Cuba ERP - TLS 证书生成工具${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""

# 创建证书目录
mkdir -p "$CERT_DIR"

# 1. 生成 CA 证书
echo -e "${YELLOW}步骤 1/4: 生成 CA 证书...${NC}"
if [ ! -f "$CERT_DIR/ca-key.pem" ]; then
    openssl genrsa -out "$CERT_DIR/ca-key.pem" 4096
    openssl req -new -x509 -days 3650 -key "$CERT_DIR/ca-key.pem" \
      -out "$CERT_DIR/ca-cert.pem" \
      -subj "/C=CN/ST=Beijing/L=Beijing/O=Cuba ERP/OU=IT/CN=Cuba CA"
    echo -e "${GREEN}✓ CA 证书生成完成${NC}"
else
    echo -e "${GREEN}✓ CA 证书已存在，跳过${NC}"
fi

# 2. 生成服务证书函数
generate_service_cert() {
    SERVICE=$1
    DNS_NAMES=$2

    echo -e "${YELLOW}步骤: 生成 ${SERVICE} 证书...${NC}"

    # 生成私钥
    openssl genrsa -out "$CERT_DIR/${SERVICE}-key.pem" 2048

    # 生成 CSR
    openssl req -new -key "$CERT_DIR/${SERVICE}-key.pem" \
      -out "$CERT_DIR/${SERVICE}.csr" \
      -subj "/C=CN/ST=Beijing/L=Beijing/O=Cuba ERP/OU=IT/CN=${SERVICE}"

    # 创建扩展配置
    cat > "$CERT_DIR/${SERVICE}-ext.cnf" <<EOF
subjectAltName=${DNS_NAMES}
extendedKeyUsage=serverAuth,clientAuth
EOF

    # 签发证书
    openssl x509 -req -days $DAYS_VALID \
      -in "$CERT_DIR/${SERVICE}.csr" \
      -CA "$CERT_DIR/ca-cert.pem" \
      -CAkey "$CERT_DIR/ca-key.pem" \
      -CAcreateserial \
      -out "$CERT_DIR/${SERVICE}-cert.pem" \
      -extfile "$CERT_DIR/${SERVICE}-ext.cnf"

    # 清理临时文件
    rm "$CERT_DIR/${SERVICE}.csr" "$CERT_DIR/${SERVICE}-ext.cnf"

    # 设置权限
    chmod 600 "$CERT_DIR/${SERVICE}-key.pem"
    chmod 644 "$CERT_DIR/${SERVICE}-cert.pem"

    echo -e "${GREEN}✓ ${SERVICE} 证书生成完成${NC}"
}

# 3. 生成各服务证书
echo -e "${YELLOW}步骤 2/4: 生成 Gateway Envoy 证书...${NC}"
generate_service_cert "gateway-envoy" "DNS:gateway-envoy,DNS:localhost,IP:127.0.0.1"

echo -e "${YELLOW}步骤 3/4: 生成 IAM Envoy 证书...${NC}"
generate_service_cert "iam-access-envoy" "DNS:iam-access-envoy,DNS:iam-envoy,DNS:localhost,IP:127.0.0.1"

echo -e "${YELLOW}步骤 4/4: 生成 Consul 证书...${NC}"
generate_service_cert "consul" "DNS:consul,DNS:localhost,IP:127.0.0.1"

# 4. 验证证书
echo ""
echo -e "${YELLOW}验证证书...${NC}"
for cert in gateway-envoy iam-access-envoy consul; do
    if openssl verify -CAfile "$CERT_DIR/ca-cert.pem" "$CERT_DIR/${cert}-cert.pem" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ ${cert} 证书验证通过${NC}"
    else
        echo -e "${RED}✗ ${cert} 证书验证失败${NC}"
        exit 1
    fi
done

# 5. 显示证书信息
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}证书生成完成！${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "证书位置: $CERT_DIR/"
echo ""
echo "生成的文件:"
ls -lh "$CERT_DIR"/*.pem | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo "证书有效期: $DAYS_VALID 天"
echo ""
echo "下一步:"
echo "  1. 备份 CA 私钥: cp $CERT_DIR/ca-key.pem /secure/location/"
echo "  2. 启动 TLS 环境: cd deploy/docker && docker-compose -f docker-compose.envoy-tls.yml up -d"
echo "  3. 验证 TLS: curl --cacert $CERT_DIR/ca-cert.pem https://localhost:50051/health"
echo ""
