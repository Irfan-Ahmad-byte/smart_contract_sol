use crypsol_logger::log;
use deadpool_redis::Pool;
use log::Level;
use sqlx::{PgPool, query, query_as};

use crate::cache::users::{get_user_from_cache, increment_all_users_cache, set_users_cache};
use crate::entities::users::Users;
use crate::responses::error_msgs::Error;
use crate::responses::success_msgs::SuccessMessages;
use crate::structs::users::UserCreate;
use crate::utils::time::TimeHandler;

pub async fn create_user(pool: &PgPool, redis_pool: &Pool, user: UserCreate) -> Result<SuccessMessages, Error> {
    let time_handler = TimeHandler::new();
    let now = time_handler.get_current_time().naive_utc();

    let result = query!(
        r#"
        INSERT INTO users (user_id, created_at, updated_at, event_id)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id) DO UPDATE
        SET updated_at = now(),
            event_id = $4,
            event_status = 1
        RETURNING id, user_id, created_at, updated_at
        "#,
        user.user_id,
        now,
        now,
        user.event_id
    )
    .fetch_one(pool)
    .await;

    let new_user = match result {
        Ok(record) => Users { id: record.id, user_id: record.user_id, created_at: record.created_at, updated_at: record.updated_at },
        Err(sqlx::Error::RowNotFound) => {
            log!(Level::Error, "Duplicate Entry: for User {:?}", user);
            return Err(Error::DuplicateEntry);
        }
        Err(e) => {
            log!(Level::Error, "Database error on insert: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };
    let _ = set_users_cache(redis_pool, &new_user).await;
    let _ = increment_all_users_cache(redis_pool, &new_user).await;

    Ok(SuccessMessages::CreatedUser { user_id: new_user.user_id })
}

pub async fn rollback_user_creation(pool: &PgPool, _redis_pool: &Pool, event_id: i64) -> Result<SuccessMessages, Error> {
    let result = query!(
        r#"
        UPDATE users
        SET event_status = 2
        WHERE event_id = $1
        RETURNING user_id
        "#,
        event_id
    )
    .fetch_one(pool)
    .await;

    let _record = match result {
        Ok(users) => users,
        Err(e) => {
            log!(Level::Error, "Database error: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };

    Ok(SuccessMessages::UserCreationFailed)
}
pub async fn fetch_a_user(pool: &PgPool, redis_pool: &Pool, user_id: i64) -> Result<Users, Error> {
    let user = match get_user_from_cache(redis_pool, &user_id).await {
        Some(user) => user,
        _ => {
            let user = get_a_user_from_db(pool, user_id).await?;
            let _ = set_users_cache(redis_pool, &user).await;
            let _ = increment_all_users_cache(redis_pool, &user).await;
            user
        }
    };

    Ok(user)
}

pub async fn get_a_user_from_db(pool: &PgPool, user_id: i64) -> Result<Users, Error> {
    let result = query_as!(
        Users,
        r#"
        SELECT id, user_id, created_at, updated_at FROM users WHERE user_id = $1 AND event_status=1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(user) => Ok(user),
        Err(sqlx::Error::RowNotFound) => {
            log!(Level::Error, "User not found with user ID: {}", user_id);
            Err(Error::NotFound("User not found".to_string()))
        }
        Err(e) => {
            log!(Level::Error, "Database error: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}