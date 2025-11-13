#!/bin/bash
################################################################################
# DGX-Pixels Observability Stack Setup Script
#
# This script sets up and verifies the complete observability stack including:
#   - DCGM Exporter (GPU metrics)
#   - Prometheus (metrics storage)
#   - Grafana (visualization)
#   - Node Exporter (system metrics)
#
# Usage:
#   ./scripts/setup-observability.sh
#
# Prerequisites:
#   - Docker Compose installed
#   - NVIDIA Container Toolkit installed
#   - DGX-Spark hardware with GPU
#
################################################################################

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOCKER_DIR="${PROJECT_ROOT}/docker"

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        exit 1
    fi
    log_success "Docker is installed"

    # Check Docker Compose
    if ! docker compose version &> /dev/null; then
        log_error "Docker Compose is not installed"
        exit 1
    fi
    log_success "Docker Compose is installed"

    # Check NVIDIA GPU
    if ! nvidia-smi &> /dev/null; then
        log_error "nvidia-smi not found - NVIDIA drivers may not be installed"
        exit 1
    fi
    log_success "NVIDIA GPU detected"

    # Check NVIDIA Container Toolkit
    if ! docker run --rm --gpus all nvidia/cuda:12.1.0-base-ubuntu22.04 nvidia-smi &> /dev/null; then
        log_error "NVIDIA Container Toolkit not working"
        exit 1
    fi
    log_success "NVIDIA Container Toolkit is working"
}

create_directories() {
    log_info "Creating required directories..."

    mkdir -p "${PROJECT_ROOT}/models"
    mkdir -p "${PROJECT_ROOT}/outputs"
    mkdir -p "${PROJECT_ROOT}/config"

    log_success "Directories created"
}

start_observability_stack() {
    log_info "Starting observability stack..."

    cd "${DOCKER_DIR}"

    # Start services
    docker compose up -d dcgm-exporter prometheus grafana node-exporter

    log_success "Services started"
}

wait_for_services() {
    log_info "Waiting for services to be healthy..."

    local max_attempts=30
    local attempt=0

    # Wait for DCGM Exporter
    while ! curl -s http://localhost:9400/metrics > /dev/null; do
        attempt=$((attempt + 1))
        if [ $attempt -ge $max_attempts ]; then
            log_error "DCGM Exporter failed to start"
            return 1
        fi
        echo -n "."
        sleep 2
    done
    log_success "DCGM Exporter is ready"

    # Wait for Prometheus
    attempt=0
    while ! curl -s http://localhost:9090/-/healthy > /dev/null; do
        attempt=$((attempt + 1))
        if [ $attempt -ge $max_attempts ]; then
            log_error "Prometheus failed to start"
            return 1
        fi
        echo -n "."
        sleep 2
    done
    log_success "Prometheus is ready"

    # Wait for Grafana
    attempt=0
    while ! curl -s http://localhost:3000/api/health > /dev/null; do
        attempt=$((attempt + 1))
        if [ $attempt -ge $max_attempts ]; then
            log_error "Grafana failed to start"
            return 1
        fi
        echo -n "."
        sleep 2
    done
    log_success "Grafana is ready"

    # Wait for Node Exporter
    attempt=0
    while ! curl -s http://localhost:9100/metrics > /dev/null; do
        attempt=$((attempt + 1))
        if [ $attempt -ge $max_attempts ]; then
            log_error "Node Exporter failed to start"
            return 1
        fi
        echo -n "."
        sleep 2
    done
    log_success "Node Exporter is ready"
}

verify_metrics() {
    log_info "Verifying metrics collection..."

    # Check DCGM metrics
    if curl -s http://localhost:9400/metrics | grep -q "DCGM_FI_DEV_GPU_UTIL"; then
        log_success "DCGM GPU metrics available"
    else
        log_error "DCGM GPU metrics not found"
        return 1
    fi

    # Check Prometheus targets
    local targets_up=$(curl -s http://localhost:9090/api/v1/targets | jq -r '.data.activeTargets[] | select(.health=="up") | .scrapePool' | wc -l)
    log_info "Prometheus targets UP: $targets_up"

    if [ "$targets_up" -ge 3 ]; then
        log_success "Prometheus is scraping metrics"
    else
        log_warning "Some Prometheus targets may be down"
    fi

    # Check Grafana datasource
    local datasources=$(curl -s -u admin:admin http://localhost:3000/api/datasources | jq 'length')
    if [ "$datasources" -ge 1 ]; then
        log_success "Grafana datasource configured"
    else
        log_error "Grafana datasource not found"
        return 1
    fi
}

display_urls() {
    log_info "Observability stack is ready!"
    echo ""
    echo "========================================"
    echo "Access URLs:"
    echo "========================================"
    echo -e "${GREEN}Grafana:${NC}         http://localhost:3000"
    echo "                   Username: admin"
    echo "                   Password: admin"
    echo ""
    echo -e "${GREEN}Prometheus:${NC}      http://localhost:9090"
    echo ""
    echo -e "${GREEN}DCGM Metrics:${NC}    http://localhost:9400/metrics"
    echo ""
    echo -e "${GREEN}Node Metrics:${NC}    http://localhost:9100/metrics"
    echo ""
    echo "========================================"
    echo "Pre-Built Dashboards:"
    echo "========================================"
    echo "  1. GPU Performance"
    echo "  2. Generation Quality"
    echo "  3. System Health"
    echo ""
    echo "Navigate to Grafana → Dashboards → DGX-Pixels folder"
    echo ""
}

show_next_steps() {
    log_info "Next Steps:"
    echo ""
    echo "1. Open Grafana: http://localhost:3000"
    echo "2. Login with admin/admin (change password on first login)"
    echo "3. Navigate to Dashboards → DGX-Pixels"
    echo "4. View GPU Performance dashboard"
    echo ""
    echo "To start generating images and see metrics:"
    echo "  cd ${PROJECT_ROOT}"
    echo "  # Start your generation pipeline"
    echo ""
    echo "For troubleshooting:"
    echo "  docker compose logs -f grafana prometheus dcgm-exporter"
    echo ""
    echo "Documentation:"
    echo "  ${PROJECT_ROOT}/docs/observability-guide.md"
    echo ""
}

cleanup_on_error() {
    log_error "Setup failed. Cleaning up..."
    cd "${DOCKER_DIR}"
    docker compose down
    exit 1
}

main() {
    echo "========================================"
    echo "DGX-Pixels Observability Stack Setup"
    echo "========================================"
    echo ""

    # Set trap for cleanup on error
    trap cleanup_on_error ERR

    check_prerequisites
    create_directories
    start_observability_stack
    wait_for_services
    verify_metrics
    display_urls
    show_next_steps

    log_success "Setup complete!"
}

main "$@"
