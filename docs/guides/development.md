# 开发指南

## 环境搭建

### 前置条件

- **Rust**: 1.75 或更高版本
- **Docker**: 20.10 或更高版本
- **Docker Compose**: 2.0 或更高版本
- **just**: 命令运行器（可选但推荐）
- **protoc**: Protocol Buffers 编译器
- **buf**: Proto 文件管理工具（可选）

### 安装 Rust

```bash
# 使用 rustup 安装
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 验证安装
rustc --version
cargo --version
```

### 安装 just

```bash
# macOS
brew install just

# Linux
cargo install just

# 验证安装
just --version
```

### 安装 protoc

```bash
# macOS
brew install protobuf

# Linux (Ubuntu/Debian)
sudo apt install protobuf-compiler

# 验证安装
protoc --version
```

### 安装 buf（可选）

```bash
# macOS
brew install bufbuild/buf/buf

# Linux
curl -sSL "https://github.com/bufbuild/buf/releases/latest/download/buf-$(uname -s)-$(uname -m)" \
  -o /usr/local/bin/buf
chmod +x /usr/local/bin/buf

# 验证安装
buf --version
```

## 克隆项目

```bash
git clone https://github.com/cuba-erp/cuba-erp.git
cd cuba-erp
```

## 启动基础设施

项目使用 Docker Compose 管理基础设施服务。

```bash
# 启动所有基础设施服务
just infra-up

# 或使用 docker compose
docker compose -f deploy/docker/docker-compose.yml up -d
```

基础设施服务包括：
- **PostgreSQL** (端口 5432) - 主数据库
- **Redis** (端口 6379) - 缓存和会话存储
- **Kafka** (端口 9092) - 消息队列
- **Zookeeper** (端口 2181) - Kafka 依赖
- **ClickHouse** (端口 8123, 9000) - 分析数据库

### 验证基础设施

```bash
# 检查服务状态
docker compose -f deploy/docker/docker-compose.yml ps

# 查看日志
docker compose -f deploy/docker/docker-compose.yml logs -f

# 测试 PostgreSQL 连接
psql -h localhost -U postgres -d cuba

# 测试 Redis 连接
redis-cli ping
```

## 构建项目

```bash
# 构建所有服务
just build

# 或使用 cargo
cargo build

# 构建发布版本
just build-release
# 或
cargo build --release
```

## 运行数据库迁移

```bash
# 安装 sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# 运行 IAM Identity 服务迁移
just migrate iam-identity

# 或手动运行
cd services/iam-identity
sqlx migrate run --database-url "postgres://postgres:postgres@localhost:5432/cuba"
```

## 运行服务

### 开发模式

```bash
# 运行 IAM Identity 服务
just dev iam-identity

# 或使用 cargo
cd services/iam-identity
cargo run
```

### 生产模式

```bash
# 构建发布版本
just build-release

# 运行
./target/release/iam-identity
```

### 使用环境变量

```bash
# 复制环境变量模板
cp .env.example .env

# 编辑环境变量
vim .env

# 使用环境变量运行
source .env
cargo run
```

## 运行测试

### 单元测试

```bash
# 运行所有单元测试
just test

# 运行特定服务的测试
just test iam-identity

# 或使用 cargo
cargo test --lib

# 运行特定测试
cargo test test_user_creation
```

### 集成测试

```bash
# 运行集成测试（需要数据库）
just test-integration

# 或使用 cargo
cargo test --test '*'
```

### 测试覆盖率

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html --output-dir coverage

# 查看报告
open coverage/index.html
```

## 代码质量检查

### Clippy（Lint）

```bash
# 运行 clippy
just lint

# 或使用 cargo
cargo clippy --all-targets --all-features -- -D warnings
```

### 格式化

```bash
# 检查格式
just fmt-check

# 自动格式化
just fmt

# 或使用 cargo
cargo fmt --all -- --check
cargo fmt --all
```

### 安全审计

```bash
# 安装 cargo-audit
cargo install cargo-audit

# 运行安全审计
cargo audit
```

## 开发工具

### IDE 推荐

#### VS Code

推荐扩展：
- **rust-analyzer**: Rust 语言服务器
- **CodeLLDB**: 调试器
- **Even Better TOML**: TOML 文件支持
- **Error Lens**: 内联错误显示
- **crates**: Cargo.toml 依赖管理

配置文件 (`.vscode/settings.json`):
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

#### IntelliJ IDEA / CLion

安装插件：
- **Rust**: 官方 Rust 插件
- **TOML**: TOML 文件支持

### 调试

#### VS Code 调试配置

`.vscode/launch.json`:
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug IAM Identity",
      "cargo": {
        "args": [
          "build",
          "--bin=iam-identity",
          "--package=iam-identity"
        ],
        "filter": {
          "name": "iam-identity",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "debug"
      }
    }
  ]
}
```

#### 命令行调试

```bash
# 使用 rust-gdb
rust-gdb target/debug/iam-identity

# 使用 rust-lldb
rust-lldb target/debug/iam-identity
```

### gRPC 测试工具

#### grpcurl

```bash
# 安装
brew install grpcurl

# 列出服务
grpcurl -plaintext localhost:50051 list

# 调用方法
grpcurl -plaintext -d '{"username":"admin","password":"password","tenant_id":"default"}' \
  localhost:50051 cuba.iam.auth.AuthService/Login
```

#### BloomRPC

图形化 gRPC 客户端：https://github.com/bloomrpc/bloomrpc

### 数据库工具

#### psql

```bash
# 连接数据库
psql -h localhost -U postgres -d cuba

# 常用命令
\dt          # 列出所有表
\d users     # 查看表结构
\q           # 退出
```

#### DBeaver

