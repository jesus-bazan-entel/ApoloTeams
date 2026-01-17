#!/bin/bash
# Deployment script for Rust Teams
# Run this script from the project root directory

set -e

echo "=========================================="
echo "  Rust Teams Deployment Script"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Please run this script from the rust-teams directory${NC}"
    exit 1
fi

# Step 1: Build the frontend
echo -e "\n${YELLOW}Step 1: Building frontend...${NC}"
cd frontend

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo -e "${RED}trunk is not installed. Installing...${NC}"
    cargo install trunk
fi

# Check if wasm32 target is installed
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo -e "${YELLOW}Installing wasm32-unknown-unknown target...${NC}"
    rustup target add wasm32-unknown-unknown
fi

# Build frontend
trunk build --release

echo -e "${GREEN}Frontend built successfully!${NC}"

# Step 2: Copy frontend files to backend static directory
echo -e "\n${YELLOW}Step 2: Copying frontend files to backend...${NC}"
cd ..

# Create static directory in backend
mkdir -p backend/static

# Copy all files from frontend/dist to backend/static
cp -r frontend/dist/* backend/static/

echo -e "${GREEN}Frontend files copied to backend/static/${NC}"

# Step 3: Build the backend
echo -e "\n${YELLOW}Step 3: Building backend...${NC}"
cd backend
cargo build --release

echo -e "${GREEN}Backend built successfully!${NC}"

# Step 4: Create necessary directories
echo -e "\n${YELLOW}Step 4: Creating necessary directories...${NC}"
mkdir -p data
mkdir -p uploads

# Step 5: Display instructions
echo -e "\n${GREEN}=========================================="
echo "  Deployment Complete!"
echo "==========================================${NC}"
echo ""
echo "To run the server:"
echo "  cd backend && ../target/release/backend"
echo ""
echo "Or with systemd (recommended for production):"
echo "  sudo systemctl restart rust-teams"
echo ""
echo "The application will be available at:"
echo "  - Frontend: http://localhost:8080/"
echo "  - API: http://localhost:8080/api/v1"
echo "  - WebSocket: ws://localhost:8080/ws"
echo "  - Health Check: http://localhost:8080/health"
echo ""
echo "Troubleshooting:"
echo "  1. Check logs: journalctl -u rust-teams -f"
echo "  2. Check if port 8080 is in use: ss -tlnp | grep 8080"
echo "  3. Check backend status: systemctl status rust-teams"
echo ""
