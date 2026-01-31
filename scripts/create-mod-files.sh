#!/bin/bash

# 创建所有 mod.rs 文件的脚本
# 这个脚本会自动生成所有必要的模块组织文件

set -e

PROJECT_ROOT="/Users/x/init"
SRC_DIR="$PROJECT_ROOT/services/iam-identity/src"

echo "=========================================="
echo "创建 mod.rs 文件"
echo "=========================================="
echo ""

# ============================================
# Domain 层 mod.rs
# ============================================

echo "📝 创建 Domain 层 mod.rs..."

# domain/auth/mod.rs
cat > "$SRC_DIR/domain/auth/mod.rs" << 'EOF'
//! 认证领域实体

pub mod backup_code;
pub mod login_log;
pub mod password_reset_token;
pub mod session;
pub mod webauthn_credential;

pub use backup_code::*;
pub use login_log::*;
pub use password_reset_token::*;
pub use session::*;
pub use webauthn_credential::*;
EOF

# domain/user/mod.rs
cat > "$SRC_DIR/domain/user/mod.rs" << 'EOF'
//! 用户领域实体

pub mod email_verification;
pub mod phone_verification;
pub mod tenant;
pub mod user;

pub use email_verification::*;
pub use phone_verification::*;
pub use tenant::*;
pub use user::*;
EOF

# domain/oauth/mod.rs
cat > "$SRC_DIR/domain/oauth/mod.rs" << 'EOF'
//! OAuth 领域实体

pub mod access_token;
pub mod authorization_code;
pub mod oauth_client;
pub mod refresh_token;

pub use access_token::*;
pub use authorization_code::*;
pub use oauth_client::*;
pub use refresh_token::*;
EOF

# domain/repositories/auth/mod.rs
cat > "$SRC_DIR/domain/repositories/auth/mod.rs" << 'EOF'
//! 认证仓储接口

pub mod backup_code_repository;
pub mod login_log_repository;
pub mod password_reset_repository;
pub mod session_repository;
pub mod webauthn_credential_repository;

pub use backup_code_repository::*;
pub use login_log_repository::*;
pub use password_reset_repository::*;
pub use session_repository::*;
pub use webauthn_credential_repository::*;
EOF

# domain/repositories/user/mod.rs
cat > "$SRC_DIR/domain/repositories/user/mod.rs" << 'EOF'
//! 用户仓储接口

pub mod email_verification_repository;
pub mod phone_verification_repository;
pub mod tenant_repository;
pub mod user_repository;

pub use email_verification_repository::*;
pub use phone_verification_repository::*;
pub use tenant_repository::*;
pub use user_repository::*;
EOF

# domain/repositories/oauth/mod.rs
cat > "$SRC_DIR/domain/repositories/oauth/mod.rs" << 'EOF'
//! OAuth 仓储接口

pub mod access_token_repository;
pub mod authorization_code_repository;
pub mod oauth_client_repository;
pub mod refresh_token_repository;

pub use access_token_repository::*;
pub use authorization_code_repository::*;
pub use oauth_client_repository::*;
pub use refresh_token_repository::*;
EOF

# domain/repositories/mod.rs
cat > "$SRC_DIR/domain/repositories/mod.rs" << 'EOF'
//! 仓储接口

pub mod auth;
pub mod oauth;
pub mod user;
EOF

# domain/services/auth/mod.rs
cat > "$SRC_DIR/domain/services/auth/mod.rs" << 'EOF'
//! 认证领域服务

pub mod backup_code_service;
pub mod login_attempt_service;
pub mod password_reset_service;
pub mod password_service;
pub mod suspicious_login_detector;
pub mod totp_service;
pub mod webauthn_service;

pub use backup_code_service::*;
pub use login_attempt_service::*;
pub use password_reset_service::*;
pub use password_service::*;
pub use suspicious_login_detector::*;
pub use totp_service::*;
pub use webauthn_service::*;
EOF

# domain/services/user/mod.rs
cat > "$SRC_DIR/domain/services/user/mod.rs" << 'EOF'
//! 用户领域服务

pub mod email_verification_service;
pub mod phone_verification_service;

pub use email_verification_service::*;
pub use phone_verification_service::*;
EOF

# domain/services/oauth/mod.rs
cat > "$SRC_DIR/domain/services/oauth/mod.rs" << 'EOF'
//! OAuth 领域服务

