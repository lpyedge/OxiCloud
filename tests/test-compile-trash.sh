#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== OxiCloud Trash Feature Compilation Test ===${NC}"

# Set working directory
cd /home/torrefacto/OxiCloud

# 1. Check if storage and trash directories exist
echo -e "${YELLOW}Checking trash directories...${NC}"
./check-trash-dirs.sh

# 2. Build the project to verify our changes
echo -e "\n${YELLOW}Building project to verify changes...${NC}"
cargo build

if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed, please check the errors above${NC}"
    exit 1
fi

echo -e "${GREEN}Build successful!${NC}"

# 3. Run a simple test to verify that the trash feature works
echo -e "\n${YELLOW}Running trash feature test...${NC}"
RUST_LOG=debug cargo run &
SERVER_PID=$!

# Wait for the server to start
echo -e "${YELLOW}Waiting for the server to start (5 seconds)...${NC}"
sleep 5

# Run our debug script
echo -e "${YELLOW}Running trash debug script...${NC}"
python3 debug-trash.py

# Shutdown the server
echo -e "${YELLOW}Shutting down the server...${NC}"
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null

echo -e "${GREEN}Test completed!${NC}"