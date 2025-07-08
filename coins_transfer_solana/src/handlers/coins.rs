use crate::config::app_config::AppState;
use crate::responses::http_response_impl::return_response;
use crate::responses::success_msgs_impl::create_error_response;
use crate::services::coins::{
    create_coin, generate_address, get_all_coins
};
use crate::structs::coins::{AddressGenerationRequest, CoinCreate};
use actix_web::{Responder, web};
use std::sync::Arc;

pub async fn create_coin_handler(app_state: web::Data<Arc<AppState>>, coin: web::Json<CoinCreate>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = create_coin(&pool, &redis_pool, coin.into_inner()).await;
    return_response(result).await
}

pub async fn get_all_coins_handler(app_state: web::Data<Arc<AppState>>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = get_all_coins(&pool, &redis_pool).await;
    return_response(result).await
}

pub async fn address_generation_handler(app_state: web::Data<Arc<AppState>>, coin: web::Json<AddressGenerationRequest>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = generate_address(&pool, &redis_pool, coin.into_inner()).await;
    return_response(result).await
}
