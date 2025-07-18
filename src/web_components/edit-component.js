import { eventBus } from './event-bus.js';

/**
 * Edit Component Web Component
 * 
 * Attributes:
 * - username: The username to edit
 * - theme: (optional) Theme name to apply
 * - return-url: (optional) URL to redirect after successful edit
 */
class EditComponent extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({ mode: 'open' });
    this.error = null;
    this.success = null;
  }
  
  static get observedAttributes() {
    return ['username', 'theme', 'return-url'];
  }
  
  connectedCallback() {
    this.render();
    
    // Add event listeners
    const form = this.shadowRoot.querySelector('form');
    if (form) {
      form.addEventListener('submit', this._handleSubmit.bind(this));
    }
  }
  
  disconnectedCallback() {
    // Clean up event listeners
    const form = this.shadowRoot.querySelector('form');
    if (form) {
      form.removeEventListener('submit', this._handleSubmit);
    }
  }
  
  attributeChangedCallback(name, oldValue, newValue) {
    if (oldValue !== newValue) {
      this.render();
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
  
  async _handleSubmit(event) {
    event.preventDefault();
    
    // Clear previous messages
    this.error = null;
    this.success = null;
    
    const form = event.target;
    const formData = new FormData(form);
    const displayName = formData.get('display_name');
    const username = this.getAttribute('username');
    
    if (!username) {
      this._handleError(new Error('No username provided for update'));
      return;
    }
    
    try {
      const response = await fetch('/api/username', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'application/json'
        },
        body: JSON.stringify({ 
          username: username,
          display_name: displayName 
        }),
        credentials: 'same-origin' // Include cookies
      });
      
      if (response.ok) {
        const data = await response.json();
        this.success = data.message || 'Display name updated successfully';
        
        // Notify other components about the update
        eventBus.dispatch('micro-frontend-user-updated', {
          username: username,
          displayName: displayName
        });
        
        // Also dispatch as a DOM event for backward compatibility
        const updateEvent = new CustomEvent('micro-frontend-user-updated', {
          bubbles: true,
          composed: true,
          detail: {
            username: username,
            displayName: displayName
          }
        });
        this.dispatchEvent(updateEvent);
        
        // Redirect if return URL is provided
        const returnUrl = this.getAttribute('return-url');
        if (returnUrl) {
          window.location.href = returnUrl;
        } else {
          this.render();
        }
      } else {
        const errorData = await response.json().catch(() => ({}));
        this._handleError(new Error(errorData.message || `Failed to update: ${response.status}`));
      }
    } catch (error) {
      this._handleError(error);
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
        component: 'edit',
        message: this.error,
        code: 'EDIT_ERROR',
      }
    });
    this.dispatchEvent(errorEvent);
  }
  
  async render() {
    const username = this.getAttribute('username');
    let displayName = '';
    
    // Fetch current display name if we have a username
    if (username) {
      const userData = await this._fetchUserData(username);
      if (userData) {
        displayName = userData.display_name || '';
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
          --micro-frontend-button-color: #0078d7;
          --micro-frontend-button-text: #ffffff;
        }
        
        .theme-dark {
          --micro-frontend-primary-color: #4cc2ff;
          --micro-frontend-background-color: #2d2d2d;
          --micro-frontend-text-color: #e0e0e0;
          --micro-frontend-button-color: #4cc2ff;
          --micro-frontend-button-text: #000000;
        }
        
        .theme-high-contrast {
          --micro-frontend-primary-color: #ffff00;
          --micro-frontend-background-color: #000000;
          --micro-frontend-text-color: #ffffff;
          --micro-frontend-button-color: #ffff00;
          --micro-frontend-button-text: #000000;
        }
        
        h2 {
          margin-top: 0;
          color: var(--micro-frontend-text-color, #333333);
        }
        
        .form-group {
          margin-bottom: 1rem;
        }
        
        label {
          display: block;
          margin-bottom: 0.5rem;
          font-weight: bold;
          color: var(--micro-frontend-text-color, #333333);
        }
        
        input {
          width: 100%;
          padding: 0.5rem;
          font-size: 1rem;
          border: 1px solid #ccc;
          border-radius: var(--micro-frontend-border-radius, 4px);
          box-sizing: border-box;
        }
        
        button {
          background-color: var(--micro-frontend-button-color, #0078d7);
          color: var(--micro-frontend-button-text, #ffffff);
          border: none;
          border-radius: var(--micro-frontend-border-radius, 4px);
          padding: 0.5rem 1rem;
          font-size: 1rem;
          cursor: pointer;
          transition: background-color 0.3s;
        }
        
        button:hover {
          opacity: 0.9;
        }
        
        .error {
          color: var(--micro-frontend-error-color, #d32f2f);
          padding: 0.5rem;
          margin: 0.5rem 0;
          background-color: var(--micro-frontend-error-bg, #fdecea);
          border-radius: var(--micro-frontend-border-radius, 4px);
        }
        
        .success {
          color: var(--micro-frontend-success-color, #2e7d32);
          padding: 0.5rem;
          margin: 0.5rem 0;
          background-color: var(--micro-frontend-success-bg, #edf7ed);
          border-radius: var(--micro-frontend-border-radius, 4px);
        }
        
        .info {
          margin-bottom: 0.5rem;
        }
        
        .form-help {
          display: block;
          font-size: 0.8rem;
          color: #666;
          margin-top: 0.25rem;
        }
      </style>
      
      <div class="card ${themeClass}">
        <h2>Edit Display Name</h2>
        
        ${this.error ? `<div class="error" role="alert">${this.error}</div>` : ''}
        ${this.success ? `<div class="success" role="alert">${this.success}</div>` : ''}
        
        ${username ? `
          <div class="info">
            <strong>Username:</strong> ${this._escapeHtml(username)}
          </div>
          
          <form id="editForm" aria-label="Update display name form">
            <div class="form-group">
              <label for="display_name">Display Name</label>
              <input 
                type="text" 
                id="display_name" 
                name="display_name" 
                value="${this._escapeHtml(displayName)}" 
                placeholder="Enter your display name"
                aria-describedby="display-name-help"
                required
              >
              <small id="display-name-help" class="form-help">
                This is the name that will be displayed publicly.
              </small>
            </div>
            
            <button type="submit">Update Display Name</button>
          </form>
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
  reset() {
    const form = this.shadowRoot.querySelector('form');
    if (form) {
      form.reset();
    }
    this.error = null;
    this.success = null;
    this.render();
  }
}

// Define the custom element
customElements.define('micro-frontend-edit', EditComponent);
