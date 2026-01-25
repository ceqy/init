#!/bin/bash

# IAM Identity 服务测试运行脚本

set -e

echo "========================================="
echo "IAM Identity 服务测试套件"
echo "========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 测试类型
TEST_TYPE=${1:-all}

# 运行单元测试
run_unit_tests() {
    echo -e "${YELLOW}运行单元测试...${NC}"
    cargo test -p iam-identity --lib -- --nocapture
    echo -e "${GREEN}✓ 单元测试完成${NC}"
    echo ""
}

# 运行集成测试
run_integration_tests() {
    echo -e "${YELLOW}运行集成测试...${NC}"
    cargo test -p iam-identity --test '*' -- --nocapture
    echo -e "${GREEN}✓ 集成测试完成${NC}"
    echo ""
}

# 运行所有测试
run_all_tests() {
    echo -e "${YELLOW}运行所有测试...${NC}"
    cargo test -p iam-identity -- --nocapture
    echo -e "${GREEN}✓ 所有测试完成${NC}"
    echo ""
}

# 生成覆盖率报告
generate_coverage() {
    echo -e "${YELLOW}生成测试覆盖率报告...${NC}"
    
    # 检查是否安装了 tarpaulin
    if ! command -v cargo-tarpaulin &> /dev/null; then
        echo -e "${RED}错误: cargo-tarpaulin 未安装${NC}"
        echo "请运行: cargo install cargo-tarpaulin"
        exit 1
    fi
    
    cargo tarpaulin -p iam-identity \
        --out Html \
        --output-dir coverage \
        --exclude-files 'tests/*' \
        --exclude-files '*/mod.rs' \
        --exclude-files '*/main.rs'
    
    echo -e "${GREEN}✓ 覆盖率报告已生成到 coverage/index.html${NC}"
    echo ""
}

# 运行性能基准测试
run_benchmarks() {
    echo -e "${YELLOW}运行性能基准测试...${NC}"
    cargo bench -p iam-identity
    echo -e "${GREEN}✓ 基准测试完成${NC}"
    echo ""
}

# 运行特定模块测试
run_module_tests() {
    local module=$1
    echo -e "${YELLOW}运行 ${module} 模块测试...${NC}"
    cargo test -p iam-identity ${module} -- --nocapture
    echo -e "${GREEN}✓ ${module} 模块测试完成${NC}"
    echo ""
}

# 主逻辑
case $TEST_TYPE in
    unit)
        run_unit_tests
        ;;
    integration)
        run_integration_tests
        ;;
    coverage)
        generate_coverage
        ;;
    benchmark)
        run_benchmarks
        ;;
    value_objects)
        run_module_tests "value_objects"
        ;;
    entity)
        run_module_tests "entity_tests"
        ;;
    domain_service)
        run_module_tests "domain_service"
        ;;
    oauth)
        run_module_tests "oauth_tests"
        ;;
    tenant)
        run_module_tests "tenant_isolation"
        ;;
    all)
        run_all_tests
        ;;
    *)
        echo -e "${RED}错误: 未知的测试类型 '${TEST_TYPE}'${NC}"
        echo ""
        echo "用法: $0 [test_type]"
        echo ""
        echo "测试类型:"
        echo "  all              - 运行所有测试（默认）"
        echo "  unit             - 运行单元测试"
        echo "  integration      - 运行集成测试"
        echo "  coverage         - 生成覆盖率报告"
        echo "  benchmark        - 运行性能基准测试"
        echo "  value_objects    - 运行值对象测试"
        echo "  entity           - 运行实体测试"
        echo "  domain_service   - 运行领域服务测试"
        echo "  oauth            - 运行 OAuth 测试"
        echo "  tenant           - 运行租户隔离测试"
        echo ""
        exit 1
        ;;
esac

echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}测试完成！${NC}"
echo -e "${GREEN}=========================================${NC}"
