use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WithdrawalCreate {
    pub user_id: i64,
    pub coin_id: i16,
    pub usd_amount: BigDecimal,
    pub address: String,
    pub status: bool,
    pub event_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WithdrawalUpdate {
    pub status: bool,
}
