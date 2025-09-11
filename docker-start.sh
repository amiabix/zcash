#!/bin/bash

# ZisK-Zcash Docker Startup Script
# This script helps you start the complete ZisK-Zcash integration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

# Function to check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
    print_success "Docker is running"
}

# Function to build images
build_images() {
    print_status "Building Docker images..."
    
    # Build Zcash node
    print_status "Building Zcash node image..."
    docker build -f Dockerfile.zcash -t zcash-zkvm-node .
    
    # Build ZisK integration
    print_status "Building ZisK integration image..."
    docker build -f Dockerfile.zisk -t zisk-zkvm-rpc .
    
    print_success "All images built successfully"
}

# Function to start services
start_services() {
    print_status "Starting services with Docker Compose..."
    
    # Start services
    docker-compose up -d
    
    print_success "Services started successfully"
    print_status "Waiting for services to be ready..."
    
    # Wait for services to be healthy
    sleep 30
    
    # Check service health
    if docker-compose ps | grep -q "healthy"; then
        print_success "All services are healthy"
    else
        print_warning "Some services may not be fully ready yet"
    fi
}

# Function to show service status
show_status() {
    print_status "Service Status:"
    echo ""
    docker-compose ps
    echo ""
    
    print_status "Service URLs:"
    echo "  🌐 ZisK RPC API: http://localhost:8080"
    echo "  📡 Zcash RPC: http://localhost:8232"
    echo "  📊 Monitoring: http://localhost:9090 (if enabled)"
    echo ""
    
    print_status "Health Checks:"
    echo "  ZisK RPC: curl http://localhost:8080/health"
    echo "  Zcash RPC: curl -u test:test http://localhost:8232"
    echo ""
}

# Function to stop services
stop_services() {
    print_status "Stopping services..."
    docker-compose down
    print_success "Services stopped"
}

# Function to show logs
show_logs() {
    print_status "Showing service logs..."
    docker-compose logs -f
}

# Function to clean up
cleanup() {
    print_status "Cleaning up Docker resources..."
    docker-compose down -v
    docker system prune -f
    print_success "Cleanup completed"
}

# Main script logic
case "${1:-start}" in
    "build")
        check_docker
        build_images
        ;;
    "start")
        check_docker
        build_images
        start_services
        show_status
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
    "cleanup")
        cleanup
        ;;
    "help")
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  build    - Build Docker images"
        echo "  start    - Build and start all services (default)"
        echo "  stop     - Stop all services"
        echo "  restart  - Restart all services"
        echo "  status   - Show service status"
        echo "  logs     - Show service logs"
        echo "  cleanup  - Clean up Docker resources"
        echo "  help     - Show this help message"
        ;;
    *)
        print_error "Unknown command: $1"
        echo "Use '$0 help' for available commands"
        exit 1
        ;;
esac
