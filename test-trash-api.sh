#!/bin/bash

# Configuration
BASE_URL="http://localhost:8085/api"
AUTH_TOKEN=""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Get auth token
get_auth_token() {
  echo -e "${YELLOW}Getting auth token...${NC}"
  
  response=$(curl -s -X POST "$BASE_URL/auth/login" \
    -H "Content-Type: application/json" \
    -d '{"username":"test","password":"test123"}')
  
  AUTH_TOKEN=$(echo "$response" | grep -o '"token":"[^"]*' | cut -d'"' -f4)
  
  if [ -z "$AUTH_TOKEN" ]; then
    echo -e "${RED}Failed to get auth token${NC}"
    exit 1
  else
    echo -e "${GREEN}Auth token: ${AUTH_TOKEN:0:10}...${NC}"
  fi
}

# Create a test file
create_test_file() {
  echo -e "${YELLOW}Creating test file...${NC}"
  
  local content="Test file content $(date)"
  local filename="test-file-$(date +%s).txt"
  
  echo "$content" > "$filename"
  
  response=$(curl -s -X POST "$BASE_URL/files/upload" \
    -H "Authorization: Bearer $AUTH_TOKEN" \
    -F "file=@$filename")
  
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
    -H "Authorization: Bearer $AUTH_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"$folder_name\"}")
  
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
  
  response=$(curl -s -X DELETE "$BASE_URL/files/trash/$file_id" \
    -H "Authorization: Bearer $AUTH_TOKEN")
  
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
  
  response=$(curl -s -X DELETE "$BASE_URL/folders/trash/$folder_id" \
    -H "Authorization: Bearer $AUTH_TOKEN")
  
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
  
  response=$(curl -s -X GET "$BASE_URL/trash" \
    -H "Authorization: Bearer $AUTH_TOKEN")
  
  echo "$response" | jq
  return 0
}

# Restore an item from trash
restore_from_trash() {
  local trash_id=$1
  echo -e "${YELLOW}Restoring item $trash_id from trash...${NC}"
  
  response=$(curl -s -X POST "$BASE_URL/trash/$trash_id/restore" \
    -H "Authorization: Bearer $AUTH_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{}")
  
  if echo "$response" | grep -q "success"; then
    echo -e "${GREEN}Successfully restored item from trash${NC}"
    return 0
  else
    echo -e "${RED}Failed to restore item from trash: $response${NC}"
    return 1
  fi
}

# Delete an item permanently
delete_permanently() {
  local trash_id=$1
  echo -e "${YELLOW}Permanently deleting item $trash_id...${NC}"
  
  response=$(curl -s -X DELETE "$BASE_URL/trash/$trash_id" \
    -H "Authorization: Bearer $AUTH_TOKEN")
  
  if echo "$response" | grep -q "success"; then
    echo -e "${GREEN}Successfully deleted item permanently${NC}"
    return 0
  else
    echo -e "${RED}Failed to delete item permanently: $response${NC}"
    return 1
  fi
}

# Empty the trash
empty_trash() {
  echo -e "${YELLOW}Emptying trash...${NC}"
  
  response=$(curl -s -X DELETE "$BASE_URL/trash/empty" \
    -H "Authorization: Bearer $AUTH_TOKEN")
  
  if echo "$response" | grep -q "success"; then
    echo -e "${GREEN}Successfully emptied trash${NC}"
    return 0
  else
    echo -e "${RED}Failed to empty trash: $response${NC}"
    return 1
  fi
}

# Check if a file exists
check_file_exists() {
  local file_id=$1
  echo -e "${YELLOW}Checking if file $file_id exists...${NC}"
  
  response=$(curl -s -o /dev/null -w "%{http_code}" -X GET "$BASE_URL/files/$file_id" \
    -H "Authorization: Bearer $AUTH_TOKEN")
  
  if [ "$response" == "200" ]; then
    echo -e "${GREEN}File exists${NC}"
    return 0
  else
    echo -e "${RED}File does not exist (HTTP $response)${NC}"
    return 1
  fi
}

