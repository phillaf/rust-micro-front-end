/**
 * Display Component Isolation Tests
 * 
 * Tests that verify the display component works properly in isolation.
 */

import { JSDOM } from 'jsdom';
import fetch from 'node-fetch';
import assert from 'assert';

// Configuration
const SERVER_URL = process.env.TEST_SERVER_URL || 'http://localhost:3000';
const TEST_USERNAME = 'test_user';

describe('Display Component Isolation Tests', () => {
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
    container = document.createElement('div');
    container.id = 'component-container';
    document.body.appendChild(container);
  });

  afterEach(() => {
    // Clean up DOM
    if (container && container.parentNode) {
      container.parentNode.removeChild(container);
    }
  });

  async function loadDisplayComponent(username) {
    try {
      // Fetch the component HTML
      const response = await fetch(`${SERVER_URL}/display/username/${username}`);
      if (!response.ok) {
        throw new Error(`Failed to load component: ${response.status}`);
      }
      
      const html = await response.text();
      container.innerHTML = html;
      
      return {
        success: true,
        html,
        element: container.firstElementChild
      };
    } catch (error) {
      console.error('Error loading display component:', error);
      return {
        success: false,
        error
      };
    }
  }

  it('should load the display component', async () => {
    const result = await loadDisplayComponent(TEST_USERNAME);
    assert.strictEqual(result.success, true, 'Component should load successfully');
    assert.ok(result.html.length > 0, 'Component HTML should not be empty');
    assert.ok(result.element, 'Component should have a root element');
  });

  it('should contain the username when loaded', async () => {
    const result = await loadDisplayComponent(TEST_USERNAME);
    assert.strictEqual(result.success, true);
    
    const usernameElement = container.querySelector('[data-test="display-username"]');
    assert.ok(usernameElement, 'Component should contain username element');
    assert.ok(usernameElement.textContent.includes(TEST_USERNAME), 'Username should be displayed');
  });

  it('should handle display name properly', async () => {
    const result = await loadDisplayComponent(TEST_USERNAME);
    assert.strictEqual(result.success, true);
    
    const displayNameElement = container.querySelector('[data-test="display-name"]');
    assert.ok(displayNameElement, 'Component should contain display name element');
    
    // Either a display name is shown or a fallback message
    assert.ok(
      displayNameElement.textContent.length > 0, 
      'Display name element should not be empty'
    );
  });

  it('should handle non-existent username gracefully', async () => {
    const result = await loadDisplayComponent('non_existent_user_12345');
    assert.strictEqual(result.success, true, 'Component should load even with invalid username');
    
    const errorElement = container.querySelector('[data-test="display-error"]');
    assert.ok(errorElement, 'Component should show error element for non-existent user');
  });

  it('should maintain style isolation', async () => {
    // Add some potentially conflicting styles to parent document
    const style = document.createElement('style');
    style.textContent = `
      div { background-color: red !important; }
      p { display: none !important; }
      .display-name { color: purple !important; }
    `;
    document.head.appendChild(style);
    
    const result = await loadDisplayComponent(TEST_USERNAME);
    assert.strictEqual(result.success, true);
    
    const componentRoot = container.firstElementChild;
    assert.ok(componentRoot, 'Component root should exist');
    
    // Get computed styles for elements
    const displayNameElement = container.querySelector('[data-test="display-name"]');
    if (displayNameElement) {
      const computedStyle = window.getComputedStyle(displayNameElement);
      
      // The component should use its own styles, not be hidden by parent document style
      assert.notStrictEqual(computedStyle.display, 'none', 'Component styles should not be affected by parent document');
    }
  });

  it('should not leak global variables', async () => {
    const beforeGlobals = Object.keys(window);
    
    await loadDisplayComponent(TEST_USERNAME);
    
    const afterGlobals = Object.keys(window);
    const newGlobals = afterGlobals.filter(key => !beforeGlobals.includes(key));
    
    // The component shouldn't create any new global variables
    assert.deepStrictEqual(newGlobals, [], 'Component should not create global variables');
  });
});
