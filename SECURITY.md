# 安全指南

## 🔐 凭证管理规则

### ❌ 绝对禁止

1. **不要提交真实凭证到 Git**
   - 数据库密码
   - API 密钥
   - JWT secrets
   - Vault role_id/secret_id
   - 私有 IP 地址和端口

2. **不要在代码中硬编码敏感信息**
   - 使用环境变量
   - 使用 Vault 存储
   - 使用配置文件（不提交）

### ✅ 正确做法

1. **使用 .env.local 存储本地凭证**
   ```bash
   cp .env.example .env.local
   # 编辑 .env.local，填入真实凭证
   # .env.local 已被 .gitignore 忽略
   ```

2. **文档和示例使用占位符**
   ```bash
   # ❌ 错误
   VAULT_ADDR=http://10.0.0.10:10018
   
   # ✅ 正确
   VAULT_ADDR=http://your-vault-server:8200
   ```

3. **生产环境使用 Vault**
   - 所有密钥存储在 Vault
   - 通过 AppRole 认证访问
   - 定期轮换密钥

## 🛡️ 安全检查工具

### 1. Pre-commit Hook（自动）

安装 Git hook：
```bash
# 复制 hook 到 .git/hooks/
cp .githooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

每次提交时自动检查敏感信息。

### 2. 手动扫描

运行安全检查脚本：
```bash
./scripts/check-secrets.sh
```

### 3. Git Secrets（推荐）

安装并配置：
```bash
# macOS
brew install git-secrets

# 初始化
git secrets --install
git secrets --register-aws

# 添加自定义规则
git secrets --add '10\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}:[0-9]+'
git secrets --add '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}'

# 扫描历史记录
git secrets --scan-history
```

## 🚨 如果不小心提交了敏感信息

### 立即行动

1. **轮换所有泄露的凭证**
   ```bash
   # 重新生成 Vault secret_id
   vault write -f auth/approle/role/cuba-services/secret-id
   
   # 修改数据库密码
   vault kv put secret/database password="new-password"
   ```

2. **从 Git 历史中删除**
   ```bash
   # 使用 BFG Repo-Cleaner
   brew install bfg
   bfg --replace-text passwords.txt
   git reflog expire --expire=now --all
   git gc --prune=now --aggressive
   
   # 强制推送（警告：会改写历史）
   git push --force
   ```

3. **通知团队**
   - 告知所有开发者
   - 更新所有环境的凭证
   - 检查是否有未授权访问

## 📋 检查清单

提交代码前：

- [ ] 运行 `./scripts/check-secrets.sh`
- [ ] 确认 `.env.local` 未被暂存
- [ ] 文档中使用占位符
- [ ] 测试代码从环境变量读取凭证
- [ ] 没有硬编码的 IP 地址/端口

## 🔗 相关资源

- [Git Secrets](https://github.com/awslabs/git-secrets)
- [BFG Repo-Cleaner](https://rtyley.github.io/bfg-repo-cleaner/)
- [Vault 最佳实践](https://www.vaultproject.io/docs/internals/security)
