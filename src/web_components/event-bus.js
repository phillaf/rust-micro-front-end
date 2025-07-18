// Event bus for cross-component communication
export class MicroFrontendEventBus {
  constructor() {
    this.eventTarget = new EventTarget();
    
    // Use BroadcastChannel if available for cross-tab/window communication
    this.broadcastChannel = typeof BroadcastChannel !== 'undefined' ? 
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
