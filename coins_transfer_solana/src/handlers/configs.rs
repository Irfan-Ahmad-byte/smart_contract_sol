use crate::config::app_config::AppState;
use crate::responses::http_response_impl::return_response;
use crate::responses::success_msgs_impl::create_error_response;
use crate::services::configs::{create_config, delete_all_configs, delete_config, get_all_configs, get_config_by_name, update_config};
use crate::structs::configs::{ConfigCreate, ConfigUpdate};
use actix_web::{Responder, web};
use std::sync::Arc;

pub async fn create_config_handler(app_state: web::Data<Arc<AppState>>, configs: web::Json<ConfigCreate>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = create_config(&pool, &redis_pool, configs.into_inner()).await;
    return_response(result).await
}

pub async fn get_config_handler(app_state: web::Data<Arc<AppState>>, name: web::Path<String>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = get_config_by_name(&pool, &redis_pool, name.into_inner()).await;
    return_response(result).await
}

pub async fn list_configs_handler(app_state: web::Data<Arc<AppState>>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = get_all_configs(&pool, &redis_pool).await;
    return_response(result).await
}

pub async fn update_config_handler(app_state: web::Data<Arc<AppState>>, name: web::Path<String>, configs_update: web::Json<ConfigUpdate>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = update_config(&pool, &redis_pool, name.into_inner(), configs_update.into_inner()).await;
    return_response(result).await
}

pub async fn delete_config_handler(app_state: web::Data<Arc<AppState>>, name: web::Path<String>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = delete_config(&pool, &redis_pool, name.into_inner()).await;
    return_response(result).await
}

pub async fn delete_all_config_handler(app_state: web::Data<Arc<AppState>>) -> impl Responder {
    let pool = app_state.db.clone();
    let redis_pool = match app_state.get_redis_pool() {
        Ok(pool) => pool,
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            return create_error_response(message_key, message, data, status_code);
        }
    };
    let result = delete_all_configs(&pool, &redis_pool).await;
    return_response(result).await
}
