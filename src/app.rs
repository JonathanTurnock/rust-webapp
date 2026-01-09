use crate::users::UserRepo;

pub struct Application<U: UserRepo> {
    pub users: U,
}

impl<U: UserRepo> Application<U> {
    pub fn new(users: U) -> Self {
        Application { users }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::users::{TestUserRepo, SqliteUserRepo, MongoUserRepo};
    use testcontainers::{runners::AsyncRunner, GenericImage};

    #[tokio::test]
    async fn test_add_user() {
        let users = TestUserRepo::new(":memory:").await;
        let mut app = Application::new(users);
        assert_eq!(app.users.list_users().await.len(), 0);
        app.users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .await
            .unwrap();
        assert_eq!(app.users.list_users().await.len(), 1);
    }

    #[tokio::test]
    async fn test_get_user() {
        let users = TestUserRepo::new(":memory:").await;
        let mut app = Application::new(users);
        let user = app
            .users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .await
            .unwrap();
        let fetched = app.users.get_user(user.id.to_string()).await.unwrap();
        assert_eq!(fetched.username, "johndoe");
        assert_eq!(fetched.email, "johndoe@example.com");
    }

    #[tokio::test]
    async fn test_list_users() {
        let users = TestUserRepo::new(":memory:").await;
        let mut app = Application::new(users);
        assert_eq!(app.users.list_users().await.len(), 0);
        app.users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .await
            .unwrap();
        app.users
            .add_user("janedoe".to_string(), "janedoe@example.com".to_string())
            .await
            .unwrap();
        assert_eq!(app.users.list_users().await.len(), 2);
    }

    #[tokio::test]
    async fn test_remove_user() {
        let users = TestUserRepo::new(":memory:").await;
        let mut app = Application::new(users);
        let user = app
            .users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .await
            .unwrap();
        assert_eq!(app.users.list_users().await.len(), 1);
        let returned = app.users.remove_user(&user.id.to_string()).await.unwrap();
        assert_eq!(returned.username, "johndoe");
        assert_eq!(returned.email, "johndoe@example.com");
        assert_eq!(app.users.list_users().await.len(), 0);
    }

    #[tokio::test]
    async fn test_sqlite_add_user() {
        let users = SqliteUserRepo::new("sqlite::memory:").await;
        let mut app = Application::new(users);
        assert_eq!(app.users.list_users().await.len(), 0);
        app.users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .await
            .unwrap();
        assert_eq!(app.users.list_users().await.len(), 1);
    }

    #[tokio::test]
    async fn test_sqlite_get_user() {
        let users = SqliteUserRepo::new("sqlite::memory:").await;
        let mut app = Application::new(users);
        let user = app
            .users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .await
            .unwrap();
        let fetched = app.users.get_user(user.id.to_string()).await.unwrap();
        assert_eq!(fetched.username, "johndoe");
        assert_eq!(fetched.email, "johndoe@example.com");
    }

    #[tokio::test]
    async fn test_sqlite_list_users() {
        let users = SqliteUserRepo::new("sqlite::memory:").await;
        let mut app = Application::new(users);
        assert_eq!(app.users.list_users().await.len(), 0);
        app.users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .await
            .unwrap();
        app.users
            .add_user("janedoe".to_string(), "janedoe@example.com".to_string())
            .await
            .unwrap();
        assert_eq!(app.users.list_users().await.len(), 2);
    }

    #[tokio::test]
    async fn test_sqlite_remove_user() {
        let users = SqliteUserRepo::new("sqlite::memory:").await;
        let mut app = Application::new(users);
        let user = app
            .users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .await
            .unwrap();
        assert_eq!(app.users.list_users().await.len(), 1);
        let returned = app.users.remove_user(&user.id.to_string()).await.unwrap();
        assert_eq!(returned.username, "johndoe");
        assert_eq!(returned.email, "johndoe@example.com");
        assert_eq!(app.users.list_users().await.len(), 0);
    }

    #[tokio::test]
    async fn test_mongo_add_user() {
        let mongo_container = GenericImage::new("mongo", "7.0")
            .with_exposed_port(27017.into())
            .start()
            .await
            .expect("Failed to start MongoDB container");
        
        let mongo_port = mongo_container.get_host_port_ipv4(27017).await.expect("Failed to get MongoDB port");
        let mongo_url = format!("mongodb://localhost:{}", mongo_port);
        
        let users = MongoUserRepo::new(&mongo_url).await;
        let mut app = Application::new(users);
        assert_eq!(app.users.list_users().await.len(), 0);
        app.users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .await
            .unwrap();
        assert_eq!(app.users.list_users().await.len(), 1);
    }

    #[tokio::test]
    async fn test_mongo_get_user() {
        let mongo_container = GenericImage::new("mongo", "7.0")
            .with_exposed_port(27017.into())
            .start()
            .await
            .expect("Failed to start MongoDB container");
        
        let mongo_port = mongo_container.get_host_port_ipv4(27017).await.expect("Failed to get MongoDB port");
        let mongo_url = format!("mongodb://localhost:{}", mongo_port);
        
        let users = MongoUserRepo::new(&mongo_url).await;
        let mut app = Application::new(users);
        let user = app
            .users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .await
            .unwrap();
        let fetched = app.users.get_user(user.id.to_string()).await.unwrap();
        assert_eq!(fetched.username, "johndoe");
        assert_eq!(fetched.email, "johndoe@example.com");
    }

    #[tokio::test]
    async fn test_mongo_list_users() {
        let mongo_container = GenericImage::new("mongo", "7.0")
            .with_exposed_port(27017.into())
            .start()
            .await
            .expect("Failed to start MongoDB container");
        
        let mongo_port = mongo_container.get_host_port_ipv4(27017).await.expect("Failed to get MongoDB port");
        let mongo_url = format!("mongodb://localhost:{}", mongo_port);
        
        let users = MongoUserRepo::new(&mongo_url).await;
        let mut app = Application::new(users);
        assert_eq!(app.users.list_users().await.len(), 0);
        app.users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .await
            .unwrap();
        app.users
            .add_user("janedoe".to_string(), "janedoe@example.com".to_string())
            .await
            .unwrap();
        assert_eq!(app.users.list_users().await.len(), 2);
    }

    #[tokio::test]
    async fn test_mongo_remove_user() {
        let mongo_container = GenericImage::new("mongo", "7.0")
            .with_exposed_port(27017.into())
            .start()
            .await
            .expect("Failed to start MongoDB container");
        
        let mongo_port = mongo_container.get_host_port_ipv4(27017).await.expect("Failed to get MongoDB port");
        let mongo_url = format!("mongodb://localhost:{}", mongo_port);
        
        let users = MongoUserRepo::new(&mongo_url).await;
        let mut app = Application::new(users);
        let user = app
            .users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .await
            .unwrap();
        assert_eq!(app.users.list_users().await.len(), 1);
        let returned = app.users.remove_user(&user.id.to_string()).await.unwrap();
        assert_eq!(returned.username, "johndoe");
        assert_eq!(returned.email, "johndoe@example.com");
        assert_eq!(app.users.list_users().await.len(), 0);
    }
}
