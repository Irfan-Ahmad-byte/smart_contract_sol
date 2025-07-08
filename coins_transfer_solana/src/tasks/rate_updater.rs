use crate::services::configs::get_a_config;
use crate::services::conversion_rates::fetch_and_store_coin_rates;
use crypsol_logger::log;
use deadpool_redis::Pool;
use log::Level;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;

pub async fn start_rate_updater(pool: PgPool, redis_pool: Pool) {
    // Environment variable se seconds read karen

    log!(Level::Info, "Starting rate updater task");
    let sleep_duration: u64 = match get_a_config(&pool, &redis_pool, "RATE_UPDATE_INTERVAL_IN_SECONDS".to_string()).await {
        Ok(value) => value.parse::<u64>().unwrap_or(3600 * 24),
        Err(_) => {
            log!(Level::Error, "Failed to read RATE_UPDATE_INTERVAL_IN_SECONDS from config, using default value 60000");
            3600 * 24 //12 hours
        }
    };

    tokio::spawn(async move {
        loop {
            let db_clone = pool.clone();
            let redis_clone = redis_pool.clone();
            match fetch_and_store_coin_rates(db_clone.clone(), redis_clone.clone()).await {
                Ok(_) => log!(Level::Info, "Rates updated successfully"),
                Err(e) => log!(Level::Error, "Failed to update rates: {}", e),
            }
            sleep(Duration::from_secs(sleep_duration)).await;
        }
    });
}
