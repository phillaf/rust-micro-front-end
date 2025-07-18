// Bundle all web components for easy loading
import './display-component.js';
import './edit-component.js';
import { eventBus } from './event-bus.js';

// Export the eventBus for use in host applications
window.MicroFrontendEventBus = eventBus;

/**
 * Initialize the micro frontend components with configuration
 * @param {Object} config - Configuration options
 * @param {string} config.theme - Theme to apply ('default', 'dark', 'high-contrast')
 * @param {Object} config.i18n - Internationalization settings
 */
export function initMicroFrontend(config = {}) {
  // Apply theme to document root if provided
  if (config.theme) {
    document.documentElement.dataset.microFrontendTheme = config.theme;
    
    // Apply theme to all existing components
    document.querySelectorAll('[micro-frontend-display], [micro-frontend-edit]')
      .forEach(component => {
        component.setAttribute('theme', config.theme);
      });
  }
  
  // Return API for the host application
  return {
    eventBus,
    
    // Create and attach a display component
    createDisplayComponent(container, options = {}) {
      const component = document.createElement('micro-frontend-display');
      
      if (options.username) {
        component.setAttribute('username', options.username);
      }
      
      if (options.displayName) {
        component.setAttribute('display-name', options.displayName);
      }
      
      if (options.theme || config.theme) {
        component.setAttribute('theme', options.theme || config.theme);
      }
      
      if (options.autoRefresh) {
        component.setAttribute('auto-refresh', '');
      }
      
      const targetContainer = typeof container === 'string' 
        ? document.querySelector(container) 
        : container;
        
      if (targetContainer) {
        targetContainer.appendChild(component);
      }
      
      return component;
    },
    
    // Create and attach an edit component
    createEditComponent(container, options = {}) {
      const component = document.createElement('micro-frontend-edit');
      
      if (options.username) {
        component.setAttribute('username', options.username);
      }
      
      if (options.theme || config.theme) {
        component.setAttribute('theme', options.theme || config.theme);
      }
      
      if (options.returnUrl) {
        component.setAttribute('return-url', options.returnUrl);
      }
      
      const targetContainer = typeof container === 'string' 
        ? document.querySelector(container) 
        : container;
        
      if (targetContainer) {
        targetContainer.appendChild(component);
      }
      
      return component;
    }
  };
}

// Auto-initialize if script is loaded with data-auto-init attribute
if (document.currentScript && document.currentScript.hasAttribute('data-auto-init')) {
  window.microFrontend = initMicroFrontend({
    theme: document.currentScript.getAttribute('data-theme')
  });
}
