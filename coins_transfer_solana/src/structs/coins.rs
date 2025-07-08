use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CoinCreate {
    pub coin_name: String,
    pub symbol: String,
    pub status: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoinUpdate {
    pub coin_name: Option<String>,
    pub symbol: Option<String>,
    pub status: Option<bool>,
}

#[derive(Clone, Deserialize)]
pub struct AddressGenerationRequest {
    pub user_id: i64,
    pub coin_id: i16,
}

#[derive(Deserialize)]
pub struct AddressValidationRequest {
    pub coin_id: i16,
    pub address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoinInfo {
    pub id: i16,
    pub coin_name: String,
    pub symbol: String,
    pub status: bool,
    pub current_rate: BigDecimal,
}
