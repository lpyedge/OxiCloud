#!/bin/bash

# Run unit tests for the trash feature
echo "Running unit tests for the trash feature..."
RUST_LOG=debug cargo test application::services::trash_service_test::tests -- --nocapture

# Set up environment for API tests
echo "Setting up environment for API tests..."
cargo build

# Start the server in the background
echo "Starting the server..."
RUST_LOG=debug cargo run &
SERVER_PID=$!

# Wait for the server to start
echo "Waiting for the server to start..."
sleep 5

# Run the API tests
echo "Running API tests for the trash feature..."
python3 test-trash-api.py

# Clean up
echo "Cleaning up..."
kill $SERVER_PID

echo "All tests completed!"