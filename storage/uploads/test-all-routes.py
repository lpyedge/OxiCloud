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
