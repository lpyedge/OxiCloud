#!/usr/bin/env python3
import requests
import json
import time
import uuid
import sys
import os

# Configuration
BASE_URL = "http://localhost:8085/api"
DEBUG = True

# Functions for testing
def log(message):
    if DEBUG:
        print(f"[DEBUG] {message}")

def get_auth_token():
    """Get authentication token for testing"""
    auth_url = f"{BASE_URL}/auth/login"
    payload = {
        "username": "test",
        "password": "test123"
    }
    
    response = requests.post(auth_url, json=payload)
    if response.status_code != 200:
        print(f"Failed to get auth token: {response.text}")
        sys.exit(1)
    
    return response.json()["token"]

def create_test_file(token, folder_id=None):
    """Create a test file and return its ID"""
    url = f"{BASE_URL}/files/upload"
    
    headers = {
        "Authorization": f"Bearer {token}"
    }
    
    # Generate unique filename
    filename = f"test-file-{uuid.uuid4()}.txt"
    
    # Create test file content
    file_content = f"This is a test file content for trash testing: {uuid.uuid4()}"
    
    files = {
        'file': (filename, file_content.encode(), 'text/plain')
    }
    
    data = {}
    if folder_id:
        data['folder_id'] = folder_id
    
    response = requests.post(url, headers=headers, files=files, data=data)
    
    if response.status_code != 201:
        print(f"Failed to create test file: {response.text}")
        return None
    
    log(f"Created test file: {response.json()}")
    return response.json()["id"]

def create_test_folder(token, parent_id=None):
    """Create a test folder and return its ID"""
    url = f"{BASE_URL}/folders"
    
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    
    # Generate unique folder name
    folder_name = f"test-folder-{uuid.uuid4()}"
    
    payload = {
        "name": folder_name
    }
    
    if parent_id:
        payload["parent_id"] = parent_id
    
    response = requests.post(url, headers=headers, json=payload)
    
    if response.status_code != 201:
        print(f"Failed to create test folder: {response.text}")
        return None
    
    log(f"Created test folder: {response.json()}")
    return response.json()["id"]

def move_file_to_trash(token, file_id):
    """Move a file to trash"""
    url = f"{BASE_URL}/files/trash/{file_id}"
    
    headers = {
        "Authorization": f"Bearer {token}"
    }
    
    response = requests.delete(url, headers=headers)
    log(f"Move file to trash response: {response.status_code} - {response.text}")
    
    return response.status_code == 200

def move_folder_to_trash(token, folder_id):
    """Move a folder to trash"""
    url = f"{BASE_URL}/folders/trash/{folder_id}"
    
    headers = {
        "Authorization": f"Bearer {token}"
    }
    
    response = requests.delete(url, headers=headers)
    log(f"Move folder to trash response: {response.status_code} - {response.text}")
    
    return response.status_code == 200

def list_trash_items(token):
    """List all items in trash"""
    url = f"{BASE_URL}/trash"
    
    headers = {
        "Authorization": f"Bearer {token}"
    }
    
    response = requests.get(url, headers=headers)
    log(f"List trash items response: {response.status_code}")
    
    if response.status_code != 200:
        print(f"Failed to list trash items: {response.text}")
        return []
    
    items = response.json()
    log(f"Trash items: {items}")
    return items

def restore_from_trash(token, trash_id):
    """Restore an item from trash"""
    url = f"{BASE_URL}/trash/{trash_id}/restore"
    
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }
    
    response = requests.post(url, headers=headers, json={})
    log(f"Restore from trash response: {response.status_code} - {response.text}")
    
    return response.status_code == 200

def delete_permanently(token, trash_id):
    """Delete an item permanently from trash"""
    url = f"{BASE_URL}/trash/{trash_id}"
    
    headers = {
        "Authorization": f"Bearer {token}"
    }
    
    response = requests.delete(url, headers=headers)
    log(f"Delete permanently response: {response.status_code} - {response.text}")
    
    return response.status_code == 200

def empty_trash(token):
    """Empty the trash (delete all items)"""
    url = f"{BASE_URL}/trash/empty"
    
    headers = {
        "Authorization": f"Bearer {token}"
    }
    
    response = requests.delete(url, headers=headers)
    log(f"Empty trash response: {response.status_code} - {response.text}")
    
    return response.status_code == 200

def check_file_exists(token, file_id):
    """Check if a file exists"""
    url = f"{BASE_URL}/files/{file_id}"
    
    headers = {
        "Authorization": f"Bearer {token}"
    }
    
    response = requests.get(url, headers=headers)
    exists = response.status_code == 200
    log(f"File {file_id} exists: {exists}")
    return exists

def check_folder_exists(token, folder_id):
    """Check if a folder exists"""
    url = f"{BASE_URL}/folders/{folder_id}"
    
    headers = {
        "Authorization": f"Bearer {token}"
    }
    
    response = requests.get(url, headers=headers)
    exists = response.status_code == 200
    log(f"Folder {folder_id} exists: {exists}")
    return exists

