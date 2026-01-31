#!/bin/bash
# 性能测试执行脚本

set -e

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# 配置
BASE_URL="${BASE_URL:-http://localhost:8080}"
REPORT_DIR="tests/performance/reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

log() {
    echo -e "${GREEN}[$(date +'%H:%M:%S')]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(date +'%H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[$(date +'%H:%M:%S')]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log "检查依赖..."

    if ! command -v k6 &> /dev/null; then
        error "k6 未安装，请先安装: https://k6.io/docs/getting-started/installation/"
        exit 1
    fi

    if ! command -v jq &> /dev/null; then
        warn "jq 未安装，部分功能可能不可用"
    fi

    log "✓ 依赖检查通过"
}

# 检查服务健康
check_service_health() {
    log "检查服务健康状态..."

    if ! curl -sf "${BASE_URL}/health" > /dev/null; then
        error "服务不可用: ${BASE_URL}"
        exit 1
    fi

    log "✓ 服务健康检查通过"
}

# 准备测试数据
prepare_test_data() {
    log "准备测试数据..."

    # 创建测试用户
    for i in {0..99}; do
        curl -sf -X POST "${BASE_URL}/api/auth/register" \
            -H "Content-Type: application/json" \
            -d "{\"email\":\"test${i}@example.com\",\"password\":\"Test123456!\",\"name\":\"Test User ${i}\"}" \
            > /dev/null 2>&1 || true
    done

    log "✓ 测试数据准备完成"
}

# 运行性能测试
run_performance_test() {
    local test_type=$1
    local test_file=$2

    log "开始 ${test_type} 测试..."

    mkdir -p "${REPORT_DIR}"

    # 运行 k6 测试
    k6 run \
        --out json="${REPORT_DIR}/${test_type}_${TIMESTAMP}.json" \
        --summary-export="${REPORT_DIR}/${test_type}_${TIMESTAMP}_summary.json" \
        "${test_file}"

    local exit_code=$?

    if [ $exit_code -eq 0 ]; then
        log "✓ ${test_type} 测试完成"
    else
        error "✗ ${test_type} 测试失败"
        return $exit_code
    fi
}

# 生成测试报告
generate_report() {
    log "生成测试报告..."

    local summary_file="${REPORT_DIR}/load-test_${TIMESTAMP}_summary.json"

    if [ ! -f "$summary_file" ]; then
        warn "摘要文件不存在，跳过报告生成"
        return
    fi

    # 提取关键指标
    if command -v jq &> /dev/null; then
        log "关键指标:"
        echo ""

        jq -r '
            "  总请求数: \(.metrics.http_reqs.values.count)",
            "  请求速率: \(.metrics.http_reqs.values.rate | tostring) req/s",
            "  平均响应时间: \(.metrics.http_req_duration.values.avg | tostring) ms",
            "  P95 响应时间: \(.metrics.http_req_duration.values["p(95)"] | tostring) ms",
            "  P99 响应时间: \(.metrics.http_req_duration.values["p(99)"] | tostring) ms",
            "  错误率: \((.metrics.http_req_failed.values.rate * 100) | tostring)%"
        ' "$summary_file"

        echo ""
    fi

    log "报告已保存到: ${REPORT_DIR}/"
}

# 清理测试数据
cleanup_test_data() {
    log "清理测试数据..."

    # 删除测试用户（可选）
    # for i in {0..99}; do
    #     curl -sf -X DELETE "${BASE_URL}/api/users/test${i}@example.com" > /dev/null 2>&1 || true
    # done

    log "✓ 清理完成"
}

# 主函数
main() {
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}ERP 性能测试${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""

    # 解析参数
    TEST_TYPE="${1:-load}"
    SKIP_SETUP="${2:-false}"

    log "测试类型: ${TEST_TYPE}"
    log "目标地址: ${BASE_URL}"
    echo ""

    # 执行测试流程
    check_dependencies
    check_service_health

    if [ "$SKIP_SETUP" != "true" ]; then
        prepare_test_data
    fi

    case "$TEST_TYPE" in
        load)
            run_performance_test "load-test" "tests/performance/load-test.js"
            ;;
        stress)
            run_performance_test "stress-test" "tests/performance/stress-test.js"
            ;;
        spike)
            run_performance_test "spike-test" "tests/performance/spike-test.js"
            ;;
        soak)
            run_performance_test "soak-test" "tests/performance/soak-test.js"
            ;;
        all)
            run_performance_test "load-test" "tests/performance/load-test.js"
            run_performance_test "stress-test" "tests/performance/stress-test.js"
            run_performance_test "spike-test" "tests/performance/spike-test.js"
            ;;
        *)
            error "未知的测试类型: ${TEST_TYPE}"
            echo "可用类型: load, stress, spike, soak, all"
            exit 1
            ;;
    esac

    generate_report

    if [ "$SKIP_SETUP" != "true" ]; then
        cleanup_test_data
    fi

    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}测试完成！${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
}

# 执行主函数
main "$@"
