use crate::entities::deposits::Deposits;
use crate::entities::withdrawals::Withdrawals;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WithdrawalsHistoryRequest {
    pub user_id: i64,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub status: Option<bool>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WithdrawalsHistoryResult {
    pub page: i32,
    pub per_page: i32,
    pub total_results: i32,
    pub total_pages: i32,
    pub data: Vec<Withdrawals>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionsHistoryRequest {
    pub user_id: i64,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub status: Option<bool>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionsHistoryResult {
    pub page: i32,
    pub per_page: i32,
    pub total_results: i32,
    pub total_pages: i32,
    pub data: Vec<Deposits>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionsHistoryBulkRequest {
    pub ids: Vec<i64>,
}
