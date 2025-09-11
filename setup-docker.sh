#!/bin/bash

# ZisK-Zcash Docker Setup Script
# Complete setup and management for the ZisK-Zcash integration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${PURPLE}[ZisK-Zcash]${NC} $1"
}

# Function to check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
    print_success "Docker is running"
}

# Function to check if Docker Compose is available
check_docker_compose() {
    if ! command -v docker-compose > /dev/null 2>&1 && ! docker compose version > /dev/null 2>&1; then
        print_error "Docker Compose is not available. Please install Docker Compose."
        exit 1
    fi
    print_success "Docker Compose is available"
}

# Function to build and start services
start_services() {
    print_header "Starting ZisK-Zcash Integration"
    echo ""
    
    print_status "Building and starting services with Docker Compose..."
    
    # Use docker-compose or docker compose based on availability
    if command -v docker-compose > /dev/null 2>&1; then
        COMPOSE_CMD="docker-compose"
    else
        COMPOSE_CMD="docker compose"
    fi
    
    # Start services
    $COMPOSE_CMD up --build -d
    
    print_success "Services started successfully"
    echo ""
    
    # Wait for services to be ready
    print_status "Waiting for services to be ready..."
    sleep 30
    
    # Check service health
    print_status "Checking service health..."
    if $COMPOSE_CMD ps | grep -q "healthy"; then
        print_success "All services are healthy"
    else
        print_warning "Some services may not be fully ready yet"
    fi
}

# Function to show service status
show_status() {
    print_header "Service Status"
    echo ""
    
    # Use docker-compose or docker compose based on availability
    if command -v docker-compose > /dev/null 2>&1; then
        COMPOSE_CMD="docker-compose"
    else
        COMPOSE_CMD="docker compose"
    fi
    
    $COMPOSE_CMD ps
    echo ""
    
    print_status "Service URLs:"
    echo "  🌐 ZisK RPC API: http://localhost:8080"
    echo "  📡 Zcash RPC: http://localhost:8232"
    echo "  🔗 Zcash P2P: localhost:18232"
    echo ""
    
    print_status "Health Checks:"
    echo "  ZisK RPC: curl http://localhost:8080/health"
    echo "  Zcash RPC: curl -u test:test http://localhost:8232"
    echo ""
    
    print_status "Test Commands:"
    echo "  # Test ZisK RPC health"
    echo "  curl http://localhost:8080/health"
    echo ""
    echo "  # Test Zcash RPC"
    echo "  curl -u test:test http://localhost:8232"
    echo ""
    echo "  # Get Zcash blockchain info"
    echo "  curl -u test:test -X POST -H 'Content-Type: application/json' -d '{\"jsonrpc\":\"1.0\",\"id\":\"test\",\"method\":\"getblockchaininfo\",\"params\":[]}' http://localhost:8232"
    echo ""
}

# Function to stop services
stop_services() {
    print_header "Stopping Services"
    echo ""
    
    # Use docker-compose or docker compose based on availability
    if command -v docker-compose > /dev/null 2>&1; then
        COMPOSE_CMD="docker-compose"
    else
        COMPOSE_CMD="docker compose"
    fi
    
    print_status "Stopping services..."
    $COMPOSE_CMD down
    print_success "Services stopped"
}

# Function to show logs
show_logs() {
    print_header "Service Logs"
    echo ""
    
    # Use docker-compose or docker compose based on availability
    if command -v docker-compose > /dev/null 2>&1; then
        COMPOSE_CMD="docker-compose"
    else
        COMPOSE_CMD="docker compose"
    fi
    
    print_status "Showing service logs (Ctrl+C to exit)..."
    $COMPOSE_CMD logs -f
}

# Function to clean up
cleanup() {
    print_header "Cleanup"
    echo ""
    
    # Use docker-compose or docker compose based on availability
    if command -v docker-compose > /dev/null 2>&1; then
        COMPOSE_CMD="docker-compose"
    else
        COMPOSE_CMD="docker compose"
    fi
    
    print_status "Cleaning up Docker resources..."
    $COMPOSE_CMD down -v
    docker system prune -f
    print_success "Cleanup completed"
}

# Function to test the integration
test_integration() {
    print_header "Testing Integration"
    echo ""
    
    print_status "Testing ZisK RPC health..."
    if curl -s http://localhost:8080/health > /dev/null; then
        print_success "ZisK RPC is responding"
    else
        print_error "ZisK RPC is not responding"
    fi
    
    print_status "Testing Zcash RPC..."
    if curl -s -u test:test http://localhost:8232 > /dev/null; then
        print_success "Zcash RPC is responding"
    else
        print_error "Zcash RPC is not responding"
    fi
    
    echo ""
}

# Function to show help
show_help() {
    print_header "ZisK-Zcash Docker Setup"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  start     - Build and start all services (default)"
    echo "  stop      - Stop all services"
    echo "  restart   - Restart all services"
    echo "  status    - Show service status and URLs"
    echo "  logs      - Show service logs"
    echo "  test      - Test the integration"
    echo "  cleanup   - Clean up Docker resources"
    echo "  help      - Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 start    # Start all services"
    echo "  $0 status   # Check service status"
    echo "  $0 logs     # View logs"
    echo "  $0 test     # Test integration"
    echo ""
}

# Main script logic
case "${1:-start}" in
    "start")
        check_docker
        check_docker_compose
        start_services
        show_status
        test_integration
        ;;
    "stop")
        stop_services
        ;;
    "restart")
        stop_services
        start_services
        show_status
        ;;
    "status")
        show_status
        ;;
    "logs")
        show_logs
        ;;
    "test")
        test_integration
        ;;
    "cleanup")
        cleanup
        ;;
    "help")
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac
