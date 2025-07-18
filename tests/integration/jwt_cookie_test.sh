#!/bin/bash

# JWT Cookie Authentication Test
# Tests the JWT middleware's cookie-based authentication method

# Only set strict mode when running as a script, not when sourced
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    set -euo pipefail
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== JWT Cookie Authentication Test ===${NC}"

# Check if server is running
if ! curl -s http://localhost:8080/health > /dev/null; then
    echo -e "${RED}Server is not running on port 8080. Please start the server first.${NC}"
    exit 1
fi

# Source the JWT test helper to reuse token generation logic
if [ -f "scripts/jwt_test_helper.sh" ]; then
    source scripts/jwt_test_helper.sh
else
    echo -e "${RED}JWT test helper script not found at scripts/jwt_test_helper.sh${NC}"
    exit 1
fi

# Generate a test token
echo "1. Generating test token..."
test_token=$(generate_test_jwt "testuser")

if [ -z "$test_token" ]; then
    echo -e "${RED}❌ Failed to generate token${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Token generated successfully${NC}"

# Test with Authorization header (as a reference)
echo "2. Testing Authorization header..."
auth_response=$(curl -s -w "%{http_code}" -o /dev/null -H "Authorization: Bearer $test_token" http://localhost:8080/edit)
if [ "$auth_response" == "200" ] || [ "$auth_response" == "302" ]; then
    echo -e "${GREEN}✅ Authorization header works (HTTP $auth_response)${NC}"
else
    echo -e "${RED}❌ Authorization header failed (HTTP $auth_response)${NC}"
fi

# Test with cookie
echo "3. Testing Cookie header..."
cookie_response=$(curl -s -w "%{http_code}" -o /dev/null -H "Cookie: jwt_token=$test_token" http://localhost:8080/edit)
if [ "$cookie_response" == "200" ] || [ "$cookie_response" == "302" ]; then
    echo -e "${GREEN}✅ Cookie header works (HTTP $cookie_response)${NC}"
else
    echo -e "${RED}❌ Cookie header failed (HTTP $cookie_response)${NC}"
fi

# Test with invalid cookie
echo "4. Testing invalid Cookie..."
invalid_response=$(curl -s -w "%{http_code}" -o /dev/null -H "Cookie: jwt_token=invalid_token_here" http://localhost:8080/edit)
if [ "$invalid_response" == "401" ]; then
    echo -e "${GREEN}✅ Invalid cookie correctly rejected (HTTP 401)${NC}"
else
    echo -e "${RED}❌ Invalid cookie test failed (Expected HTTP 401, got $invalid_response)${NC}"
fi

echo -e "${GREEN}🎉 JWT cookie authentication tests complete!${NC}"
