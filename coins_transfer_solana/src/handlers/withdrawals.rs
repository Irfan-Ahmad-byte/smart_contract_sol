use crate::config::app_config::AppState;
use crate::responses::http_response_impl::return_response;
use crate::responses::success_msgs_impl::create_error_response;
use crate::services::withdrawals::{create_withdrawal, get_withdrawals_history, rollback_withdrawal_request};
use crate::structs::history::WithdrawalsHistoryRequest;
use crate::structs::withdrawals::WithdrawalCreate;
use actix_web::{Responder, web};
use std::sync::Arc;

pub async fn create_withdrawal_handler(app_state: web::Data<Arc<AppState>>, withdrawal: web::Json<WithdrawalCreate>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = create_withdrawal(&pool, &redis_pool, withdrawal.into_inner()).await;
    return_response(result).await
}
pub async fn rollback_withdrawal_request_handler(app_state: web::Data<Arc<AppState>>, event_id: web::Path<i64>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = rollback_withdrawal_request(&pool, &redis_pool, event_id.into_inner()).await;
    return_response(result).await
}

pub async fn get_withdrawal_history_handler(app_state: web::Data<Arc<AppState>>, query: web::Query<WithdrawalsHistoryRequest>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = get_withdrawals_history(&pool, &redis_pool, query.into_inner()).await;
    return_response(result).await
}