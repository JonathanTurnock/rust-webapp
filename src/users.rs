use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use log::{info, error};
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
    fn add_user(&mut self, username: String, email: String) -> Result<User, String>;
    fn remove_user(&mut self, username: &String) -> Option<User>;
    fn get_user(&self, username: String) -> Option<User>;
    fn list_users(&self) -> Vec<User>;
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

    fn add_user(&mut self, username: String, email: String) -> Result<User, String> {
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

    fn remove_user(&mut self, id: &String) -> Option<User> {
        info!(target: "Users", "Removing user: {}", id);
        self.users.remove(id)
    }

    fn get_user(&self, id: String) -> Option<User> {
        info!(target: "Users", "Getting user: {}", id);
        self.users.get(&id).cloned()
    }

    fn list_users(&self) -> Vec<User> {
        info!(target: "Users", "Getting list of users");
        self.users.values().cloned().collect()
    }
}

pub struct SqliteUserRepo {
    connection: Connection,
}

impl UserRepo for SqliteUserRepo {
    fn new() -> Self {
        let connection = match sqlite::open(":memory:") {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to open SQLite connection: {}", e);
                panic!("Cannot create database connection");
            }
        };
        
        if let Err(e) = connection.execute(
            "
            CREATE TABLE users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                email TEXT NOT NULL
            )
            ",
        ) {
            error!("Failed to create users table: {}", e);
            panic!("Cannot initialize database schema");
        }
        
        SqliteUserRepo { 
            connection,
        }
    }

    fn add_user(&mut self, username: String, email: String) -> Result<User, String> {
        info!(target: "Users", "Adding user: {}", username);
        let id = Uuid::new_v4();
        let id_str = id.to_string();
        
        let query = "INSERT INTO users (id, username, email) VALUES (?, ?, ?)";
        let mut statement = match self.connection.prepare(query) {
            Ok(stmt) => stmt,
            Err(e) => return Err(format!("Failed to prepare statement: {}", e)),
        };
        
        if let Err(e) = statement.bind((1, id_str.as_str())) {
            return Err(format!("Failed to bind id: {}", e));
        }
        if let Err(e) = statement.bind((2, username.as_str())) {
            return Err(format!("Failed to bind username: {}", e));
        }
        if let Err(e) = statement.bind((3, email.as_str())) {
            return Err(format!("Failed to bind email: {}", e));
        }
        
        match statement.next() {
            Ok(_) => {
                Ok(User {
                    id,
                    username,
                    email,
                })
            }
            Err(err) => Err(format!("Failed to add user: {}", err)),
        }
    }

    fn remove_user(&mut self, id: &String) -> Option<User> {
        info!(target: "Users", "Removing user: {}", id);
        
        // First, get the user before deleting
        let user = self.get_user(id.clone());
        
        if user.is_some() {
            let query = "DELETE FROM users WHERE id = ?";
            match self.connection.prepare(query) {
                Ok(mut statement) => {
                    if let Err(e) = statement.bind((1, id.as_str())) {
                        error!("Failed to bind id in remove_user: {}", e);
                        return None;
                    }
                    if let Err(e) = statement.next() {
                        error!("Failed to execute DELETE in remove_user: {}", e);
                        return None;
                    }
                }
                Err(e) => {
                    error!("Failed to prepare DELETE statement in remove_user: {}", e);
                    return None;
                }
            }
        }
        
        user
    }

    fn get_user(&self, id: String) -> Option<User> {
        info!(target: "Users", "Getting user: {}", id);
        
        let query = "SELECT id, username, email FROM users WHERE id = ?";
        let mut statement = match self.connection.prepare(query) {
            Ok(stmt) => stmt,
            Err(e) => {
                error!("Failed to prepare get_user statement: {}", e);
                return None;
            }
        };
        
        if let Err(e) = statement.bind((1, id.as_str())) {
            error!("Failed to bind id in get_user: {}", e);
            return None;
        }
        
        if let Ok(State::Row) = statement.next() {
            let id_str = match statement.read::<String, _>("id") {
                Ok(val) => val,
                Err(e) => {
                    error!("Failed to read id: {}", e);
                    return None;
                }
            };
            let username = match statement.read::<String, _>("username") {
                Ok(val) => val,
                Err(e) => {
                    error!("Failed to read username: {}", e);
                    return None;
                }
            };
            let email = match statement.read::<String, _>("email") {
                Ok(val) => val,
                Err(e) => {
                    error!("Failed to read email: {}", e);
                    return None;
                }
            };
            let id = match Uuid::parse_str(&id_str) {
                Ok(val) => val,
                Err(e) => {
                    error!("Failed to parse UUID: {}", e);
                    return None;
                }
            };
            
            Some(User {
                id,
                username,
                email,
            })
        } else {
            None
        }
    }

    fn list_users(&self) -> Vec<User> {
        info!(target: "Users", "Getting list of users");
        
        let query = "SELECT id, username, email FROM users";
        let mut statement = match self.connection.prepare(query) {
            Ok(stmt) => stmt,
            Err(e) => {
                error!("Failed to prepare list_users statement: {}", e);
                return Vec::new();
            }
        };
        
        let mut users = Vec::new();
        while let Ok(State::Row) = statement.next() {
            let id_str = match statement.read::<String, _>("id") {
                Ok(val) => val,
                Err(e) => {
                    error!("Failed to read id in list_users: {}", e);
                    continue;
                }
            };
            let username = match statement.read::<String, _>("username") {
                Ok(val) => val,
                Err(e) => {
                    error!("Failed to read username in list_users: {}", e);
                    continue;
                }
            };
            let email = match statement.read::<String, _>("email") {
                Ok(val) => val,
                Err(e) => {
                    error!("Failed to read email in list_users: {}", e);
                    continue;
                }
            };
            let id = match Uuid::parse_str(&id_str) {
                Ok(val) => val,
                Err(e) => {
                    error!("Failed to parse UUID in list_users: {}", e);
                    continue;
                }
            };
            
            users.push(User {
                id,
                username,
                email,
            });
        }
        
        users
    }
}