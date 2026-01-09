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
    use crate::users::{TestUserRepo, SqliteUserRepo};

    #[test]
    fn test_add_user() {
        let users = TestUserRepo::new();
        let mut app = Application::new(users);
        assert_eq!(app.users.list_users().len(), 0);
        app.users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .unwrap();
        assert_eq!(app.users.list_users().len(), 1);
    }

    #[test]
    fn test_get_user() {
        let users = TestUserRepo::new();
        let mut app = Application::new(users);
        let user = app
            .users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string()).unwrap();
        let fetched = app.users.get_user(user.id.to_string()).unwrap();
        assert_eq!(fetched.username, "johndoe");
        assert_eq!(fetched.email, "johndoe@example.com");
    }

    #[test]
    fn test_list_users() {
        let users = TestUserRepo::new();
        let mut app = Application::new(users);
        assert_eq!(app.users.list_users().len(), 0);
        app.users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string()).unwrap();
        app.users
            .add_user("janedoe".to_string(), "janedoe@example.com".to_string()).unwrap();
        assert_eq!(app.users.list_users().len(), 2);
    }

    #[test]
    fn test_remove_user() {
        let users = TestUserRepo::new();
        let mut app = Application::new(users);
        let user = app
            .users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .unwrap();
        assert_eq!(app.users.list_users().len(), 1);
        let returned = app.users.remove_user(&user.id.to_string()).unwrap();
        assert_eq!(returned.username, "johndoe");
        assert_eq!(returned.email, "johndoe@example.com");
        assert_eq!(app.users.list_users().len(), 0);
    }

    #[test]
    fn test_sqlite_add_user() {
        let users = SqliteUserRepo::new();
        let mut app = Application::new(users);
        assert_eq!(app.users.list_users().len(), 0);
        app.users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .unwrap();
        assert_eq!(app.users.list_users().len(), 1);
    }

    #[test]
    fn test_sqlite_get_user() {
        let users = SqliteUserRepo::new();
        let mut app = Application::new(users);
        let user = app
            .users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string()).unwrap();
        let fetched = app.users.get_user(user.id.to_string()).unwrap();
        assert_eq!(fetched.username, "johndoe");
        assert_eq!(fetched.email, "johndoe@example.com");
    }

    #[test]
    fn test_sqlite_list_users() {
        let users = SqliteUserRepo::new();
        let mut app = Application::new(users);
        assert_eq!(app.users.list_users().len(), 0);
        app.users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string()).unwrap();
        app.users
            .add_user("janedoe".to_string(), "janedoe@example.com".to_string()).unwrap();
        assert_eq!(app.users.list_users().len(), 2);
    }

    #[test]
    fn test_sqlite_remove_user() {
        let users = SqliteUserRepo::new();
        let mut app = Application::new(users);
        let user = app
            .users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .unwrap();
        assert_eq!(app.users.list_users().len(), 1);
        let returned = app.users.remove_user(&user.id.to_string()).unwrap();
        assert_eq!(returned.username, "johndoe");
        assert_eq!(returned.email, "johndoe@example.com");
        assert_eq!(app.users.list_users().len(), 0);
    }
}
