# justfile - 统一命令入口

# 默认命令
default:
    @just --list

# 开发环境启动 - Gateway
dev:
    cargo run --package cuba-gateway

# 启动 IAM Identity 服务
iam:
    cd services/iam-identity && cargo run --package iam-identity

# 启动所有服务 (Gateway + IAM) - 使用 Shell 并行运行
all:
    @echo "Starting all services..."
    @trap 'kill 0' SIGINT; \
    just iam & \
    sleep 5 && just dev & \
    wait

# 启动所有服务（手动提示版本）
start-all:
    @echo "请在两个终端分别运行："
    @echo "  终端1: just dev      # Gateway (HTTP :8080)"
    @echo "  终端2: just iam      # IAM Identity (gRPC :50051)"

# 构建所有
build:
    cargo build --workspace

# 构建发布版本
build-release:
    cargo build --workspace --release

# 运行测试
test:
    cargo test --workspace

# 代码检查
check:
    cargo check --workspace

# 代码格式化
fmt:
    cargo fmt --all

# Clippy 检查
lint:
    cargo clippy --workspace -- -D warnings

# 生成 proto
proto-gen:
    cargo build --package cuba-bootstrap --features proto-gen

# 数据库迁移
migrate service:
    sqlx migrate run --source services/{{service}}/migrations

# 创建新服务
new-service name:
    mkdir -p services/{{name}}/src/{domain/{aggregates,entities,events,repositories,services,value_objects},application/{commands,queries,handlers,dto},infrastructure/{persistence,messaging,services},api/grpc}
    mkdir -p services/{{name}}/{proto,migrations,config}
    @echo "Service {{name}} created. Don't forget to add it to Cargo.toml"

# Docker compose 启动基础设施
infra-up:
    docker compose -f deploy/docker/docker-compose.yml up -d

# Docker compose 停止基础设施
infra-down:
    docker compose -f deploy/docker/docker-compose.yml down

# 清理构建产物
clean:
    cargo clean
