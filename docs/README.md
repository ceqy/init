# Cuba ERP 文档中心

欢迎来到 Cuba ERP 项目文档中心。

## 文档导航

### 📚 开发指南
- [开发指南](guides/development.md) - 项目开发规范和流程
- [安全指南](guides/security.md) - 安全最佳实践
- [多租户指南](guides/multi-tenancy.md) - 多租户架构说明
- [gRPC API 测试指南](guides/GRPC_API_TESTING_GUIDE.md) - gRPC 接口测试
- [REST API 测试指南](guides/REST_API_TESTING_GUIDE.md) - REST 接口测试
- [前端集成指南](guides/FRONTEND_INTEGRATION_GUIDE.md) - 前端对接说明
- [前端 API 指南](guides/FRONTEND_API_GUIDE.md) - 前端 API 使用
- [项目启动清单](guides/PROJECT_STARTUP_CHECKLIST.md) - 新项目启动步骤

### 🏗️ 架构文档
- [架构概览](architecture/architecture.md) - 系统整体架构
- [架构详细设计](architecture/architecture-detailed.md) - 详细架构说明

### 📡 API 文档
- [API 总览](api/README.md)
- [IAM 服务](api/iam/README.md)
  - [认证服务](api/iam/auth-service.md)
  - [用户服务](api/iam/user-service.md)

### 📊 报告文档

#### 实现报告
- [OAuth2 领域模型完成](reports/implementation/OAUTH2_DOMAIN_MODEL_COMPLETION.md)
- [OAuth2 实现完成](reports/implementation/OAUTH2_IMPLEMENTATION_COMPLETE.md)
- [可观测性实现总结](reports/implementation/OBSERVABILITY_IMPLEMENTATION_SUMMARY.md)
- [密码重置完成](reports/implementation/PASSWORD_RESET_COMPLETION.md)
- [WebAuthn 总结](reports/implementation/WEBAUTHN_SUMMARY.md)
- [网关实现总结](reports/implementation/GATEWAY_IMPLEMENTATION_SUMMARY.md)
- [验证实现总结](reports/implementation/VERIFICATION_IMPLEMENTATION_SUMMARY.md)
- [文档实现总结](reports/implementation/DOCUMENTATION_IMPLEMENTATION_SUMMARY.md)

#### 测试报告
- [完整测试报告](reports/testing/COMPLETE_TEST_REPORT.md)
- [仓储测试总结](reports/testing/REPOSITORY_TEST_SUMMARY.md)
- [测试实现总结](reports/testing/TEST_IMPLEMENTATION_SUMMARY.md)
- [单元测试改进](reports/testing/UNIT_TESTS_IMPROVEMENT.md)

#### 状态报告
- [多租户状态](reports/status/MULTI_TENANT_STATUS.md)
- [多租户第二阶段总结](reports/status/MULTI_TENANT_PHASE2_SUMMARY.md)
- [安全问题状态报告](reports/status/SECURITY_ISSUES_STATUS_REPORT.md)
- [代码质量问题状态](reports/status/CODE_QUALITY_ISSUES_STATUS.md)
- [gRPC 反射状态](reports/status/GRPC_REFLECTION_STATUS.md)

### 📦 历史归档
- [2026年1月归档](archive/2026-01/) - 历史文档和完成报告

## 快速开始

1. 新开发者请先阅读 [开发指南](guides/development.md)
2. 了解系统架构请查看 [架构概览](architecture/architecture.md)
3. API 对接请参考 [API 文档](api/README.md)
4. 查看项目进展请访问 [报告文档](#-报告文档)

## 文档维护

- 所有新文档应放在对应的分类目录下
- 历史文档和完成报告归档到 `archive/YYYY-MM/` 目录
- 每个子目录应包含 README.md 说明文档用途

## 贡献指南

更新文档时请遵循：
1. 使用清晰的标题和结构
2. 提供代码示例和配置说明
3. 保持文档与代码同步更新
4. 使用相对链接引用其他文档
