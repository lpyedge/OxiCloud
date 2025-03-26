#!/bin/bash

# Configuration 
BASE_URL="http://localhost:8086/api"
USER_ID="00000000-0000-0000-0000-000000000000"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Create a test file directly (no authentication)
create_test_file() {
  echo -e "${YELLOW}Creating test file...${NC}"
  
  local content="Test file content $(date)"
  local filename="test-file-$(date +%s).txt"
  
  echo "$content" > "$filename"
  
  response=$(curl -s -X POST "$BASE_URL/files/upload" \
    -F "file=@$filename" \
    -F "userId=$USER_ID")
  
  file_id=$(echo "$response" | grep -o '"id":"[^"]*' | cut -d'"' -f4)
  
  rm "$filename"
  
  if [ -z "$file_id" ]; then
    echo -e "${RED}Failed to create test file${NC}"
    return 1
  else
    echo -e "${GREEN}Created file with ID: $file_id${NC}"
    echo "$file_id"
    return 0
  fi
}

# Create a test folder
create_test_folder() {
  echo -e "${YELLOW}Creating test folder...${NC}"
  
  local folder_name="test-folder-$(date +%s)"
  
  response=$(curl -s -X POST "$BASE_URL/folders" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"$folder_name\", \"userId\":\"$USER_ID\"}")
  
  folder_id=$(echo "$response" | grep -o '"id":"[^"]*' | cut -d'"' -f4)
  
  if [ -z "$folder_id" ]; then
    echo -e "${RED}Failed to create test folder${NC}"
    return 1
  else
    echo -e "${GREEN}Created folder with ID: $folder_id${NC}"
    echo "$folder_id"
    return 0
  fi
}

# Move a file to trash
move_file_to_trash() {
  local file_id=$1
  echo -e "${YELLOW}Moving file $file_id to trash...${NC}"
  
  response=$(curl -s -X DELETE "$BASE_URL/files/trash/$file_id")
  
  if echo "$response" | grep -q "success"; then
    echo -e "${GREEN}Successfully moved file to trash${NC}"
    return 0
  else
    echo -e "${RED}Failed to move file to trash: $response${NC}"
    return 1
  fi
}

# Move a folder to trash
move_folder_to_trash() {
  local folder_id=$1
  echo -e "${YELLOW}Moving folder $folder_id to trash...${NC}"
  
  response=$(curl -s -X DELETE "$BASE_URL/folders/trash/$folder_id")
  
  if echo "$response" | grep -q "success"; then
    echo -e "${GREEN}Successfully moved folder to trash${NC}"
    return 0
  else
    echo -e "${RED}Failed to move folder to trash: $response${NC}"
    return 1
  fi
}

# List trash items
list_trash_items() {
  echo -e "${YELLOW}Listing trash items...${NC}"
  
  response=$(curl -s -X GET "$BASE_URL/trash?userId=$USER_ID")
  
  echo "$response"
  return 0
}

# Run simple trash test
run_test() {
  echo -e "${GREEN}=== Starting Simple Trash Test ===${NC}"
  
  # Test: Create a file and move it to trash
  echo -e "${GREEN}\n=== Test: File to Trash ===${NC}"
  file_id=$(create_test_file)
  if [ $? -ne 0 ]; then
    echo -e "${RED}Test failed: Could not create test file${NC}"
    exit 1
  fi
  
  # Move file to trash
  move_file_to_trash "$file_id"
  if [ $? -ne 0 ]; then
    echo -e "${RED}Test failed: Could not move file to trash${NC}"
    exit 1
  fi
  
  # List trash items to confirm
  echo -e "${GREEN}Listing trash items after file deletion:${NC}"
  list_trash_items
  
  # Test: Create a folder and move it to trash
  echo -e "${GREEN}\n=== Test: Folder to Trash ===${NC}"
  folder_id=$(create_test_folder)
  if [ $? -ne 0 ]; then
    echo -e "${RED}Test failed: Could not create test folder${NC}"
    exit 1
  fi
  
  # Move folder to trash
  move_folder_to_trash "$folder_id"
  if [ $? -ne 0 ]; then
    echo -e "${RED}Test failed: Could not move folder to trash${NC}"
    exit 1
  fi
  
  # List trash items to confirm
  echo -e "${GREEN}Listing trash items after folder deletion:${NC}"
  list_trash_items
  
  echo -e "${GREEN}\n=== Test Completed ===${NC}"
  return 0
}

# Run the test
run_test
exit $?