use crate::cache::cache_helper::flush_cache;
use crate::config::app_config::AppState;
use crate::config::db_connection::connect_to_db;
use crate::responses::error_msgs::ModuleError;
use crate::tasks::task_manager::TaskManager;
use crate::utils::struct_validation::custom_error_handler;
use actix_web::{App, HttpServer, middleware::DefaultHeaders, web};
use crypsol_logger::log;
use crypsol_logger::logs::initialize_logs;
use dotenvy::dotenv;
use lazy_static::lazy_static;
use log::Level;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::config::redis_connection::initialize_redis_connection;

mod cache;
mod config;
mod entities;
mod handlers;
mod middlewares;
mod responses;
mod routes;
mod services;
mod structs;
mod tasks;
mod utils;

lazy_static! {
    static ref TASK_MANAGER: Mutex<TaskManager> = Mutex::new(TaskManager::new(None, None));
}

#[actix_web::main]
async fn main() -> Result<(), ModuleError> {
    dotenv().ok();

    // Logs initializing
    initialize_logs();

    // PostgreSQL Connection
    let db_connection = connect_to_db().await?;
    // Redis Connection Manager
    let redis_data = initialize_redis_connection().await?;

    // Initialize the Task Manager with the connection pools
    {
        let mut task_manager = TASK_MANAGER.lock().await;
        task_manager.redis_pool = redis_data.clone();
        task_manager.pg_pool = Some(db_connection.clone());
    }

    // Start the task manager
    TASK_MANAGER.lock().await.start_manager();

    // Server host and port
    let port = env::var("SERVER_PORT").expect("SERVER_PORT is not set in .env file");
    let host = env::var("SERVER_HOST").expect("SERVER_HOST is not set in .env file");

    if let Some(pool) = &redis_data {
        log!(Level::Info, "Flushing Redis Cache for Coins module V2.");
        let _ = flush_cache(pool).await;
    }
    // Making dynamic server URL by using host and port
    let server_url = format!("{host}:{port}");

    log!(Level::Info, "Application is starting");

    let shared_state = Arc::new(AppState { db: db_connection, redis: redis_data });
    let server = HttpServer::new(move || {
        let app = App::new()
            .wrap(
                DefaultHeaders::new()
                    .add(("Content-Security-Policy", "default-src 'self'"))
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add(("X-Frame-Options", "DENY"))
                    .add(("Strict-Transport-Security", "max-age=31536000; includeSubDomains"))
                    .add(("X-XSS-Protection", "1; mode=block"))
                    .add(("Referrer-Policy", "no-referrer"))
                    .add(("Expect-CT", "max-age=86400, enforce")),
            )
            .app_data(web::JsonConfig::default().error_handler(custom_error_handler))
            .app_data(web::Data::new(shared_state.clone()));

        // if let Some(pool) = &redis_pool.unwrap().as_ref() {
        //     log!(Level::Info, "Attaching Redis pool to App data");
        //     app = app.app_data(web::Data::new(pool.clone()));
        // }

        app.configure(routes::route_list::config)
    })
    .bind(&server_url)?
    .run();

    // Log after server successfully starts
    log!(Level::Info, "Application is Started Successfully on {}", server_url);

    server.await?;

    Ok(())
}
