use crypsol_logger::log;
use deadpool_redis::Pool;
use log::Level;
use serde_json::from_str;

use crate::cache::cache_helper::{get_cache, get_full_cache_list, push_to_cache_list, set_cache};
use crate::config::constants::COIN_CACHE_EXPIRATION;
use crate::config::redis_keys::RedisKeys;
use crate::entities::coins::Coins;
use crate::responses::error_msgs::Error;

/// Helper function to retrieve a single coin from cache using a provided Redis key.
async fn get_coin_from_cache(pool: &Pool, key: &str) -> Option<Coins> {
    match get_cache(pool, key).await {
        Ok(Some(coin_str)) => match from_str::<Coins>(&coin_str) {
            Ok(coin) => Some(coin),
            Err(e) => {
                log!(Level::Error, "Error::RedisIssue decoding JSON for key {}: {}", key, e);
                None
            }
        },
        Ok(None) => None,
        Err(e) => {
            log!(Level::Error, "Error::RedisIssue getting coin from cache for key {}: {}", key, e);
            None
        }
    }
}

pub async fn get_coin_from_cache_by_id(pool: &Pool, coin_id: i16) -> Option<Coins> {
    let key = RedisKeys::CoinById { id: coin_id }.to_string();
    get_coin_from_cache(pool, &key).await
}
pub async fn get_all_coins_from_cache(pool: &Pool) -> Option<Vec<Coins>> {
    let key = RedisKeys::AllCoins.to_string();
    match get_full_cache_list(pool, &key).await {
        Ok(coins) => {
            if coins.is_empty() {
                return None;
            }
            Some(coins)
        }
        Err(e) => {
            log!(Level::Error, "Error::RedisIssue getting all coins from cache: {}", e);
            None
        }
    }
}

pub async fn set_coin_cache_by_id(pool: &Pool, coin: &Coins) -> Result<(), Error> {
    let key = RedisKeys::CoinById { id: coin.id }.to_string();
    if let Err(_e) = set_cache(pool, &key, coin, Some(COIN_CACHE_EXPIRATION)).await {
        return Err(Error::RedisIssue);
    }
    Ok(())
}

pub async fn set_coin_cache_by_name(pool: &Pool, coin: &Coins) -> Result<(), Error> {
    let key = RedisKeys::CoinByName { name: coin.coin_name.clone() }.to_string();
    if let Err(e) = set_cache(pool, &key, coin, Some(COIN_CACHE_EXPIRATION)).await {
        log!(Level::Error, "Error::RedisIssue setting coin cache for name {}: {}", coin.coin_name, e);
        return Err(Error::RedisIssue);
    }
    Ok(())
}

pub async fn set_coin_cache_by_symbol(pool: &Pool, coin: &Coins) -> Result<(), Error> {
    let key = RedisKeys::CoinBySymbol { symbol: coin.symbol.clone() }.to_string();
    if let Err(e) = set_cache(pool, &key, coin, Some(COIN_CACHE_EXPIRATION)).await {
        log!(Level::Error, "Error::RedisIssue setting coin cache for symbol {}: {}", coin.symbol, e);
        return Err(Error::RedisIssue);
    }
    Ok(())
}

pub async fn set_all_coins_cache(pool: &Pool, coins: &[Coins]) -> Result<(), Error> {
    let key = RedisKeys::AllCoins.to_string();
    for c in coins {
        if let Err(e) = push_to_cache_list(pool, &key, c, Some(COIN_CACHE_EXPIRATION)).await {
            log!(Level::Error, "Error::RedisIssue setting coin cache for id {}: {}", c.id, e);
            return Err(Error::RedisIssue);
        }
    }
    Ok(())
}

pub async fn increment_all_coins_cache(pool: &Pool, coin: &Coins) -> Result<(), Error> {
    let key = RedisKeys::AllCoins.to_string();
    if let Err(e) = push_to_cache_list(pool, &key, coin, Some(COIN_CACHE_EXPIRATION)).await {
        log!(Level::Error, "Error incrementing all coins cache: {}", e);
        return Err(e);
    }
    Ok(())
}