# Cross-App Integration Patterns

## Overview

This document outlines the patterns, best practices, and implementation guidelines for integrating our Rust Micro Front-End components with other micro-apps in our ecosystem. These patterns ensure consistent behavior, performance, and security across all integrated applications.

## Table of Contents

1. [Integration Strategies](#integration-strategies)
2. [Communication Patterns](#communication-patterns)
3. [Styling and Theming](#styling-and-theming)
4. [State Management](#state-management)
5. [Authentication and Authorization](#authentication-and-authorization)
6. [Error Handling](#error-handling)
7. [Performance Considerations](#performance-considerations)
8. [Implementation Examples](#implementation-examples)
9. [Testing Cross-App Integration](#testing-cross-app-integration)

## Integration Strategies

### Web Component Embedding

Our micro front-ends are delivered as self-contained web components that can be embedded in any application.

```html
<!-- Example of embedding our display component in another app -->
<div id="user-display-container">
  <micro-frontend-display username="current-user"></micro-frontend-display>
</div>
```

**Implementation Requirements:**

1. Components must be fully self-contained with no external dependencies
2. Shadow DOM should be used for style isolation
3. All components must expose a clear public API through attributes/properties
4. Components must clean up all resources when disconnected

### Server-Side Includes

For applications that cannot use client-side web components, we provide server-side includes:

```html
<!-- Using Edge-Side Includes (ESI) -->
<esi:include src="https://micro-frontend-api.example.com/component/display?username=test" />
```

## Communication Patterns

### Event-Based Communication

Components communicate with other applications through custom events:

```javascript
// Broadcasting a change
const event = new CustomEvent('micro-frontend-user-updated', {
  bubbles: true,
  composed: true,
  detail: { username: 'updated-username', displayName: 'New Display Name' }
});
this.dispatchEvent(event);

// Listening in another application
document.addEventListener('micro-frontend-user-updated', (e) => {
  console.log('User updated:', e.detail);
});
```

**Standard Events:**

| Event Name | Description | Payload |
|------------|-------------|---------|
| `micro-frontend-user-updated` | User details have been changed | `{username: string, displayName: string}` |
| `micro-frontend-auth-required` | Authentication needed | `{redirectUrl?: string}` |
| `micro-frontend-error` | Error occurred in component | `{code: string, message: string}` |

### Direct API Communication

For more complex operations, components can communicate with other apps via exposed methods:

```javascript
// Exposing methods on the web component
class MicroFrontendDisplay extends HTMLElement {
  refreshData() {
    // Implementation
  }
}

// Another app calling the method
document.querySelector('micro-frontend-display').refreshData();
```

## Styling and Theming

### CSS Custom Properties

Components expose CSS custom properties for theming:

```css
/* Default theme values in component */
:host {
  --micro-frontend-primary-color: #0078d7;
  --micro-frontend-background-color: #ffffff;
  --micro-frontend-text-color: #333333;
  --micro-frontend-border-radius: 4px;
}

/* Customization in host application */
micro-frontend-display {
  --micro-frontend-primary-color: #00a651;
}
```

### Style Hooks

Critical elements have CSS classes for styling hooks:

```css
/* Host app styling the component */
micro-frontend-display .mf-username {
  font-weight: bold;
}
```

## State Management

### Isolated State

Each component maintains its own state and only exposes what's necessary via events and API.

### Cross-App State

For state that needs to be shared across applications:

1. **URL Parameters** - For shareable, bookmarkable state
2. **LocalStorage** - For persistence across sessions in the same origin
3. **BroadcastChannel API** - For real-time state sharing between tabs/windows

## Authentication and Authorization

### JWT Authentication

Components use JWT tokens for authentication, which can be provided in multiple ways:

1. **Authorization Header** - For API requests
2. **Cookie-Based** - For seamless SSR experiences
3. **Component Attribute** - For direct component injection

### Token Management

Components do not generate tokens but can request authentication via events.

## Error Handling

### Graceful Degradation

Components implement graceful degradation when services are unavailable:

1. Fallback to cached data when possible
2. Clear error states with actionable messages
3. Automatic retry with exponential backoff

### Error Events

Standard error events allow host applications to handle errors consistently.

## Performance Considerations

### Lazy Loading

Components support lazy loading to improve initial page load:

```html
<!-- Example of lazy loading a component -->
<script type="module" src="/components/micro-frontend-display.js" defer></script>
```

### Resource Preloading

Critical resources are declaratively preloaded:

```html
<link rel="preload" href="/components/micro-frontend-display.js" as="script">
```

## Implementation Examples

### Basic Component Integration

```javascript
// Initialize and mount the component
const container = document.getElementById('user-display');
const component = document.createElement('micro-frontend-display');
component.setAttribute('username', 'current-user');
container.appendChild(component);

// Listen for events
component.addEventListener('micro-frontend-user-updated', (e) => {
  console.log('User updated:', e.detail);
});
```

### Server-Side Rendering with Hydration

```html
<!-- Server renders initial state -->
<micro-frontend-display 
  username="server-user" 
  display-name="Server Rendered"
  data-hydrate="true">
  <!-- Fallback content until JS loads -->
  <div class="loading-placeholder">Loading user display...</div>
</micro-frontend-display>

<script type="module">
  // Client hydrates the component without re-rendering
  import '/components/micro-frontend-display.js';
</script>
```

## Testing Cross-App Integration

### Integration Tests

We provide integration test helpers to verify cross-app communication:

```javascript
// In your test suite
import { setupMicroFrontendTest } from '@micro-frontend/test-utils';

describe('Cross-app integration', () => {
  let { container, components } = setupMicroFrontendTest(['display', 'edit']);
  
  test('Edit component updates display component', async () => {
    // Test implementation
  });
});
```

### Contract Tests

Contract tests ensure API compatibility between versions:

```javascript
// In contract test
test('Component adheres to event contract', () => {
  const component = document.createElement('micro-frontend-display');
  const eventSpy = jest.fn();
  
  document.addEventListener('micro-frontend-user-updated', eventSpy);
  component.setAttribute('username', 'new-user');
  
  expect(eventSpy).toHaveBeenCalledWith(
    expect.objectContaining({
      detail: expect.objectContaining({
        username: 'new-user'
      })
    })
  );
});
```
