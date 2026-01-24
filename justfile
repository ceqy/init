# justfile - 统一命令入口

# 默认命令
default:
    @just --list

# 开发环境启动
dev:
    cargo run --package gateway

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
