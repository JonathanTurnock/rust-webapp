use crate::users::{User, UserRepo, UserRepoError};
use async_trait::async_trait;
use futures_util::TryStreamExt;
use log::info;
use mongodb::bson::oid::ObjectId;
use mongodb::{
    bson::{doc, Document}, Collection,
    Database,
};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct MongoUserDoc {
    #[serde(rename = "_id")]
    id: ObjectId,
    uuid: String,
    username: String,
    email: String,
}

impl MongoUserDoc {
    fn from_user(user: &User) -> Self {
        Self {
            id: ObjectId::new(),
            uuid: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
        }
    }

    fn try_into_user(self) -> Result<User, UserRepoError> {
        Ok(User {
            id: Uuid::parse_str(&self.uuid).map_err(|e| UserRepoError::Unexpected(Box::new(e)))?,
            username: self.username,
            email: self.email,
        })
    }
}

pub struct MongoUserRepo {
    users: Collection<MongoUserDoc>,
}

impl MongoUserRepo {
    pub async fn new(db: Database) -> Self {
        let users = db.collection::<MongoUserDoc>("users");

        users
            .create_index(
                mongodb::IndexModel::builder()
                    .keys(doc! { "uuid": 1 })
                    .options(
                        mongodb::options::IndexOptions::builder()
                            .unique(true)
                            .build(),
                    )
                    .build(),
            )
            .await
            .expect("Failed to create index on 'uuid' field");

        Self { users }
    }
}

fn map_mongo_err(e: mongodb::error::Error) -> UserRepoError {
    // Mongo error typing varies between versions; message string is the stable fallback.
    let msg = e.to_string();

    // “Unavailable” bucket: can’t talk to mongo.
    if msg.contains("server selection")
        || msg.contains("timed out")
        || msg.contains("timeout")
        || msg.contains("connection")
        || msg.contains("Connection refused")
        || msg.contains("pool")
    {
        return UserRepoError::Unavailable;
    }

    UserRepoError::Unexpected(Box::new(e))
}

#[async_trait]
impl UserRepo for MongoUserRepo {
    async fn add_user(&self, username: &str, email: &str) -> Result<User, UserRepoError> {
        info!(target: "Users", "Adding user: {}", username);

        let user = User {
            id: Uuid::new_v4(),
            username: username.to_owned(),
            email: email.to_owned(),
        };

        self.users
            .insert_one(MongoUserDoc::from_user(&user))
            .await
            .map_err(map_mongo_err)?;

        Ok(user)
    }

    async fn get_user(&self, id: Uuid) -> Result<Option<User>, UserRepoError> {
        info!(target: "Users", "Getting user: {}", id);

        let doc_opt = self
            .users
            .find_one(doc! { "uuid": id.to_string() })
            .await
            .map_err(map_mongo_err)?;

        doc_opt.map(MongoUserDoc::try_into_user).transpose()
    }

    async fn remove_user(&self, id: Uuid) -> Result<Option<User>, UserRepoError> {
        info!(target: "Users", "Removing user: {}", id);

        let doc_opt = self
            .users
            .find_one_and_delete(doc! { "uuid": id.to_string() })
            .await
            .map_err(map_mongo_err)?;

        doc_opt.map(MongoUserDoc::try_into_user).transpose()
    }

    async fn list_users(&self) -> Result<Vec<User>, UserRepoError> {
        info!(target: "Users", "Listing users");

        let cursor = self
            .users
            .find(Document::new())
            .await
            .map_err(map_mongo_err)?;

        let docs: Vec<MongoUserDoc> = cursor.try_collect().await.map_err(map_mongo_err)?;

        docs.into_iter().map(MongoUserDoc::try_into_user).collect()
    }
}
