#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== OxiCloud Trash Feature Debug Script ===${NC}"

# 1. Ensure we have debug logging enabled for the server
export RUST_LOG=debug

# 2. Build and run the server in the background
echo -e "${YELLOW}Building and starting the server...${NC}"
cargo build
if [ $? -ne 0 ]; then
    echo -e "${RED}Failed to build the server${NC}"
    exit 1
fi

echo -e "${YELLOW}Starting the server with debug logging...${NC}"
cargo run > server_debug.log 2>&1 &
SERVER_PID=$!

# Wait for the server to start
echo -e "${YELLOW}Waiting for the server to start (5 seconds)...${NC}"
sleep 5

# Verify the server is running
if ! ps -p $SERVER_PID > /dev/null; then
    echo -e "${RED}Server failed to start. Check server_debug.log for details.${NC}"
    exit 1
fi

echo -e "${GREEN}Server started successfully with PID $SERVER_PID${NC}"

# 3. Run the debug script
echo -e "${YELLOW}Running the trash debug script...${NC}"
python3 debug-trash.py

# 4. Shutdown the server
echo -e "${YELLOW}Shutting down the server...${NC}"
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null

echo -e "${GREEN}Debug run completed. Check server_debug.log for server output.${NC}"