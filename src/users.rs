use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use log::{info, error};
use uuid::Uuid;
use sqlx::{SqlitePool, FromRow};

#[derive(Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

// Dedicated sqlx struct for database operations
#[derive(FromRow)]
struct SqlxUserRow {
    id: String,
    username: String,
    email: String,
}

impl From<SqlxUserRow> for User {
    fn from(row: SqlxUserRow) -> Self {
        User {
            id: Uuid::parse_str(&row.id).unwrap_or_else(|_| Uuid::new_v4()),
            username: row.username,
            email: row.email,
        }
    }
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

pub struct SqliteUserRepo {
    pool: SqlitePool,
}

impl UserRepo for SqliteUserRepo {
    async fn new(db_url: &str) -> Self {
        let pool = match SqlitePool::connect(db_url).await {
            Ok(pool) => pool,
            Err(e) => {
                error!("Failed to connect to SQLite at '{}': {}", db_url, e);
                panic!("Cannot create database connection");
            }
        };
        
        if let Err(e) = sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                email TEXT NOT NULL
            )"
        )
        .execute(&pool)
        .await
        {
            error!("Failed to create users table: {}", e);
            panic!("Cannot initialize database schema");
        }
        
        SqliteUserRepo { pool }
    }

    async fn add_user(&mut self, username: String, email: String) -> Result<User, String> {
        info!(target: "Users", "Adding user: {}", username);
        let id = Uuid::new_v4();
        let id_str = id.to_string();
        
        match sqlx::query(
            "INSERT INTO users (id, username, email) VALUES (?, ?, ?)"
        )
        .bind(&id_str)
        .bind(&username)
        .bind(&email)
        .execute(&self.pool)
        .await
        {
            Ok(_) => Ok(User {
                id,
                username,
                email,
            }),
            Err(e) => Err(format!("Failed to add user: {}", e)),
        }
    }

    async fn remove_user(&mut self, id: &String) -> Option<User> {
        info!(target: "Users", "Removing user: {}", id);
        
        // First, get the user before deleting
        let user = self.get_user(id.clone()).await;
        
        if user.is_some() {
            match sqlx::query("DELETE FROM users WHERE id = ?")
                .bind(id)
                .execute(&self.pool)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to execute DELETE in remove_user: {}", e);
                    return None;
                }
            }
        }
        
        user
    }

    async fn get_user(&self, id: String) -> Option<User> {
        info!(target: "Users", "Getting user: {}", id);
        
        match sqlx::query_as::<_, SqlxUserRow>(
            "SELECT id, username, email FROM users WHERE id = ?"
        )
        .bind(&id)
        .fetch_one(&self.pool)
        .await
        {
            Ok(row) => Some(row.into()),
            Err(e) => {
                info!("User not found: {}", e);
                None
            }
        }
    }

    async fn list_users(&self) -> Vec<User> {
        info!(target: "Users", "Getting list of users");
        
        match sqlx::query_as::<_, SqlxUserRow>(
            "SELECT id, username, email FROM users"
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(rows) => rows.into_iter().map(|row| row.into()).collect(),
            Err(e) => {
                error!("Failed to list users: {}", e);
                Vec::new()
            }
        }
    }
}