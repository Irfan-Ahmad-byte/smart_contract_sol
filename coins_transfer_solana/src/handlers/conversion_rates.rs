use crate::config::app_config::AppState;
use crate::responses::http_response_impl::return_response;
use crate::responses::success_msgs_impl::create_error_response;
use crate::services::conversion_rates::get_rate;
use actix_web::{Responder, web};
use std::sync::Arc;

pub async fn conversion_rate_get_handler(app_state: web::Data<Arc<AppState>>, symbol: web::Path<String>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let symbol = symbol.into_inner();
    let result = get_rate(&pool, &redis_pool, symbol).await;
    return_response(result).await
}
