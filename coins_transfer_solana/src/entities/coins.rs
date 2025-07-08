use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Coins {
    pub id: i16,
    pub coin_name: String,
    pub symbol: String,
    pub status: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
