use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Withdrawals {
    pub id: i64,
    pub user_id: i64,
    pub coin_id: i16,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub usd_amount: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub coin_amount: BigDecimal,
    pub fee_usd_amount: BigDecimal,
    pub fee_coin_amount: BigDecimal,
    pub transaction_hash: Option<String>,
    pub address: Option<String>,
    pub status: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
