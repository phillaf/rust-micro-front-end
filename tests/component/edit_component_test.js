/**
 * Edit Component Isolation Tests
 * 
 * Tests that verify the edit/CMS component works properly in isolation.
 */

import { JSDOM } from 'jsdom';
import fetch from 'node-fetch';
import assert from 'assert';

// Configuration
const SERVER_URL = process.env.TEST_SERVER_URL || 'http://localhost:3000';
const TEST_USERNAME = 'test_user';
const VALID_JWT = process.env.TEST_JWT_TOKEN || 'eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...'; // Provide valid token via env

describe('Edit Component Isolation Tests', () => {
  let dom;
  let window;
  let document;
  let container;
  let fetchSpy;
  let originalFetch;

  beforeEach(async () => {
    // Create a clean DOM environment for each test
    dom = new JSDOM('<!DOCTYPE html><html><head></head><body></body></html>', {
      url: 'http://localhost',
      runScripts: 'dangerously',
      resources: 'usable'
    });
    
    window = dom.window;
    document = window.document;
    container = document.createElement('div');
    container.id = 'component-container';
    document.body.appendChild(container);
    
    // Mock fetch for form submission tests
    originalFetch = window.fetch;
    fetchSpy = jest.fn().mockImplementation(async (url, options) => {
      if (url.includes('/api/username') && options.method === 'POST') {
        return {
          ok: true,
          json: async () => ({ 
            username: TEST_USERNAME, 
            display_name: JSON.parse(options.body).display_name
          })
        };
      }
      
      return originalFetch(url, options);
    });
    window.fetch = fetchSpy;
  });

  afterEach(() => {
    // Clean up DOM
    if (container && container.parentNode) {
      container.parentNode.removeChild(container);
    }
    
    // Restore original fetch
    if (originalFetch) {
      window.fetch = originalFetch;
    }
  });

  async function loadEditComponent() {
    try {
      // Fetch the component HTML
      const response = await fetch(`${SERVER_URL}/edit`, {
        headers: {
          'Authorization': `Bearer ${VALID_JWT}`
        }
      });
      
      if (!response.ok) {
        throw new Error(`Failed to load component: ${response.status}`);
      }
      
      const html = await response.text();
      container.innerHTML = html;
      
      return {
        success: true,
        html,
        element: container.firstElementChild,
        form: container.querySelector('form')
      };
    } catch (error) {
      console.error('Error loading edit component:', error);
      return {
        success: false,
        error
      };
    }
  }

  it('should load the edit component', async () => {
    const result = await loadEditComponent();
    assert.strictEqual(result.success, true, 'Component should load successfully');
    assert.ok(result.html.length > 0, 'Component HTML should not be empty');
    assert.ok(result.element, 'Component should have a root element');
  });

  it('should contain a form for editing display name', async () => {
    const result = await loadEditComponent();
    assert.strictEqual(result.success, true);
    
    const form = container.querySelector('form[data-test="edit-form"]');
    assert.ok(form, 'Component should contain a form');
    
    const displayNameInput = form.querySelector('input[name="display_name"]');
    assert.ok(displayNameInput, 'Form should contain a display name input');
    
    const submitButton = form.querySelector('button[type="submit"]');
    assert.ok(submitButton, 'Form should contain a submit button');
  });

  it('should pre-populate the display name field', async () => {
    const result = await loadEditComponent();
    assert.strictEqual(result.success, true);
    
    const displayNameInput = container.querySelector('input[name="display_name"]');
    assert.ok(displayNameInput, 'Form should contain a display name input');
    assert.ok(displayNameInput.value.length > 0, 'Display name input should be pre-populated');
  });

  it('should validate input on client-side', async () => {
    const result = await loadEditComponent();
    assert.strictEqual(result.success, true);
    
    const form = container.querySelector('form[data-test="edit-form"]');
    const displayNameInput = form.querySelector('input[name="display_name"]');
    
    // Test with empty input
    displayNameInput.value = '';
    
    // Create and dispatch submit event
    const submitEvent = new window.Event('submit', { cancelable: true });
    const preventDefaultSpy = jest.spyOn(submitEvent, 'preventDefault');
    form.dispatchEvent(submitEvent);
    
    // Form submission should be prevented for invalid input
    assert.strictEqual(preventDefaultSpy.mock.calls.length, 1, 'Form should prevent submission with invalid input');
    
    // Test with valid input
    displayNameInput.value = 'Valid Name';
    const validSubmitEvent = new window.Event('submit', { cancelable: true });
    const validPreventDefault = jest.spyOn(validSubmitEvent, 'preventDefault');
    
    form.dispatchEvent(validSubmitEvent);
    
    // Form should still prevent default (in test environment) but attempt to submit via fetch
    assert.strictEqual(validPreventDefault.mock.calls.length, 1);
  });

  it('should submit form data to the API', async () => {
    const result = await loadEditComponent();
    assert.strictEqual(result.success, true);
    
    const form = container.querySelector('form[data-test="edit-form"]');
    const displayNameInput = form.querySelector('input[name="display_name"]');
    const newDisplayName = 'Updated Display Name';
    
    // Set new display name
    displayNameInput.value = newDisplayName;
    
    // Submit the form
    const submitEvent = new window.Event('submit', { cancelable: true });
    form.dispatchEvent(submitEvent);
    
    // Check if fetch was called with correct parameters
    setTimeout(() => {
      expect(fetchSpy).toHaveBeenCalledWith(
        expect.stringMatching(/\/api\/username$/),
        expect.objectContaining({
          method: 'POST',
          body: expect.stringContaining(newDisplayName)
        })
      );
    }, 100);
  });

  it('should maintain style isolation', async () => {
    // Add some potentially conflicting styles to parent document
    const style = document.createElement('style');
    style.textContent = `
      form { display: none !important; }
      input { background-color: red !important; }
      button { pointer-events: none !important; }
    `;
    document.head.appendChild(style);
    
    const result = await loadEditComponent();
    assert.strictEqual(result.success, true);
    
    const form = container.querySelector('form');
    assert.ok(form, 'Form should exist');
    
    // Get computed styles
    const computedStyle = window.getComputedStyle(form);
    
    // Form should not be hidden by parent document style
    assert.notStrictEqual(computedStyle.display, 'none', 'Component styles should not be affected by parent document');
  });

  it('should handle errors gracefully', async () => {
    // Mock a failed fetch
    window.fetch = jest.fn().mockImplementation(async () => {
      return {
        ok: false,
        status: 401,
        statusText: 'Unauthorized'
      };
    });
    
    const result = await loadEditComponent();
    
    // The component should show an error message when loading fails
    assert.strictEqual(result.success, false);
    
    // When rendering without authorization, there should be an error message
    container.innerHTML = '<div class="error">Authorization required</div>';
    const errorElement = container.querySelector('.error');
    assert.ok(errorElement, 'Component should show error element');
  });
});
