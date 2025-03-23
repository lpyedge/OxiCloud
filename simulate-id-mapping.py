#!/usr/bin/env python3
import os
import json
import uuid
from pathlib import Path

def ensure_directory(path):
    if isinstance(path, str):
        path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)

def create_test_file(path, content="This is a test file content"):
    # Make sure parent directories exist
    ensure_directory(path)
    
    with open(path, 'w') as f:
        f.write(content)
    print(f"Created test file: {path}")

def update_id_mapping(file_path, storage_path):
    # Create a mapping file to simulate what the server would do
    file_ids_path = "/home/torrefacto/OxiCloud/storage/file_ids.json"
    folder_ids_path = "/home/torrefacto/OxiCloud/storage/folder_ids.json"
    
    # Make sure directory exists
    ensure_directory(Path(file_ids_path).parent)
    
    # Generate a UUID for the file
    file_id = str(uuid.uuid4())
    
    # Load existing file mapping if it exists
    file_mapping = {"path_to_id": {}, "id_to_path": {}, "version": 1}
    if os.path.exists(file_ids_path):
        try:
            with open(file_ids_path, 'r') as f:
                file_mapping = json.load(f)
        except json.JSONDecodeError:
            print(f"Warning: Could not parse {file_ids_path}, creating new mapping")
    
    # Add the new mapping
    file_mapping["path_to_id"][storage_path] = file_id
    file_mapping["id_to_path"][file_id] = storage_path
    file_mapping["version"] += 1
    
    # Save the updated mapping
    with open(file_ids_path, 'w') as f:
        json.dump(file_mapping, f, indent=2, sort_keys=True)
    
    print(f"Updated file ID mapping: {storage_path} -> {file_id}")
    
    # Check folder mapping too and update it for parent folders
    folder_mapping = {"path_to_id": {}, "id_to_path": {}, "version": 1}
    if os.path.exists(folder_ids_path):
        try:
            with open(folder_ids_path, 'r') as f:
                folder_mapping = json.load(f)
        except json.JSONDecodeError:
            print(f"Warning: Could not parse {folder_ids_path}, creating new mapping")
    
    # Get parent folders and add them to the mapping
    storage_path_parts = storage_path.split('/')
    if len(storage_path_parts) > 1:  # Has parent folder(s)
        current_path = ""
        for i in range(len(storage_path_parts) - 1):  # All but the last part (file name)
            if i > 0:
                current_path += "/"
            current_path += storage_path_parts[i]
            
            # Check if folder already has an ID
            if current_path not in folder_mapping["path_to_id"]:
                folder_id = str(uuid.uuid4())
                folder_mapping["path_to_id"][current_path] = folder_id
                folder_mapping["id_to_path"][folder_id] = current_path
                print(f"Added folder mapping: {current_path} -> {folder_id}")
    
    # Save the folder mapping
    folder_mapping["version"] += 1
    with open(folder_ids_path, 'w') as f:
        json.dump(folder_mapping, f, indent=2, sort_keys=True)
        
    print(f"Folder mapping now has {len(folder_mapping['path_to_id'])} entries")
    
    return file_id

def main():
    # Create multiple test files in different folders
    test_files = [
        # Basic file in root
        ("/home/torrefacto/OxiCloud/storage/test-simulation-file.txt", "test-simulation-file.txt"),
        
        # File in a subfolder
        ("/home/torrefacto/OxiCloud/storage/documents/important-doc.txt", "documents/important-doc.txt"),
        
        # File in a deeper subfolder
        ("/home/torrefacto/OxiCloud/storage/projects/2023/notes.txt", "projects/2023/notes.txt"),
        
        # File with spaces in name
        ("/home/torrefacto/OxiCloud/storage/My Documents/report with spaces.pdf", "My Documents/report with spaces.pdf")
    ]
    
    # Create and map each file
    for file_path, storage_path in test_files:
        create_test_file(file_path)
        file_id = update_id_mapping(file_path, storage_path)
        print(f"Created file with ID: {file_id}")
    
    print(f"Simulation complete. Created {len(test_files)} files with proper ID mappings.")
    print(f"You can now test accessing these files through the web interface using their IDs.")

if __name__ == "__main__":
    main()