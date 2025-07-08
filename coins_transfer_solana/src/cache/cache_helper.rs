use crate::config::constants::MODULE_NAME;
use crate::responses::error_msgs::Error;
use deadpool_redis::Pool;
use deadpool_redis::redis::AsyncCommands;
use log::{Level, log};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json;

pub fn is_cache_enabled() -> bool {
    let cache_enabled = std::env::var("REDIS_ENABLED").unwrap_or("false".to_string());
    let enabled_values = ["true", "yes", "1", "enabled"];
    enabled_values.contains(&cache_enabled.to_lowercase().as_str())
}

/// Sets a key in Redis, optionally with an expiration (TTL in seconds).
pub async fn set_cache<T: Serialize + ?Sized>(pool: &Pool, key: &str, value: &T, expiration: Option<usize>) -> Result<(), Error> {
    if !is_cache_enabled() {
        return Ok(());
    }

    let mut conn = pool.get().await.map_err(|e| {
        log!(Level::Error, "Failed to get redis connection: {e:?}");
        Error::RedisIssue
    })?;

    let value_str = serde_json::to_string(value).map_err(|e| {
        log!(Level::Error, "Serialization error: {e}");
        Error::RedisIssue
    })?;

    if let Some(exp) = expiration {
        // `set_ex` requires `u64` for the TTL
        conn.set_ex::<&str, String, ()>(key, value_str, exp as u64).await.map_err(|e| {
            log!(Level::Error, "Failed to set cache for key {key}: {e}");
            Error::RedisIssue
        })?;
    } else {
        conn.set::<&str, String, ()>(key, value_str).await.map_err(|e| {
            log!(Level::Error, "Failed to set cache for key {key}: {e}");
            Error::RedisIssue
        })?;
    }
    Ok(())
}

/// Pushes `value` to the tail of a Redis list, optionally setting TTL.
pub async fn push_to_cache_list<T: Serialize + ?Sized>(pool: &Pool, key: &str, value: &T, expiration: Option<usize>) -> Result<(), Error> {
    if !is_cache_enabled() {
        return Ok(());
    }

    let mut conn = pool.get().await.map_err(|e| {
        log!(Level::Error, "Failed to get redis connection: {e:?}");
        Error::RedisIssue
    })?;

    let value_str = serde_json::to_string(value).map_err(|e| {
        log!(Level::Error, "Serialization error: {e}");
        Error::RedisIssue
    })?;

    // `rpush` returns the new length (i64) of the list
    conn.rpush::<&str, String, i64>(key, value_str).await.map_err(|e| {
        log!(Level::Error, "Failed to push to list for key {key}: {e}");
        Error::RedisIssue
    })?;

    if let Some(exp) = expiration {
        // `expire` requires an `i64` for the TTL
        conn.expire::<&str, bool>(key, exp as i64).await.map_err(|e| {
            log!(Level::Error, "Failed to expire key {key}: {e}");
            Error::RedisIssue
        })?;
    }
    Ok(())
}

/// Removes `value` from a Redis list.
pub async fn drop_from_cache_list<T: Serialize>(pool: &Pool, key: &str, value: &T) -> Result<(), Error> {
    if !is_cache_enabled() {
        return Ok(());
    }

    let mut conn = pool.get().await.map_err(|e| {
        log!(Level::Error, "Failed to get redis connection: {e:?}");
        Error::RedisIssue
    })?;

    let value_str = serde_json::to_string(value).map_err(|e| {
        log!(Level::Error, "Serialization error: {e}");
        Error::RedisIssue
    })?;

    // `lrem` returns the number of removed items (i64)
    conn.lrem::<&str, String, i64>(key, 0, value_str).await.map_err(|e| {
        log!(Level::Error, "Failed to drop from list for key {key}: {e}");
        Error::RedisIssue
    })?;
    Ok(())
}

/// Retrieves all items of a Redis list and deserializes them into a `Vec<T>`.
pub async fn get_full_cache_list<T: DeserializeOwned>(pool: &Pool, key: &str) -> Result<Vec<T>, Error> {
    if !is_cache_enabled() {
        return Ok(Vec::new());
    }

    let mut conn = pool.get().await.map_err(|e| {
        log!(Level::Error, "Failed to get redis connection: {e:?}");
        Error::RedisIssue
    })?;

    let items = conn.lrange::<&str, Vec<String>>(key, 0, -1).await.map_err(|e| {
        log!(Level::Error, "Failed to get list for key {key}: {e}");
        Error::RedisIssue
    })?;

    let mut parsed = Vec::new();
    for item_str in items {
        let deserialized = serde_json::from_str(&item_str).map_err(|e| {
            log!(Level::Error, "Deserialization error: {e}");
            Error::RedisIssue
        })?;
        parsed.push(deserialized);
    }
    Ok(parsed)
}

/// Paginates a list in Redis, returning a portion of items (as `Vec<String>`).
#[allow(dead_code)]
pub async fn paginate_cache(pool: &Pool, key: &str, page: usize, page_size: usize) -> Result<Vec<String>, Error> {
    if !is_cache_enabled() {
        return Ok(Vec::new());
    }

    let mut conn = pool.get().await.map_err(|e| {
        log!(Level::Error, "Failed to get redis connection: {e:?}");
        Error::RedisIssue
    })?;

    let start = (page.saturating_sub(1)) * page_size;
    let end = start + page_size - 1;

    conn.lrange::<&str, Vec<String>>(key, start as isize, end as isize).await.map_err(|e| {
        log!(Level::Error, "Failed to paginate key {key}: {e}");
        Error::RedisIssue
    })
}

