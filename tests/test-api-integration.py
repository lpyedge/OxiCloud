#!/usr/bin/env python3
import requests
import os
import time
import json

print("Testing API integration with OxiCloud API...")

SERVER_URL = "http://localhost:8086"
TEST_FILE = "api-test-file.txt"

# Create a test file
with open(TEST_FILE, "w") as f:
    f.write("This is a test file for API integration testing\n")
    f.write("We'll check if the ID mapping system works correctly\n")
    f.write("The file should be retrievable after upload\n")

print(f"Created test file: {TEST_FILE}")

try:
    # 1. Upload file
    print("\n1. Uploading file...")
    files = {'file': open(TEST_FILE, 'rb')}
    data = {'folder_id': 'folder-storage:1'}
    
    upload_response = requests.post(f"{SERVER_URL}/api/files/upload", files=files, data=data)
    files['file'].close()
    
    print(f"Upload status: {upload_response.status_code}")
    
    if upload_response.status_code == 201:
        upload_data = upload_response.json()
        file_id = upload_data.get('id')
        file_name = upload_data.get('name')
        
        print(f"File uploaded successfully with ID: {file_id}")
        print(f"Response data: {json.dumps(upload_data, indent=2)}")
        
        # 2. List files to see if our file appears
        print("\n2. Listing files...")
        time.sleep(1)  # Small delay to allow server processing
        
        list_response = requests.get(f"{SERVER_URL}/api/files?folder_id=folder-storage:1")
        print(f"List files status: {list_response.status_code}")
        
        if list_response.status_code == 200:
            files_list = list_response.json()
            print(f"Found {len(files_list)} files")
            
            # Look for our file
            found = False
            for file in files_list:
                if file.get('id') == file_id:
                    found = True
                    print(f"Found our file in the list! ID: {file.get('id')}, Name: {file.get('name')}")
                    print(f"File details: {json.dumps(file, indent=2)}")
            
            if not found:
                print(f"ERROR: Our file with ID {file_id} was not found in the list")
                print(f"Files in list: {json.dumps(files_list, indent=2)}")
        else:
            print(f"Error listing files: {list_response.text}")
        
        # 3. Try to download the file
        print("\n3. Downloading file...")
        download_response = requests.get(f"{SERVER_URL}/api/files/{file_id}")
        print(f"Download status: {download_response.status_code}")
        
        if download_response.status_code == 200:
            print("File downloaded successfully")
            print(f"Downloaded content length: {len(download_response.content)} bytes")
            print(f"Content preview: {download_response.content[:50]}...")
        else:
            print(f"Error downloading file: {download_response.text}")
        
    else:
        print(f"Upload failed: {upload_response.text}")

except Exception as e:
    print(f"Error during test: {e}")

# Clean up
if os.path.exists(TEST_FILE):
    os.remove(TEST_FILE)
    print(f"\nRemoved test file: {TEST_FILE}")

print("\nAPI integration test completed.")