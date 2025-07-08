use actix_web::HttpResponse;

use crate::responses::error_msgs::Error;
use crate::responses::success_msgs::SuccessMessages;
use crate::responses::success_msgs_impl::{create_error_response, create_success_response};

pub async fn return_response(result: Result<SuccessMessages, Error>) -> HttpResponse {
    match result {
        Ok(response) => {
            let (message_key, message, data, _) = response.to_response();
            create_success_response(message_key, message, data)
        }
        Err(e) => {
            let (message_key, message, data, status_code) = e.to_response();
            create_error_response(message_key, message, data, status_code)
        }
    }
}
