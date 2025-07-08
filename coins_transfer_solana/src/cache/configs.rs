use crypsol_logger::log;
use deadpool_redis::Pool;
use log::Level;

use crate::cache::cache_helper::{delete_cache, drop_from_cache_list, get_cache, get_full_cache_list, push_to_cache_list, set_cache};
use crate::config::constants::CONFIGS_CACHE_EXPIRATION;
use crate::config::redis_keys::RedisKeys;
use crate::entities::configs::Configs;
use crate::responses::error_msgs::Error;

/// Fetch a `Configs` entry from cache by name.
pub async fn get_config_by_name_from_cache(pool: &Pool, name: &str) -> Option<Configs> {
    let key = RedisKeys::Config { name: name.to_string() }.to_string();

    match get_cache(pool, &key).await {
        Ok(Some(config_str)) => match serde_json::from_str::<Configs>(&config_str) {
            Ok(config) => Some(config),
            Err(e) => {
                log!(Level::Error, "RedisIssue decoding JSON for key {}: {}", key, e);
                None
            }
        },
        Ok(None) => None,
        Err(e) => {
            log!(Level::Error, "Error getting config from cache for name {}: {}", name, e);
            None
        }
    }
}

/// Set/Update a config entry in cache by name.
pub async fn set_config_cache(pool: &Pool, config: &Configs) -> Result<(), Error> {
    let key = RedisKeys::Config { name: config.name.clone() }.to_string();
    if let Err(e) = set_cache(pool, &key, config, Some(CONFIGS_CACHE_EXPIRATION)).await {
        log!(Level::Error, "Error setting config cache for name {}: {}", config.name, e);
        return Err(e);
    }
    Ok(())
}

/// Retrieve all configs from cache, if any.
pub async fn get_all_configs_from_cache(pool: &Pool) -> Option<Vec<Configs>> {
    let key = RedisKeys::AllConfigs.to_string();
    match get_full_cache_list(pool, &key).await {
        Ok(configs) => {
            if configs.is_empty() {
                log!(Level::Warn, "No configs found in cache");
                None
            } else {
                Some(configs)
            }
        }
        Err(e) => {
            log!(Level::Error, "Error getting all configs from cache: {}", e);
            None
        }
    }
}

/// Push a list of configs into the cache for the AllConfigs key.
pub async fn set_all_config_cache(pool: &Pool, config: &[Configs]) -> Result<(), Error> {
    let key = RedisKeys::AllConfigs.to_string();
    for c in config {
        if let Err(e) = push_to_cache_list(pool, &key, c, Some(CONFIGS_CACHE_EXPIRATION)).await {
            log!(Level::Error, "Error setting all config cache: {}", e);
            return Err(e);
        }
    }
    Ok(())
}

/// Push one additional config into the AllConfigs cache list.
pub async fn increment_all_config_cache(pool: &Pool, config: &Configs) -> Result<(), Error> {
    let key = RedisKeys::AllConfigs.to_string();
    if let Err(e) = push_to_cache_list(pool, &key, config, Some(CONFIGS_CACHE_EXPIRATION)).await {
        log!(Level::Error, "Error incrementing all config cache: {}", e);
        return Err(e);
    }
    Ok(())
}

/// Remove one config item from the AllConfigs cache list.
pub async fn drop_a_config_from_all_config_cache(pool: &Pool, config: &Configs) -> Result<(), Error> {
    let key = RedisKeys::AllConfigs.to_string();
    if let Err(e) = drop_from_cache_list(pool, &key, config).await {
        log!(Level::Error, "Error dropping a config from all config cache: {}", e);
        return Err(e);
    }
    Ok(())
}

/// Delete a config entry from cache by name.
pub async fn delete_config_cache(pool: &Pool, name: &str) -> Result<(), Error> {
    let key = RedisKeys::Config { name: name.to_string() }.to_string();
    if let Err(e) = delete_cache(pool, &key).await {
        log!(Level::Error, "Error deleting config cache for name {}: {}", name, e);
        return Err(e);
    }
    Ok(())
}
