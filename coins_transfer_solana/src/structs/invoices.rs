use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvoiceCreate {
    pub admin_id: i16,
    pub user_id: i64,
    pub coin_id: i16,
    pub trx_id: i64,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub coin_amount: BigDecimal,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub fiat_amount: BigDecimal,
    pub address: String,
    pub transaction_hash: Option<String>,
    pub description: String,
    pub is_paid: bool,
    pub is_expired: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceUpdate {
    pub coin_amount: Option<BigDecimal>,
    pub fiat_amount: Option<BigDecimal>,
    pub transaction_hash: Option<String>,
    pub description: Option<String>,
    pub is_paid: Option<bool>,
    pub is_expired: Option<bool>,
}
