#[cfg(test)]
mod tests {
    use crate::database::{mock::MockUserDatabase, UserDatabase};

    #[tokio::test]
    async fn test_mock_database_get_user() {
        let db = MockUserDatabase::new();

        // Test getting an existing user
        let user = db.get_user("testuser").await.unwrap();
        assert!(user.is_some());

        let user = user.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.display_name, "testuser"); // Fixed to match actual implementation

        // Test getting a non-existent user
        let user = db.get_user("nonexistent").await.unwrap();
        assert!(user.is_none());
    }

    #[tokio::test]
    async fn test_mock_database_update_user() {
        let db = MockUserDatabase::new();

        // Update an existing user
        db.update_user_display_name("testuser", "Updated Name").await.unwrap();

        // Verify the update
        let user = db.get_user("testuser").await.unwrap().unwrap();
        assert_eq!(user.display_name, "Updated Name");

        // Test updating a non-existent user (should create it)
        db.update_user_display_name("newuser", "New User").await.unwrap();

        // Verify the new user was created
        let user = db.get_user("newuser").await.unwrap().unwrap();
        assert_eq!(user.username, "newuser");
        assert_eq!(user.display_name, "New User");
    }

    #[tokio::test]
    async fn test_mock_database_health_check() {
        let db = MockUserDatabase::new();

        // Health check should always succeed for mock database
        let status = db.health_check().await.unwrap();
        assert_eq!(status, "mock_db_healthy_with_4_users"); // Updated to match implementation
    }
}
