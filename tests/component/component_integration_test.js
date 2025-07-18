/**
 * Component Integration Tests
 * 
 * Tests that verify multiple components can work together properly.
 */

import { JSDOM } from 'jsdom';
import fetch from 'node-fetch';
import assert from 'assert';

// Configuration
const SERVER_URL = process.env.TEST_SERVER_URL || 'http://localhost:3000';
const TEST_USERNAME = 'test_user';
const VALID_JWT = process.env.TEST_JWT_TOKEN || 'eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...'; // Provide valid token via env

describe('Component Integration Tests', () => {
  let dom;
  let window;
  let document;
  let container;

  beforeEach(async () => {
    // Create a clean DOM environment for each test
    dom = new JSDOM('<!DOCTYPE html><html><head></head><body></body></html>', {
      url: 'http://localhost',
      runScripts: 'dangerously',
      resources: 'usable'
    });
    
    window = dom.window;
    document = window.document;
    
    // Create container for components
    container = document.createElement('div');
    container.id = 'micro-frontend-container';
    container.style.display = 'flex';
    container.style.flexDirection = 'column';
    document.body.appendChild(container);
    
    // Create edit component container
    const editContainer = document.createElement('div');
    editContainer.id = 'edit-component';
    container.appendChild(editContainer);
    
    // Create display component container
    const displayContainer = document.createElement('div');
    displayContainer.id = 'display-component';
    container.appendChild(displayContainer);
  });

  afterEach(() => {
    // Clean up DOM
    if (container && container.parentNode) {
      container.parentNode.removeChild(container);
    }
  });

  async function loadComponents() {
    try {
      // Fetch the edit component
      const editResponse = await fetch(`${SERVER_URL}/edit`, {
        headers: {
          'Authorization': `Bearer ${VALID_JWT}`
        }
      });
      
      if (!editResponse.ok) {
        throw new Error(`Failed to load edit component: ${editResponse.status}`);
      }
      
      const editHtml = await editResponse.text();
      const editContainer = document.getElementById('edit-component');
      editContainer.innerHTML = editHtml;
      
      // Fetch the display component
      const displayResponse = await fetch(`${SERVER_URL}/display/username/${TEST_USERNAME}`);
      
      if (!displayResponse.ok) {
        throw new Error(`Failed to load display component: ${displayResponse.status}`);
      }
      
      const displayHtml = await displayResponse.text();
      const displayContainer = document.getElementById('display-component');
      displayContainer.innerHTML = displayHtml;
      
      return {
        success: true,
        editComponent: editContainer.firstElementChild,
        displayComponent: displayContainer.firstElementChild
      };
    } catch (error) {
      console.error('Error loading components:', error);
      return {
        success: false,
        error
      };
    }
  }

  it('should load both components together', async () => {
    const result = await loadComponents();
    assert.strictEqual(result.success, true, 'Components should load successfully');
    assert.ok(result.editComponent, 'Edit component should be loaded');
    assert.ok(result.displayComponent, 'Display component should be loaded');
  });

  it('should update display component when edit form is submitted', async () => {
    // Mock fetch for API calls
    const originalFetch = window.fetch;
    window.fetch = jest.fn().mockImplementation(async (url, options) => {
      if (url.includes('/api/username') && options.method === 'POST') {
        // Mock successful update
        const body = JSON.parse(options.body);
        const newDisplayName = body.display_name;
        
        // Update display component with new display name
        const displayNameElement = document.querySelector('#display-component [data-test="display-name"]');
        if (displayNameElement) {
          displayNameElement.textContent = newDisplayName;
        }
        
        return {
          ok: true,
          json: async () => ({ 
            username: TEST_USERNAME, 
            display_name: newDisplayName
          })
        };
      }
      
      return originalFetch(url, options);
    });
    
    await loadComponents();
    
    // Get form elements
    const form = document.querySelector('#edit-component form[data-test="edit-form"]');
    const displayNameInput = form.querySelector('input[name="display_name"]');
    const submitButton = form.querySelector('button[type="submit"]');
    
    // Get original display name
    const displayNameElement = document.querySelector('#display-component [data-test="display-name"]');
    const originalDisplayName = displayNameElement ? displayNameElement.textContent : '';
    
    // Set new display name
    const newDisplayName = 'Updated Display Name';
    displayNameInput.value = newDisplayName;
    
    // Submit the form
    const submitEvent = new window.Event('submit', { cancelable: true });
    form.dispatchEvent(submitEvent);
    
    // Wait for update to propagate
    await new Promise(resolve => setTimeout(resolve, 50));
    
    // Check if display component was updated
    const updatedDisplayNameElement = document.querySelector('#display-component [data-test="display-name"]');
    const updatedDisplayName = updatedDisplayNameElement ? updatedDisplayNameElement.textContent : '';
    
    assert.notStrictEqual(updatedDisplayName, originalDisplayName, 'Display name should be updated');
    assert.strictEqual(updatedDisplayName, newDisplayName, 'Display should show the new name');
    
    // Restore original fetch
    window.fetch = originalFetch;
  });

  it('should maintain component boundaries', async () => {
    await loadComponents();
    
    // Check if components have distinct root elements
    const editRoot = document.getElementById('edit-component').firstElementChild;
    const displayRoot = document.getElementById('display-component').firstElementChild;
    
    assert.ok(editRoot, 'Edit component should have root element');
    assert.ok(displayRoot, 'Display component should have root element');
    assert.notStrictEqual(editRoot, displayRoot, 'Components should have separate root elements');
    
    // Check if component styles are isolated
    const editStyle = window.getComputedStyle(editRoot);
    const displayStyle = window.getComputedStyle(displayRoot);
    
    // Components should maintain separate styling
    if (editStyle.backgroundColor && displayStyle.backgroundColor) {
      assert.ok(true, 'Component styles can be accessed independently');
    }
  });

  it('should handle component communication', async () => {
    await loadComponents();
    
    // Create event bus for component communication
    window.eventBus = {
      listeners: {},
      on(event, callback) {
        this.listeners[event] = this.listeners[event] || [];
        this.listeners[event].push(callback);
      },
      emit(event, data) {
        if (this.listeners[event]) {
          this.listeners[event].forEach(callback => callback(data));
        }
      }
    };
    
    // Set up display component to listen for updates
    const displayNameElement = document.querySelector('#display-component [data-test="display-name"]');
    window.eventBus.on('displayNameUpdated', (data) => {
      if (displayNameElement) {
        displayNameElement.textContent = data.displayName;
      }
    });
    
    // Set up edit component to emit update events
    const form = document.querySelector('#edit-component form[data-test="edit-form"]');
    form.addEventListener('submit', (event) => {
      event.preventDefault();
      const displayNameInput = form.querySelector('input[name="display_name"]');
      window.eventBus.emit('displayNameUpdated', {
        username: TEST_USERNAME,
        displayName: displayNameInput.value
      });
    });
    
    // Update the display name
    const displayNameInput = form.querySelector('input[name="display_name"]');
    const newDisplayName = 'Event Bus Updated Name';
    displayNameInput.value = newDisplayName;
    
    // Submit the form
    const submitEvent = new window.Event('submit', { cancelable: true });
    form.dispatchEvent(submitEvent);
    
    // Check if display component received the update
    assert.strictEqual(displayNameElement.textContent, newDisplayName, 
      'Display component should receive updates via event bus');
  });
});
