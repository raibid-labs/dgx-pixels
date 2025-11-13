#!/bin/bash
################################################################################
# DGX-Pixels Metrics Testing Script
#
# This script tests the observability stack by verifying:
#   - All metrics endpoints are accessible
#   - Metrics contain expected data
#   - Prometheus is scraping correctly
#   - Grafana dashboards are loaded
#
# Usage:
#   ./scripts/test-metrics.sh
#
################################################################################

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

FAILED_TESTS=0
PASSED_TESTS=0

log_info() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

log_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    PASSED_TESTS=$((PASSED_TESTS + 1))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    FAILED_TESTS=$((FAILED_TESTS + 1))
}

test_endpoint() {
    local name=$1
    local url=$2
    local expected_pattern=$3

    log_info "Testing $name endpoint..."

    if curl -sf "$url" | grep -q "$expected_pattern"; then
        log_pass "$name is working correctly"
        return 0
    else
        log_fail "$name failed (expected pattern not found)"
        return 1
    fi
}

test_dcgm_metrics() {
    log_info "Testing DCGM GPU metrics..."

    local metrics=$(curl -s http://localhost:9400/metrics)

    # Check GPU utilization metric
    if echo "$metrics" | grep -q "DCGM_FI_DEV_GPU_UTIL"; then
        log_pass "GPU utilization metric present"
    else
        log_fail "GPU utilization metric missing"
    fi

    # Check VRAM metrics
    if echo "$metrics" | grep -q "DCGM_FI_DEV_FB_USED"; then
        log_pass "VRAM usage metric present"
    else
        log_fail "VRAM usage metric missing"
    fi

    # Check temperature metric
    if echo "$metrics" | grep -q "DCGM_FI_DEV_GPU_TEMP"; then
        log_pass "GPU temperature metric present"
    else
        log_fail "GPU temperature metric missing"
    fi

    # Check power metric
    if echo "$metrics" | grep -q "DCGM_FI_DEV_POWER_USAGE"; then
        log_pass "GPU power metric present"
    else
        log_fail "GPU power metric missing"
    fi

    # Check NVLink metric (Grace Blackwell specific)
    if echo "$metrics" | grep -q "DCGM_FI_DEV_NVLINK_BANDWIDTH_TOTAL"; then
        log_pass "NVLink bandwidth metric present"
    else
        log_fail "NVLink bandwidth metric missing"
    fi

    # Check Tensor Core metric
    if echo "$metrics" | grep -q "DCGM_FI_PROF_PIPE_TENSOR_ACTIVE"; then
        log_pass "Tensor Core utilization metric present"
    else
        log_fail "Tensor Core utilization metric missing"
    fi
}

test_prometheus_targets() {
    log_info "Testing Prometheus scrape targets..."

    local targets=$(curl -s http://localhost:9090/api/v1/targets)

    # Check DCGM target
    if echo "$targets" | jq -e '.data.activeTargets[] | select(.scrapePool=="dcgm" and .health=="up")' > /dev/null; then
        log_pass "DCGM target is UP"
    else
        log_fail "DCGM target is DOWN"
    fi

    # Check Node Exporter target
    if echo "$targets" | jq -e '.data.activeTargets[] | select(.scrapePool=="node" and .health=="up")' > /dev/null; then
        log_pass "Node Exporter target is UP"
    else
        log_fail "Node Exporter target is DOWN"
    fi

    # Check Prometheus self-monitoring
    if echo "$targets" | jq -e '.data.activeTargets[] | select(.scrapePool=="prometheus" and .health=="up")' > /dev/null; then
        log_pass "Prometheus self-monitoring is UP"
    else
        log_fail "Prometheus self-monitoring is DOWN"
    fi
}

test_prometheus_queries() {
    log_info "Testing Prometheus queries..."

    # Test GPU utilization query
    local gpu_util=$(curl -s 'http://localhost:9090/api/v1/query?query=DCGM_FI_DEV_GPU_UTIL' | jq -r '.data.result | length')
    if [ "$gpu_util" -gt 0 ]; then
        log_pass "GPU utilization query returns data"
    else
        log_fail "GPU utilization query returns no data"
    fi

    # Test VRAM percentage query
    local vram_pct=$(curl -s 'http://localhost:9090/api/v1/query?query=(DCGM_FI_DEV_FB_USED/DCGM_FI_DEV_FB_TOTAL)*100' | jq -r '.data.result | length')
    if [ "$vram_pct" -gt 0 ]; then
        log_pass "VRAM percentage query returns data"
    else
        log_fail "VRAM percentage query returns no data"
    fi

    # Test node metrics
    local node_mem=$(curl -s 'http://localhost:9090/api/v1/query?query=node_memory_MemAvailable_bytes' | jq -r '.data.result | length')
    if [ "$node_mem" -gt 0 ]; then
        log_pass "Node memory query returns data"
    else
        log_fail "Node memory query returns no data"
    fi
}

test_grafana_datasource() {
    log_info "Testing Grafana datasource..."

    local datasources=$(curl -s -u admin:admin http://localhost:3000/api/datasources)

    # Check Prometheus datasource exists
    if echo "$datasources" | jq -e '.[] | select(.type=="prometheus")' > /dev/null; then
        log_pass "Prometheus datasource configured"
    else
        log_fail "Prometheus datasource not found"
    fi

    # Check datasource is default
    if echo "$datasources" | jq -e '.[] | select(.type=="prometheus" and .isDefault==true)' > /dev/null; then
        log_pass "Prometheus is default datasource"
    else
        log_fail "Prometheus is not default datasource"
    fi
}

test_grafana_dashboards() {
    log_info "Testing Grafana dashboards..."

    local dashboards=$(curl -s -u admin:admin http://localhost:3000/api/search?type=dash-db)

    # Check GPU Performance dashboard
    if echo "$dashboards" | jq -e '.[] | select(.uid=="dgx-pixels-gpu")' > /dev/null; then
        log_pass "GPU Performance dashboard loaded"
    else
        log_fail "GPU Performance dashboard not found"
    fi

    # Check Generation Quality dashboard
    if echo "$dashboards" | jq -e '.[] | select(.uid=="dgx-pixels-quality")' > /dev/null; then
        log_pass "Generation Quality dashboard loaded"
    else
        log_fail "Generation Quality dashboard not found"
    fi

    # Check System Health dashboard
    if echo "$dashboards" | jq -e '.[] | select(.uid=="dgx-pixels-health")' > /dev/null; then
        log_pass "System Health dashboard loaded"
    else
        log_fail "System Health dashboard not found"
    fi
}

test_alert_rules() {
    log_info "Testing Prometheus alert rules..."

    local rules=$(curl -s http://localhost:9090/api/v1/rules)

    # Check if alert rules are loaded
    local rule_count=$(echo "$rules" | jq '.data.groups | length')
    if [ "$rule_count" -gt 0 ]; then
        log_pass "Alert rules loaded ($rule_count groups)"
    else
        log_fail "No alert rules loaded"
    fi

    # Check for critical GPU alerts
    if echo "$rules" | jq -e '.data.groups[].rules[] | select(.name=="GPUHighVRAMUsage")' > /dev/null; then
        log_pass "GPU VRAM alert rule present"
    else
        log_fail "GPU VRAM alert rule missing"
    fi

    if echo "$rules" | jq -e '.data.groups[].rules[] | select(.name=="GPUXIDErrors")' > /dev/null; then
        log_pass "GPU XID error alert rule present"
    else
        log_fail "GPU XID error alert rule missing"
    fi
}

test_metric_cardinality() {
    log_info "Testing metric cardinality (performance check)..."

    # Check total number of time series
    local series_count=$(curl -s 'http://localhost:9090/api/v1/query?query=count({__name__=~".+"})' | jq -r '.data.result[0].value[1]')
    log_info "Total time series: $series_count"

    if [ "$series_count" -lt 10000 ]; then
        log_pass "Metric cardinality is reasonable (<10k series)"
    else
        log_fail "Metric cardinality is high (>10k series) - may impact performance"
    fi
}

show_summary() {
    echo ""
    echo "========================================"
    echo "Test Summary"
    echo "========================================"
    echo -e "${GREEN}Passed:${NC} $PASSED_TESTS"
    echo -e "${RED}Failed:${NC} $FAILED_TESTS"
    echo "========================================"

    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}All tests passed!${NC}"
        echo ""
        echo "Your observability stack is fully operational."
        echo ""
        echo "Next steps:"
        echo "  1. Open Grafana: http://localhost:3000"
        echo "  2. View dashboards in DGX-Pixels folder"
        echo "  3. Start generating images to see live metrics"
        echo ""
        return 0
    else
        echo -e "${RED}Some tests failed.${NC}"
        echo ""
        echo "Troubleshooting:"
        echo "  - Check logs: docker compose logs grafana prometheus dcgm-exporter"
        echo "  - Verify services are running: docker compose ps"
        echo "  - Review documentation: docs/observability-guide.md"
        echo ""
        return 1
    fi
}

main() {
    echo "========================================"
    echo "DGX-Pixels Metrics Testing"
    echo "========================================"
    echo ""

    log_info "Starting metrics tests..."
    echo ""

    # Run all tests
    test_endpoint "DCGM Exporter" "http://localhost:9400/metrics" "DCGM_FI_DEV"
    test_endpoint "Prometheus" "http://localhost:9090/-/healthy" "Prometheus"
    test_endpoint "Grafana" "http://localhost:3000/api/health" "ok"
    test_endpoint "Node Exporter" "http://localhost:9100/metrics" "node_"

    echo ""
    test_dcgm_metrics
    echo ""
    test_prometheus_targets
    echo ""
    test_prometheus_queries
    echo ""
    test_grafana_datasource
    echo ""
    test_grafana_dashboards
    echo ""
    test_alert_rules
    echo ""
    test_metric_cardinality
    echo ""

    show_summary
}

main "$@"
