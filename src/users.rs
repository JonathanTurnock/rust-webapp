use std::error::Error;
use std::fmt;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ConflictField { Username, Email }

#[derive(Debug)]
pub enum UserRepoError {
    /// Database is unavailable / network / pool closed / timeouts, etc.
    Unavailable,

    /// The request was valid, but violates a constraint (unique username/email).
    Conflict {
        field: ConflictField,
        value: String,
    },

    /// Everything else you didn’t classify yet.
    /// Keep source for logs/telemetry, but don’t leak it to callers.
    Unexpected(Box<dyn Error + Send + Sync>),
}

impl UserRepoError {
    pub fn unexpected<E>(e: E) -> Self
    where E: Error + Send + Sync + 'static,
    {
        UserRepoError::Unexpected(Box::new(e))
    }
}

impl fmt::Display for UserRepoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => write!(f, "repository unavailable"),
            Self::Conflict { field, .. } => write!(f, "conflict on field {:?}", field),
            Self::Unexpected(_) => write!(f, "unexpected repository error"),
        }
    }
}

impl std::error::Error for UserRepoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unexpected(e) => Some(&**e),
            _ => None,
        }
    }
}

#[async_trait::async_trait]
pub trait UserRepo: Send + Sync {
    async fn add_user(&self, username: &str, email: &str) -> Result<User, UserRepoError>;
    async fn get_user(&self, id: Uuid) -> Result<Option<User>, UserRepoError>;
    async fn remove_user(&self, id: Uuid) -> Result<Option<User>, UserRepoError>;
    async fn list_users(&self) -> Result<Vec<User>, UserRepoError>;
}
