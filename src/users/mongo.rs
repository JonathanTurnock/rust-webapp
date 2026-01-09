use crate::users::{User, UserRepo};
use log::{info, error};
use uuid::Uuid;
use mongodb::{Client, Collection, bson::doc};
use serde::{Serialize, Deserialize};
use futures_util::stream::TryStreamExt;

// MongoDB document struct
#[derive(Debug, Serialize, Deserialize)]
struct MongoUserDoc {
    #[serde(rename = "_id")]
    id: String,
    username: String,
    email: String,
}

impl From<MongoUserDoc> for User {
    fn from(doc: MongoUserDoc) -> Self {
        let id = Uuid::parse_str(&doc.id).unwrap_or_else(|e| {
            error!("Failed to parse UUID '{}': {}. Generating new UUID.", doc.id, e);
            Uuid::new_v4()
        });
        User {
            id,
            username: doc.username,
            email: doc.email,
        }
    }
}

pub struct MongoUserRepo {
    collection: Collection<MongoUserDoc>,
}

impl UserRepo for MongoUserRepo {
    async fn new(db_url: &str) -> Self {
        let client = match Client::with_uri_str(db_url).await {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to connect to MongoDB at '{}': {}", db_url, e);
                panic!("Cannot create database connection");
            }
        };
        
        let database = client.database("rust_webapp");
        let collection = database.collection::<MongoUserDoc>("users");
        
        MongoUserRepo { collection }
    }

    async fn add_user(&mut self, username: String, email: String) -> Result<User, String> {
        info!(target: "Users", "Adding user: {}", username);
        let id = Uuid::new_v4();
        let id_str = id.to_string();
        
        let doc = MongoUserDoc {
            id: id_str,
            username: username.clone(),
            email: email.clone(),
        };
        
        match self.collection.insert_one(doc).await {
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
            match self.collection.delete_one(doc! { "_id": id }).await {
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
        
        match self.collection.find_one(doc! { "_id": &id }).await {
            Ok(Some(doc)) => Some(doc.into()),
            Ok(None) => {
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
        
        match self.collection.find(doc! {}).await {
            Ok(mut cursor) => {
                let mut users = Vec::new();
                while let Ok(Some(doc)) = cursor.try_next().await {
                    users.push(doc.into());
                }
                users
            }
            Err(e) => {
                error!("Failed to list users: {}", e);
                Vec::new()
            }
        }
    }
}
