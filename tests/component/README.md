# Web Component Isolation Testing

This directory contains tests for ensuring proper isolation and independent functionality of the micro front-end components.

## Testing Approach

The component isolation tests verify that:

1. Components can be independently loaded and rendered
2. Components do not have side effects on the parent DOM
3. Components maintain proper styling encapsulation
4. Components can communicate via defined interfaces
5. Components gracefully handle error conditions

## Test Structure

- `display_component_test.js`: Tests for the display component
- `edit_component_test.js`: Tests for the edit/CMS component
- `component_integration_test.js`: Tests for cross-component communication

## Running the Tests

Use the provided justfile command:

```bash
just test-components
```

## Test Environment

Tests run in a headless browser environment with:

- Isolated DOM for each test
- Mocked API responses
- Network request interception
- Performance metrics collection
