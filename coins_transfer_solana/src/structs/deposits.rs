use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DepositCreate {
    pub user_id: i64,
    pub coin_id: i16,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub amount: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub fiat_amount: BigDecimal,
    pub address_id: i64,
    pub transaction_hash: String,
    pub status: bool,
    pub event_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepositUpdate {
    pub amount: Option<BigDecimal>,
    pub fiat_amount: Option<BigDecimal>,
    pub address_id: Option<i64>,
    pub status: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepositsDetails {
    pub id: i64,
    pub user_id: i64,
    pub coin_id: i16,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub amount: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub fiat_amount: BigDecimal,
    pub address: Option<String>,
    pub transaction_hash: String,
    pub status: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
