use crate::entities::coins::Coins;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone)]
pub struct NotifyPaymentQuery {
    pub txid: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NotifyUsdPayment {
    pub user_id: i64,
    pub txid: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotifyPaymentTask {
    pub txid: String,
    pub coin: Coins,
}

#[derive(Serialize, Deserialize)]
pub struct ConfirmationsPendingNotify {
    pub message_key: String,
    pub user_id: i64,
    pub data: Value,
}
