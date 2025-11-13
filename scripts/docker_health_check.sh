#!/bin/bash
################################################################################
# Docker Health Check Script
#
# Comprehensive health check for all DGX-Pixels Docker services
#
# Usage:
#   ./scripts/docker_health_check.sh
#
################################################################################

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}DGX-Pixels Health Check${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

cd "$PROJECT_ROOT/docker"

# ============================================================================
# Container Status
# ============================================================================

echo -e "${BLUE}Container Status:${NC}"
echo ""

docker compose ps

echo ""

# Check if all services are running
RUNNING=$(docker compose ps --services --filter "status=running" | wc -l)
TOTAL=$(docker compose ps --services | wc -l)

if [ "$RUNNING" -eq "$TOTAL" ]; then
    echo -e "${GREEN}✓${NC} All services running ($RUNNING/$TOTAL)"
else
    echo -e "${YELLOW}⚠${NC} Some services not running ($RUNNING/$TOTAL)"
fi

echo ""

# ============================================================================
# Service Health Checks
# ============================================================================

echo -e "${BLUE}Service Health Checks:${NC}"
echo ""

check_service() {
    local service=$1
    local url=$2
    local expected=$3

    printf "%-20s " "$service:"

    if curl -s -f "$url" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ HEALTHY${NC}"
        return 0
    else
        echo -e "${RED}✗ UNHEALTHY${NC}"
        return 1
    fi
}

# Check ComfyUI
check_service "ComfyUI" "http://localhost:8188/system_stats" || true

# Check Backend Metrics
check_service "Backend Metrics" "http://localhost:8000/metrics" || true

# Check Prometheus
check_service "Prometheus" "http://localhost:9090/-/healthy" || true

# Check Grafana
check_service "Grafana" "http://localhost:3000/api/health" || true

# Check DCGM Exporter
check_service "DCGM Exporter" "http://localhost:9400/metrics" || true

# Check Node Exporter
check_service "Node Exporter" "http://localhost:9100/metrics" || true

echo ""

# ============================================================================
# GPU Check
# ============================================================================

echo -e "${BLUE}GPU Access:${NC}"
echo ""

printf "%-20s " "ComfyUI GPU:"
if docker compose exec -T comfyui nvidia-smi > /dev/null 2>&1; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAILED${NC}"
fi

printf "%-20s " "Backend GPU:"
if docker compose exec -T backend-worker nvidia-smi > /dev/null 2>&1; then
    echo -e "${GREEN}✓ OK${NC}"
else
    echo -e "${RED}✗ FAILED${NC}"
fi

echo ""

# ============================================================================
# Volume Check
# ============================================================================

echo -e "${BLUE}Volume Status:${NC}"
echo ""

docker volume ls | grep dgx-pixels

echo ""

# ============================================================================
# Network Check
# ============================================================================

echo -e "${BLUE}Network Status:${NC}"
echo ""

docker network ls | grep dgx-pixels

echo ""

# ============================================================================
# Resource Usage
# ============================================================================

echo -e "${BLUE}Resource Usage:${NC}"
echo ""

docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}"

echo ""

# ============================================================================
# Recent Errors
# ============================================================================

echo -e "${BLUE}Recent Errors (last 5 minutes):${NC}"
echo ""

SINCE=$(date -u -d '5 minutes ago' '+%Y-%m-%dT%H:%M:%S')

for service in comfyui backend-worker mcp-server prometheus grafana; do
    ERRORS=$(docker compose logs --since="$SINCE" "$service" 2>/dev/null | grep -i "error\|exception\|failed" | wc -l)

    printf "%-20s " "$service:"
    if [ "$ERRORS" -eq 0 ]; then
        echo -e "${GREEN}No errors${NC}"
    else
        echo -e "${YELLOW}$ERRORS errors found${NC}"
    fi
done

echo ""

# ============================================================================
# Summary
# ============================================================================

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}Summary${NC}"
echo -e "${BLUE}================================${NC}"
echo ""
echo -e "Services running: $RUNNING/$TOTAL"
echo -e "Timestamp: $(date)"
echo ""
echo -e "For detailed logs:"
echo -e "  ${YELLOW}docker compose logs -f${NC}"
echo ""
echo -e "To restart a service:"
echo -e "  ${YELLOW}docker compose restart <service-name>${NC}"
echo ""
