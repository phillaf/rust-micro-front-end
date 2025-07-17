#[cfg(test)]
mod tests {
    use crate::validation::{ValidatedDisplayName, ValidatedUsername};
    use crate::errors::ErrorCode;
    
    #[test]
    fn test_validated_username_new() {
        // Valid usernames
        let username = ValidatedUsername::new("testuser".to_string()).unwrap();
        assert_eq!(username.as_str(), "testuser");
        assert_eq!(username.into_string(), "testuser");
        
        let username = ValidatedUsername::new("user123".to_string()).unwrap();
        assert_eq!(username.as_str(), "user123");
        
        let username = ValidatedUsername::new("test-user".to_string()).unwrap();
        assert_eq!(username.as_str(), "test-user");
        
        // Invalid usernames
        let result = ValidatedUsername::new("".to_string());
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(err.code, ErrorCode::ValidationFailed));
        }
        
        let result = ValidatedUsername::new("a".to_string());
        assert!(result.is_err());
        
        let result = ValidatedUsername::new("test@user".to_string());
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validated_display_name_new() {
        // Valid display names
        let display_name = ValidatedDisplayName::new("John Doe".to_string()).unwrap();
        assert_eq!(display_name.as_str(), "John Doe");
        assert_eq!(display_name.into_string(), "John Doe");
        
        let display_name = ValidatedDisplayName::new("User 123".to_string()).unwrap();
        assert_eq!(display_name.as_str(), "User 123");
        
        // Invalid display names
        let result = ValidatedDisplayName::new("".to_string());
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(err.code, ErrorCode::ValidationFailed));
        }
        
        let result = ValidatedDisplayName::new("<script>alert('xss')</script>".to_string());
        assert!(result.is_err());
        
        // Test with extremely long name (over 100 chars)
        let long_name = "a".repeat(101);
        let result = ValidatedDisplayName::new(long_name);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_display_trait() {
        let username = ValidatedUsername::new("testuser".to_string()).unwrap();
        assert_eq!(format!("{}", username), "testuser");
        
        let display_name = ValidatedDisplayName::new("John Doe".to_string()).unwrap();
        assert_eq!(format!("{}", display_name), "John Doe");
    }
}