/// Retrieves the string value for a key (if it exists).
pub async fn get_cache(pool: &Pool, key: &str) -> Result<Option<String>, Error> {
    if !is_cache_enabled() {
        return Ok(None);
    }

    let mut conn = pool.get().await.map_err(|e| {
        log!(Level::Error, "Failed to get redis connection: {e:?}");
        Error::RedisIssue
    })?;

    conn.get::<&str, Option<String>>(key).await.map_err(|e| {
        log!(Level::Error, "Failed to get key {key}: {e}");
        Error::RedisIssue
    })
}

/// Gets the length of a list as `i32`.
#[allow(dead_code)]
pub async fn cache_list_length(pool: &Pool, key: &str) -> Result<i32, Error> {
    if !is_cache_enabled() {
        return Ok(0);
    }

    let mut conn = pool.get().await.map_err(|e| {
        log!(Level::Error, "Failed to get redis connection: {e:?}");
        Error::RedisIssue
    })?;

    let length = conn.llen::<&str, i64>(key).await.map_err(|e| {
        log!(Level::Error, "Failed to get length for key {key}: {e}");
        Error::RedisIssue
    })?;
    Ok(length as i32)
}

/// Uses `scan_match` to find all keys that match `MODULE_NAME:pattern`.
#[allow(dead_code)]
pub async fn get_keys_with_pattern(pool: &Pool, pattern: &str) -> Result<Vec<String>, Error> {
    if !is_cache_enabled() {
        return Ok(Vec::new());
    }

    let mut conn = pool.get().await.map_err(|e| {
        log!(Level::Error, "Failed to get redis connection: {e:?}");
        Error::RedisIssue
    })?;

    let pattern = format!("{MODULE_NAME}:{pattern}");

    // `scan_match` requires two generic parameters in `redis >= 0.27`
    let mut iter = conn.scan_match::<String, String>(pattern.clone()).await.map_err(|e| {
        log!(Level::Error, "Failed to scan for keys with pattern {pattern}: {e}");
        Error::RedisIssue
    })?;

    let mut keys = Vec::new();
    while let Some(key) = iter.next_item().await {
        keys.push(key);
    }
    Ok(keys)
}

/// Deletes a single key from Redis.
pub async fn delete_cache(pool: &Pool, key: &str) -> Result<(), Error> {
    if !is_cache_enabled() {
        return Ok(());
    }

    let mut conn = pool.get().await.map_err(|e| {
        log!(Level::Error, "Failed to get redis connection: {e:?}");
        Error::RedisIssue
    })?;

    conn.del::<&str, i64>(key).await.map_err(|e| {
        log!(Level::Error, "Failed to delete key {key}: {e}");
        Error::RedisIssue
    })?;
    Ok(())
}

/// Increments a key by the specified `amount`.
pub async fn _increment_cache(pool: &Pool, key: &str, amount: i64) -> Result<i64, Error> {
    if !is_cache_enabled() {
        return Ok(0);
    }

    let mut conn = pool.get().await.map_err(|e| {
        log!(Level::Error, "Failed to get redis connection: {e:?}");
        Error::RedisIssue
    })?;

    conn.incr::<&str, i64, i64>(key, amount).await.map_err(|e| {
        log!(Level::Error, "Failed to increment key {key} by {amount}: {e}");
        Error::RedisIssue
    })
}

/// Sets an expiration on a key (TTL in seconds).
pub async fn _expire_cache(pool: &Pool, key: &str, expiration: usize) -> Result<(), Error> {
    if !is_cache_enabled() {
        return Ok(());
    }

    let mut conn = pool.get().await.map_err(|e| {
        log!(Level::Error, "Failed to get redis connection: {e:?}");
        Error::RedisIssue
    })?;

    // `expire` expects `i64`
    conn.expire::<&str, bool>(key, expiration as i64).await.map_err(|e| {
        log!(Level::Error, "Failed to expire key {key}: {e}");
        Error::RedisIssue
    })?;
    Ok(())
}

/// Flushes all keys under `MODULE_NAME` prefix.
pub async fn flush_cache(pool: &Pool) -> Result<(), Error> {
    if !is_cache_enabled() {
        return Ok(());
    }

    let mut conn = pool.get().await.map_err(|e| {
        log!(Level::Error, "Failed to get redis connection: {e:?}");
        Error::RedisIssue
    })?;

    let key_pattern = format!("{MODULE_NAME}:*");

    // `keys` returns `RedisResult<Vec<String>>`
    let keys = conn.keys::<String, Vec<String>>(key_pattern.clone()).await.map_err(|e| {
        log!(Level::Error, "Failed to scan keys with pattern {key_pattern}: {e}");
        Error::RedisIssue
    })?;

    // `del` returns the number of keys removed (i64)
    for k in &keys {
        conn.del::<&str, i64>(k).await.map_err(|e| {
            log!(Level::Error, "Failed to delete key {k}: {e}");
            Error::RedisIssue
        })?;
    }
    Ok(())
}
