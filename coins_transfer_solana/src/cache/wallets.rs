use crypsol_logger::log;
use deadpool_redis::Pool;
use log::Level;
use serde_json::from_str;

use crate::cache::cache_helper::{drop_from_cache_list, get_cache, get_full_cache_list, push_to_cache_list, set_cache};
use crate::config::constants::WALLETS_CACHE_EXPIRATION;
use crate::config::redis_keys::RedisKeys;
use crate::entities::users_wallets::UsersWallets;
use crate::responses::error_msgs::Error;

/// Retrieve a single wallet from cache by `wallet_id`.
pub async fn get_wallet_from_cache_by_id(pool: &Pool, wallet_id: &i64) -> Option<UsersWallets> {
    let key = RedisKeys::UserWallet { id: *wallet_id }.to_string();
    match get_cache(pool, &key).await {
        Ok(Some(wallet_str)) => match from_str::<UsersWallets>(&wallet_str) {
            Ok(wallet_obj) => Some(wallet_obj),
            Err(e) => {
                log!(Level::Error, "Error::RedisIssue decoding JSON for key {}: {}", key, e);
                None
            }
        },
        Ok(None) => None,
        Err(e) => {
            log!(Level::Error, "Error getting wallet from cache for wallet ID {}: {}", wallet_id, e);
            None
        }
    }
}

/// Retrieve multiple wallets from cache for a specific `user_id`.
pub async fn get_wallets_from_cache_for_user_id(pool: &Pool, user_id: &i64) -> Option<Vec<UsersWallets>> {
    let key = RedisKeys::WalletsByUserId { user_id: *user_id }.to_string();
    match get_full_cache_list(pool, &key).await {
        Ok(wallets_list) => {
            if wallets_list.is_empty() {
                log!(Level::Warn, "No wallets found in cache for user ID: {}", user_id);
                return None;
            }
            Some(wallets_list)
        }
        Err(e) => {
            log!(Level::Error, "Error getting wallets from cache for user ID {}: {}", user_id, e);
            None
        }
    }
}

/// Retrieve all wallets from cache (if any).
pub async fn get_all_wallets_from_cache(pool: &Pool) -> Option<Vec<UsersWallets>> {
    let key = RedisKeys::AllWallets.to_string();
    match get_full_cache_list(pool, &key).await {
        Ok(wallets_list) => {
            if wallets_list.is_empty() {
                log!(Level::Warn, "No wallets found in cache");
                None
            } else {
                Some(wallets_list)
            }
        }
        Err(e) => {
            log!(Level::Error, "Error getting all wallets from cache: {}", e);
            None
        }
    }
}

/// Retrieve multiple wallets from cache for a specific `coin_id`.
pub async fn get_wallets_from_cache_for_coin_id(pool: &Pool, coin_id: &i16) -> Option<Vec<UsersWallets>> {
    let key = RedisKeys::WalletsByCoinId { coin_id: *coin_id }.to_string();
    match get_full_cache_list(pool, &key).await {
        Ok(wallets_list) => {
            if wallets_list.is_empty() {
                log!(Level::Warn, "No wallets found in cache for coin ID: {}", coin_id);
                return None;
            }
            Some(wallets_list)
        }
        Err(e) => {
            log!(Level::Error, "Error getting wallets from cache for coin ID {}: {}", coin_id, e);
            None
        }
    }
}

/// Store (or update) a single wallet in the cache by `wallet.id`.
pub async fn set_wallet_cache_for_id(pool: &Pool, wallet: &UsersWallets) -> Result<(), Error> {
    let key = RedisKeys::UserWallet { id: wallet.id }.to_string();
    if let Err(e) = set_cache(pool, &key, wallet, Some(WALLETS_CACHE_EXPIRATION)).await {
        log!(Level::Error, "Error setting wallet cache for user ID {}: {}", wallet.user_id, e);
        return Err(Error::RedisIssue);
    }
    Ok(())
}

/// Incrementally add multiple wallets to the cache for a specific `coin_id`.
/// (Assumes `wallet` slice is not empty—`wallet[0]` is used.)
pub async fn increment_wallet_cache_for_coin_id(pool: &Pool, wallet: &[UsersWallets]) -> Result<(), Error> {
    let key = RedisKeys::WalletsByCoinId { coin_id: wallet[0].coin_id }.to_string();
    for c in wallet {
        if let Err(e) = push_to_cache_list(pool, &key, c, Some(WALLETS_CACHE_EXPIRATION)).await {
            log!(Level::Error, "Error setting wallet cache for coin ID {}: {}", c.coin_id, e);
            return Err(Error::RedisIssue);
        }
    }
    Ok(())
}

/// Incrementally add multiple wallets to the cache for a specific `user_id`.
/// (Assumes `wallet` slice is not empty—`wallet[0]` is used.)
pub async fn increment_wallets_cache_for_user_id(pool: &Pool, wallet: &[UsersWallets]) -> Result<(), Error> {
    let key = RedisKeys::WalletsByUserId { user_id: wallet[0].user_id }.to_string();
    for w in wallet {
        if let Err(e) = push_to_cache_list(pool, &key, w, Some(WALLETS_CACHE_EXPIRATION)).await {
            log!(Level::Error, "Error setting wallets cache for user ID {}: {}", w.user_id, e);
            return Err(Error::RedisIssue);
        }
    }
    Ok(())
}

/// Incrementally add multiple wallets to the `AllWallets` cache.
pub async fn increment_all_wallets_cache(pool: &Pool, wallets: &[UsersWallets]) -> Result<(), Error> {
    let key = RedisKeys::AllWallets.to_string();
    for w in wallets {
        if let Err(e) = push_to_cache_list(pool, &key, w, Some(WALLETS_CACHE_EXPIRATION)).await {
            log!(Level::Error, "Error setting all wallets cache: {}", e);
            return Err(Error::RedisIssue);
        }
    }
    Ok(())
}

/// Remove a single wallet from the user ID-based cache list.
pub async fn drop_a_wallet_from_all_user_id_wallets(pool: &Pool, coin: &UsersWallets) -> Result<(), Error> {
    let key = RedisKeys::WalletsByUserId { user_id: coin.user_id }.to_string();
    if let Err(e) = drop_from_cache_list(pool, &key, coin).await {
        log!(Level::Error, "Error dropping wallet from all user ID wallets: {}", e);
        return Err(e);
    }
    Ok(())
}

/// Remove a single wallet from the coin ID-based cache list.
pub async fn drop_a_wallet_from_all_coin_id_wallets(pool: &Pool, coin: &UsersWallets) -> Result<(), Error> {
    let key = RedisKeys::WalletsByCoinId { coin_id: coin.coin_id }.to_string();
    if let Err(e) = drop_from_cache_list(pool, &key, coin).await {
        log!(Level::Error, "Error dropping wallet from all coin ID wallets: {}", e);
        return Err(e);
    }
    Ok(())
}

/// Remove a single wallet from the `AllWallets` cache list.
pub async fn drop_a_wallet_from_all_wallets(pool: &Pool, coin: &UsersWallets) -> Result<(), Error> {
    let key = RedisKeys::AllWallets.to_string();
    if let Err(e) = drop_from_cache_list(pool, &key, coin).await {
        log!(Level::Error, "Error dropping wallet from all wallets: {}", e);
        return Err(e);
    }
    Ok(())
}
