use crate::config::app_config::AppState;
use crate::responses::success_msgs_impl::{create_error_response, create_success_response};
use crate::services::notifications::validate_usdt_deposit;
use crate::structs::coin_clients::LitecoinClient;
use crate::structs::notify_payment::NotifyUsdPayment;
use actix_web::{HttpResponse, web};
use crypsol_logger::log;
use log::Level;
use std::collections::HashMap;
use std::sync::Arc;

pub async fn generate_litecoin_address(app_state: web::Data<Arc<AppState>>, web::Query(info): web::Query<HashMap<String, String>>) -> HttpResponse {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let client = match LitecoinClient::new(&pool, &redis_pool).await {
        Ok(client) => client,
        Err(e) => {
            log!(Level::Error, "Error initializing Litecoin client: {}", e);
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let label = info.get("label").cloned().unwrap_or_else(|| "default".to_string());
    match client.generate_new_address(&label).await {
        Ok(addr) => HttpResponse::Ok().json(addr),
        Err(e) => {
            log!(Level::Error, "Error generating Litecoin address: {}", e);
            let (message_key, message, data, status_code) = e.to_response();
            create_error_response(message_key, message, data, status_code)
        }
    }
}

pub async fn validate_usdt_payment(app_state: web::Data<Arc<AppState>>, query: web::Json<NotifyUsdPayment>) -> HttpResponse {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let tx_id = query.txid.clone();
    let user_id = query.user_id.clone();
    match validate_usdt_deposit(&pool, &redis_pool, &tx_id, user_id).await {
        Ok(_) => {
            let message_key = "USDTDepositSuccessful";
            let message = "USDT Deposit successful".to_string();
            let data = serde_json::Value::Null;
            create_success_response(message_key, message, data)
        }
        Err(e) => {
            log!(Level::Error, "Error Validating USDT deposit: {}", e);
            let (message_key, message, data, status) = e.to_response();
            create_error_response(message_key, message, data, status)
        }
    }
}