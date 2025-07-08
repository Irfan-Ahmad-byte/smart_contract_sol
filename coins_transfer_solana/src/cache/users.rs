use crypsol_logger::log;
use deadpool_redis::Pool;
use log::Level;

use crate::cache::cache_helper::{get_cache, push_to_cache_list, set_cache};
use crate::config::constants::USER_CACHE_EXPIRATION;
use crate::config::redis_keys::RedisKeys;
use crate::entities::users::Users;
use crate::responses::error_msgs::Error;

/// Retrieve a single user from cache by `user_id`.
pub async fn get_user_from_cache(pool: &Pool, user_id: &i64) -> Option<Users> {
    let key = RedisKeys::User { user_id: *user_id }.to_string();

    match get_cache(pool, &key).await {
        Ok(Some(user_str)) => match serde_json::from_str::<Users>(&user_str) {
            Ok(user_obj) => Some(user_obj),
            Err(e) => {
                log!(Level::Error, "RedisIssue decoding JSON for key {}: {}", key, e);
                None
            }
        },
        Ok(None) => None,
        Err(e) => {
            log!(Level::Error, "Error getting users from cache for ID {}: {}", user_id, e);
            None
        }
    }
}

/// Store (or update) a single user in the cache.
pub async fn set_users_cache(pool: &Pool, users: &Users) -> Result<(), Error> {
    let key = RedisKeys::User { user_id: users.user_id }.to_string();

    if let Err(e) = set_cache(pool, &key, users, Some(USER_CACHE_EXPIRATION)).await {
        log!(Level::Error, "Error setting users cache for ID {}: {}", users.user_id, e);
        return Err(e);
    }
    Ok(())
}

/// Incrementally add a single user to the existing all-users cache list.
pub async fn increment_all_users_cache(pool: &Pool, users: &Users) -> Result<(), Error> {
    let key = RedisKeys::AllUsers.to_string();
    if let Err(e) = push_to_cache_list(pool, &key, users, Some(USER_CACHE_EXPIRATION)).await {
        log!(Level::Error, "Error incrementing all users cache: {}", e);
        return Err(e);
    }
    Ok(())
}
