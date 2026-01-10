mod common;

use log::info;
use rust_webapp::app::Application;
use rust_webapp::users::UserRepo;

async fn scenario_add_user<R: UserRepo>(app: &mut Application<R>) {
    assert_eq!(app.users.list_users().await.expect("Failed to get users").len(), 0);
    app.users.add_user("johndoe".into(), "johndoe@example.com".into()).await.expect("Failed to add user");
    assert_eq!(app.users.list_users().await.expect("Failed to get users").len(), 1);
}

async fn scenario_remove_user<R: UserRepo>(app: &mut Application<R>) {
    let addeduser = app.users.add_user("janedoe".into(), "johndoe@example.com".into()).await.expect("Failed to add user");
    info!(target: "Users", "Removing user: {:?}", addeduser);
    let users = app.users.list_users().await.expect("Failed to get users");
    assert_eq!(users.len(), 1);
    let removed = app.users.remove_user(addeduser.id).await.expect("Failed to remove user");
    info!(target: "Users", "Removed user: {:?}", removed);
    assert!(removed.is_some());
    assert_eq!(app.users.list_users().await.expect("Failed to get users").len(), 0);
}

async fn scenario_list_users<R: UserRepo>(app: &mut Application<R>) {
    assert_eq!(app.users.list_users().await.expect("Failed to get users").len(), 0);
    app.users.add_user("alice".into(), "alice@example.com".into()).await.expect("Failed to add user");
    app.users.add_user("bob".into(), "bob@example.com".into()).await.expect("Failed to add user");
    let users = app.users.list_users().await.expect("Failed to get users");
    assert_eq!(users.len(), 2);
}

backend_tests!(scenario_add_user);
backend_tests!(scenario_remove_user);
backend_tests!(scenario_list_users);