def run_tests():
    print("=== Starting Trash API Tests ===")
    
    # Get auth token
    token = get_auth_token()
    print(f"Auth token: {token[:10]}...")
    
    # Test 1: Create a file and move it to trash
    print("\n=== Test 1: File to Trash ===")
    file_id = create_test_file(token)
    assert file_id, "Failed to create test file"
    print(f"Created test file with ID: {file_id}")
    
    # Check file exists before trashing
    assert check_file_exists(token, file_id), "File should exist before moving to trash"
    
    # Move file to trash
    assert move_file_to_trash(token, file_id), "Failed to move file to trash"
    print("Moved file to trash successfully")
    
    # Verify file is no longer accessible in main interface
    assert not check_file_exists(token, file_id), "File should not be accessible after moving to trash"
    
    # Verify file appears in trash
    trash_items = list_trash_items(token)
    file_in_trash = any(item["original_id"] == file_id and item["item_type"] == "file" for item in trash_items)
    assert file_in_trash, "File should appear in trash listing"
    print("File correctly appears in trash")
    
    # Get the trash item ID
    file_trash_id = next(item["id"] for item in trash_items if item["original_id"] == file_id)
    
    # Test 2: Create a folder and move it to trash
    print("\n=== Test 2: Folder to Trash ===")
    folder_id = create_test_folder(token)
    assert folder_id, "Failed to create test folder"
    print(f"Created test folder with ID: {folder_id}")
    
    # Check folder exists before trashing
    assert check_folder_exists(token, folder_id), "Folder should exist before moving to trash"
    
    # Move folder to trash
    assert move_folder_to_trash(token, folder_id), "Failed to move folder to trash"
    print("Moved folder to trash successfully")
    
    # Verify folder is no longer accessible
    assert not check_folder_exists(token, folder_id), "Folder should not be accessible after moving to trash"
    
    # Verify folder appears in trash
    trash_items = list_trash_items(token)
    folder_in_trash = any(item["original_id"] == folder_id and item["item_type"] == "folder" for item in trash_items)
    assert folder_in_trash, "Folder should appear in trash listing"
    print("Folder correctly appears in trash")
    
    # Get the trash item ID
    folder_trash_id = next(item["id"] for item in trash_items if item["original_id"] == folder_id)
    
    # Test 3: Restore file from trash
    print("\n=== Test 3: Restore File from Trash ===")
    assert restore_from_trash(token, file_trash_id), "Failed to restore file from trash"
    print("Restored file from trash successfully")
    
    # Verify file is now accessible again
    assert check_file_exists(token, file_id), "File should be accessible after restoring from trash"
    
    # Verify file no longer appears in trash
    trash_items = list_trash_items(token)
    file_in_trash = any(item["id"] == file_trash_id for item in trash_items)
    assert not file_in_trash, "File should not appear in trash after restoration"
    print("File no longer appears in trash")
    
    # Test 4: Permanently delete folder from trash
    print("\n=== Test 4: Permanently Delete Folder from Trash ===")
    assert delete_permanently(token, folder_trash_id), "Failed to permanently delete folder"
    print("Permanently deleted folder successfully")
    
    # Verify folder is still not accessible
    assert not check_folder_exists(token, folder_id), "Folder should not be accessible after permanent deletion"
    
    # Verify folder no longer appears in trash
    trash_items = list_trash_items(token)
    folder_in_trash = any(item["id"] == folder_trash_id for item in trash_items)
    assert not folder_in_trash, "Folder should not appear in trash after permanent deletion"
    print("Folder no longer appears in trash")
    
    # Test 5: Test Empty Trash functionality
    print("\n=== Test 5: Empty Trash ===")
    
    # Create multiple files and folders and move them to trash
    print("Creating multiple test items...")
    test_files = [create_test_file(token) for _ in range(3)]
    test_folders = [create_test_folder(token) for _ in range(2)]
    
    # Move all to trash
    for file_id in test_files:
        move_file_to_trash(token, file_id)
    
    for folder_id in test_folders:
        move_folder_to_trash(token, folder_id)
    
    # Verify items are in trash
    trash_items = list_trash_items(token)
    assert len(trash_items) >= 5, "All test items should be in trash"
    print(f"Trash contains {len(trash_items)} items")
    
    # Empty trash
    assert empty_trash(token), "Failed to empty trash"
    print("Emptied trash successfully")
    
    # Verify trash is empty
    trash_items = list_trash_items(token)
    assert len(trash_items) == 0, "Trash should be empty"
    print("Trash is empty as expected")
    
    print("\n=== All Trash API Tests Passed! ===")
    return True

if __name__ == "__main__":
    try:
        run_tests()
    except Exception as e:
        print(f"Test failed: {e}")
        sys.exit(1)