use crate::users::{User, UserRepo};
use log::{info, error};
use uuid::Uuid;
use sqlx::{SqlitePool, FromRow};

// Dedicated sqlx struct for database operations
#[derive(FromRow)]
struct SqlxUserRow {
    id: String,
    username: String,
    email: String,
}

impl From<SqlxUserRow> for User {
    fn from(row: SqlxUserRow) -> Self {
        let id = Uuid::parse_str(&row.id).unwrap_or_else(|e| {
            error!("Failed to parse UUID '{}': {}. Generating new UUID.", row.id, e);
            Uuid::new_v4()
        });
        User {
            id,
            username: row.username,
            email: row.email,
        }
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
            Err(sqlx::Error::RowNotFound) => {
                info!("User not found: {}", id);
                None
            }
            Err(e) => {
                error!("Database error while fetching user {}: {}", id, e);
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
