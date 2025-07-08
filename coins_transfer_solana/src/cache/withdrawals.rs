use crypsol_logger::log;
use deadpool_redis::Pool;
use log::Level;
use crate::cache::cache_helper::{delete_cache, get_full_cache_list, push_to_cache_list, set_cache};
use crate::config::constants::WITHDRAWALS_CACHE_EXPIRATION;
use crate::config::redis_keys::RedisKeys;
use crate::entities::withdrawals::Withdrawals;
use crate::responses::error_msgs::Error;

/// Retrieve all withdrawals from cache for a given `user_id`.
pub async fn get_withdrawals_from_cache_by_user_id(pool: &Pool, user_id: i64) -> Option<Vec<Withdrawals>> {
    let key = RedisKeys::WithdrawalsByUserId { user_id }.to_string();
    match get_full_cache_list(pool, &key).await {
        Ok(withdrawals_list) => {
            if withdrawals_list.is_empty() {
                return None;
            }
            Some(withdrawals_list)
        }
        Err(e) => {
            log!(Level::Error, "Error::RedisIssue getting withdrawals from cache for user_id {}: {}", user_id, e);
            None
        }
    }
}

/// Store (or update) a single withdrawal in the cache by `withdrawal.id`.
pub async fn set_withdrawal_cache_by_id(pool: &Pool, withdrawal: &Withdrawals) -> Result<(), Error> {
    let key = RedisKeys::Withdrawal { id: withdrawal.id }.to_string();
    if let Err(e) = set_cache(pool, &key, withdrawal, Some(WITHDRAWALS_CACHE_EXPIRATION)).await {
        log!(Level::Error, "Error::RedisIssue setting withdrawal cache for id {}: {}", withdrawal.id, e);
        return Err(Error::RedisIssue);
    }
    Ok(())
}

/// Store (or update) multiple withdrawals in the cache for a specific `user_id`.
pub async fn set_withdrawals_cache_by_user_id(pool: &Pool, user_id: i64, withdrawals: &[Withdrawals]) -> Result<(), Error> {
    let key = RedisKeys::WithdrawalsByUserId { user_id }.to_string();
    for withdraw in withdrawals {
        if let Err(e) = push_to_cache_list(pool, &key, withdraw, Some(WITHDRAWALS_CACHE_EXPIRATION)).await {
            log!(Level::Error, "Error::RedisIssue setting withdrawals cache for user_id {}: {}", user_id, e);
            return Err(Error::RedisIssue);
        }
    }
    Ok(())
}

/// Delete a single withdrawal from cache by `withdrawal_id`.
#[allow(dead_code, unused)]
pub async fn drop_a_withdrawal_from_cache_by_id(pool: &Pool, withdrawal_id: i64) -> Result<(), Error> {
    let key = RedisKeys::Withdrawal { id: withdrawal_id }.to_string();
    match delete_cache(pool, &key).await {
        Ok(_) => Ok(()),
        Err(e) => {
            log!(Level::Error, "Error::RedisIssue deleting withdrawal from cache for id {}: {}", withdrawal_id, e);
            Err(Error::RedisIssue)
        }
    }
}