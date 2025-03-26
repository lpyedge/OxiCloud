#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== OxiCloud Trash Directory Check Script ===${NC}"

# Configuration
STORAGE_DIR="./storage"
TRASH_DIR="$STORAGE_DIR/.trash"
TRASH_FILES_DIR="$TRASH_DIR/files"

# Check if storage directory exists
echo -e "${YELLOW}Checking if storage directory exists: $STORAGE_DIR${NC}"
if [ ! -d "$STORAGE_DIR" ]; then
    echo -e "${RED}Storage directory does not exist. Creating it...${NC}"
    mkdir -p "$STORAGE_DIR"
    if [ $? -ne 0 ]; then
        echo -e "${RED}Failed to create storage directory${NC}"
        exit 1
    fi
    echo -e "${GREEN}Storage directory created successfully${NC}"
else
    echo -e "${GREEN}Storage directory exists${NC}"
fi

# Check if trash directory exists
echo -e "${YELLOW}Checking if trash directory exists: $TRASH_DIR${NC}"
if [ ! -d "$TRASH_DIR" ]; then
    echo -e "${RED}Trash directory does not exist. Creating it...${NC}"
    mkdir -p "$TRASH_DIR"
    if [ $? -ne 0 ]; then
        echo -e "${RED}Failed to create trash directory${NC}"
        exit 1
    fi
    echo -e "${GREEN}Trash directory created successfully${NC}"
else
    echo -e "${GREEN}Trash directory exists${NC}"
fi

# Check if trash files directory exists
echo -e "${YELLOW}Checking if trash files directory exists: $TRASH_FILES_DIR${NC}"
if [ ! -d "$TRASH_FILES_DIR" ]; then
    echo -e "${RED}Trash files directory does not exist. Creating it...${NC}"
    mkdir -p "$TRASH_FILES_DIR"
    if [ $? -ne 0 ]; then
        echo -e "${RED}Failed to create trash files directory${NC}"
        exit 1
    fi
    echo -e "${GREEN}Trash files directory created successfully${NC}"
else
    echo -e "${GREEN}Trash files directory exists${NC}"
fi

# Check if trash index file exists
echo -e "${YELLOW}Checking if trash index file exists: $TRASH_DIR/trash_index.json${NC}"
if [ ! -f "$TRASH_DIR/trash_index.json" ]; then
    echo -e "${RED}Trash index file does not exist. Creating it...${NC}"
    echo "[]" > "$TRASH_DIR/trash_index.json"
    if [ $? -ne 0 ]; then
        echo -e "${RED}Failed to create trash index file${NC}"
        exit 1
    fi
    echo -e "${GREEN}Trash index file created successfully${NC}"
else
    echo -e "${GREEN}Trash index file exists${NC}"
    echo -e "${YELLOW}Current trash index file content:${NC}"
    cat "$TRASH_DIR/trash_index.json"
fi

echo -e "\n${GREEN}All trash directories and files are ready!${NC}"