use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct UsersWallets {
    pub id: i64,
    pub user_id: i64,
    pub coin_id: i16,
    pub address: Option<String>,
    pub status: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
