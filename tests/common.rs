#[macro_export] macro_rules! backend_tests {
    ($scenario:ident) => {
        mod $scenario {
            use rust_webapp::app::Application;

            use rust_webapp::adapters::mongo::MongoUserRepo;
            use rust_webapp::adapters::sqlite::SqliteUserRepo;
            use testcontainers::{runners::AsyncRunner, GenericImage};
            use std::sync::Once;
            use log::error;
            use log4rs;
            use sqlx::SqlitePool;
            use rust_webapp::adapters::memory::MemoryUserRepo;

            static LOG_INIT: Once = Once::new();

            fn init_log4rs() {
                LOG_INIT.call_once(|| {
                    log4rs::init_file("log4rs_tests.yaml", Default::default()).or_else(|e| {
                        error!("Failed to initialize log4rs: {}", e);
                        Err(e)
                    }).ok();
                })
            }

            #[tokio::test]
            async fn in_memory() {
                init_log4rs();
                let users = MemoryUserRepo::new();
                let mut app = Application::new(users);
                super::$scenario(&mut app).await;
            }

            #[tokio::test]
            async fn sqlite() {
                init_log4rs();
                let pool = SqlitePool::connect("sqlite::memory:").await.expect("Failed to create SQLite in-memory database");
                let users = SqliteUserRepo::new(pool).await.expect("Failed to create SQLiteUserRepo");
                let mut app = Application::new(users);
                super::$scenario(&mut app).await;
            }

            #[tokio::test]
            async fn mongo() {
                init_log4rs();
                let mongo_container = GenericImage::new("mongo", "7.0")
                    .with_exposed_port(27017.into())
                    .start()
                    .await
                    .expect("Failed to start MongoDB container");

                let mongo_port = mongo_container.get_host_port_ipv4(27017).await.expect("Failed to get MongoDB port");
                let mongo_url = format!("mongodb://localhost:{}", mongo_port);
                let db = mongodb::Client::with_uri_str(&mongo_url)
                    .await
                    .expect("Failed to connect to MongoDB")
                    .database("test_db");
                let users = MongoUserRepo::new(db).await;
                let mut app = Application::new(users);
                super::$scenario(&mut app).await;
            }
        }
    };
}