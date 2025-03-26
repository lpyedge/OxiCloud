#!/usr/bin/env python3
import requests
import json
import os
import time

# Configuration
BASE_URL = "http://localhost:8086/api"
DEFAULT_USER_ID = "00000000-0000-0000-0000-000000000000"

# Create a test file and get its ID
def create_test_file():
    # Create temp file
    filename = f"test-file-{int(time.time())}.txt"
    with open(filename, 'w') as f:
        f.write(f"Test content {time.time()}")
    
    # Upload file
    files = {'file': open(filename, 'rb')}
    response = requests.post(f"{BASE_URL}/files/upload?userId={DEFAULT_USER_ID}", files=files)
    
    # Clean up
    os.remove(filename)
    
    if response.status_code in [200, 201, 202]:
        data = response.json()
        file_id = data.get('id')
        print(f"Created file with ID: {file_id}")
        return file_id
    else:
        print(f"Failed to create test file: {response.status_code} - {response.text}")
        return None

# Delete the file
def delete_file(file_id):
    print(f"Deleting file with ID: {file_id}")
    
    # Delete the file
    response = requests.delete(f"{BASE_URL}/files/{file_id}?userId={DEFAULT_USER_ID}")
    
    if response.status_code in [200, 201, 202, 204]:
        print(f"File deleted successfully with status code: {response.status_code}")
        return True
    else:
        print(f"Failed to delete file: {response.status_code} - {response.text}")
        return False

# List items in trash
def list_trash():
    print("Listing trash items...")
    
    response = requests.get(f"{BASE_URL}/trash?userId={DEFAULT_USER_ID}")
    
    if response.status_code in [200, 201]:
        items = response.json()
        print(f"Found {len(items)} items in trash:")
        for item in items:
            print(f"- {item['id']} ({item['item_type']}): {item['name']} (original ID: {item['original_id']})")
        return items
    else:
        print(f"Failed to list trash: {response.status_code} - {response.text}")
        return []

def main():
    # Create a test file
    file_id = create_test_file()
    if not file_id:
        print("Could not create test file")
        return
    
    # Delete the file
    if not delete_file(file_id):
        print("Could not delete file")
        return
    
    # Wait for trash operation to complete
    print("Waiting 2 seconds for trash operation to complete...")
    time.sleep(2)
    
    # List trash items
    trash_items = list_trash()
    
    # Check if file is in trash
    file_in_trash = next((item for item in trash_items if item['original_id'] == file_id), None)
    
    if file_in_trash:
        print(f"File found in trash with trash ID: {file_in_trash['id']}")
    else:
        print(f"File not found in trash! Debug the trash implementation.")

if __name__ == "__main__":
    main()