# Check if a folder exists
check_folder_exists() {
  local folder_id=$1
  echo -e "${YELLOW}Checking if folder $folder_id exists...${NC}"
  
  response=$(curl -s -o /dev/null -w "%{http_code}" -X GET "$BASE_URL/folders/$folder_id" \
    -H "Authorization: Bearer $AUTH_TOKEN")
  
  if [ "$response" == "200" ]; then
    echo -e "${GREEN}Folder exists${NC}"
    return 0
  else
    echo -e "${RED}Folder does not exist (HTTP $response)${NC}"
    return 1
  fi
}

# Run tests
run_tests() {
  echo -e "${GREEN}=== Starting Trash API Tests ===${NC}"
  
  # Get auth token
  get_auth_token
  
  # Test 1: Create a file and move it to trash
  echo -e "${GREEN}\n=== Test 1: File to Trash ===${NC}"
  file_id=$(create_test_file)
  if [ $? -ne 0 ]; then
    echo -e "${RED}Test 1 failed: Could not create test file${NC}"
    exit 1
  fi
  
  # Check file exists before trashing
  check_file_exists "$file_id"
  if [ $? -ne 0 ]; then
    echo -e "${RED}Test 1 failed: File should exist before moving to trash${NC}"
    exit 1
  fi
  
  # Move file to trash
  move_file_to_trash "$file_id"
  if [ $? -ne 0 ]; then
    echo -e "${RED}Test 1 failed: Could not move file to trash${NC}"
    exit 1
  fi
  
  # Verify file is no longer accessible in main interface
  check_file_exists "$file_id"
  if [ $? -eq 0 ]; then
    echo -e "${RED}Test 1 failed: File should not be accessible after moving to trash${NC}"
    exit 1
  else
    echo -e "${GREEN}File correctly inaccessible after moving to trash${NC}"
  fi
  
  # Verify file appears in trash
  response=$(curl -s -X GET "$BASE_URL/trash" \
    -H "Authorization: Bearer $AUTH_TOKEN")
  
  file_trash_id=$(echo "$response" | jq -r ".[] | select(.original_id == \"$file_id\" and .item_type == \"file\") | .id")
  
  if [ -z "$file_trash_id" ]; then
    echo -e "${RED}Test 1 failed: File should appear in trash listing${NC}"
    exit 1
  else
    echo -e "${GREEN}File correctly appears in trash with trash ID: $file_trash_id${NC}"
  fi
  
  # Test 2: Create a folder and move it to trash
  echo -e "${GREEN}\n=== Test 2: Folder to Trash ===${NC}"
  folder_id=$(create_test_folder)
  if [ $? -ne 0 ]; then
    echo -e "${RED}Test 2 failed: Could not create test folder${NC}"
    exit 1
  fi
  
  # Check folder exists before trashing
  check_folder_exists "$folder_id"
  if [ $? -ne 0 ]; then
    echo -e "${RED}Test 2 failed: Folder should exist before moving to trash${NC}"
    exit 1
  fi
  
  # Move folder to trash
  move_folder_to_trash "$folder_id"
  if [ $? -ne 0 ]; then
    echo -e "${RED}Test 2 failed: Could not move folder to trash${NC}"
    exit 1
  fi
  
  # Verify folder is no longer accessible
  check_folder_exists "$folder_id"
  if [ $? -eq 0 ]; then
    echo -e "${RED}Test 2 failed: Folder should not be accessible after moving to trash${NC}"
    exit 1
  else
    echo -e "${GREEN}Folder correctly inaccessible after moving to trash${NC}"
  fi
  
  # Verify folder appears in trash
  response=$(curl -s -X GET "$BASE_URL/trash" \
    -H "Authorization: Bearer $AUTH_TOKEN")
  
  folder_trash_id=$(echo "$response" | jq -r ".[] | select(.original_id == \"$folder_id\" and .item_type == \"folder\") | .id")
  
  if [ -z "$folder_trash_id" ]; then
    echo -e "${RED}Test 2 failed: Folder should appear in trash listing${NC}"
    exit 1
  else
    echo -e "${GREEN}Folder correctly appears in trash with trash ID: $folder_trash_id${NC}"
  fi
  
  # Test 3: Restore file from trash
  echo -e "${GREEN}\n=== Test 3: Restore File from Trash ===${NC}"
  restore_from_trash "$file_trash_id"
  if [ $? -ne 0 ]; then
    echo -e "${RED}Test 3 failed: Could not restore file from trash${NC}"
    exit 1
  fi
  
  # Verify file is now accessible again
  check_file_exists "$file_id"
  if [ $? -ne 0 ]; then
    echo -e "${RED}Test 3 failed: File should be accessible after restoring from trash${NC}"
    exit 1
  fi
  
  # Verify file no longer appears in trash
  response=$(curl -s -X GET "$BASE_URL/trash" \
    -H "Authorization: Bearer $AUTH_TOKEN")
  
  file_still_in_trash=$(echo "$response" | jq -r ".[] | select(.id == \"$file_trash_id\") | .id")
  
  if [ ! -z "$file_still_in_trash" ]; then
    echo -e "${RED}Test 3 failed: File should not appear in trash after restoration${NC}"
    exit 1
  else
    echo -e "${GREEN}File no longer appears in trash${NC}"
  fi
  
  # Test 4: Permanently delete folder from trash
  echo -e "${GREEN}\n=== Test 4: Permanently Delete Folder from Trash ===${NC}"
  delete_permanently "$folder_trash_id"
  if [ $? -ne 0 ]; then
    echo -e "${RED}Test 4 failed: Could not permanently delete folder${NC}"
    exit 1
  fi
  
  # Verify folder is still not accessible
  check_folder_exists "$folder_id"
  if [ $? -eq 0 ]; then
    echo -e "${RED}Test 4 failed: Folder should not be accessible after permanent deletion${NC}"
    exit 1
  fi
  
  # Verify folder no longer appears in trash
  response=$(curl -s -X GET "$BASE_URL/trash" \
    -H "Authorization: Bearer $AUTH_TOKEN")
  
  folder_still_in_trash=$(echo "$response" | jq -r ".[] | select(.id == \"$folder_trash_id\") | .id")
  
  if [ ! -z "$folder_still_in_trash" ]; then
    echo -e "${RED}Test 4 failed: Folder should not appear in trash after permanent deletion${NC}"
    exit 1
  else
    echo -e "${GREEN}Folder no longer appears in trash${NC}"
  fi
  
  # Test 5: Test Empty Trash functionality
  echo -e "${GREEN}\n=== Test 5: Empty Trash ===${NC}"
  
  # Create multiple files and folders and move them to trash
  echo -e "${YELLOW}Creating multiple test items...${NC}"
  file_ids=()
  folder_ids=()
  
  for i in {1..3}; do
    file_id=$(create_test_file)
    file_ids+=("$file_id")
    move_file_to_trash "$file_id"
  done
  
  for i in {1..2}; do
    folder_id=$(create_test_folder)
    folder_ids+=("$folder_id")
    move_folder_to_trash "$folder_id"
  done
  
  # Verify items are in trash
  response=$(curl -s -X GET "$BASE_URL/trash" \
    -H "Authorization: Bearer $AUTH_TOKEN")
  
  trash_count=$(echo "$response" | jq '. | length')
  echo -e "${GREEN}Trash contains $trash_count items${NC}"
  
  # Empty trash
  empty_trash
  if [ $? -ne 0 ]; then
    echo -e "${RED}Test 5 failed: Could not empty trash${NC}"
    exit 1
  fi
  
  # Verify trash is empty
  response=$(curl -s -X GET "$BASE_URL/trash" \
    -H "Authorization: Bearer $AUTH_TOKEN")
  
  trash_count=$(echo "$response" | jq '. | length')
  
  if [ "$trash_count" -ne 0 ]; then
    echo -e "${RED}Test 5 failed: Trash should be empty, but contains $trash_count items${NC}"
    exit 1
  else
    echo -e "${GREEN}Trash is empty as expected${NC}"
  fi
  
  echo -e "${GREEN}\n=== All Trash API Tests Passed! ===${NC}"
  return 0
}

# Run the tests
run_tests
exit $?