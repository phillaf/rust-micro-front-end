# Test Suite

This directory contains the test suite for the Rust Micro Front-End application.

## Directory Structure

```text
tests/
├── integration/           # Integration tests
│   └── mysql_integration_test.sh  # MySQL database integration tests
└── README.md             # This file
```

## Integration Tests

### MySQL Integration Test (`mysql_integration_test.sh`)

Tests the complete MySQL integration including:

- Database connection health check
- User data retrieval from MySQL
- User data updates in MySQL
- New user creation
- 404 handling for non-existent users
- Data persistence verification

**Prerequisites:**

- `curl` command available
- `jq` command available for JSON parsing
- Application server running (use `just dev`)

**Usage:**

```bash
# Run all integration tests
just test-integration

# Run MySQL integration tests directly
./tests/integration/mysql_integration_test.sh
```

**Test Output:**

- ✅ Green checkmarks for passing tests
- ❌ Red X marks for failing tests
- Detailed test results summary
- Color-coded log messages for easy reading

## Running Tests

### All Tests

```bash
# Run unit tests
just test

# Run integration tests
just test-integration
```

### Individual Test Suites

```bash
# MySQL integration tests only
./tests/integration/mysql_integration_test.sh
```

## Test Philosophy

The test suite follows these principles:

1. **Containerized Testing**: All tests run against the containerized application
2. **Real Environment Testing**: Integration tests use actual MySQL database
3. **Comprehensive Coverage**: Tests cover happy paths, error cases, and edge cases
4. **Clear Output**: Tests provide detailed, color-coded output for easy debugging
5. **Maintainable**: Tests are organized and documented for long-term maintenance

## Adding New Tests

When adding new integration tests:

1. Create test scripts in the appropriate subdirectory
2. Follow the naming convention: `{feature}_integration_test.sh`
3. Make scripts executable: `chmod +x test_script.sh`
4. Add test commands to the justfile if needed
5. Update this README with new test documentation

## Test Environment

Integration tests expect:

- MySQL database running (`just db-up`)
- Application server running (`just dev`)
- Environment configured with MySQL adapter (`DATABASE_ADAPTER=mysql`)
- Standard test dependencies (`curl`, `jq`)
