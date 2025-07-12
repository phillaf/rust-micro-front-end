use super::{User, UserDatabase};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MockUserDatabase {
    users: Arc<RwLock<HashMap<String, User>>>,
}

impl MockUserDatabase {
    pub fn new() -> Self {
        let mut users = HashMap::new();

        users.insert(
            "admin".to_string(),
            User {
                username: "admin".to_string(),
                display_name: "Administrator".to_string(),
            },
        );

        users.insert(
            "johndoe".to_string(),
            User {
                username: "johndoe".to_string(),
                display_name: "John Doe".to_string(),
            },
        );

        users.insert(
            "alice".to_string(),
            User {
                username: "alice".to_string(),
                display_name: "Alice Smith".to_string(),
            },
        );

        tracing::info!("Mock database initialized with {} sample users", users.len());

        Self { users: Arc::new(RwLock::new(users)) }
    }

    #[allow(dead_code)]
    pub fn new_empty() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn user_count(&self) -> usize {
        let users = self.users.read().await;
        users.len()
    }

    #[allow(dead_code)]
    async fn user_exists(&self, username: &str) -> Result<bool> {
        let users = self.users.read().await;
        Ok(users.contains_key(username))
    }
}

#[async_trait]
impl UserDatabase for MockUserDatabase {
    async fn get_user(&self, username: &str) -> Result<Option<User>> {
        let users = self.users.read().await;
        Ok(users.get(username).cloned())
    }

    async fn update_user_display_name(&self, username: &str, display_name: &str) -> Result<()> {
        let mut users = self.users.write().await;

        match users.get_mut(username) {
            Some(user) => {
                user.display_name = display_name.to_string();
                tracing::info!("📝 Updated display name for user '{}': '{}'", username, display_name);
            }
            None => {
                let user = User {
                    username: username.to_string(),
                    display_name: display_name.to_string(),
                };
                users.insert(username.to_string(), user);
                tracing::info!("➕ Created new user '{}' with display name: '{}'", username, display_name);
            }
        }

        Ok(())
    }

    async fn health_check(&self) -> Result<String> {
        let user_count = self.user_count().await;
        Ok(format!("mock_db_healthy_with_{user_count}_users"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_database_new() {
        let db = MockUserDatabase::new();
        assert_eq!(db.user_count().await, 3);

        let admin = db.get_user("admin").await.unwrap();
        assert!(admin.is_some());
        assert_eq!(admin.unwrap().display_name, "Administrator");
    }

    #[tokio::test]
    async fn test_mock_database_empty() {
        let db = MockUserDatabase::new_empty();
        assert_eq!(db.user_count().await, 0);

        let user = db.get_user("nonexistent").await.unwrap();
        assert!(user.is_none());
    }

    #[tokio::test]
    async fn test_update_existing_user() {
        let db = MockUserDatabase::new();

        db.update_user_display_name("admin", "Super Admin").await.unwrap();

        let user = db.get_user("admin").await.unwrap().unwrap();
        assert_eq!(user.display_name, "Super Admin");
        assert_eq!(db.user_count().await, 3); // Should still have 3 users
    }

    #[tokio::test]
    async fn test_create_new_user() {
        let db = MockUserDatabase::new_empty();

        // Create new user
        db.update_user_display_name("newuser", "New User").await.unwrap();

        let user = db.get_user("newuser").await.unwrap().unwrap();
        assert_eq!(user.username, "newuser");
        assert_eq!(user.display_name, "New User");
        assert_eq!(db.user_count().await, 1);
    }

    #[tokio::test]
    async fn test_user_exists() {
        let db = MockUserDatabase::new();

        assert!(db.user_exists("admin").await.unwrap());
        assert!(!db.user_exists("nonexistent").await.unwrap());
    }

    #[tokio::test]
    async fn test_health_check() {
        let db = MockUserDatabase::new();
        let health = db.health_check().await.unwrap();
        assert!(health.contains("mock_db_healthy"));
        assert!(health.contains("3_users"));
    }
}