pub mod oauth_service;

pub use oauth_service::*;
EOF

# domain/services/mod.rs
cat > "$SRC_DIR/domain/services/mod.rs" << 'EOF'
//! 领域服务

pub mod auth;
pub mod oauth;
pub mod user;
EOF

# domain/value_objects/mod.rs
cat > "$SRC_DIR/domain/value_objects/mod.rs" << 'EOF'
//! 值对象

pub mod email;
pub mod password;
pub mod tenant_context;
pub mod username;

pub use email::*;
pub use password::*;
pub use tenant_context::*;
pub use username::*;
EOF

# domain/events/mod.rs
cat > "$SRC_DIR/domain/events/mod.rs" << 'EOF'
//! 领域事件

pub mod user_events;

pub use user_events::*;
EOF

# domain/mod.rs
cat > "$SRC_DIR/domain/mod.rs" << 'EOF'
//! 领域层
//!
//! 包含所有业务实体、值对象、仓储接口、领域服务和领域事件

pub mod auth;
pub mod events;
pub mod oauth;
pub mod repositories;
pub mod services;
pub mod user;
pub mod value_objects;
EOF

echo "✅ Domain 层 mod.rs 创建完成"

# ============================================
# Application 层 mod.rs
# ============================================

echo "📝 创建 Application 层 mod.rs..."

# application/commands/auth/mod.rs
cat > "$SRC_DIR/application/commands/auth/mod.rs" << 'EOF'
//! 认证命令

pub mod login_command;
pub mod request_password_reset_command;
pub mod reset_password_command;

pub use login_command::*;
pub use request_password_reset_command::*;
pub use reset_password_command::*;
EOF

# application/commands/user/mod.rs
cat > "$SRC_DIR/application/commands/user/mod.rs" << 'EOF'
//! 用户命令

// 用户相关命令将在这里添加
EOF

# application/commands/oauth/mod.rs
cat > "$SRC_DIR/application/commands/oauth/mod.rs" << 'EOF'
//! OAuth 命令

pub mod authorize_command;
pub mod create_client_command;
pub mod token_command;

pub use authorize_command::*;
pub use create_client_command::*;
pub use token_command::*;
EOF

# application/commands/mod.rs
cat > "$SRC_DIR/application/commands/mod.rs" << 'EOF'
//! 命令（写操作）

pub mod auth;
pub mod oauth;
pub mod user;
EOF

# application/queries/auth/mod.rs
cat > "$SRC_DIR/application/queries/auth/mod.rs" << 'EOF'
//! 认证查询

pub mod validate_token_query;

pub use validate_token_query::*;
EOF

# application/queries/user/mod.rs
cat > "$SRC_DIR/application/queries/user/mod.rs" << 'EOF'
//! 用户查询

// 用户查询将在这里添加
EOF

# application/queries/oauth/mod.rs
cat > "$SRC_DIR/application/queries/oauth/mod.rs" << 'EOF'
//! OAuth 查询

// OAuth 查询将在这里添加
EOF

# application/queries/mod.rs
cat > "$SRC_DIR/application/queries/mod.rs" << 'EOF'
//! 查询（读操作）

pub mod auth;
pub mod oauth;
pub mod user;
EOF

# application/handlers/auth/mod.rs
cat > "$SRC_DIR/application/handlers/auth/mod.rs" << 'EOF'
//! 认证处理器

pub mod login_handler;
pub mod request_password_reset_handler;
pub mod reset_password_handler;

pub use login_handler::*;
pub use request_password_reset_handler::*;
pub use reset_password_handler::*;
EOF

# application/handlers/user/mod.rs
cat > "$SRC_DIR/application/handlers/user/mod.rs" << 'EOF'
//! 用户处理器

// 检查是否有文件
use std::fs;

// 动态导出所有处理器
EOF

# application/handlers/oauth/mod.rs
cat > "$SRC_DIR/application/handlers/oauth/mod.rs" << 'EOF'
//! OAuth 处理器

pub mod authorize_handler;
pub mod create_client_handler;
pub mod token_handler;

pub use authorize_handler::*;
pub use create_client_handler::*;
pub use token_handler::*;
EOF

# application/handlers/mod.rs
cat > "$SRC_DIR/application/handlers/mod.rs" << 'EOF'
//! 命令和查询处理器

pub mod auth;
pub mod oauth;
pub mod user;
EOF

