mod app;
mod users;

use crate::app::Application;
use crate::users::{User, UserRepo, SqliteUserRepo};
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;
use utoipa;
use utoipa::OpenApi;
use log::{info};
use actix_cors::Cors;

type _Application = Application<SqliteUserRepo>;
struct AppState {
    application: Mutex<_Application>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        get_users,
        create_user,
        get_user,
        delete_user
    ),
    components(
        schemas(UserDto, CreateUserDto)
    ),
    tags(
        (name = "users", description = "User management")
    )
)]
struct ApiDoc;

#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
struct UserDto {

    /// Unique identifier for the user
    /// example = "550e8400-e29b-41d4-a716-446655440000"
    /// format = "uuid"
    id: String,

    /// Username of the user
    /// example = "johndoe"
    username: String,

    /// Email address of the user
    /// example = "johndoe@example.com"
    email: String,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
struct CreateUserDto {
    username: String,
    email: String,
}

impl From<&User> for UserDto {
    fn from(user: &User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
        }
    }
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
        }
    }
}


#[utoipa::path(
    responses(
        (status = 200, description = "List all users", body = [UserDto])
    )
)]
#[get("/api/users")]
async fn get_users(data: Data<AppState>) -> impl Responder {
    info!("Fetching all users");
    let application = data.application.lock().unwrap();

    let users: Vec<User> = application.users.list_users().await;
    HttpResponse::Ok().json(users.into_iter().map(UserDto::from).collect::<Vec<_>>())
}

#[utoipa::path(
    request_body = CreateUserDto,
    responses(
        (status = 200, description = "User created successfully", body = UserDto),
        (status = 409, description = "User already exists")
    )
)]
#[post("/api/users")]
async fn create_user(data: Data<AppState>, user_dto: Json<CreateUserDto>) -> impl Responder {
    info!("Creating user: {}", user_dto.username);
    let mut application = data.application.lock().unwrap();

    let add_result =
        (application.users).add_user(user_dto.username.clone(), user_dto.email.clone()).await;

    match add_result {
        Ok(user) => HttpResponse::Ok().json(UserDto::from(user)),
        Err(err_msg) => HttpResponse::Conflict().body(err_msg),
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "Get user by ID", body = UserDto),
        (status = 404, description = "User not found")
    )
)]
#[get("/api/users/{id}")]
async fn get_user(data: Data<AppState>, id: Path<String>) -> impl Responder {
    info!("Fetching user: {}", id);
    let application = data.application.lock().unwrap();
    match application.users.get_user(id.clone()).await {
        Some(user) => HttpResponse::Ok().json(UserDto::from(user)),
        None => HttpResponse::NotFound().body("User not found"),
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "User deleted successfully", body = UserDto),
        (status = 404, description = "User not found")
    )
)]
#[delete("/api/users/{id}")]
async fn delete_user(data: Data<AppState>, id: Path<String>) -> impl Responder {
    info!("Deleting user: {}", id);
    let mut application = data.application.lock().unwrap();
    match application.users.remove_user(&id).await {
        Some(user) => HttpResponse::Ok().json(UserDto::from(user)),
        None => HttpResponse::NotFound().body("User not found"),
    }
}

#[get("/v3/api-docs")]
async fn api_docs() -> impl Responder {
    HttpResponse::Ok().json(ApiDoc::openapi())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let users_impl = SqliteUserRepo::new("sqlite:data.sqlite").await;
    let application = Application::new(users_impl);
    let data = Data::new(AppState {
        application: Mutex::new(application),
    });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
            )
            .app_data(data.clone())
            .service(get_users)
            .service(create_user)
            .service(get_user)
            .service(api_docs)
            .service(delete_user)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
