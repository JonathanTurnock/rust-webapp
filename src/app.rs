use crate::users::Users;

pub struct Application<U: Users> {
    pub users: U,
}

impl<U: Users> Application<U> {
    pub fn new(users: U) -> Self {
        Application { users }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::users::UsersImpl;

    #[test]
    fn test_add_user() {
        let users = UsersImpl::new();
        let mut app = Application::new(users);
        assert_eq!(app.users.list_users().len(), 0);
        app.users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .unwrap();
        assert_eq!(app.users.list_users().len(), 1);
    }

    #[test]
    fn test_get_user() {
        let users = UsersImpl::new();
        let mut app = Application::new(users);
        let user = app
            .users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string()).unwrap().clone();
        let fetched = app.users.get_user(user.id.to_string()).unwrap();
        assert_eq!(fetched.username, "johndoe");
        assert_eq!(fetched.email, "johndoe@example.com");
    }

    #[test]
    fn test_list_users() {
        let users = UsersImpl::new();
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
        let users = UsersImpl::new();
        let mut app = Application::new(users);
        let user = app
            .users
            .add_user("johndoe".to_string(), "johndoe@example.com".to_string())
            .unwrap().clone();
        assert_eq!(app.users.list_users().len(), 1);
        let returned = app.users.remove_user(&user.id.to_string()).unwrap();
        assert_eq!(returned.username, "johndoe");
        assert_eq!(returned.email, "johndoe@example.com");
        assert_eq!(app.users.list_users().len(), 0);
    }
}
