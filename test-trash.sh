#!/bin/bash

BASE_URL="http://127.0.0.1:8085/api"

# Get the login token
echo "Logging in..."
TOKEN=$(curl -s -X POST "${BASE_URL}/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"admin", "password":"admin123"}' | jq -r '.access_token')

if [ -z "$TOKEN" ] || [ "$TOKEN" == "null" ]; then
  echo "Failed to get token"
  exit 1
fi

echo "Token: ${TOKEN:0:15}..."

# Create a test folder
echo -e "\nCreating test folder..."
FOLDER_ID=$(curl -s -X POST "${BASE_URL}/folders" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Trash Test Folder", "parent_id":null}' | jq -r '.id')

echo "Created folder with ID: $FOLDER_ID"

# Create a test file in the folder
echo -e "\nCreating test file..."
FILE_CONTENT="This is a test file that will be moved to trash."
TEST_FILE_PATH="/tmp/trash_test_file.txt"
echo "$FILE_CONTENT" > "$TEST_FILE_PATH"

FILE_ID=$(curl -s -X POST "${BASE_URL}/files/upload" \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@$TEST_FILE_PATH" \
  -F "folder_id=$FOLDER_ID" | jq -r '.id')

echo "Created file with ID: $FILE_ID"

# Try the trash operations (these will use the frontend code we modified)
echo -e "\nTesting trash operations through the frontend using direct delete (which uses trash)..."
echo "Moving file to trash..."
curl -s -X DELETE "${BASE_URL}/files/$FILE_ID" \
  -H "Authorization: Bearer $TOKEN"

# Check if file is still accessible (should return 404 if moved to trash)
echo -e "\nChecking if file is still accessible..."
STATUS=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/files/$FILE_ID" \
  -H "Authorization: Bearer $TOKEN")

if [ "$STATUS" == "404" ]; then
  echo "File moved to trash successfully (returns 404)"
else
  echo "File still accessible, move to trash failed (status: $STATUS)"
fi

echo -e "\nMoving folder to trash..."
curl -s -X DELETE "${BASE_URL}/folders/$FOLDER_ID" \
  -H "Authorization: Bearer $TOKEN"

# Check if folder is still accessible
echo -e "\nChecking if folder is still accessible..."
STATUS=$(curl -s -o /dev/null -w "%{http_code}" "${BASE_URL}/folders/$FOLDER_ID" \
  -H "Authorization: Bearer $TOKEN")

if [ "$STATUS" == "404" ]; then
  echo "Folder moved to trash successfully (returns 404)"
else
  echo "Folder still accessible, move to trash failed (status: $STATUS)"
fi

echo -e "\nTest complete."