图形化数据库客户端：https://dbeaver.io/

## 项目结构

```
cuba-erp/
├── crates/                    # 共享库
│   ├── common/                # 通用类型和工具
│   ├── errors/                # 统一错误处理
│   ├── config/                # 配置加载
│   ├── telemetry/             # 可观测性
│   ├── auth-core/             # 认证核心
│   ├── ports/                 # 抽象 trait 层
│   ├── domain-core/           # 跨 context 领域核心
│   ├── cqrs-core/             # CQRS 核心
│   ├── event-core/            # 事件核心
│   └── adapters/              # 基础设施适配器
│       ├── postgres/
│       ├── redis/
│       ├── clickhouse/
│       └── kafka/
├── proto/                     # Protocol Buffers 定义
│   ├── common/                # 通用消息类型
│   └── iam/                   # IAM 服务定义
├── services/                  # 微服务
│   └── iam-identity/          # 身份服务
│       ├── src/
│       │   ├── main.rs        # 服务入口
│       │   ├── lib.rs         # 库入口
│       │   ├── config.rs      # 配置
│       │   ├── error.rs       # 错误定义
│       │   ├── api/           # API 层（gRPC）
│       │   ├── application/   # 应用层（CQRS）
│       │   ├── domain/        # 领域层（DDD）
│       │   └── infrastructure/ # 基础设施层
│       ├── tests/             # 测试
│       │   ├── unit/          # 单元测试
│       │   └── integration/   # 集成测试
│       ├── migrations/        # 数据库迁移
│       └── config/            # 配置文件
├── gateway/                   # API 网关
├── bootstrap/                 # 统一启动骨架
├── deploy/                    # 部署配置
│   ├── docker/                # Docker 配置
│   └── k3s/                   # Kubernetes 配置
├── docs/                      # 文档
│   ├── api/                   # API 文档
│   ├── guides/                # 开发指南
│   └── architecture.md        # 架构文档
├── justfile                   # Just 命令定义
├── Cargo.toml                 # Workspace 配置
└── README.md                  # 项目说明
```

## 开发工作流

### 1. 创建新功能分支

```bash
git checkout -b feature/user-profile-update
```

### 2. 开发功能

```bash
# 编写代码
vim services/iam-identity/src/...

# 运行测试
just test

# 检查代码质量
just lint
just fmt-check
```

### 3. 提交代码

```bash
# 添加文件
git add .

# 提交（使用中文提交信息）
git commit -m "feat: 添加用户资料更新功能"

# 推送到远程
git push origin feature/user-profile-update
```

### 4. 创建 Pull Request

在 GitHub 上创建 Pull Request，等待代码审查。

## 常见任务

### 添加新的依赖

```bash
# 添加到 workspace
vim Cargo.toml

# 或添加到特定服务
cd services/iam-identity
cargo add tokio --features full
```

### 生成 Proto 代码

```bash
# 使用 build.rs 自动生成
cargo build

# 或手动生成
protoc --rust_out=. proto/**/*.proto
```

### 创建数据库迁移

```bash
cd services/iam-identity

# 创建新迁移
sqlx migrate add create_new_table

# 编辑迁移文件
vim migrations/XXXXXX_create_new_table.sql

# 运行迁移
sqlx migrate run
```

### 添加新的测试

```bash
# 单元测试（在源文件中）
vim services/iam-identity/src/domain/entities/user.rs

# 集成测试（独立文件）
vim services/iam-identity/tests/integration/user_test.rs
```

## 性能优化

### 编译优化

在 `Cargo.toml` 中配置：

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

### 运行时优化

```bash
# 使用 release 模式
cargo build --release

# 设置环境变量
export RUST_LOG=info  # 减少日志输出
export TOKIO_WORKER_THREADS=4  # 限制线程数
```

### 性能分析

```bash
# 安装 flamegraph
cargo install flamegraph

# 生成火焰图
cargo flamegraph --bin iam-identity

# 查看火焰图
open flamegraph.svg
```

## 故障排查

### 编译错误

```bash
# 清理构建缓存
cargo clean

# 更新依赖
cargo update

# 重新构建
cargo build
```

### 运行时错误

```bash
# 启用详细日志
export RUST_LOG=debug
cargo run

# 使用 backtrace
export RUST_BACKTRACE=1
cargo run
```

### 数据库连接问题

```bash
# 检查 PostgreSQL 状态
docker compose -f deploy/docker/docker-compose.yml ps postgres

# 查看日志
docker compose -f deploy/docker/docker-compose.yml logs postgres

# 重启服务
docker compose -f deploy/docker/docker-compose.yml restart postgres
```

## 最佳实践

### 代码风格

- 遵循 Rust 官方风格指南
- 使用 `cargo fmt` 自动格式化
- 使用 `cargo clippy` 检查代码质量
- 添加文档注释（`///`）

### 错误处理

- 使用 `Result<T, E>` 返回可能失败的操作
- 使用 `?` 操作符传播错误
- 为自定义错误实现 `std::error::Error`
- 使用 `thiserror` 简化错误定义

### 测试

- 为所有公共 API 编写测试
- 使用 `#[cfg(test)]` 模块组织测试
- 使用 `#[tokio::test]` 测试异步代码
- 使用 `#[sqlx::test]` 测试数据库操作

### 文档

- 为所有公共 API 添加文档注释
- 使用示例代码展示用法
- 保持文档与代码同步

## 相关资源

- [Rust 官方文档](https://doc.rust-lang.org/)
- [Tokio 文档](https://tokio.rs/)
- [Tonic 文档](https://github.com/hyperium/tonic)
- [SQLx 文档](https://github.com/launchbadge/sqlx)
- [Cuba ERP 架构文档](../architecture.md)
