-- Create users table
CREATE TABLE users (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- Indexes for performance
    INDEX idx_username (username),
    INDEX idx_created_at (created_at)
);

-- Insert some initial test data
INSERT INTO users (username, display_name) VALUES
    ('admin', 'Administrator'),
    ('testuser', 'Test User'),
    ('demo', 'Demo User');
