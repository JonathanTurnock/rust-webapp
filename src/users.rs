mod sqlite;
mod mongo;

use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use log::info;
use uuid::Uuid;

pub use sqlite::SqliteUserRepo;
pub use mongo::MongoUserRepo;

#[derive(Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

pub trait UserRepo {
    async fn new(db_url: &str) -> Self;
    async fn add_user(&mut self, username: String, email: String) -> Result<User, String>;
    async fn remove_user(&mut self, username: &String) -> Option<User>;
    async fn get_user(&self, username: String) -> Option<User>;
    async fn list_users(&self) -> Vec<User>;
}

pub struct TestUserRepo {
    users: HashMap<String, User>,
}

impl UserRepo for TestUserRepo {
    async fn new(_db_url: &str) -> Self {
        TestUserRepo {
            users: HashMap::new(),
        }
    }

    async fn add_user(&mut self, username: String, email: String) -> Result<User, String> {
        info!(target: "Users", "Adding user: {}", username);
        let user = User {
            id: Uuid::new_v4(),
            username: username.clone(),
            email,
        };

        match self.users.entry(user.id.to_string()) {
            Vacant(entry) => {
                let user_copy = entry.insert(user).clone();
                Ok(user_copy)
            }
            _ => Err(format!("User with username '{}' already exists", username)),
        }
    }

    async fn remove_user(&mut self, id: &String) -> Option<User> {
        info!(target: "Users", "Removing user: {}", id);
        self.users.remove(id)
    }

    async fn get_user(&self, id: String) -> Option<User> {
        info!(target: "Users", "Getting user: {}", id);
        self.users.get(&id).cloned()
    }

    async fn list_users(&self) -> Vec<User> {
        info!(target: "Users", "Getting list of users");
        self.users.values().cloned().collect()
    }
}