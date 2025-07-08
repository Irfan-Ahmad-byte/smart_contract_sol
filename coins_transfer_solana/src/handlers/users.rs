use crate::config::app_config::AppState;
use crate::responses::http_response_impl::return_response;
use crate::responses::success_msgs_impl::create_error_response;
use crate::services::users::{create_user, rollback_user_creation};
use crate::structs::users::UserCreate;
use actix_web::{Responder, web};
use std::sync::Arc;

pub async fn create_user_handler(app_state: web::Data<Arc<AppState>>, user: web::Json<UserCreate>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = create_user(&pool, &redis_pool, user.into_inner()).await;
    return_response(result).await
}

pub async fn rollback_user_creation_handler(app_state: web::Data<Arc<AppState>>, event_id: web::Path<i64>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = rollback_user_creation(&pool, &redis_pool, event_id.into_inner()).await;
    return_response(result).await
}