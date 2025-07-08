use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Deposits {
    pub id: i64,
    pub user_id: i64,
    pub coin_id: i16,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub amount: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub fiat_amount: BigDecimal,
    pub user_address_id: i64,
    pub transaction_hash: String,
    pub status: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
