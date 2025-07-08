use crypsol_logger::log;
use deadpool_redis::Pool;
use log::Level;
use crate::cache::cache_helper::{get_full_cache_list, push_to_cache_list, set_cache};
use crate::config::constants::USER_TRANSACTION_CACHE_EXPIRATION;
use crate::config::redis_keys::RedisKeys;
use crate::entities::deposits::Deposits;
use crate::responses::error_msgs::Error;

/// Retrieve all transactions from cache for a specific `user_id`.
pub async fn get_transactions_from_cache_by_user_id(pool: &Pool, user_id: i64) -> Option<Vec<Deposits>> {
    let key = RedisKeys::UserDeposits { user_id }.to_string();
    match get_full_cache_list(pool, &key).await {
        Ok(transactions) => {
            if transactions.is_empty() {
                log!(Level::Warn, "No transactions found in cache for user_id: {}", user_id);
                return None;
            }
            Some(transactions)
        }
        Err(e) => {
            log!(Level::Error, "Error::RedisIssue getting transactions from cache for user_id {}: {}", user_id, e);
            None
        }
    }
}

/// Store a single transaction in the cache by `transaction.id`.
pub async fn set_transaction_cache_by_id(pool: &Pool, transaction: &Deposits) -> Result<(), Error> {
    let key = RedisKeys::Deposits { id: transaction.id }.to_string();

    if let Err(e) = set_cache(pool, &key, transaction, Some(USER_TRANSACTION_CACHE_EXPIRATION)).await {
        log!(Level::Error, "Error::RedisIssue setting transaction cache for id {}: {}", transaction.id, e);
        return Err(Error::RedisIssue);
    }
    Ok(())
}

/// Store multiple transactions in the cache for a given `user_id`.
pub async fn set_transactions_cache_by_user_id(pool: &Pool, user_id: i64, transactions: &[Deposits]) -> Result<(), Error> {
    let key = RedisKeys::UserDeposits { user_id }.to_string();

    for txn in transactions {
        if let Err(e) = push_to_cache_list(pool, &key, txn, Some(USER_TRANSACTION_CACHE_EXPIRATION)).await {
            log!(Level::Error, "Error::RedisIssue setting transactions cache for user_id {}: {}", user_id, e);
            return Err(Error::RedisIssue);
        }
    }
    Ok(())
}
