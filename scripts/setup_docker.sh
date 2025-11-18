#!/bin/bash
################################################################################
# DGX-Pixels Docker Setup Script
#
# This script sets up the complete Docker Compose environment for DGX-Pixels,
# including prerequisites checking, directory creation, and initial configuration.
#
# Usage:
#   ./scripts/setup_docker.sh
#
# Prerequisites:
#   - Docker v20.10+
#   - Docker Compose v2+
#   - NVIDIA Container Toolkit
#   - DGX-Spark GB10 hardware
#
################################################################################

set -e  # Exit on error
set -u  # Exit on undefined variable

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}DGX-Pixels Docker Setup${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# ============================================================================
# Function Definitions
# ============================================================================

check_command() {
    if command -v "$1" &> /dev/null; then
        echo -e "${GREEN}✓${NC} $2 installed"
        return 0
    else
        echo -e "${RED}✗${NC} $2 not installed"
        return 1
    fi
}

check_docker_compose_version() {
    local version=$(docker compose version --short 2>/dev/null || echo "0")
    local major=$(echo "$version" | cut -d. -f1)

    if [ "$major" -ge 2 ]; then
        echo -e "${GREEN}✓${NC} Docker Compose v2+ installed ($version)"
        return 0
    else
        echo -e "${RED}✗${NC} Docker Compose v2+ required (found: $version)"
        return 1
    fi
}

check_nvidia_docker() {
    if docker run --rm --gpus all nvidia/cuda:12.0.0-base-ubuntu22.04 nvidia-smi &> /dev/null; then
        echo -e "${GREEN}✓${NC} NVIDIA Container Toolkit working"
        return 0
    else
        echo -e "${RED}✗${NC} NVIDIA Container Toolkit not working"
        echo -e "${YELLOW}  Install with: https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/install-guide.html${NC}"
        return 1
    fi
}

create_directory() {
    if [ ! -d "$1" ]; then
        mkdir -p "$1"
        echo -e "${GREEN}✓${NC} Created directory: $1"
    else
        echo -e "${BLUE}ℹ${NC} Directory exists: $1"
    fi
}

# ============================================================================
# Prerequisites Check
# ============================================================================

echo -e "${BLUE}Step 1: Checking prerequisites...${NC}"
echo ""

ALL_CHECKS_PASSED=true

# Check Docker
if ! check_command "docker" "Docker"; then
    ALL_CHECKS_PASSED=false
    echo -e "${YELLOW}  Install from: https://docs.docker.com/engine/install/${NC}"
fi

# Check Docker Compose
if ! check_docker_compose_version; then
    ALL_CHECKS_PASSED=false
    echo -e "${YELLOW}  Upgrade Docker to get Compose v2${NC}"
fi

# Check NVIDIA Container Toolkit
if ! check_nvidia_docker; then
    ALL_CHECKS_PASSED=false
fi

# Check git
check_command "git" "Git" || true

# Check curl
check_command "curl" "curl" || true

echo ""

if [ "$ALL_CHECKS_PASSED" = false ]; then
    echo -e "${RED}✗ Some prerequisites are missing. Please install them and try again.${NC}"
    exit 1
fi

echo -e "${GREEN}✓ All prerequisites met${NC}"
echo ""

# ============================================================================
# Directory Creation
# ============================================================================

echo -e "${BLUE}Step 2: Creating directories...${NC}"
echo ""

cd "$PROJECT_ROOT"

# Core directories
create_directory "models"
create_directory "models/checkpoints"
create_directory "models/loras"
create_directory "models/vae"
create_directory "models/embeddings"
create_directory "outputs"
create_directory "workflows"
create_directory "config"
create_directory "logs"

# ComfyUI directories (if ComfyUI not present)
if [ ! -d "ComfyUI" ]; then
    create_directory "ComfyUI"
fi

echo ""

# ============================================================================
# Environment Configuration
# ============================================================================

echo -e "${BLUE}Step 3: Setting up environment configuration...${NC}"
echo ""

if [ ! -f "docker/.env" ]; then
    echo -e "${YELLOW}Creating docker/.env from template...${NC}"
    cp docker/.env.production docker/.env
    echo -e "${GREEN}✓${NC} Created docker/.env"
    echo -e "${YELLOW}  ⚠ Please review and update docker/.env with your settings${NC}"
else
    echo -e "${BLUE}ℹ${NC} docker/.env already exists (not overwriting)"
fi

echo ""

# ============================================================================
# ComfyUI Setup
# ============================================================================

echo -e "${BLUE}Step 4: Setting up ComfyUI...${NC}"
echo ""

if [ ! -d "ComfyUI/.git" ]; then
    echo -e "${YELLOW}Cloning ComfyUI repository...${NC}"
    if [ -d "ComfyUI" ]; then
        rm -rf "ComfyUI"
    fi
    git clone https://github.com/comfyanonymous/ComfyUI.git
    echo -e "${GREEN}✓${NC} ComfyUI cloned"
else
    echo -e "${BLUE}ℹ${NC} ComfyUI already exists"

    # Offer to update (only in interactive mode)
    if [ -t 0 ]; then
        echo -e "${YELLOW}Update ComfyUI to latest version? (y/N)${NC}"
        read -r response
        if [[ "$response" =~ ^[Yy]$ ]]; then
            cd ComfyUI
            git pull
            cd ..
            echo -e "${GREEN}✓${NC} ComfyUI updated"
        fi
    else
        echo -e "${BLUE}ℹ${NC} Skipping ComfyUI update (non-interactive mode)"
        echo -e "${YELLOW}  Run 'cd ComfyUI && git pull' to update manually${NC}"
    fi
fi

echo ""

# ============================================================================
# Build Docker Images
# ============================================================================

echo -e "${BLUE}Step 5: Building Docker images...${NC}"
echo ""

cd "$PROJECT_ROOT"

echo -e "${YELLOW}This may take 10-15 minutes on first build...${NC}"
echo ""

# Build all images
docker compose -f docker/docker-compose.yml build

echo ""
echo -e "${GREEN}✓${NC} Docker images built successfully"
echo ""

# ============================================================================
# Validate Configuration
# ============================================================================

echo -e "${BLUE}Step 6: Validating Docker Compose configuration...${NC}"
echo ""

cd docker
if docker compose config > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Docker Compose configuration is valid"
else
    echo -e "${RED}✗${NC} Docker Compose configuration has errors"
    docker compose config
    exit 1
fi
cd ..

echo ""

# ============================================================================
# Setup Complete
# ============================================================================

echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}✓ Setup Complete!${NC}"
echo -e "${GREEN}================================${NC}"
echo ""
echo -e "${BLUE}Next Steps:${NC}"
echo ""
echo -e "  1. Review configuration:"
echo -e "     ${YELLOW}nano docker/.env${NC}"
echo ""
echo -e "  2. Download SDXL model (if not already present):"
echo -e "     ${YELLOW}just download-model${NC}"
echo -e "     OR manually download to: ${YELLOW}models/checkpoints/${NC}"
echo ""
echo -e "  3. Start the stack:"
echo -e "     ${YELLOW}cd docker && docker compose up -d${NC}"
echo ""
echo -e "  4. Check service status:"
echo -e "     ${YELLOW}docker compose ps${NC}"
echo ""
echo -e "  5. View logs:"
echo -e "     ${YELLOW}docker compose logs -f${NC}"
echo ""
echo -e "  6. Access services:"
echo -e "     - ComfyUI:   ${BLUE}http://localhost:8188${NC}"
echo -e "     - Grafana:   ${BLUE}http://localhost:3000${NC} (admin/admin)"
echo -e "     - Prometheus: ${BLUE}http://localhost:9090${NC}"
echo ""
echo -e "${YELLOW}Documentation:${NC}"
echo -e "  - Deployment Guide: ${BLUE}docs/docker-deployment.md${NC}"
echo -e "  - Architecture:     ${BLUE}docs/07-rust-python-architecture.md${NC}"
echo ""
