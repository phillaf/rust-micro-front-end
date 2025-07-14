use crate::errors::AppError;
use regex::Regex;
use std::sync::OnceLock;

// Username validation regex (Twitter-style handles: alphanumeric, underscores, hyphens)
static USERNAME_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_username_regex() -> &'static Regex {
    USERNAME_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9_-]{3,50}$").expect("Invalid username regex")
    })
}

#[derive(Debug, Clone)]
pub struct ValidatedUsername(String);

impl ValidatedUsername {
    pub fn new(username: String) -> Result<Self, AppError> {
        validate_username(&username)?;
        Ok(ValidatedUsername(username))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for ValidatedUsername {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedDisplayName(String);

impl ValidatedDisplayName {
    pub fn new(display_name: String) -> Result<Self, AppError> {
        validate_display_name(&display_name)?;
        Ok(ValidatedDisplayName(display_name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for ValidatedDisplayName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Validate username format and length
pub fn validate_username(username: &str) -> Result<(), AppError> {
    if username.is_empty() {
        return Err(AppError::validation_failed("Username cannot be empty"));
    }

    if username.len() < 3 {
        return Err(AppError::validation_failed("Username must be at least 3 characters long"));
    }

    if username.len() > 50 {
        return Err(AppError::validation_failed("Username must be at most 50 characters long"));
    }

    if !get_username_regex().is_match(username) {
        return Err(AppError::validation_failed(
            "Username must contain only alphanumeric characters, underscores, and hyphens"
        ));
    }

    Ok(())
}

/// Validate display name format and length
pub fn validate_display_name(display_name: &str) -> Result<(), AppError> {
    if display_name.is_empty() {
        return Err(AppError::validation_failed("Display name cannot be empty"));
    }

    if display_name.len() > 100 {
        return Err(AppError::validation_failed("Display name must be at most 100 characters long"));
    }

    // Check for dangerous characters that could be used for XSS
    if display_name.contains('<') || display_name.contains('>') || display_name.contains('&') {
        return Err(AppError::validation_failed(
            "Display name cannot contain HTML characters"
        ));
    }

    // Check for control characters
    if display_name.chars().any(|c| c.is_control() && c != '\t' && c != '\n' && c != '\r') {
        return Err(AppError::validation_failed(
            "Display name cannot contain control characters"
        ));
    }

    Ok(())
}

/// Sanitize display name by trimming whitespace and normalizing
pub fn sanitize_display_name(display_name: &str) -> String {
    display_name.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_validation() {
        // Valid usernames
        assert!(validate_username("test_user").is_ok());
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("test-user").is_ok());
        assert!(validate_username("a1b").is_ok()); // minimum length
        assert!(validate_username(&"a".repeat(50)).is_ok()); // maximum length

        // Invalid usernames
        assert!(validate_username("").is_err()); // empty
        assert!(validate_username("ab").is_err()); // too short
        assert!(validate_username(&"a".repeat(51)).is_err()); // too long
        assert!(validate_username("test user").is_err()); // space
        assert!(validate_username("test@user").is_err()); // special char
        assert!(validate_username("test.user").is_err()); // dot
    }

    #[test]
    fn test_display_name_validation() {
        // Valid display names
        assert!(validate_display_name("John Doe").is_ok());
        assert!(validate_display_name("User 123").is_ok());
        assert!(validate_display_name("Test User!").is_ok());
        assert!(validate_display_name("A").is_ok()); // minimum length
        assert!(validate_display_name(&"A".repeat(100)).is_ok()); // maximum length

        // Invalid display names
        assert!(validate_display_name("").is_err()); // empty
        assert!(validate_display_name(&"A".repeat(101)).is_err()); // too long
        assert!(validate_display_name("Test<script>").is_err()); // HTML
        assert!(validate_display_name("Test&amp;").is_err()); // HTML entity
        assert!(validate_display_name("Test>User").is_err()); // HTML
    }

    #[test]
    fn test_sanitization() {
        assert_eq!(sanitize_username("  TestUser  "), "testuser");
        assert_eq!(sanitize_username("User123"), "user123");
        assert_eq!(sanitize_display_name("  John Doe  "), "John Doe");
        assert_eq!(sanitize_display_name("Test User"), "Test User");
    }
}
