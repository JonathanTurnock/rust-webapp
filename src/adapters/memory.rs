use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::users::{User, UserRepo, UserRepoError};

pub struct MemoryUserRepo {
    users: RwLock<HashMap<Uuid, User>>,
}

impl MemoryUserRepo {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl UserRepo for MemoryUserRepo {
    async fn add_user(&self, username: &str, email: &str) -> Result<User, UserRepoError> {
        log::debug!(target: "Users", "Adding user: {username}");

        let user = User {
            id: Uuid::new_v4(),
            username: username.to_owned(),
            email: email.to_owned(),
        };

        // No need for Entry/Vacant: UUID collision is not a thing you handle here.
        let mut users = self.users.write().await;
        users.insert(user.id, user.clone());

        Ok(user)
    }

    async fn get_user(&self, id: Uuid) -> Result<Option<User>, UserRepoError> {
        log::debug!(target: "Users", "Getting user: {id}");

        let users = self.users.read().await;
        Ok(users.get(&id).cloned())
    }

    async fn remove_user(&self, id: Uuid) -> Result<Option<User>, UserRepoError> {
        log::debug!(target: "Users", "Removing user: {id}");

        let mut users = self.users.write().await;
        Ok(users.remove(&id))
    }

    async fn list_users(&self) -> Result<Vec<User>, UserRepoError> {
        log::debug!(target: "Users", "Listing users");

        let users = self.users.read().await;
        Ok(users.values().cloned().collect())
    }
}
