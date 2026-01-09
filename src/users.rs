use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use log::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

pub trait UserRepo {
    fn new() -> Self;
    fn add_user(&mut self, username: String, email: String) -> Result<&User, String>;
    fn remove_user(&mut self, username: &String) -> Option<User>;
    fn get_user(&self, username: String) -> Option<&User>;
    fn list_users(&self) -> Vec<&User>;
}

pub struct TestUserRepo {
    users: HashMap<String, User>,
}

impl UserRepo for TestUserRepo {
    fn new() -> Self {
        TestUserRepo {
            users: HashMap::new(),
        }
    }

    fn add_user(&mut self, username: String, email: String) -> Result<&User, String> {
        info!(target: "Users", "Adding user: {}", username);
        let user = User {
            id: Uuid::new_v4(),
            username: username.clone(),
            email,
        };

        match self.users.entry(user.id.to_string()) {
            Vacant(entry) => Ok(entry.insert(user)),
            _ => Err(format!("User with username '{}' already exists", username)),
        }
    }

    fn remove_user(&mut self, id: &String) -> Option<User> {
        info!(target: "Users", "Removing user: {}", id);
        self.users.remove(id)
    }

    fn get_user(&self, id: String) -> Option<&User> {
        info!(target: "Users", "Getting user: {}", id);
        self.users.get(&id)
    }

    fn list_users(&self) -> Vec<&User> {
        info!(target: "Users", "Getting list of users");
        self.users.values().collect()
    }
}