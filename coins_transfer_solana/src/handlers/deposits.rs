use crate::config::app_config::AppState;
use crate::responses::http_response_impl::return_response;
use crate::responses::success_msgs_impl::create_error_response;
use crate::services::deposits::get_transactions_history;
use crate::structs::history::TransactionsHistoryRequest;
use actix_web::{Responder, web};
use std::sync::Arc;

pub async fn get_transaction_history_handler(app_state: web::Data<Arc<AppState>>, query: web::Query<TransactionsHistoryRequest>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = get_transactions_history(&pool, &redis_pool, query.into_inner()).await;
    return_response(result).await
}