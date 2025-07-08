use crate::responses::error_msgs::Error;
use deadpool_redis::Pool;
use sqlx::PgPool;

pub struct AppState {
    pub db: PgPool,
    pub redis: Option<Pool>,
}

impl AppState {
    pub fn get_redis_pool(&self) -> Result<Pool, Error> {
        self.redis.clone().ok_or(Error::RedisIssue)
    }
}
