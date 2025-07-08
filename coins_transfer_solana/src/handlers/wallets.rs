use crate::config::app_config::AppState;
use crate::responses::http_response_impl::return_response;
use crate::responses::success_msgs_impl::create_error_response;
use crate::services::wallets::{create_wallet, get_wallets, update_wallet};
use crate::structs::wallets::{WalletCreate, WalletQuery, WalletUpdate};
use actix_web::{Responder, web};
use std::sync::Arc;

#[allow(dead_code, unused)]
pub async fn create_wallet_handler(app_state: web::Data<Arc<AppState>>, wallet: web::Json<WalletCreate>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = create_wallet(&pool, &redis_pool, wallet.into_inner()).await;
    return_response(result).await
}

pub async fn get_wallet_handler(app_state: web::Data<Arc<AppState>>, wallet_query: web::Query<WalletQuery>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = get_wallets(&pool, &redis_pool, wallet_query.into_inner()).await;
    return_response(result).await
}

pub async fn update_wallet_handler(app_state: web::Data<Arc<AppState>>, wallet_id: web::Path<i64>, wallet_update: web::Json<WalletUpdate>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let wallet_id = wallet_id.into_inner();
    let result = update_wallet(&pool, &redis_pool, wallet_id, None, wallet_update.into_inner()).await;
    return_response(result).await
}