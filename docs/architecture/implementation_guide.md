# Cross-App Integration Implementation Guide

This guide provides practical steps for implementing the cross-app integration patterns described in our [Cross App Integration Patterns](./cross_app_integration.md) document.

## Table of Contents

1. [Web Component Implementation](#web-component-implementation)
2. [Cross-App Communication Implementation](#cross-app-communication-implementation)
3. [Authentication Integration](#authentication-integration)
4. [Styling and Theming Implementation](#styling-and-theming-implementation)
5. [Error Handling Implementation](#error-handling-implementation)
6. [Testing Implementation](#testing-implementation)

## Web Component Implementation

### Converting Existing Templates to Web Components

Our existing templates can be converted to web components using the following approach:

```javascript
// display-component.js
class DisplayComponent extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({ mode: 'open' });
    this.render();
  }
  
  static get observedAttributes() {
    return ['username', 'display-name'];
  }
  
  attributeChangedCallback(name, oldValue, newValue) {
    if (oldValue !== newValue) {
      this.render();
    }
  }
  
  async render() {
    const username = this.getAttribute('username');
    let displayName = this.getAttribute('display-name');
    
    if (username && !displayName) {
      // Fetch from API if display-name not provided
      try {
        const response = await fetch(`/api/username/${username}`);
        if (response.ok) {
          const data = await response.json();
          displayName = data.display_name;
        }
      } catch (error) {
        this.handleError(error);
      }
    }
    
    this.shadowRoot.innerHTML = `
      <style>
        /* Component-specific styles */
        :host {
          display: block;
          font-family: var(--micro-frontend-font-family, system-ui, sans-serif);
          color: var(--micro-frontend-text-color, #333);
        }
        .card {
          background-color: var(--micro-frontend-background-color, #fff);
          border-radius: var(--micro-frontend-border-radius, 4px);
          box-shadow: 0 2px 5px rgba(0,0,0,0.1);
          padding: 1.5rem;
        }
        .display-name {
          font-size: 1.5rem;
          margin-top: 1rem;
          color: var(--micro-frontend-primary-color, #0078d7);
        }
        .error {
          color: var(--micro-frontend-error-color, #d32f2f);
          padding: 0.5rem;
          margin: 0.5rem 0;
          background-color: var(--micro-frontend-error-bg, #fdecea);
          border-radius: var(--micro-frontend-border-radius, 4px);
        }
      </style>
      
      <div class="card">
        <h2>Username Display</h2>
        
        ${this.error ? `<div class="error" role="alert">${this.error}</div>` : ''}
        
        ${username ? `
          <div class="info">
            <strong>Username:</strong> <span class="mf-username">${username}</span>
          </div>
          
          ${displayName ? `
            <div class="display-name" id="display-name">${displayName}</div>
          ` : `
            <div class="display-name" id="display-name">No display name set</div>
          `}
        ` : `
          <div class="error" role="alert">No username provided</div>
        `}
      </div>
    `;
  }
  
  handleError(error) {
    this.error = 'Failed to load user data';
    console.error('DisplayComponent error:', error);
    
    const errorEvent = new CustomEvent('micro-frontend-error', {
      bubbles: true,
      composed: true,
      detail: {
        code: 'DISPLAY_FETCH_ERROR',
        message: 'Failed to fetch user display data',
        originalError: error.message
      }
    });
    
    this.dispatchEvent(errorEvent);
  }
}

customElements.define('micro-frontend-display', DisplayComponent);
```

### Registering Components

Components should be registered in a central location:

```javascript
// components/index.js
import './display-component.js';
import './edit-component.js';

// Export a function that verifies all components are registered
export function ensureComponentsRegistered() {
  const components = [
    'micro-frontend-display',
    'micro-frontend-edit'
  ];
  
  const missingComponents = components.filter(
    component => !customElements.get(component)
  );
  
  if (missingComponents.length > 0) {
    console.warn('Missing component registrations:', missingComponents);
    return false;
  }
  
  return true;
}
```

## Cross-App Communication Implementation

### Event Bus Implementation

```javascript
// event-bus.js
export class MicroFrontendEventBus {
  constructor() {
    this.eventTarget = new EventTarget();
    this.broadcastChannel = 'BroadcastChannel' in window ? 
      new BroadcastChannel('micro-frontend-events') : null;
      
    if (this.broadcastChannel) {
      this.broadcastChannel.onmessage = (event) => {
        this.dispatch(event.data.type, event.data.detail);
      };
    }
  }
  
  dispatch(eventName, detail) {
    // Local dispatch
    const event = new CustomEvent(eventName, { 
      bubbles: false, 
      detail 
    });
    this.eventTarget.dispatchEvent(event);
    
    // Cross-window dispatch if available
    if (this.broadcastChannel) {
      this.broadcastChannel.postMessage({
        type: eventName,
        detail
      });
    }
  }
  
  on(eventName, callback) {
    this.eventTarget.addEventListener(eventName, (event) => {
      callback(event.detail);
    });
  }
  
  off(eventName, callback) {
    this.eventTarget.removeEventListener(eventName, callback);
  }
}

// Singleton instance
export const eventBus = new MicroFrontendEventBus();
```

### Component Communication Example

```javascript
// edit-component.js
import { eventBus } from './event-bus.js';

class EditComponent extends HTMLElement {
  // ... constructor and other methods
  
  async handleSubmit(event) {
    event.preventDefault();
    const form = event.target;
    const formData = new FormData(form);
    const displayName = formData.get('display_name');
    
    try {
      const response = await fetch('/api/username', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ display_name: displayName }),
      });
      
      if (response.ok) {
        // Notify other components about the update
        eventBus.dispatch('micro-frontend-user-updated', {
          username: this.getAttribute('username'),
          displayName: displayName
        });
        
        this.showSuccess('Display name updated successfully');
      } else {
        this.handleError(new Error(`Failed to update: ${response.status}`));
      }
    } catch (error) {
      this.handleError(error);
    }
  }
}

customElements.define('micro-frontend-edit', EditComponent);
```

## Authentication Integration

### JWT Token Handling

```javascript
// auth-service.js
export class AuthService {
  constructor() {
    this.tokenKey = 'jwt_token';
    this.listeners = [];
  }
  
  getToken() {
    return localStorage.getItem(this.tokenKey) || this.getCookieToken();
  }
  
  getCookieToken() {
    const cookieValue = document.cookie
      .split('; ')
      .find(row => row.startsWith(`${this.tokenKey}=`))
      ?.split('=')[1];
      
    return cookieValue || null;
  }
  
  setAuthHeader(headers = {}) {
    const token = this.getToken();
    if (token) {
      return {
        ...headers,
        'Authorization': `Bearer ${token}`
      };
    }
    return headers;
  }
  
  isAuthenticated() {
    return !!this.getToken();
  }
  
  requestAuthentication(redirectUrl) {
    const event = new CustomEvent('micro-frontend-auth-required', {
      bubbles: true,
      composed: true,
      detail: { redirectUrl }
    });
    document.dispatchEvent(event);
  }
}

export const authService = new AuthService();
```

## Styling and Theming Implementation

### Design System Integration

```javascript
// theme-provider.js
export class ThemeProvider {
  constructor() {
    this.defaultTheme = {
      'micro-frontend-primary-color': '#0078d7',
      'micro-frontend-background-color': '#ffffff',
      'micro-frontend-text-color': '#333333',
      'micro-frontend-border-radius': '4px',
      'micro-frontend-font-family': 'system-ui, sans-serif',
      'micro-frontend-error-color': '#d32f2f',
      'micro-frontend-error-bg': '#fdecea',
      'micro-frontend-success-color': '#2e7d32',
      'micro-frontend-success-bg': '#edf7ed'
    };
  }
  
  applyTheme(theme = {}) {
    const combinedTheme = {...this.defaultTheme, ...theme};
    const root = document.documentElement;
    
    Object.entries(combinedTheme).forEach(([key, value]) => {
      root.style.setProperty(`--${key}`, value);
    });
  }
}

export const themeProvider = new ThemeProvider();
```

## Error Handling Implementation

### Standardized Error Handler

```javascript
// error-handler.js
export class ErrorHandler {
  constructor() {
    this.errorListeners = new Map();
  }
  
  handleError(component, error, code = 'UNKNOWN_ERROR') {
    console.error(`[${component}] Error:`, error);
    
    // Structured error object
    const errorDetail = {
      component,
      code,
      message: error.message || 'An unknown error occurred',
      timestamp: new Date().toISOString()
    };
    
    // Dispatch to global listeners
    const event = new CustomEvent('micro-frontend-error', {
      bubbles: true,
      composed: true,
      detail: errorDetail
    });
    
    document.dispatchEvent(event);
    
    // Call specific listeners for this error code
    const listeners = this.errorListeners.get(code) || [];
    listeners.forEach(listener => listener(errorDetail));
    
    return errorDetail;
  }
  
  registerErrorListener(code, callback) {
    if (!this.errorListeners.has(code)) {
      this.errorListeners.set(code, []);
    }
    
    this.errorListeners.get(code).push(callback);
  }
}

export const errorHandler = new ErrorHandler();
```

## Testing Implementation

### Integration Test Helpers

```javascript
// test-utils.js
import { JSDOM } from 'jsdom';

export function setupMicroFrontendTest(componentNames = []) {
  // Create DOM environment
  const dom = new JSDOM('<!DOCTYPE html><html><head></head><body></body></html>', {
    url: 'http://localhost',
    runScripts: 'dangerously',
    resources: 'usable'
  });
  
  const window = dom.window;
  const document = window.document;
  
  // Create container
  const container = document.createElement('div');
  container.id = 'test-container';
  document.body.appendChild(container);
  
  // Create component instances
  const components = {};
  
  componentNames.forEach(name => {
    // Create element container
    const componentContainer = document.createElement('div');
    componentContainer.id = `${name}-container`;
    container.appendChild(componentContainer);
    
    // Create actual component
    const component = document.createElement(`micro-frontend-${name}`);
    componentContainer.appendChild(component);
    components[name] = component;
  });
  
  return {
    window,
    document,
    container,
    components,
    
    // Helper to simulate events
    async simulateEvent(componentName, eventName, detail = {}) {
      const component = components[componentName];
      const event = new window.CustomEvent(eventName, {
        bubbles: true,
        composed: true,
        detail
      });
      component.dispatchEvent(event);
      
      // Wait for any async operations
      await new Promise(resolve => setTimeout(resolve, 0));
    },
    
    // Clean up
    cleanup() {
      container.remove();
    }
  };
}
```
