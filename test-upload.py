#!/usr/bin/env python3
import requests
import argparse
import os
import json

def upload_file(url, file_path, folder_id=None):
    """Upload a file to the server"""
    if not os.path.exists(file_path):
        print(f"Error: File not found: {file_path}")
        return

    # Create the multipart form data
    files = {'file': open(file_path, 'rb')}
    data = {}
    
    if folder_id:
        data['folder_id'] = folder_id
    
    # Send the request
    try:
        response = requests.post(f"{url}/api/files/upload", files=files, data=data)
        return response.json()
    except Exception as e:
        print(f"Error during upload: {e}")
        return None

def list_files(url, folder_id=None):
    """List files in a folder"""
    params = {}
    if folder_id:
        params['folder_id'] = folder_id
    
    try:
        response = requests.get(f"{url}/api/files", params=params)
        return response.json()
    except Exception as e:
        print(f"Error listing files: {e}")
        return None

def download_file(url, file_id, output_path=None):
    """Download a file by its ID"""
    try:
        response = requests.get(f"{url}/api/files/{file_id}")
        
        if output_path is None:
            output_path = file_id.split('/')[-1]  # Use the last part of the path as filename
        
        # Save the file
        with open(output_path, 'wb') as f:
            f.write(response.content)
        
        return output_path
    except Exception as e:
        print(f"Error downloading file: {e}")
        return None

def main():
    parser = argparse.ArgumentParser(description='Test OxiCloud API')
    parser.add_argument('--url', type=str, default="http://localhost:8086", help='Server URL')
    parser.add_argument('--action', type=str, required=True, choices=['upload', 'list', 'download'], help='Action to perform')
    parser.add_argument('--file', type=str, help='Path to file for upload or output path for download')
    parser.add_argument('--folder', type=str, help='Folder ID for upload or list actions')
    parser.add_argument('--id', type=str, help='File ID for download action')
    
    args = parser.parse_args()
    
    if args.action == 'upload':
        if not args.file:
            print("Error: --file is required for upload action")
            return
        
        result = upload_file(args.url, args.file, args.folder)
        if result:
            if isinstance(result, dict):
                print(json.dumps(result, indent=2))
                print(f"File uploaded successfully with ID: {result.get('id', 'unknown')}")
            else:
                print(json.dumps(result, indent=2))
                print("Received unexpected response format")
    
    elif args.action == 'list':
        result = list_files(args.url, args.folder)
        if result:
            print(json.dumps(result, indent=2))
            print(f"Found {len(result)} files")
    
    elif args.action == 'download':
        if not args.id:
            print("Error: --id is required for download action")
            return
        
        output_path = download_file(args.url, args.id, args.file)
        if output_path:
            print(f"File downloaded successfully to {output_path}")

if __name__ == "__main__":
    main()