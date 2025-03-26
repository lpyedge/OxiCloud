#!/usr/bin/env python3
import requests
import os
import json

file_path = "./test-api-file.txt"
url = "http://localhost:8086/api/files/upload"

# Check file existence
if not os.path.exists(file_path):
    print(f"Error: File not found at {file_path}")
    exit(1)

# Create multipart form
files = {'file': open(file_path, 'rb')}

# Make request
try:
    response = requests.post(url, files=files)
    print(f"Status Code: {response.status_code}")
    print("Response Headers:")
    for key, value in response.headers.items():
        print(f"{key}: {value}")
    print("\nResponse Content:")
    try:
        data = response.json()
        print(json.dumps(data, indent=2))
    except:
        print(response.text)
except Exception as e:
    print(f"Error: {e}")