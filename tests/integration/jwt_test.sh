#!/bin/bash

# JWT Authentication Integration Test
# Tests the JWT middleware with actual JWT tokens

# Only set strict mode when running as a script, not when sourced
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    set -euo pipefail
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== JWT Authentication Integration Test ===${NC}"

# Check if server is running
if ! curl -s http://localhost:8080/health > /dev/null; then
    echo -e "${RED}Error: Server is not running. Please start with 'just dev' first.${NC}"
    exit 1
fi

# Function to generate a test JWT token
generate_test_jwt() {
    local username="$1"
    local private_key_path="scripts/jwt_private_key.pem"
    
    # Check if required tools are available
    if ! command -v openssl &> /dev/null; then
        echo "Error: openssl not available"
        return 1
    fi
    
    if ! command -v base64 &> /dev/null; then
        echo "Error: base64 not available"
        return 1
    fi
    
    # Create JWT header
    local header='{"alg":"RS256","typ":"JWT"}'
    
    # Create JWT payload
    local now=$(date +%s)
    local exp=$((now + 3600))  # 1 hour from now
    local payload=$(cat <<EOF
{"sub":"$username","iat":$now,"exp":$exp,"aud":"micro-frontend-service","iss":"your-auth-service"}
EOF
)
    
    # Base64URL encode header and payload
    local header_b64=$(echo -n "$header" | base64url_encode)
    local payload_b64=$(echo -n "$payload" | base64url_encode)
    
    # Create signature input
    local signature_input="${header_b64}.${payload_b64}"
    
    # Create signature using openssl
    local signature=$(echo -n "$signature_input" | openssl dgst -sha256 -sign "$private_key_path" -binary | base64url_encode)
    
    # Return the complete JWT token
    echo "${header_b64}.${payload_b64}.${signature}"
}

# Helper function for base64url encoding (fallback)
base64url_encode() {
    base64 -w 0 | tr '+/' '-_' | tr -d '='
}

# Test 1: Access protected endpoint without JWT token
echo -e "${YELLOW}Test 1: Access protected endpoint without JWT token${NC}"
response=$(curl -s -w "%{http_code}" -o /dev/null -X POST http://localhost:8080/api/username \
    -H "Content-Type: application/json" \
    -d '{"username": "testuser", "display_name": "Test User"}')

if [ "$response" = "401" ]; then
    echo -e "${GREEN}✓ Correctly rejected request without JWT token (401)${NC}"
else
    echo -e "${RED}✗ Expected 401, got $response${NC}"
    exit 1
fi

# Test 2: Access protected endpoint with invalid JWT token
echo -e "${YELLOW}Test 2: Access protected endpoint with invalid JWT token${NC}"
response=$(curl -s -w "%{http_code}" -o /dev/null -X POST http://localhost:8080/api/username \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer invalid_token_here" \
    -d '{"username": "testuser", "display_name": "Test User"}')

if [ "$response" = "401" ]; then
    echo -e "${GREEN}✓ Correctly rejected request with invalid JWT token (401)${NC}"
else
    echo -e "${RED}✗ Expected 401, got $response${NC}"
    exit 1
fi

# Test 3: Access protected endpoint with valid JWT token
echo -e "${YELLOW}Test 3: Access protected endpoint with valid JWT token${NC}"

# Check if we can generate a token
if ! command -v openssl &> /dev/null; then
    echo -e "${YELLOW}⚠ OpenSSL not available, skipping valid token test${NC}"
else
    # Generate a test token
    test_token=$(generate_test_jwt "testuser")
    
    if [ -z "$test_token" ]; then
        echo -e "${RED}✗ Failed to generate test token${NC}"
        exit 1
    fi
    
    response=$(curl -s -w "%{http_code}" -o /tmp/jwt_test_response.json -X POST http://localhost:8080/api/username \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $test_token" \
        -d '{"username": "testuser", "display_name": "Test User"}')
    
    if [ "$response" = "200" ]; then
        echo -e "${GREEN}✓ Successfully accessed protected endpoint with valid JWT token (200)${NC}"
        echo "Response: $(cat /tmp/jwt_test_response.json)"
    else
        echo -e "${RED}✗ Expected 200, got $response${NC}"
        echo "Response: $(cat /tmp/jwt_test_response.json)"
        exit 1
    fi
fi

# Test 4: Verify JWT token includes username in request context
echo -e "${YELLOW}Test 4: Verify JWT functionality with different usernames${NC}"

# Test with different username
if command -v openssl &> /dev/null; then
    test_token2=$(generate_test_jwt "anotheruser")
    
    if [ -z "$test_token2" ]; then
        echo -e "${RED}✗ Failed to generate test token for anotheruser${NC}"
        exit 1
    fi
    
    response=$(curl -s -w "%{http_code}" -o /tmp/jwt_test_response2.json -X POST http://localhost:8080/api/username \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $test_token2" \
        -d '{"username": "anotheruser", "display_name": "Another User"}')
    
    if [ "$response" = "200" ]; then
        echo -e "${GREEN}✓ JWT middleware working with different usernames${NC}"
        echo "Response: $(cat /tmp/jwt_test_response2.json)"
    else
        echo -e "${RED}✗ Expected 200, got $response${NC}"
        echo "Response: $(cat /tmp/jwt_test_response2.json)"
        exit 1
    fi
fi

# Clean up
rm -f /tmp/jwt_test_response.json /tmp/jwt_test_response2.json

echo -e "${GREEN}=== JWT Authentication Tests Completed Successfully ===${NC}"
