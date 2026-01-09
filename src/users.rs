use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use log::info;
use uuid::Uuid;
use sqlite::{Connection, State};

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

pub struct SqliteUserRepo {
    connection: Connection,
    cache: HashMap<String, User>,
}

impl SqliteUserRepo {
    fn sync_from_db(&mut self) {
        self.cache.clear();
        let query = "SELECT id, username, email FROM users";
        let mut statement = self.connection.prepare(query).unwrap();
        
        while let Ok(State::Row) = statement.next() {
            let id_str = statement.read::<String, _>("id").unwrap();
            let username = statement.read::<String, _>("username").unwrap();
            let email = statement.read::<String, _>("email").unwrap();
            let id = Uuid::parse_str(&id_str).unwrap();
            
            let user = User {
                id,
                username,
                email,
            };
            self.cache.insert(id_str, user);
        }
    }
}

impl UserRepo for SqliteUserRepo {
    fn new() -> Self {
        let connection = sqlite::open(":memory:").unwrap();
        
        connection
            .execute(
                "
                CREATE TABLE users (
                    id TEXT PRIMARY KEY,
                    username TEXT NOT NULL,
                    email TEXT NOT NULL
                )
                ",
            )
            .unwrap();
        
        SqliteUserRepo { 
            connection,
            cache: HashMap::new(),
        }
    }

    fn add_user(&mut self, username: String, email: String) -> Result<&User, String> {
        info!(target: "Users", "Adding user: {}", username);
        let id = Uuid::new_v4();
        let id_str = id.to_string();
        
        let query = "INSERT INTO users (id, username, email) VALUES (?, ?, ?)";
        let mut statement = self.connection.prepare(query).unwrap();
        statement.bind((1, id_str.as_str())).unwrap();
        statement.bind((2, username.as_str())).unwrap();
        statement.bind((3, email.as_str())).unwrap();
        
        match statement.next() {
            Ok(_) => {
                let user = User {
                    id,
                    username: username.clone(),
                    email: email.clone(),
                };
                self.cache.insert(id_str.clone(), user);
                Ok(self.cache.get(&id_str).unwrap())
            }
            Err(err) => Err(format!("Failed to add user: {}", err)),
        }
    }

    fn remove_user(&mut self, id: &String) -> Option<User> {
        info!(target: "Users", "Removing user: {}", id);
        
        let query = "DELETE FROM users WHERE id = ?";
        let mut statement = self.connection.prepare(query).unwrap();
        statement.bind((1, id.as_str())).unwrap();
        
        match statement.next() {
            Ok(_) => self.cache.remove(id),
            Err(_) => None,
        }
    }

    fn get_user(&self, id: String) -> Option<&User> {
        info!(target: "Users", "Getting user: {}", id);
        self.cache.get(&id)
    }

    fn list_users(&self) -> Vec<&User> {
        info!(target: "Users", "Getting list of users");
        self.cache.values().collect()
    }
}