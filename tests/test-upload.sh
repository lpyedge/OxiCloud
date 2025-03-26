#!/bin/bash

# Test file upload to the server
echo "Testing file upload to OxiCloud server..."

# Build the form data
curl -X POST \
     -F "file=@test-upload.txt" \
     -F "folder_id=folder-storage:1" \
     http://localhost:8086/api/files/upload

echo ""
echo "Upload test completed."