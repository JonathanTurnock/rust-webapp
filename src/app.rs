use crate::users::UserRepo;

pub struct Application<U: UserRepo> {
    pub users: U,
}

impl<U: UserRepo> Application<U> {
    pub fn new(users: U) -> Self {
        Application { users }
    }
}

