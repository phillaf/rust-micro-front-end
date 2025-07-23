#!/bin/bash

# JWT Token Generator for Browser Testing
# Generates a JWT token and provides instructions for browser setup

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default username
DEFAULT_USERNAME="testuser"

# Function to generate a test JWT token (from jwt_test.sh)
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
    local payload='{"sub":"'$username'","iat":'$now',"exp":'$exp',"aud":"micro-frontend-service","iss":"test-auth-service"}'
    
    # Base64URL encode header and payload
    local header_b64=$(echo -n "$header" | base64 -w 0 | tr '+/' '-_' | tr -d '=')
    local payload_b64=$(echo -n "$payload" | base64 -w 0 | tr '+/' '-_' | tr -d '=')
    
    # Create signature input
    local signature_input="${header_b64}.${payload_b64}"
    
    # Create signature using openssl
    local signature=$(echo -n "$signature_input" | openssl dgst -sha256 -sign "$private_key_path" -binary | base64 -w 0 | tr '+/' '-_' | tr -d '=')
    
    # Return the complete JWT token
    echo "${header_b64}.${payload_b64}.${signature}"
}

echo -e "${BLUE}=== JWT Token Generator for Browser Testing ===${NC}"
echo

# Get username from command line or use default
username="${1:-$DEFAULT_USERNAME}"

echo -e "${YELLOW}Generating JWT token for username: ${username}${NC}"

# Generate the token
token=$(generate_test_jwt "$username")

if [ -z "$token" ]; then
    echo -e "${RED}Failed to generate JWT token${NC}"
    exit 1
fi

echo -e "${GREEN}✓ JWT Token generated successfully!${NC}"
echo
echo -e "${YELLOW}Token (copy this):${NC}"
echo "$token"
echo
echo -e "${BLUE}=== Browser Setup Instructions ===${NC}"
echo
echo -e "${YELLOW}Option 1: Automatic Token Injection (Recommended)${NC}"
echo "1. Make sure your dev server is running (just dev)"
echo "2. Open your browser and navigate to this URL:"
echo
echo -e "${GREEN}http://localhost/debug/set-token/$username?token=$token${NC}"
echo
echo "3. You'll see a confirmation page, then navigate to: http://localhost/edit"
echo "4. The token is now automatically injected and ready to use!"
echo
echo -e "${YELLOW}Option 2: Test with curl${NC}"
echo "Test the API directly with curl:"
echo
echo -e "${GREEN}curl -X POST http://localhost/api/username \\"
echo "  -H \"Content-Type: application/json\" \\"
echo "  -H \"Authorization: Bearer $token\" \\"
echo "  -d '{\"username\": \"$username\", \"display_name\": \"Updated Name\"}'${NC}"
echo
echo -e "${YELLOW}Option 3: Copy token to clipboard (if xclip available)${NC}"
if command -v xclip &> /dev/null; then
    echo "$token" | xclip -selection clipboard
    echo -e "${GREEN}✓ Token copied to clipboard!${NC}"
else
    echo -e "${RED}xclip not available - copy token manually${NC}"
fi
echo
echo -e "${BLUE}=== Token Information ===${NC}"
echo "Username: $username"
echo "Expires: $(date -d @$(($(date +%s) + 3600)))"
echo "Algorithm: RS256"
echo
echo -e "${YELLOW}Note: This token is valid for 1 hour${NC}"
echo -e "${YELLOW}To generate a token for a different user: $0 <username>${NC}"
