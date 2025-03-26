#!/usr/bin/env python3
import requests
import json
import time
import sys
import os

# Configuration
BASE_URL = "http://localhost:8085/api"
DEFAULT_USER_ID = "00000000-0000-0000-0000-000000000000"
DEBUG = True

# Save the current directory
CURRENT_DIR = os.getcwd()

def log(message):
    if DEBUG:
        print(f"[DEBUG] {message}")

def create_test_file():
    """Create a test file and return its ID"""
    url = f"{BASE_URL}/files/upload"
    
    # Create a unique filename
    filename = f"test-file-{int(time.time())}.txt"
    file_content = f"Test content created at {time.time()}"
    
    files = {'file': (filename, file_content.encode(), 'text/plain')}
    log(f"Uploading file: {filename}")
    
    response = requests.post(url, files=files)
    log(f"Upload response: {response.status_code}")
    
    if response.status_code in [200, 201]:
        data = response.json()
        file_id = data.get('id')
        log(f"File created with ID: {file_id}")
        return file_id
    else:
        log(f"Failed to create file: {response.text}")
        return None

def delete_file_to_trash(file_id):
    """Delete a file (should move to trash)"""
    url = f"{BASE_URL}/files/{file_id}"
    log(f"Deleting file: {file_id} (should move to trash)")
    
    response = requests.delete(url)
    log(f"Delete response: {response.status_code}")
    
    if response.status_code in [200, 201, 202, 204]:
        return True
    else:
        log(f"Failed to delete file: {response.text}")
        return False

def list_trash_items():
    """List all items in the trash"""
    url = f"{BASE_URL}/trash?userId={DEFAULT_USER_ID}"
    log("Listing trash items")
    
    response = requests.get(url)
    log(f"List trash response: {response.status_code}")
    
    if response.status_code == 200:
        items = response.json()
        log(f"Found {len(items)} items in trash")
        for item in items:
            print(f"  - {item['name']} (ID: {item['id']}, Original ID: {item['original_id']}, Type: {item['item_type']})")
        return items
    else:
        log(f"Failed to list trash items: {response.text}")
        return []

def check_trash_structure():
    """Checks the structure of the trash directory"""
    print("\n--- Checking Trash Directory Structure ---")
    
    # Check storage directory
    storage_dir = os.path.join(CURRENT_DIR, "storage")
    if os.path.exists(storage_dir):
        print(f"Storage directory exists: {storage_dir}")
    else:
        print(f"ERROR: Storage directory does not exist: {storage_dir}")
        return False
    
    # Check trash directory
    trash_dir = os.path.join(storage_dir, ".trash")
    if os.path.exists(trash_dir):
        print(f"Trash directory exists: {trash_dir}")
    else:
        print(f"ERROR: Trash directory does not exist: {trash_dir}")
        return False
    
    # Check trash files directory
    trash_files_dir = os.path.join(trash_dir, "files")
    if os.path.exists(trash_files_dir):
        print(f"Trash files directory exists: {trash_files_dir}")
    else:
        print(f"ERROR: Trash files directory does not exist: {trash_files_dir}")
        return False
    
    # Check trash index file
    trash_index_path = os.path.join(trash_dir, "trash_index.json")
    if os.path.exists(trash_index_path):
        print(f"Trash index file exists: {trash_index_path}")
        try:
            with open(trash_index_path, 'r') as f:
                trash_index = json.load(f)
                print(f"Trash index contains {len(trash_index)} entries")
        except Exception as e:
            print(f"ERROR: Could not read trash index file: {e}")
            return False
    else:
        print(f"ERROR: Trash index file does not exist: {trash_index_path}")
        return False
    
    return True

def check_file_in_trash_fs(file_id):
    """Checks if a file exists in the trash directory filesystem"""
    print("\n--- Checking File In Trash Filesystem ---")
    
    # Check if the file exists in the trash files directory
    trash_files_dir = os.path.join(CURRENT_DIR, "storage", ".trash", "files")
    if os.path.exists(os.path.join(trash_files_dir, file_id)):
        print(f"File found in trash filesystem: {file_id}")
        return True
    else:
        print(f"File NOT found in trash filesystem: {file_id}")
        
        # List all files in the trash directory to help debugging
        print("\nFiles in trash directory:")
        try:
            files = os.listdir(trash_files_dir)
            if files:
                for f in files:
                    print(f"  - {f}")
            else:
                print("  (no files)")
        except Exception as e:
            print(f"Error listing trash directory: {e}")
        
        return False

def dump_trash_index():
    """Dumps the contents of the trash index file"""
    trash_index_path = os.path.join(CURRENT_DIR, "storage", ".trash", "trash_index.json")
    try:
        with open(trash_index_path, 'r') as f:
            trash_index = json.load(f)
            print("\n--- Trash Index Contents ---")
            print(json.dumps(trash_index, indent=2))
    except Exception as e:
        print(f"ERROR: Could not read trash index file: {e}")

def main():
    print("=== Trash Debug Tool ===")
    
    # First check the trash directory structure
    if not check_trash_structure():
        print("FAILED: Trash directory structure is not correct")
        print("Run the check-trash-dirs.sh script to fix it")
        sys.exit(1)
    
    # List current trash contents
    print("\n--- Current Trash Contents ---")
    list_trash_items()
    
    # Create a test file
    print("\n1. Creating test file...")
    file_id = create_test_file()
    if not file_id:
        print("FAILED: Could not create test file")
        sys.exit(1)
    
    print(f"Created file with ID: {file_id}")
    
    # Delete the file (should move to trash)
    print("\n2. Deleting file (should move to trash)...")
    if not delete_file_to_trash(file_id):
        print("FAILED: Could not delete file")
        sys.exit(1)
    
    print("\n3. Waiting 2 seconds for trash operation to complete...")
    time.sleep(2)
    
    # Check if the file appears in trash
    print("\n4. Checking trash contents after deletion...")
    trash_items = list_trash_items()
    
    file_in_trash = False
    for item in trash_items:
        if item.get('original_id') == file_id:
            file_in_trash = True
            break
    
    # Check if the file physically exists in the trash directory
    file_in_trash_fs = check_file_in_trash_fs(file_id)
    
    # Dump the trash index file contents
    dump_trash_index()
    
    # Final result
    if file_in_trash and file_in_trash_fs:
        print("\nSUCCESS: File was moved to trash correctly")
    elif file_in_trash:
        print("\nPARTIAL SUCCESS: File is in trash index but not in trash filesystem")
    elif file_in_trash_fs:
        print("\nPARTIAL SUCCESS: File is in trash filesystem but not in trash index")
    else:
        print("\nFAILURE: File was not found in trash")
        print("This indicates the trash feature is not working properly")

if __name__ == "__main__":
    main()