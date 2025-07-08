use actix_web::{HttpResponse, http::StatusCode};
use serde_json::{Value, json};

pub fn create_response_message(message_key: &str, message: String, data: Value) -> Value {
    let response = json!({
        "message_key": message_key,
        "message": message,
        "data": data
    });
    response
}

pub fn create_success_response(message_key: &str, message: String, data: Value) -> HttpResponse {
    let response = create_response_message(message_key, message, data);

    HttpResponse::Ok().json(response)
}

pub fn create_error_response(message_key: &str, message: String, data: Value, status_code: StatusCode) -> HttpResponse {
    let response = create_response_message(message_key, message, data);

    HttpResponse::build(status_code).json(response)
}
