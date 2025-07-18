#!/bin/bash
# Composability validation test for the micro front-end components
# This script tests the integration of our web components with other micro-apps

set -e # Exit on any error

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo "Running composability validation for web components..."

# Generate test environment if it doesn't exist
TEST_ENV="/tmp/composability-test"

if [ ! -d "$TEST_ENV" ]; then
    echo "Creating test environment at $TEST_ENV"
    mkdir -p "$TEST_ENV"
fi

# Copy our web components to the test environment
echo "Copying web components to test environment..."
cp -r /home/phil/Projects/rust-micro-front-end/src/web_components/* "$TEST_ENV/"

# Create a test HTML file that imports our components and simulates a multi-app environment
cat > "$TEST_ENV/composability_test.html" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Composability Test</title>
    <script src="./index.js" type="module"></script>
    <script src="./event-bus.js" type="module"></script>
    
    <!-- Mock external app components -->
    <script>
        // Define a mock navigation component
        customElements.define('mock-navigation', class extends HTMLElement {
            constructor() {
                super();
                const shadow = this.attachShadow({mode: 'open'});
                shadow.innerHTML = \`
                    <style>
                        :host { display: block; padding: 10px; background: #f0f0f0; margin-bottom: 10px; }
                    </style>
                    <div>
                        <h2>Navigation Component</h2>
                        <button id="navigate-display">Go to Display</button>
                        <button id="navigate-edit">Go to Edit</button>
                    </div>
                \`;
                
                shadow.querySelector('#navigate-display').addEventListener('click', () => {
                    document.dispatchEvent(new CustomEvent('app:navigation', { 
                        detail: { target: 'display', username: 'testuser' }
                    }));
                });
                
                shadow.querySelector('#navigate-edit').addEventListener('click', () => {
                    document.dispatchEvent(new CustomEvent('app:navigation', { 
                        detail: { target: 'edit', username: 'testuser' }
                    }));
                });
            }
        });

        // Define a mock authentication component
        customElements.define('mock-auth', class extends HTMLElement {
            constructor() {
                super();
                const shadow = this.attachShadow({mode: 'open'});
                shadow.innerHTML = \`
                    <style>
                        :host { display: block; padding: 10px; background: #e0e0e0; margin-bottom: 10px; }
                    </style>
                    <div>
                        <h2>Auth Component</h2>
                        <button id="login">Login</button>
                        <button id="logout">Logout</button>
                    </div>
                \`;
                
                this.isAuthenticated = false;
                
                shadow.querySelector('#login').addEventListener('click', () => {
                    this.isAuthenticated = true;
                    document.dispatchEvent(new CustomEvent('auth:status', { 
                        detail: { authenticated: true, username: 'testuser' }
                    }));
                });
                
                shadow.querySelector('#logout').addEventListener('click', () => {
                    this.isAuthenticated = false;
                    document.dispatchEvent(new CustomEvent('auth:status', { 
                        detail: { authenticated: false }
                    }));
                });
            }
        });
    </script>
</head>
<body>
    <h1>Micro Front-End Composability Test</h1>
    
    <!-- Mock external components -->
    <mock-navigation></mock-navigation>
    <mock-auth></mock-auth>
    
    <!-- Our components -->
    <display-component username="testuser"></display-component>
    <edit-component username="testuser"></edit-component>
    
    <!-- Test output -->
    <div id="test-output">
        <h2>Test Log</h2>
        <pre id="log"></pre>
    </div>
    
    <script>
        // Test harness
        const log = document.getElementById('log');
        
        function logMessage(message, type = 'info') {
            const timestamp = new Date().toISOString();
            let color = 'black';
            if (type === 'success') color = 'green';
            if (type === 'error') color = 'red';
            if (type === 'warning') color = 'orange';
            
            log.innerHTML += \`<span style="color: \${color}">\${timestamp} - \${message}</span>\n\`;
        }
        
        // Test cases
        const testCases = [
            {
                name: 'Components render properly',
                test: () => {
                    const displayComponent = document.querySelector('display-component');
                    const editComponent = document.querySelector('edit-component');
                    return displayComponent && editComponent;
                }
            },
            {
                name: 'Components respond to navigation events',
                test: () => {
                    // Trigger navigation event
                    document.dispatchEvent(new CustomEvent('app:navigation', { 
                        detail: { target: 'display', username: 'newuser' }
                    }));
                    
                    // Verify display component updated
                    const displayComponent = document.querySelector('display-component');
                    return displayComponent.getAttribute('username') === 'newuser';
                }
            },
            {
                name: 'Components respond to authentication events',
                test: () => {
                    // Trigger auth event
                    document.dispatchEvent(new CustomEvent('auth:status', { 
                        detail: { authenticated: true, username: 'authuser' }
                    }));
                    
                    // Check if edit component is enabled/disabled based on auth status
                    const editComponent = document.querySelector('edit-component');
                    return editComponent.isEnabled === true;
                }
            },
            {
                name: 'Event bus properly propagates events',
                test: () => {
                    let eventReceived = false;
                    
                    // Set up listener
                    document.addEventListener('user:updated', () => {
                        eventReceived = true;
                    }, { once: true });
                    
                    // Trigger event
                    document.dispatchEvent(new CustomEvent('user:updated', { 
                        detail: { username: 'testuser', displayName: 'Test User' }
                    }));
                    
                    return eventReceived;
                }
            }
        ];
        
        // Run tests
        window.addEventListener('load', () => {
            logMessage('Starting composability tests...');
            
            // Wait a bit for components to initialize
            setTimeout(() => {
                let passed = 0;
                let failed = 0;
                
                testCases.forEach(testCase => {
                    try {
                        const result = testCase.test();
                        if (result) {
                            logMessage(\`✅ PASS: \${testCase.name}\`, 'success');
                            passed++;
                        } else {
                            logMessage(\`❌ FAIL: \${testCase.name}\`, 'error');
                            failed++;
                        }
                    } catch (error) {
                        logMessage(\`❌ ERROR: \${testCase.name} - \${error.message}\`, 'error');
                        failed++;
                    }
                });
                
                logMessage(\`Tests completed: \${passed} passed, \${failed} failed\`);
                
                // Report results back to parent window if in iframe
                if (window.parent && window.parent !== window) {
                    window.parent.postMessage({
                        type: 'test-results',
                        passed,
                        failed,
                        total: testCases.length
                    }, '*');
                }
            }, 500);
        });
    </script>
</body>
</html>
EOF

# Create a Node.js test runner
cat > "$TEST_ENV/test_runner.js" << EOF
const puppeteer = require('puppeteer');
const http = require('http');
const fs = require('fs');
const path = require('path');

// Simple server to serve the test files
const server = http.createServer((req, res) => {
    let filePath = path.join(__dirname, req.url === '/' ? 'composability_test.html' : req.url);
    
    fs.readFile(filePath, (err, data) => {
        if (err) {
            res.writeHead(404);
            res.end('File not found');
            return;
        }
        
        // Set the content type
        let contentType = 'text/html';
        if (filePath.endsWith('.js')) contentType = 'text/javascript';
        if (filePath.endsWith('.css')) contentType = 'text/css';
        
        res.writeHead(200, { 'Content-Type': contentType });
        res.end(data);
    });
});

// Start the server
const PORT = 8888;
server.listen(PORT, async () => {
    console.log(\`Server running at http://localhost:\${PORT}/\`);
    
    // Launch browser and run tests
    const browser = await puppeteer.launch({ headless: "new" });
    const page = await browser.newPage();
    
    // Capture console messages
    page.on('console', msg => console.log(\`Browser console: \${msg.text()}\`));
    
    let testResults = null;
    
    // Listen for test results
    page.on('dialog', async dialog => {
        console.log(\`Dialog: \${dialog.message()}\`);
        await dialog.dismiss();
    });
    
    // Listen for test results via postMessage
    page.on('pageerror', err => {
        console.error(\`Page error: \${err.message}\`);
    });
    
    // Set up message listener
    await page.evaluateOnNewDocument(() => {
        window.addEventListener('message', event => {
            if (event.data.type === 'test-results') {
                console.log(JSON.stringify(event.data));
            }
        });
    });
    
    // Navigate to the test page
    await page.goto(\`http://localhost:\${PORT}/\`);
    
    // Wait for tests to complete (adjust timeout as needed)
    await page.waitForFunction(() => {
        const log = document.querySelector('#log');
        return log && log.textContent.includes('Tests completed');
    }, { timeout: 5000 });
    
    // Extract results from the page
    const results = await page.evaluate(() => {
        const log = document.querySelector('#log').textContent;
        const passedMatch = log.match(/(\d+) passed/);
        const failedMatch = log.match(/(\d+) failed/);
        return {
            passed: passedMatch ? parseInt(passedMatch[1]) : 0,
            failed: failedMatch ? parseInt(failedMatch[1]) : 0
        };
    });
    
    console.log(\`Test Results: \${results.passed} passed, \${results.failed} failed\`);
    
    // Close browser and server
    await browser.close();
    server.close();
    
    // Exit with appropriate status
    process.exit(results.failed > 0 ? 1 : 0);
});
EOF

echo -e "${YELLOW}Composability test files created at $TEST_ENV${NC}"
echo -e "${YELLOW}To run the tests, install node dependencies and execute:${NC}"
echo -e "${GREEN}cd $TEST_ENV && npm install puppeteer && node test_runner.js${NC}"

# Now let's create a wrapper script to execute these tests through our containerized environment
cat > "/home/phil/Projects/rust-micro-front-end/tests/composability/run_composability_tests.sh" << EOF
#!/bin/bash
# Run composability validation tests for our web components

# This script runs the composability tests in a containerized environment
# to ensure components work properly in a multi-app ecosystem

set -e # Exit on any error

echo "Setting up composability test environment..."

# Create temporary directory for test artifacts
TEST_DIR=\$(mktemp -d /tmp/composability-XXXXX)
SCRIPT_DIR="\$( cd "\$( dirname "\${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# Copy test files
cp -r "\$SCRIPT_DIR/../../src/web_components" "\$TEST_DIR/"

# Copy the test files from the source
cp "\$0" "\$TEST_DIR/run_test.sh"

echo "Creating composability test container..."

# Run tests in a container with puppeteer
docker run --rm -v "\$TEST_DIR:/app" -w /app node:18 bash -c "
    echo 'Installing dependencies...'
    npm init -y > /dev/null
    npm install puppeteer@19 > /dev/null
    
    echo 'Creating test files...'
    cp /app/run_test.sh /app/setup_tests.sh
    chmod +x /app/setup_tests.sh
    /app/setup_tests.sh
    
    echo 'Running tests...'
    node /tmp/composability-test/test_runner.js
"

TEST_RESULT=\$?

echo "Cleaning up test environment..."
rm -rf "\$TEST_DIR"

if [ \$TEST_RESULT -eq 0 ]; then
    echo -e \"\\033[0;32mComposability tests passed successfully!\\033[0m\"
    exit 0
else
    echo -e \"\\033[0;31mComposability tests failed!\\033[0m\"
    exit 1
fi
EOF

chmod +x "/home/phil/Projects/rust-micro-front-end/tests/composability/run_composability_tests.sh"

echo -e "${GREEN}Composability validation script created:${NC} /home/phil/Projects/rust-micro-front-end/tests/composability/run_composability_tests.sh"
echo -e "${YELLOW}To run the script, use:${NC} just run-in-container /tests/composability/run_composability_tests.sh"
