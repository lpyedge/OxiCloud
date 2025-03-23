#!/usr/bin/env python3
import requests
import time

def test_route(url, method="GET", data=None, files=None):
    """Test a route and return the response"""
    print(f"Testing {method} {url}")
    try:
        if method == "GET":
            response = requests.get(url)
        elif method == "POST":
            response = requests.post(url, data=data, files=files)
        elif method == "PUT":
            response = requests.put(url, json=data)
        elif method == "DELETE":
            response = requests.delete(url)
        else:
            print(f"Unsupported method: {method}")
            return None
        
        print(f"  Status: {response.status_code}")
        if response.status_code < 400:
            content_type = response.headers.get('Content-Type', '')
            if 'json' in content_type:
                try:
                    print(f"  Response: {response.json()}")
                except:
                    print(f"  Response: {response.text[:100]}...")
            else:
                print(f"  Response: {response.text[:100]}...")
        else:
            print(f"  Error: {response.text}")
        
        return response
    except Exception as e:
        print(f"  Error: {e}")
        return None

# Base URL
SERVER_URL = "http://localhost:8086"

print("Testing all routes to identify which ones are implemented in the custom server")
print("================================================================================")

# Test routes
routes = [
    # Base
    "/",
    
    # API endpoints
    "/api/folders",
    "/api/files",
    "/api/files?folder_id=folder-storage:1",
    "/api/files/upload",
    
    # Static files
    "/css/style.css",
    "/js/app.js",
    "/locales/en.json",
    
    # Auth routes
    "/login",
    "/api/auth/login",
]

# Run GET tests
for route in routes:
    if route == "/api/files/upload":
        continue  # Skip for now, will test POST later
    test_route(f"{SERVER_URL}{route}")
    time.sleep(0.5)  # Small delay between requests

# Test POST upload
print("\nTesting file upload...")
with open("test-all-routes.py", "rb") as f:
    files = {"file": f}
    data = {"folder_id": "folder-storage:1"}
    test_route(f"{SERVER_URL}/api/files/upload", method="POST", data=data, files=files)

print("\nTests completed")