#!/bin/bash
################################################################################
# Docker Cleanup Script
#
# Safely clean up Docker resources for DGX-Pixels
#
# Usage:
#   ./scripts/docker_cleanup.sh [--all|--images|--volumes|--logs]
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

MODE=${1:-menu}

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}DGX-Pixels Cleanup${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

cd "$PROJECT_ROOT/docker"

# ============================================================================
# Functions
# ============================================================================

cleanup_containers() {
    echo -e "${YELLOW}Stopping and removing containers...${NC}"
    docker compose down
    echo -e "${GREEN}✓${NC} Containers removed"
}

cleanup_images() {
    echo -e "${YELLOW}Removing DGX-Pixels images...${NC}"

    # Remove project images
    docker images | grep dgx-pixels | awk '{print $3}' | xargs -r docker rmi -f || true

    echo -e "${GREEN}✓${NC} Images removed"
}

cleanup_volumes() {
    echo -e "${RED}WARNING: This will delete all persistent data!${NC}"
    echo -e "${YELLOW}This includes models, outputs, and metrics.${NC}"
    echo ""
    read -p "Are you sure? (yes/no): " confirm

    if [ "$confirm" = "yes" ]; then
        echo -e "${YELLOW}Removing volumes...${NC}"
        docker compose down -v
        echo -e "${GREEN}✓${NC} Volumes removed"
    else
        echo -e "${BLUE}Skipped volume cleanup${NC}"
    fi
}

cleanup_logs() {
    echo -e "${YELLOW}Cleaning up old logs...${NC}"

    # Clean container logs
    docker compose down
    find /var/lib/docker/containers -name "*.log" -type f -exec truncate -s 0 {} \; 2>/dev/null || true

    # Clean project logs
    if [ -d "$PROJECT_ROOT/logs" ]; then
        find "$PROJECT_ROOT/logs" -name "*.log" -type f -mtime +7 -delete
    fi

    echo -e "${GREEN}✓${NC} Logs cleaned"
}

cleanup_build_cache() {
    echo -e "${YELLOW}Cleaning build cache...${NC}"
    docker builder prune -f
    echo -e "${GREEN}✓${NC} Build cache cleaned"
}

# ============================================================================
# Menu
# ============================================================================

if [ "$MODE" = "menu" ]; then
    echo "Select cleanup option:"
    echo ""
    echo "  1) Stop and remove containers"
    echo "  2) Remove images"
    echo "  3) Remove volumes (WARNING: deletes data)"
    echo "  4) Clean logs"
    echo "  5) Clean build cache"
    echo "  6) Full cleanup (containers + images + cache)"
    echo "  7) Nuclear option (everything including volumes)"
    echo "  0) Cancel"
    echo ""
    read -p "Enter choice [0-7]: " choice

    case $choice in
        1)
            cleanup_containers
            ;;
        2)
            cleanup_images
            ;;
        3)
            cleanup_volumes
            ;;
        4)
            cleanup_logs
            ;;
        5)
            cleanup_build_cache
            ;;
        6)
            cleanup_containers
            cleanup_images
            cleanup_build_cache
            ;;
        7)
            cleanup_containers
            cleanup_images
            cleanup_volumes
            cleanup_logs
            cleanup_build_cache
            echo ""
            echo -e "${RED}✓ Nuclear cleanup complete${NC}"
            ;;
        0)
            echo "Cancelled"
            exit 0
            ;;
        *)
            echo "Invalid choice"
            exit 1
            ;;
    esac

elif [ "$MODE" = "--all" ]; then
    cleanup_containers
    cleanup_images
    cleanup_build_cache

elif [ "$MODE" = "--images" ]; then
    cleanup_images

elif [ "$MODE" = "--volumes" ]; then
    cleanup_volumes

elif [ "$MODE" = "--logs" ]; then
    cleanup_logs

else
    echo "Usage: $0 [--all|--images|--volumes|--logs]"
    exit 1
fi

echo ""
echo -e "${GREEN}✓ Cleanup complete${NC}"
echo ""
