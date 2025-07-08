use crate::config::constants::{ACQUIRE_TIMEOUT_SECS, DB_MAX_CONNECTIONS_CAP, DB_MAX_CONNECTIONS_DEFAULT, DB_MIN_CONNECTIONS_DEFAULT, DB_MIN_CONNECTIONS_FLOOR, IDLE_TIMEOUT_SECS, MAX_LIFETIME_SECS};
use crate::responses::error_msgs::Error;
use crypsol_logger::{Level, log};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::{env, time::Duration};

/// Validates and applies database connection options.
async fn check_connect_options(pool_options: &mut PgPoolOptions) -> Result<(), &'static str> {
    log!(Level::Info, "Validating database connection options");

    let log_and_return = |msg: &'static str| async move {
        log!(Level::Error, "{}", msg);
        Err(msg)
    };

    // Helper function to get an environment variable as usize with a default value.
    fn get_env_usize(var: &str, default: usize) -> usize {
        env::var(var).ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(default)
    }

    // Read connection parameters from environment or use defaults.
    let max_connections_cap = get_env_usize("MAX_CONNECTIONS_CAP", DB_MAX_CONNECTIONS_CAP);
    let min_connections_floor = get_env_usize("MIN_CONNECTIONS_FLOOR", DB_MIN_CONNECTIONS_FLOOR);
    let max_connections = get_env_usize("MAX_CONNECTIONS", DB_MAX_CONNECTIONS_DEFAULT);
    let min_connections = get_env_usize("MIN_CONNECTIONS", DB_MIN_CONNECTIONS_DEFAULT);

    // Validate connection limits.
    if max_connections < min_connections_floor {
        return log_and_return("Configured maximum connections cannot be lower than the allowed minimum (floor).").await;
    }
    if max_connections > max_connections_cap {
        return log_and_return("Configured maximum connections cannot exceed the allowed cap.").await;
    }
    if min_connections > max_connections {
        return log_and_return("Configured minimum connections cannot exceed the configured maximum connections.").await;
    }

    // Get timeouts from environment or defaults.
    let acquire_timeout_secs = env::var("ACQUIRE_TIMEOUT").ok().and_then(|s| s.parse::<u64>().ok()).unwrap_or(ACQUIRE_TIMEOUT_SECS);
    let idle_timeout_secs = env::var("IDLE_TIMEOUT").ok().and_then(|s| s.parse::<u64>().ok()).unwrap_or(IDLE_TIMEOUT_SECS);
    let max_lifetime_secs = env::var("MAX_LIFETIME").ok().and_then(|s| s.parse::<u64>().ok()).unwrap_or(MAX_LIFETIME_SECS);

    // Validate timeouts.
    if acquire_timeout_secs == 0 || idle_timeout_secs == 0 || max_lifetime_secs == 0 {
        return log_and_return("Timeout values cannot be 0").await;
    }

    // Apply validated options.
    *pool_options = pool_options
        .clone()
        .max_connections(max_connections as u32)
        .min_connections(min_connections as u32)
        .acquire_timeout(Duration::from_secs(acquire_timeout_secs))
        .idle_timeout(Duration::from_secs(idle_timeout_secs))
        .max_lifetime(Duration::from_secs(max_lifetime_secs));

    Ok(())
}

/// Initializes database connection pool with validated options.
pub async fn connect_to_db() -> Result<Pool<Postgres>, Error> {
    log!(Level::Info, "Initializing database connection");

    let database_url = env::var("DATABASE_URL").map_err(|_| {
        log!(Level::Error, "DATABASE_URL not set in environment");
        Error::EnvVarMissing("DATABASE_URL".to_string())
    })?;

    if database_url.is_empty() {
        log!(Level::Error, "DATABASE_URL is empty");
        return Err(Error::InvalidConfiguration);
    }

    let mut pool_options = PgPoolOptions::new();

    if let Err(e) = check_connect_options(&mut pool_options).await {
        log!(Level::Error, "Invalid connection options: {}", e);
        return Err(Error::InvalidConfiguration);
    }

    match pool_options.connect(&database_url).await {
        Ok(pool) => {
            log!(Level::Info, "Database connection initialized successfully");
            Ok(pool)
        }
        Err(e) => {
            log!(Level::Error, "Database connection failed: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}
