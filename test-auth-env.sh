#!/bin/bash
# Environment variables for OxiCloud authentication testing
export OXICLOUD_ENABLE_AUTH=true
export OXICLOUD_JWT_SECRET="testing-secret-key-for-development-only"
export OXICLOUD_ACCESS_TOKEN_EXPIRY_SECS=3600
export OXICLOUD_REFRESH_TOKEN_EXPIRY_SECS=86400
export OXICLOUD_DB_CONNECTION_STRING="postgres://postgres:postgres@localhost/oxicloud"

# Run with: source test-auth-env.sh && cargo run
echo "Authentication environment variables set. Run 'cargo run' to start OxiCloud with auth enabled."