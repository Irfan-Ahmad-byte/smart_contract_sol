use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletCreate {
    pub user_id: i64,
    pub coin_id: i16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletUpdate {
    pub user_id: i64,
    pub coin_id: Option<i16>,
    pub address: Option<String>,
    pub private_key: Option<String>,
    pub wallet_index: Option<i64>,
    pub status: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletQuery {
    pub user_id: Option<i64>,
    pub wallet_id: Option<i64>,
    pub coin_id: Option<i16>,
}
