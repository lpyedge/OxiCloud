# OxiCloud Tests

This directory contains test scripts, utilities and troubleshooting tools for the OxiCloud project.

## API Tests

* `test-all-routes.py` - Tests all API routes to verify which ones are implemented
* `test-api.sh` - Bash script to test basic API functionality (list, upload, download)
* `test-api-integration.py` - Python-based integration tests for the API
* `test-upload.py` - Dedicated upload testing script with more options
* `test-upload.sh` - Simple shell script for testing file uploads
* `test-delete-file.py` - Tests the file deletion functionality

## Folder Tests

* `test-folder.sh` - Tests folder API operations
* `test-folder-simple.sh` - Simplified folder creation/listing tests
* `test-create-folder.sh` - Specific test for folder creation
* `test-folder.js` - JavaScript based folder tests

## Trash Tests

* `test-trash.sh` - Tests trash functionality
* `test-trash-simple.sh` - Basic trash functionality test
* `test-trash-api.py` - Python script for testing trash API
* `test-trash-api-simple.py` - Simplified version of trash API tests
* `test-trash-api.sh` - Bash scripts for trash API testing
* `test-compile-trash.sh` - Tests compilation with trash feature enabled
* `fix-trash-index.sh` - Utility to fix trash indexing issues
* `check-trash-dirs.sh` - Checks trash directories structure
* `debug-trash.py` - Debug tool for trash functionality
* `run-trash-test.sh` - Runner for trash tests

## Authentication Tests

* `test-auth-api.sh` - Tests the authentication API endpoints
* `test-auth-env.sh` - Tests authentication with environment variables

## Utilities

* `check-db.sh` - Database check utility
* `simulate-id-mapping.py` - Simulates ID mapping for testing
* `direct-upload-test.py` - Tests direct uploads bypassing certain layers

## Test Files

* `test-upload.txt` - Sample file for upload testing
* `test-api-file.txt` - Sample file for API testing

## Running Tests

Most test scripts can be run directly from this directory. Many accept command-line arguments
to customize their behavior. Check the script headers or run with `--help` for more information.

Basic usage examples:

```bash
# Test API endpoints
python test-all-routes.py

# Test file upload
./test-upload.sh --file sample.txt

# Test trash API
python test-trash-api.py

# Run folder tests
./test-folder.sh
```

Note that these tests expect a running OxiCloud server, typically on localhost:8086.