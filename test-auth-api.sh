#!/bin/bash
set -e

# Colors for prettier output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

BASE_URL="http://localhost:8085/api/auth"
TOKEN_FILE=".auth_tokens.json"
USER_ID=""

echo -e "${BLUE}=== OxiCloud Authentication Test Script ===${NC}"
echo -e "${BLUE}This script will test the authentication endpoints${NC}"
echo

cleanup() {
  echo -e "\n${BLUE}Cleaning up test files...${NC}"
  rm -f "$TOKEN_FILE"
  echo "Done."
}

trap cleanup EXIT

# Function to check if server is running
check_server() {
  echo -e "${BLUE}Checking if OxiCloud server is running...${NC}"
  if ! curl -s "http://localhost:8085/api/health" > /dev/null; then
    echo -e "${RED}Error: Server is not running. Please start the server first with 'cargo run'${NC}"
    exit 1
  fi
  echo -e "${GREEN}Server is running!${NC}"
}

# 1. Test registration
test_registration() {
  echo -e "\n${BLUE}1. Testing user registration...${NC}"
  
  USERNAME="testuser"
  EMAIL="test@example.com"
  PASSWORD="Test123!"
  
  RESPONSE=$(curl -s -X POST "$BASE_URL/register" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"$USERNAME\",\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}")
  
  # Check if registration was successful
  if [[ "$RESPONSE" == *"userId"* ]]; then
    echo -e "${GREEN}✓ Registration successful${NC}"
    USER_ID=$(echo $RESPONSE | jq -r '.userId')
    echo "User created with ID: $USER_ID"
  else
    echo -e "${RED}✗ Registration failed${NC}"
    echo "$RESPONSE"
    exit 1
  fi
}

# 2. Test login
test_login() {
  echo -e "\n${BLUE}2. Testing user login...${NC}"
  
  USERNAME="testuser"
  PASSWORD="Test123!"
  
  RESPONSE=$(curl -s -X POST "$BASE_URL/login" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}")
  
  # Check if login was successful
  if [[ "$RESPONSE" == *"accessToken"* ]]; then
    echo -e "${GREEN}✓ Login successful${NC}"
    # Save tokens to file for future requests
    echo "$RESPONSE" > "$TOKEN_FILE"
    # Extract token for logging
    ACCESS_TOKEN=$(echo "$RESPONSE" | jq -r '.accessToken')
    echo "Access token: ${ACCESS_TOKEN:0:20}...${ACCESS_TOKEN: -10}"
  else
    echo -e "${RED}✗ Login failed${NC}"
    echo "$RESPONSE"
    exit 1
  fi
}

# 3. Test getting current user
test_get_user() {
  echo -e "\n${BLUE}3. Testing get current user...${NC}"
  
  if [ ! -f "$TOKEN_FILE" ]; then
    echo -e "${RED}✗ No authentication token found. Login first.${NC}"
    exit 1
  fi
  
  ACCESS_TOKEN=$(jq -r '.accessToken' "$TOKEN_FILE")
  
  RESPONSE=$(curl -s -X GET "$BASE_URL/me" \
    -H "Authorization: Bearer $ACCESS_TOKEN")
  
  # Check if getting user was successful
  if [[ "$RESPONSE" == *"username"* ]]; then
    echo -e "${GREEN}✓ Got user details successfully${NC}"
    echo "Username: $(echo "$RESPONSE" | jq -r '.username')"
    echo "Email: $(echo "$RESPONSE" | jq -r '.email')"
    echo "Role: $(echo "$RESPONSE" | jq -r '.role')"
  else
    echo -e "${RED}✗ Getting user details failed${NC}"
    echo "$RESPONSE"
    exit 1
  fi
}

# 4. Test token refresh
test_refresh_token() {
  echo -e "\n${BLUE}4. Testing token refresh...${NC}"
  
  if [ ! -f "$TOKEN_FILE" ]; then
    echo -e "${RED}✗ No authentication token found. Login first.${NC}"
    exit 1
  fi
  
  REFRESH_TOKEN=$(jq -r '.refreshToken' "$TOKEN_FILE")
  
  RESPONSE=$(curl -s -X POST "$BASE_URL/refresh" \
    -H "Content-Type: application/json" \
    -d "{\"refreshToken\":\"$REFRESH_TOKEN\"}")
  
  # Check if refresh was successful
  if [[ "$RESPONSE" == *"accessToken"* ]]; then
    echo -e "${GREEN}✓ Token refresh successful${NC}"
    # Update tokens
    echo "$RESPONSE" > "$TOKEN_FILE"
    ACCESS_TOKEN=$(echo "$RESPONSE" | jq -r '.accessToken')
    echo "New access token: ${ACCESS_TOKEN:0:20}...${ACCESS_TOKEN: -10}"
  else
    echo -e "${RED}✗ Token refresh failed${NC}"
    echo "$RESPONSE"
    exit 1
  fi
}

