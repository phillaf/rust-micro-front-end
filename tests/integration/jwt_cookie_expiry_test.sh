#!/bin/bash
# JWT Cookie Expiration Test
# Tests that the cookie expiration matches the JWT token expiration

# Import common test utilities
source "$(dirname "$0")/test_utils.sh"

# Setup
SERVER_URL="http://localhost:8080"
TEST_USER="testuser"

echo -e "${YELLOW}=== JWT Cookie Expiration Test ===${NC}"

# 1. Get a JWT token for test user
echo "1. Getting JWT token for user: $TEST_USER"
response=$(curl -s -c cookies.txt "${SERVER_URL}/debug/set-token/${TEST_USER}")

# Verify only one cookie is set
cookie_count=$(grep -c jwt_token cookies.txt)
if [ "$cookie_count" -eq 1 ]; then
    echo -e "${GREEN}✅ Single cookie verified${NC}"
else
    echo -e "${RED}❌ Expected 1 cookie, found $cookie_count${NC}"
    grep jwt_token cookies.txt
fi

# Extract the token from the cookie file
token=$(grep jwt_token cookies.txt | awk '{print $7}')
if [ -z "$token" ]; then
    echo -e "${RED}❌ Failed to get JWT token${NC}"
    exit 1
else
    echo -e "${GREEN}✅ Got JWT token${NC}"
fi

# 2. Check the token's expiration time
echo "2. Checking token expiration time..."
token_parts=($(echo $token | tr "." "\n"))
if [ ${#token_parts[@]} -lt 2 ]; then
    echo -e "${RED}❌ Invalid JWT format${NC}"
    exit 1
fi

# Decode the JWT payload
# Note: We add padding '=' characters as needed for base64 decoding
jwt_payload=$(echo ${token_parts[1]} | tr -d '\n' | base64 -d 2>/dev/null || (padding=$(( 4 - (${#token_parts[1]} % 4) )); echo ${token_parts[1]}$(printf "%*s" $padding | tr ' ' '=') | base64 -d))

# Extract expiration time using grep and sed
exp_time=$(echo $jwt_payload | grep -o '"exp":[0-9]*' | sed 's/"exp"://')
if [ -z "$exp_time" ]; then
    echo -e "${RED}❌ Could not extract expiration time from JWT${NC}"
    exit 1
fi

# 3. Check the cookie expiration (Max-Age attribute)
echo "3. Checking cookie Max-Age attribute..."
cookie_header=$(curl -s -I "${SERVER_URL}/debug/set-token/${TEST_USER}" | grep -i "Set-Cookie")
max_age=$(echo "$cookie_header" | grep -o "Max-Age=[0-9]*" | head -1 | sed 's/Max-Age=//')

if [ -z "$max_age" ]; then
    echo -e "${RED}❌ Could not find Max-Age in cookie${NC}"
    exit 1
fi

# 4. Calculate time difference between token and cookie expiration
echo "4. Comparing expirations..."
current_time=$(date +%s)
token_remaining=$((exp_time - current_time))
max_age_int=$((max_age))

echo "   Token expiration in: $token_remaining seconds"
echo "   Cookie Max-Age: $max_age_int seconds"

# Allow a small difference (5 seconds) to account for time between requests
if [ $((token_remaining - max_age_int)) -lt 5 ] && [ $((token_remaining - max_age_int)) -gt -5 ]; then
    echo -e "${GREEN}✅ Token and cookie expirations match within 5 seconds${NC}"
else
    echo -e "${RED}❌ Token and cookie expirations do not match${NC}"
    echo "   Difference: $((token_remaining - max_age_int)) seconds"
    exit 1
fi

# Cleanup
rm -f cookies.txt

echo -e "${GREEN}✅ All tests passed${NC}"
