import { eventBus } from './event-bus.js';

/**
 * Display Component Web Component
 * 
 * Attributes:
 * - username: The username to display
 * - display-name: (optional) The display name to show (if not provided, will be fetched from API)
 * - theme: (optional) Theme name to apply
 * - auto-refresh: (optional) Whether to automatically refresh data on user updates
 */
class DisplayComponent extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({ mode: 'open' });
    this.error = null;
    
    // Bind event handlers
    this._handleUserUpdated = this._handleUserUpdated.bind(this);
  }
  
  static get observedAttributes() {
    return ['username', 'display-name', 'theme', 'auto-refresh'];
  }
  
  connectedCallback() {
    this.render();
    
    // Listen for user updates if auto-refresh is enabled
    if (this.hasAttribute('auto-refresh')) {
      eventBus.on('micro-frontend-user-updated', this._handleUserUpdated);
      
      // Also listen for the DOM event for backward compatibility
      document.addEventListener('micro-frontend-user-updated', (e) => {
        this._handleUserUpdated(e.detail);
      });
    }
  }
  
  disconnectedCallback() {
    // Clean up event listeners
    eventBus.off('micro-frontend-user-updated', this._handleUserUpdated);
    document.removeEventListener('micro-frontend-user-updated', this._handleUserUpdated);
  }
  
  attributeChangedCallback(name, oldValue, newValue) {
    if (oldValue !== newValue) {
      this.render();
    }
  }
  
  _handleUserUpdated(detail) {
    // Only update if this is for our username
    const currentUsername = this.getAttribute('username');
    if (detail.username === currentUsername) {
      this.setAttribute('display-name', detail.displayName);
    }
  }
  
  async _fetchUserData(username) {
    try {
      const response = await fetch(`/api/username/${username}`);
      if (response.ok) {
        const data = await response.json();
        return data;
      } else {
        throw new Error(`Failed to fetch user data: ${response.status}`);
      }
    } catch (error) {
      this._handleError(error);
      return null;
    }
  }
  
  _handleError(error) {
    this.error = error.message || 'An error occurred';
    this.render();
    
    // Dispatch error event
    const errorEvent = new CustomEvent('micro-frontend-error', {
      bubbles: true,
      composed: true,
      detail: {
        component: 'display',
        message: this.error,
        code: 'DISPLAY_ERROR',
      }
    });
    this.dispatchEvent(errorEvent);
  }
  
  async render() {
    const username = this.getAttribute('username');
    let displayName = this.getAttribute('display-name');
    
    // Fetch data if we have a username but no display name
    if (username && !displayName) {
      const userData = await this._fetchUserData(username);
      if (userData) {
        displayName = userData.display_name;
      }
    }
    
    // Apply theme
    const theme = this.getAttribute('theme') || 'default';
    const themeClass = `theme-${theme}`;
    
    this.shadowRoot.innerHTML = `
      <style>
        :host {
          display: block;
          font-family: var(--micro-frontend-font-family, system-ui, sans-serif);
        }
        
        .card {
          background-color: var(--micro-frontend-background-color, #ffffff);
          border-radius: var(--micro-frontend-border-radius, 4px);
          box-shadow: 0 2px 5px rgba(0,0,0,0.1);
          padding: 1.5rem;
          margin: 1rem 0;
        }
        
        .theme-default {
          --micro-frontend-primary-color: #0078d7;
          --micro-frontend-text-color: #333333;
        }
        
        .theme-dark {
          --micro-frontend-primary-color: #4cc2ff;
          --micro-frontend-background-color: #2d2d2d;
          --micro-frontend-text-color: #e0e0e0;
        }
        
        .theme-high-contrast {
          --micro-frontend-primary-color: #ffff00;
          --micro-frontend-background-color: #000000;
          --micro-frontend-text-color: #ffffff;
        }
        
        h2 {
          margin-top: 0;
          color: var(--micro-frontend-text-color, #333333);
        }
        
        .display-name {
          font-size: 1.5rem;
          margin-top: 1rem;
          color: var(--micro-frontend-primary-color, #0078d7);
          word-break: break-word;
        }
        
        .error {
          color: var(--micro-frontend-error-color, #d32f2f);
          padding: 0.5rem;
          margin: 0.5rem 0;
          background-color: var(--micro-frontend-error-bg, #fdecea);
          border-radius: var(--micro-frontend-border-radius, 4px);
        }
        
        .info {
          margin-bottom: 0.5rem;
        }
        
        .mf-username {
          font-weight: bold;
        }
      </style>
      
      <div class="card ${themeClass}">
        <h2>Username Display</h2>
        
        ${this.error ? `<div class="error" role="alert">${this.error}</div>` : ''}
        
        ${username ? `
          <div class="info">
            <strong>Username:</strong> <span class="mf-username">${this._escapeHtml(username)}</span>
          </div>
          
          ${displayName ? `
            <div class="display-name" id="display-name">${this._escapeHtml(displayName)}</div>
          ` : `
            <div class="display-name" id="display-name">No display name set</div>
          `}
        ` : `
          <div class="error" role="alert">No username provided</div>
        `}
      </div>
    `;
  }
  
  // Helper method to prevent XSS attacks
  _escapeHtml(unsafe) {
    return unsafe
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#039;");
  }
  
  // Public API methods
  refresh() {
    const username = this.getAttribute('username');
    if (username) {
      // Clear the display name to force a refresh
      this.removeAttribute('display-name');
      this.render();
    }
  }
}

// Define the custom element
customElements.define('micro-frontend-display', DisplayComponent);
