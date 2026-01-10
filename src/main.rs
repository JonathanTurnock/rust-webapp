pub mod adapters;
pub mod app;
pub mod users;

use crate::adapters::sqlite::SqliteUserRepo;
use crate::app::Application;
use crate::users::{User, UserRepo, UserRepoError};
use actix_cors::Cors;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, App, HttpServer, Responder, ResponseError, Result};
use log::{error, info};
use std::io;
use sqlx::migrate::MigrateDatabase;
use sqlx::Sqlite;
use thiserror::Error;
use utoipa;
use utoipa::OpenApi;
use uuid::Uuid;

struct AppState {
    application: Application<SqliteUserRepo>,
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

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
        }
    }
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("conflict")]
    Conflict,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("internal server error")]
    Internal,

    #[error("not found")]
    NotFound,
}

impl From<UserRepoError> for ApiError {
    fn from(e: UserRepoError) -> Self {
        match e {
            UserRepoError::Conflict { .. } => ApiError::Conflict,
            UserRepoError::Unavailable => ApiError::Internal, // or ServiceUnavailable if you add it
            UserRepoError::Unexpected(_) => ApiError::Internal,
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Conflict => StatusCode::CONFLICT,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

#[utoipa::path(
    responses(
        (status = 200, description = "List all users", body = [UserDto])
    )
)]
#[get("/api/users")]
async fn get_users(data: Data<AppState>) -> Result<Json<Vec<UserDto>>, ApiError> {
    info!("Fetching all users");
    let users = data.application.users.list_users().await?;
    let users_dto: Vec<UserDto> = users.into_iter().map(UserDto::from).collect();
    Ok(Json(users_dto))
}

#[utoipa::path(
    request_body = CreateUserDto,
    responses(
        (status = 200, description = "User created successfully", body = UserDto),
        (status = 409, description = "User already exists")
    )
)]
#[post("/api/users")]
async fn create_user(data: Data<AppState>, user_dto: Json<CreateUserDto>) -> Result<Json<UserDto>, ApiError> {
    info!("Creating user: {}", user_dto.username);

    let user = data
        .application
        .users
        .add_user(&user_dto.username, &user_dto.email)
        .await?;

    Ok(Json(UserDto::from(user)))
}

#[utoipa::path(
    responses(
        (status = 200, description = "Get user by ID", body = UserDto),
        (status = 404, description = "User not found")
    )
)]
#[get("/api/users/{id}")]
async fn get_user(data: Data<AppState>, id: Path<Uuid>) -> Result<Json<UserDto>, ApiError> {
    info!("Fetching user: {}", id);
    let user = data.application.users.get_user(*id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(UserDto::from(user)))
}

#[utoipa::path(
    responses(
        (status = 200, description = "User deleted successfully", body = UserDto),
        (status = 404, description = "User not found")
    )
)]
#[delete("/api/users/{id}")]
async fn delete_user(data: Data<AppState>, id: Path<Uuid>) -> Result<Json<UserDto>, ApiError> {
    info!("Deleting user: {}", id);

    let removed = data.application.users.remove_user(*id).await?;

    let user = removed.ok_or(ApiError::NotFound)?;
    Ok(Json(UserDto::from(user)))
}

#[get("/v3/api-docs")]
async fn api_docs() -> impl Responder {
    Json(ApiDoc::openapi())
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    Sqlite::create_database("data.sqlite").await.map_err(|e| {
        error!("Failed to create SQLite database: {}", e);
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    let pool = sqlx::SqlitePool::connect("sqlite:data.sqlite")
        .await
        .map_err(|e| {
            error!("Failed to connect to SQLite database: {}", e);
            io::Error::new(io::ErrorKind::Other, e)
        })?;

    let users_impl = SqliteUserRepo::new(pool).await.map_err(|e| {
        error!("Failed to initialize SQLiteUserRepo: {}", e);
        io::Error::new(io::ErrorKind::Other, e)
    })?;

    let application = Application::new(users_impl);
    let data = Data::new(AppState { application });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
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