# 5. Test change password
test_change_password() {
  echo -e "\n${BLUE}5. Testing password change...${NC}"
  
  if [ ! -f "$TOKEN_FILE" ]; then
    echo -e "${RED}✗ No authentication token found. Login first.${NC}"
    exit 1
  fi
  
  ACCESS_TOKEN=$(jq -r '.accessToken' "$TOKEN_FILE")
  OLD_PASSWORD="Test123!"
  NEW_PASSWORD="NewTest456!"
  
  RESPONSE=$(curl -s -X PUT "$BASE_URL/change-password" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $ACCESS_TOKEN" \
    -d "{\"oldPassword\":\"$OLD_PASSWORD\",\"newPassword\":\"$NEW_PASSWORD\"}")
  
  # Check response code
  if [ -z "$RESPONSE" ]; then
    echo -e "${GREEN}✓ Password changed successfully${NC}"
    
    # Test login with new password
    echo -e "${BLUE}   Testing login with new password...${NC}"
    LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/login" \
      -H "Content-Type: application/json" \
      -d "{\"username\":\"testuser\",\"password\":\"$NEW_PASSWORD\"}")
    
    if [[ "$LOGIN_RESPONSE" == *"accessToken"* ]]; then
      echo -e "${GREEN}   ✓ Login with new password successful${NC}"
      echo "$LOGIN_RESPONSE" > "$TOKEN_FILE"
    else
      echo -e "${RED}   ✗ Login with new password failed${NC}"
      echo "$LOGIN_RESPONSE"
    fi
  else
    echo -e "${RED}✗ Password change failed${NC}"
    echo "$RESPONSE"
  fi
}

# 6. Test logout
test_logout() {
  echo -e "\n${BLUE}6. Testing logout...${NC}"
  
  if [ ! -f "$TOKEN_FILE" ]; then
    echo -e "${RED}✗ No authentication token found. Login first.${NC}"
    exit 1
  fi
  
  ACCESS_TOKEN=$(jq -r '.accessToken' "$TOKEN_FILE")
  REFRESH_TOKEN=$(jq -r '.refreshToken' "$TOKEN_FILE")
  
  RESPONSE=$(curl -s -X POST "$BASE_URL/logout" \
    -H "Authorization: Bearer $REFRESH_TOKEN")
  
  # Check response
  if [ -z "$RESPONSE" ]; then
    echo -e "${GREEN}✓ Logout successful${NC}"
    
    # Verify token is invalidated by trying to use it
    echo -e "${BLUE}   Verifying token invalidation...${NC}"
    VERIFY_RESPONSE=$(curl -s -X GET "$BASE_URL/me" \
      -H "Authorization: Bearer $ACCESS_TOKEN")
    
    if [[ "$VERIFY_RESPONSE" == *"error"* ]]; then
      echo -e "${GREEN}   ✓ Token successfully invalidated${NC}"
    else
      echo -e "${RED}   ✗ Token still valid after logout${NC}"
      echo "$VERIFY_RESPONSE"
    fi
  else
    echo -e "${RED}✗ Logout failed${NC}"
    echo "$RESPONSE"
  fi
}

# 7. Test protected resource access
test_protected_resource() {
  echo -e "\n${BLUE}7. Testing protected resource access...${NC}"
  
  # Login first to get a fresh token
  USERNAME="testuser"
  PASSWORD="NewTest456!"  # Use the new password
  
  LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/login" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}")
  
  if [[ "$LOGIN_RESPONSE" == *"accessToken"* ]]; then
    echo "$LOGIN_RESPONSE" > "$TOKEN_FILE"
    ACCESS_TOKEN=$(jq -r '.accessToken' "$TOKEN_FILE")
    
    echo -e "${BLUE}   Accessing a protected resource (folders list)...${NC}"
    RESOURCE_RESPONSE=$(curl -s -X GET "http://localhost:8085/api/folders" \
      -H "Authorization: Bearer $ACCESS_TOKEN")
    
    if [[ "$RESOURCE_RESPONSE" != *"error"* ]]; then
      echo -e "${GREEN}   ✓ Successfully accessed protected resource${NC}"
    else
      echo -e "${RED}   ✗ Failed to access protected resource${NC}"
      echo "$RESOURCE_RESPONSE"
    fi
  else
    echo -e "${RED}✗ Login for resource test failed${NC}"
    echo "$LOGIN_RESPONSE"
  fi
}

# Main test execution
check_server
test_registration
test_login
test_get_user
test_refresh_token
test_change_password
test_logout
test_protected_resource

echo -e "\n${GREEN}All authentication tests completed successfully!${NC}"
echo -e "${BLUE}Your authentication system appears to be working correctly.${NC}"