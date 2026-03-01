#!/bin/bash
# ============================================================
# Production Deployment Script
# ============================================================
# Usage:
#   ./scripts/deploy.sh [build|start|stop|restart|logs|clean]
# ============================================================

set -e

PROJECT_NAME="code-continuum"
COMPOSE_FILE="docker-compose.prod.yml"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed"
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        log_error "Docker Compose is not installed"
        exit 1
    fi
    
    if [ ! -f ".env" ]; then
        log_warn ".env file not found. Using .env.example defaults."
        log_warn "Please copy .env.example to .env and configure for production."
    fi
    
    log_info "Prerequisites OK"
}

build_image() {
    log_info "Building Docker image..."
    
    # Pass proxy environment variables from host if defined
    PROXY_ARGS=""
    if [ -n "$HTTP_PROXY" ] || [ -n "$http_proxy" ]; then
        PROXY_ARGS="$PROXY_ARGS --build-arg HTTP_PROXY=${HTTP_PROXY:-$http_proxy}"
    fi
    if [ -n "$HTTPS_PROXY" ] || [ -n "$https_proxy" ]; then
        PROXY_ARGS="$PROXY_ARGS --build-arg HTTPS_PROXY=${HTTPS_PROXY:-$https_proxy}"
    fi
    if [ -n "$NO_PROXY" ] || [ -n "$no_proxy" ]; then
        PROXY_ARGS="$PROXY_ARGS --build-arg NO_PROXY=${NO_PROXY:-$no_proxy}"
    fi
    
    if [ -n "$PROXY_ARGS" ]; then
        log_info "Using proxy configuration from environment"
    fi
    
    docker build $PROXY_ARGS -t ${PROJECT_NAME}:latest .
    log_info "Build complete: ${PROJECT_NAME}:latest"
}

start_services() {
    log_info "Starting services..."
    docker-compose -f ${COMPOSE_FILE} up -d neo4j
    
    log_info "Waiting for Neo4j to be ready..."
    sleep 10
    
    log_info "Services started. Neo4j available at:"
    log_info "  - Browser: http://localhost:7474"
    log_info "  - Bolt: bolt://localhost:7687"
    log_info ""
    log_info "To analyze code, run:"
    log_info "  ./scripts/deploy.sh analyze /path/to/code"
}

stop_services() {
    log_info "Stopping services..."
    docker-compose -f ${COMPOSE_FILE} down
    log_info "Services stopped"
}

restart_services() {
    stop_services
    start_services
}

view_logs() {
    log_info "Viewing logs (Ctrl+C to exit)..."
    docker-compose -f ${COMPOSE_FILE} logs -f
}

analyze_code() {
    if [ -z "$1" ]; then
        log_error "Usage: ./scripts/deploy.sh analyze /path/to/code"
        exit 1
    fi
    
    CODE_PATH=$(realpath "$1")
    
    if [ ! -d "$CODE_PATH" ]; then
        log_error "Directory not found: $CODE_PATH"
        exit 1
    fi
    
    log_info "Analyzing code at: $CODE_PATH"
    
    docker-compose -f ${COMPOSE_FILE} run --rm \
        -v "${CODE_PATH}:/app/data:ro" \
        code-continuum \
        /app/data
    
    log_info "Analysis complete. View results in Neo4j Browser: http://localhost:7474"
}

clean_all() {
    log_warn "This will remove all containers, volumes, and images!"
    read -p "Are you sure? (yes/no): " -r
    if [[ $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
        log_info "Cleaning up..."
        docker-compose -f ${COMPOSE_FILE} down -v
        docker rmi ${PROJECT_NAME}:latest 2>/dev/null || true
        log_info "Cleanup complete"
    else
        log_info "Cleanup cancelled"
    fi
}

show_status() {
    log_info "Service status:"
    docker-compose -f ${COMPOSE_FILE} ps
}

show_help() {
    echo "Usage: $0 {build|start|stop|restart|logs|analyze|status|test|clean|help}"
    echo ""
    echo "Commands:"
    echo "  build    - Build Docker image"
    echo "  start    - Start Neo4j database and MCP server"
    echo "  stop     - Stop all services"
    echo "  restart  - Restart all services"
    echo "  logs     - View service logs"
    echo "  analyze  - Analyze code directory (usage: analyze /path/to/code)"
    echo "  status   - Show service status"
    echo "  test     - Test service health"
    echo "  clean    - Remove all containers and volumes"
    echo "  help     - Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 build"
    echo "  $0 start"
    echo "  $0 test"
    echo "  $0 analyze ./examples/backend/java"
    echo "  $0 logs"
}

test_services() {
    log_info "Testing service health..."
    
    # Check Neo4j
    if curl -f -s http://localhost:7474 > /dev/null 2>&1; then
        log_info "✅ Neo4j is healthy (http://localhost:7474)"
    else
        log_error "❌ Neo4j is not responding"
    fi
    
    # Check MCP Server
    if curl -f -s http://localhost:8000/api/mcp/ > /dev/null 2>&1; then
        log_info "✅ MCP Server is healthy (http://localhost:8000/api/mcp/)"
    else
        log_warn "⚠️  MCP Server is not responding (may not be started)"
    fi
}

# Main
check_prerequisites

case "${1:-help}" in
    build)
        build_image
        ;;
    start)
        start_services
        ;;
    stop)
        stop_services
        ;;
    restart)
        restart_services
        ;;
    logs)
        view_logs
        ;;
    analyze)
        analyze_code "$2"
        ;;
    status)
        show_status
        ;;
    test)
        test_services
        ;;
    clean)
        clean_all
        ;;
    help|*)
        show_help
        ;;
esac
