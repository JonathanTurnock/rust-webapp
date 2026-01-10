use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;
use crate::users::{User, UserRepo, UserRepoError};

pub struct SqliteUserRepo {
    pool: SqlitePool,
}

#[derive(FromRow, Debug)]
struct SqlxUserRow {
    id: String,
    username: String,
    email: String,
}

impl SqlxUserRow {
    fn try_into_user(self) -> Result<User, UserRepoError> {
        let id = Uuid::parse_str(&self.id).map_err(|e| UserRepoError::Unexpected(Box::new(e)))?;

        Ok(User {
            id,
            username: self.username,
            email: self.email,
        })
    }
}

impl SqliteUserRepo {
    pub async fn new(pool: SqlitePool) -> Result<Self, UserRepoError> {
        // Minimal schema init. Consider moving to migrations.
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id       TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                email    TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .map_err(map_sqlx_err)?;

        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl UserRepo for SqliteUserRepo {
    async fn add_user(&self, username: &str, email: &str) -> Result<User, UserRepoError> {
        log::debug!(target: "Users", "Adding user: {username}");

        let user = User {
            id: Uuid::new_v4(),
            username: username.to_owned(),
            email: email.to_owned(),
        };

        sqlx::query(r#"INSERT INTO users (id, username, email) VALUES (?, ?, ?)"#)
            .bind(user.id.to_string())
            .bind(&user.username)
            .bind(&user.email)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        Ok(user)
    }

    async fn get_user(&self, id: Uuid) -> Result<Option<User>, UserRepoError> {
        log::debug!(target: "Users", "Getting user: {id}");

        let row = sqlx::query_as::<_, SqlxUserRow>(
            r#"SELECT id, username, email FROM users WHERE id = ?"#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        row.map(SqlxUserRow::try_into_user).transpose()
    }

    async fn remove_user(&self, id: Uuid) -> Result<Option<User>, UserRepoError> {
        log::debug!(target: "Users", "Removing user: {id}");

        let mut tx = self.pool.begin().await.map_err(map_sqlx_err)?;

        let row = sqlx::query_as::<_, SqlxUserRow>(
            r#"SELECT id, username, email FROM users WHERE id = ?"#,
        )
        .bind(id.to_string())
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_sqlx_err)?;

        let user = match row {
            None => {
                tx.rollback().await.map_err(map_sqlx_err)?;
                return Ok(None);
            }
            Some(row) => row.try_into_user()?,
        };

        sqlx::query(r#"DELETE FROM users WHERE id = ?"#)
            .bind(id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_err)?;

        tx.commit().await.map_err(map_sqlx_err)?;

        Ok(Some(user))
    }

    async fn list_users(&self) -> Result<Vec<User>, UserRepoError> {
        log::debug!(target: "Users", "Listing users");

        let rows = sqlx::query_as::<_, SqlxUserRow>(r#"SELECT id, username, email FROM users"#)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        rows.into_iter().map(SqlxUserRow::try_into_user).collect()
    }
}

fn map_sqlx_err(e: sqlx::Error) -> UserRepoError {
    use sqlx::Error;

    match e {
        Error::PoolClosed | Error::PoolTimedOut => UserRepoError::Unavailable,
        Error::Database(db_err) => UserRepoError::Unexpected(db_err.into()),
        other => UserRepoError::Unexpected(Box::new(other)),
    }
}
