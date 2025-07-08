use crate::responses::error_msgs::Error;
use crypsol_logger::{Level, log};
use deadpool_redis::{Config, Pool, Runtime};
use std::env;

/// Initializes Redis connection pool for caching and pings the server.
// pub async fn initialize_redis_connection() -> Result<Option<Pool>, Error> {
//     log!(Level::Info, "Initializing Redis connection");
//
//     // Check if Redis is enabled via environment variable (default to REDIS_ENABLED_DEFAULT).
//     let redis_enabled = env::var("REDIS_ENABLED").map(|v| v == "true").unwrap_or(REDIS_ENABLED_DEFAULT);
//
//     if !redis_enabled {
//         log!(Level::Info, "Redis disabled via environment config");
//         return Ok(None);
//     }
//
//     // Retrieve Redis URL from environment or fallback to default.
//     let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| REDIS_URL_DEFAULT.to_string());
//     if redis_url.is_empty() {
//         log!(Level::Error, "REDIS_URL is empty");
//         return Err(Error::InvalidConfiguration);
//     }
//
//     // Create Redis configuration and connection pool.
//     let cfg = Config::from_url(redis_url);
//     let pool = cfg.create_pool(Some(Runtime::Tokio1)).map_err(|e| {
//         log!(Level::Error, "Failed to create Redis pool: {:?}", e);
//         Error::RedisIssue
//     })?;
//
//     // Acquire a connection from the pool and ping Redis to verify connectivity.
//     let mut conn = pool.get().await.map_err(|e| {
//         log!(Level::Error, "Failed to get Redis connection from pool: {:?}", e);
//         Error::RedisIssue
//     })?;
//
//     let pong: String = redis::cmd("PING").query_async(&mut conn).await.map_err(|e| {
//         log!(Level::Error, "Redis PING command failed: {:?}", e);
//         Error::RedisIssue
//     })?;
//
//     if pong != "PONG" {
//         log!(Level::Error, "Unexpected response from Redis PING: {}", pong);
//         return Err(Error::RedisIssue);
//     }
//
//     log!(Level::Info, "Redis connection initialized and PING successful");
//     Ok(Some(pool))
// }

pub async fn initialize_redis_connection() -> Result<Option<Pool>, Error> {
    log!(Level::Info, "Starting Redis connection initialization");

    let redis_url = match env::var("REDIS_URL") {
        Ok(url) => url,
        Err(_) => {
            log!(Level::Error, "REDIS_URL is not set in .env file");
            return Err(Error::EnvVarMissing("REDIS_URL".to_string()));
        }
    };

    let cfg = Config::from_url(redis_url);
    let pool = match cfg.create_pool(Some(Runtime::Tokio1)) {
        Ok(p) => p,
        Err(e) => {
            log!(Level::Error, "Failed to create Redis pool: {:?}", e);
            return Err(Error::RedisIssue);
        }
    };

    log!(Level::Info, "Redis connection initialization has been completed successfully");
    Ok(Some(pool))
}
