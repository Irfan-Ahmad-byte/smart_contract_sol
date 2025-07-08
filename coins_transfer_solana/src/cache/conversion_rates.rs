use crate::cache::cache_helper::{delete_cache, get_cache, set_cache};
use crate::config::constants::CONVERSION_RATES_CACHE_EXPIRATION;
use crate::config::redis_keys::RedisKeys;
use crate::entities::conversion_rates::ConversionRates;
use crate::responses::error_msgs::Error;

use crypsol_logger::log;
use deadpool_redis::Pool;
use log::Level;
use serde_json::from_str;

/// Fetch a conversion rate from cache using a given coin_id.
pub async fn get_conversion_rate_from_cache(pool: &Pool, coin_id: i16) -> Option<ConversionRates> {
    let key = RedisKeys::ConversionRate { coin_id }.to_string();

    match get_cache(pool, &key).await {
        Ok(Some(history_str)) => {
            // Decode JSON string into ConversionRates
            match from_str::<ConversionRates>(&history_str) {
                Ok(history) => Some(history),
                Err(e) => {
                    log!(Level::Error, "Error::RedisIssue decoding JSON for key {}: {}", key, e);
                    None
                }
            }
        }
        Ok(None) => None,
        Err(e) => {
            log!(Level::Error, "Error::RedisIssue getting conversion rate from cache for coin_id {}: {}", coin_id, e);
            None
        }
    }
}

/// Set (or update) a conversion rate in the cache.
pub async fn set_conversion_rate_cache(pool: &Pool, rate: &ConversionRates) -> Result<(), Error> {
    let key = RedisKeys::ConversionRate { coin_id: rate.coin_id }.to_string();

    if let Err(e) = set_cache(pool, &key, rate, Some(CONVERSION_RATES_CACHE_EXPIRATION)).await {
        log!(Level::Error, "Error::RedisIssue setting conversion rate cache for coin_id {}: {}", rate.coin_id, e);
        return Err(Error::RedisIssue);
    }
    Ok(())
}

/// Delete a conversion rate from the cache by coin_id.
#[allow(dead_code)]
pub async fn delete_conversion_rate_cache(pool: &Pool, coin_id: i16) -> Result<(), Error> {
    let key = RedisKeys::ConversionRate { coin_id }.to_string();

    if let Err(e) = delete_cache(pool, &key).await {
        log!(Level::Error, "Error::RedisIssue deleting conversion rate cache for coin_id {}: {}", coin_id, e);
        return Err(Error::RedisIssue);
    }
    Ok(())
}