# application/dto/auth/mod.rs
cat > "$SRC_DIR/application/dto/auth/mod.rs" << 'EOF'
//! 认证 DTO

// Auth DTOs
EOF

# application/dto/user/mod.rs
cat > "$SRC_DIR/application/dto/user/mod.rs" << 'EOF'
//! 用户 DTO

// User DTOs
EOF

# application/dto/oauth/mod.rs
cat > "$SRC_DIR/application/dto/oauth/mod.rs" << 'EOF'
//! OAuth DTO

// OAuth DTOs
EOF

# application/dto/mod.rs
cat > "$SRC_DIR/application/dto/mod.rs" << 'EOF'
//! 数据传输对象

pub mod auth;
pub mod oauth;
pub mod user;
EOF

# application/mod.rs
cat > "$SRC_DIR/application/mod.rs" << 'EOF'
//! 应用层
//!
//! 包含命令、查询、处理器和 DTO

pub mod commands;
pub mod dto;
pub mod handlers;
pub mod queries;
EOF

echo "✅ Application 层 mod.rs 创建完成"

# ============================================
# Infrastructure 层 mod.rs
# ============================================

echo "📝 创建 Infrastructure 层 mod.rs..."

# infrastructure/persistence/auth/mod.rs
cat > "$SRC_DIR/infrastructure/persistence/auth/mod.rs" << 'EOF'
//! 认证持久化实现

// 动态导出所有 repository 实现
EOF

# infrastructure/persistence/user/mod.rs
cat > "$SRC_DIR/infrastructure/persistence/user/mod.rs" << 'EOF'
//! 用户持久化实现

// 动态导出所有 repository 实现
EOF

# infrastructure/persistence/oauth/mod.rs
cat > "$SRC_DIR/infrastructure/persistence/oauth/mod.rs" << 'EOF'
//! OAuth 持久化实现

// 动态导出所有 repository 实现
EOF

# infrastructure/persistence/mod.rs
cat > "$SRC_DIR/infrastructure/persistence/mod.rs" << 'EOF'
//! 持久化实现

pub mod auth;
pub mod oauth;
pub mod user;
EOF

# infrastructure/cache/mod.rs
cat > "$SRC_DIR/infrastructure/cache/mod.rs" << 'EOF'
//! 缓存实现

// 动态导出所有 cache 实现
EOF

# infrastructure/external/mod.rs
cat > "$SRC_DIR/infrastructure/external/mod.rs" << 'EOF'
//! 外部服务集成

// 外部服务实现
EOF

# infrastructure/middleware/mod.rs (如果存在)
if [ -d "$SRC_DIR/infrastructure/middleware" ]; then
    cat > "$SRC_DIR/infrastructure/middleware/mod.rs" << 'EOF'
//! 中间件

// 中间件实现
EOF
fi

# infrastructure/mod.rs
cat > "$SRC_DIR/infrastructure/mod.rs" << 'EOF'
//! 基础设施层
//!
//! 包含持久化、缓存、外部服务等实现

pub mod cache;
pub mod external;
pub mod persistence;
EOF

echo "✅ Infrastructure 层 mod.rs 创建完成"

# ============================================
# API 层 mod.rs
# ============================================

echo "📝 创建 API 层 mod.rs..."

# api/grpc/mod.rs
cat > "$SRC_DIR/api/grpc/mod.rs" << 'EOF'
//! gRPC 服务实现

pub mod auth_service;
pub mod oauth_service;
pub mod user_service;

// Proto 模块保留在原位置
pub mod auth_proto {
    include!("../../auth/api/grpc/iam.auth.rs");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("../../auth/api/grpc/auth_descriptor.bin");
}

pub mod user_proto {
    include!("../../user/api/grpc/iam.user.rs");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("../../user/api/grpc/user_descriptor.bin");
}

pub mod oauth_proto {
    include!("../../oauth/api/grpc/iam.oauth.rs");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!("../../oauth/api/grpc/oauth_descriptor.bin");
}
EOF

# api/mod.rs
cat > "$SRC_DIR/api/mod.rs" << 'EOF'
//! API 层
//!
//! 包含 gRPC 服务实现

pub mod grpc;
EOF

echo "✅ API 层 mod.rs 创建完成"
echo ""

echo "=========================================="
echo "✅ 所有 mod.rs 文件创建完成！"
echo "=========================================="
echo ""
