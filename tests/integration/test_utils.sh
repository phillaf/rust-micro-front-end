#!/bin/bash

# Common test utilities for integration tests

# Only set strict mode when running as a script, not when sourced
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    set -euo pipefail
fi

# ANSI color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Utility functions
function check_server_running() {
    # Check if server is running on port 8080
    if ! curl -s -f http://localhost:8080/health >/dev/null; then
        echo -e "${RED}ERROR: Server is not running on port 8080${NC}"
        echo "Please start the server with 'just dev' before running tests"
        exit 1
    fi
}

# Call this at the beginning of test scripts
function initialize_test() {
    check_server_running
    echo -e "${BLUE}Starting test: $1${NC}"
}

# Initialize test environment - runs when this file is sourced
check_server_running
