#!/bin/bash

# JWT Debug Helper - Check JWT validity and inspect token contents
# Usage: scripts/check_jwt.sh [token]

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to decode base64 URL-safe
b64_decode() {
    local b64="$1"
    # Add padding if needed
    local mod4=$((${#b64} % 4))
    if [ $mod4 -eq 2 ]; then
        b64="${b64}=="
    elif [ $mod4 -eq 3 ]; then
        b64="${b64}="
    fi
    # Replace URL-safe chars and decode
    echo "$b64" | tr -- '-_' '+/' | base64 -d
}

# Function to check if jq is available
check_jq() {
    if ! command -v jq &> /dev/null; then
        echo -e "${YELLOW}Warning: jq not available. JSON will not be pretty-printed.${NC}"
        return 1
    fi
    return 0
}

echo -e "${BLUE}=== JWT Debug Helper ===${NC}"
echo

# Get JWT token - either from parameter, from env var, or from file
JWT_TOKEN="${1:-}"

if [ -z "$JWT_TOKEN" ]; then
    echo -e "${YELLOW}Looking for token in environment or debug file...${NC}"
    JWT_TOKEN="${JWT_TOKEN:-$JWT_DEBUG_TOKEN}"

    if [ -z "$JWT_TOKEN" ]; then
        if [ -f ".jwt_debug_token" ]; then
            JWT_TOKEN=$(cat .jwt_debug_token)
            echo -e "${GREEN}Token loaded from .jwt_debug_token${NC}"
        else
            echo -e "${RED}No JWT token provided. Usage: $0 [token]${NC}"
            echo "You can also set JWT_DEBUG_TOKEN environment variable or create a .jwt_debug_token file"
            exit 1
        fi
    fi
fi

echo -e "${BLUE}Analyzing JWT token:${NC}"
echo "$JWT_TOKEN" | fold -w 80

# Split token into parts
IFS='.' read -r header_b64 payload_b64 signature_b64 <<< "$JWT_TOKEN"

if [ -z "$header_b64" ] || [ -z "$payload_b64" ] || [ -z "$signature_b64" ]; then
    echo -e "${RED}❌ Invalid JWT format. Token should have three parts separated by dots.${NC}"
    exit 1
fi

echo -e "\n${BLUE}=== Header ===${NC}"
header=$(b64_decode "$header_b64")
if check_jq; then
    echo "$header" | jq .
else
    echo "$header"
fi

echo -e "\n${BLUE}=== Payload ===${NC}"
payload=$(b64_decode "$payload_b64")
if check_jq; then
    echo "$payload" | jq .
else
    echo "$payload"
fi

# Extract expiration and other info
exp=$(echo "$payload" | grep -o '"exp":[0-9]*' | cut -d':' -f2)
iat=$(echo "$payload" | grep -o '"iat":[0-9]*' | cut -d':' -f2)
sub=$(echo "$payload" | grep -o '"sub":"[^"]*' | cut -d':' -f2 | tr -d '"')
iss=$(echo "$payload" | grep -o '"iss":"[^"]*' | cut -d':' -f2 | tr -d '"')
aud=$(echo "$payload" | grep -o '"aud":"[^"]*' | cut -d':' -f2 | tr -d '"')

now=$(date +%s)

echo -e "\n${BLUE}=== Token Information ===${NC}"
echo -e "Subject (username): ${GREEN}$sub${NC}"
echo -e "Issuer: $iss"
echo -e "Audience: $aud"
echo -e "Issued at: $(date -d @$iat)"

if [ -n "$exp" ]; then
    remaining=$((exp - now))
    if [ $remaining -gt 0 ]; then
        echo -e "Expires at: ${GREEN}$(date -d @$exp)${NC} (in $remaining seconds)"
    else
        echo -e "Expired at: ${RED}$(date -d @$exp)${NC} ($((remaining * -1)) seconds ago)"
    fi
fi

echo -e "\n${BLUE}=== Debug URLs ===${NC}"
echo -e "${GREEN}http://localhost/debug/validate-token/$JWT_TOKEN${NC}"
echo -e "${GREEN}http://localhost/debug/set-token/$sub?token=$JWT_TOKEN${NC}"

echo -e "\n${BLUE}=== Test Commands ===${NC}"
echo -e "Test edit page: ${YELLOW}curl -v http://localhost/edit -b \"jwt_token=$JWT_TOKEN\"${NC}"
echo -e "Test API endpoint: ${YELLOW}curl -v http://localhost/api/username -H \"Content-Type: application/json\" -H \"Authorization: Bearer $JWT_TOKEN\" -d '{\"display_name\":\"Updated Name\"}'${NC}"

# Save token to file for future use
echo "$JWT_TOKEN" > .jwt_debug_token
echo -e "\n${GREEN}✅ Token saved to .jwt_debug_token for future debugging${NC}"
