#!/usr/bin/env python3
import requests
import json

SERVER_URL = "http://localhost:8086"

print("Checking which files are visible from the frontend...")

def make_request(url, method="GET", params=None):
    print(f"\n[{method}] {url}")
    
    try:
        if method == "GET":
            response = requests.get(url, params=params)
        else:
            return None
        
        if response.status_code == 200:
            if response.headers.get('Content-Type', '').startswith('application/json'):
                return response.json()
            else:
                return response.text[:100] + "..."
        else:
            return f"Error: {response.status_code} - {response.text}"
    except Exception as e:
        return f"Exception: {str(e)}"

# List files at root
print("Files at root level:")
root_files = make_request(f"{SERVER_URL}/api/files")
if isinstance(root_files, list):
    for file in root_files:
        print(f"- {file.get('name')} (ID: {file.get('id')})")
else:
    print(f"Error: {root_files}")

# List files in folder-storage:1
print("\nFiles in folder-storage:1:")
folder_files = make_request(f"{SERVER_URL}/api/files", params={"folder_id": "folder-storage:1"})
if isinstance(folder_files, list):
    for file in folder_files:
        print(f"- {file.get('name')} (ID: {file.get('id')})")
else:
    print(f"Error: {folder_files}")

# Check file_ids.json and folder_ids.json
print("\nContents of file_ids.json:")
try:
    with open("./storage/file_ids.json", "r") as f:
        file_ids = json.load(f)
        print(json.dumps(file_ids, indent=2))
except Exception as e:
    print(f"Error reading file_ids.json: {e}")

print("\nContents of folder_ids.json:")
try:
    with open("./storage/folder_ids.json", "r") as f:
        folder_ids = json.load(f)
        print(json.dumps(folder_ids, indent=2))
except Exception as e:
    print(f"Error reading folder_ids.json: {e}")