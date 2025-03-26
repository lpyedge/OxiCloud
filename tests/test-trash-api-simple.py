#!/usr/bin/env python3
import requests
import json
import os
import time
import random
import string

# Configuration
BASE_URL = "http://localhost:8086/api"
DEFAULT_USER_ID = "00000000-0000-0000-0000-000000000000"

def create_test_file():
    """Create a test file and upload it"""
    print("Creating test file...")
    
    # Create temp file
    filename = f"test-file-{int(time.time())}.txt"
    with open(filename, 'w') as f:
        f.write(f"Test content {time.time()}")
    
    # Upload file
    files = {'file': open(filename, 'rb')}
    response = requests.post(f"{BASE_URL}/files/upload?userId={DEFAULT_USER_ID}", files=files)
    
    # Clean up
    os.remove(filename)
    
    if response.status_code in [200, 201]:
        # The response is already a file object with ID
        data = response.json()
        file_id = data.get('id')
        print(f"Created file with ID: {file_id}")
        return file_id
    else:
        print(f"Failed to create test file: {response.status_code} - {response.text}")
        return None

def create_test_folder():
    """Create a test folder"""
    print("Creating test folder...")
    
    folder_name = f"test-folder-{int(time.time())}"
    payload = {
        "name": folder_name
    }
    
    response = requests.post(f"{BASE_URL}/folders?userId={DEFAULT_USER_ID}", json=payload)
    
    if response.status_code in [200, 201]:
        # The response is a folder object with ID
        data = response.json()
        folder_id = data.get('id')
        print(f"Created folder with ID: {folder_id}")
        return folder_id
    else:
        print(f"Failed to create test folder: {response.status_code} - {response.text}")
        return None

def move_to_trash(item_id, item_type):
    """Move an item to trash"""
    print(f"Moving {item_type} {item_id} to trash...")
    
    if item_type == 'file':
        url = f"{BASE_URL}/files/{item_id}?userId={DEFAULT_USER_ID}"
    else:
        url = f"{BASE_URL}/folders/{item_id}?userId={DEFAULT_USER_ID}"
    
    response = requests.delete(url)
    
    if response.status_code in [200, 201, 202, 204]:
        print(f"Successfully moved {item_type} to trash")
        return True
    else:
        print(f"Failed to move {item_type} to trash: {response.status_code} - {response.text}")
        return False

def list_trash():
    """List all items in trash"""
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

def restore_from_trash(trash_id):
    """Restore an item from trash"""
    print(f"Restoring item {trash_id} from trash...")
    
    response = requests.post(f"{BASE_URL}/trash/{trash_id}/restore?userId={DEFAULT_USER_ID}", json={})
    
    if response.status_code in [200, 201, 202, 204]:
        print("Successfully restored item from trash")
        return True
    else:
        print(f"Failed to restore item: {response.status_code} - {response.text}")
        return False

def delete_permanently(trash_id):
    """Delete an item permanently"""
    print(f"Permanently deleting item {trash_id}...")
    
    response = requests.delete(f"{BASE_URL}/trash/{trash_id}?userId={DEFAULT_USER_ID}")
    
    if response.status_code in [200, 201, 202, 204]:
        print("Successfully deleted item permanently")
        return True
    else:
        print(f"Failed to delete item: {response.status_code} - {response.text}")
        return False

def main():
    """Main test function"""
    print("=== Starting Trash API Tests ===")
    
    # Create test file
    file_id = create_test_file()
    if not file_id:
        print("Test failed: Could not create test file")
        return
    
    # Move file to trash
    if not move_to_trash(file_id, 'file'):
        print("Test failed: Could not move file to trash")
        return
    
    # Wait a moment for the trash operation to complete
    print("Waiting 5 seconds for trash operation to complete...")
    time.sleep(5)
    
    # List trash items
    trash_items = list_trash()
    
    # Find our file in trash
    file_trash_item = next((item for item in trash_items if item['original_id'] == file_id and item['item_type'] == 'file'), None)
    if not file_trash_item:
        print("Test failed: File not found in trash")
        return
    
    # Restore file from trash
    if not restore_from_trash(file_trash_item['id']):
        print("Test failed: Could not restore file from trash")
        return
    
    # Create test folder
    folder_id = create_test_folder()
    if not folder_id:
        print("Test failed: Could not create test folder")
        return
    
    # Move folder to trash
    if not move_to_trash(folder_id, 'folder'):
        print("Test failed: Could not move folder to trash")
        return
    
    # List trash items again
    trash_items = list_trash()
    
    # Find our folder in trash
    folder_trash_item = next((item for item in trash_items if item['original_id'] == folder_id and item['item_type'] == 'folder'), None)
    if not folder_trash_item:
        print("Test failed: Folder not found in trash")
        return
    
    # Delete folder permanently
    if not delete_permanently(folder_trash_item['id']):
        print("Test failed: Could not delete folder permanently")
        return
    
    print("=== All Trash API Tests Passed! ===")

if __name__ == "__main__":
    main()