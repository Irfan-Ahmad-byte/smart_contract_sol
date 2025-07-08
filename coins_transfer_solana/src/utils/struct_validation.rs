use actix_web::error::{InternalError, JsonPayloadError};
use actix_web::{Error as ActixError, HttpRequest, HttpResponse};
use crypsol_logger::log;
use log::Level;

use crate::responses::error_msgs::StructError;

pub fn custom_error_handler(err: JsonPayloadError, req: &HttpRequest) -> ActixError {
    let error = if let Some(len) = req.headers().get("content-length") {
        if len.to_str().unwrap_or_default() == "0" {
            StructError::EmptyData
        } else {
            match &err {
                JsonPayloadError::Deserialize(err) => {
                    let error_message = err.to_string();
                    let mut field_name = String::new();
                    let mut expected_type = String::new();
                    let mut found_type = String::new();

                    if error_message.contains("invalid type") {
                        let parts: Vec<&str> = error_message.split(" at ").collect();
                        if parts.len() > 1 {
                            let type_info: Vec<&str> = parts[0].split(", expected ").collect();
                            if type_info.len() > 1 {
                                expected_type = type_info[1].to_string();
                            }
                            let field_info: Vec<&str> = type_info[0].split(" `").collect();
                            if field_info.len() > 1 {
                                field_name = field_info[1].split("`").next().unwrap_or_default().to_string();
                                found_type = field_info[0].split("`").last().unwrap_or_default().to_string();
                            }
                        }
                    } else if error_message.contains("missing field `") {
                        field_name = error_message.split("missing field `").nth(1).unwrap_or_default().split('`').next().unwrap_or_default().to_string();
                    }

                    if !field_name.is_empty() {
                        if !expected_type.is_empty() {
                            StructError::InvalidJSON(field_name, expected_type, found_type)
                        } else {
                            StructError::DeserializationError(field_name)
                        }
                    } else {
                        StructError::InvalidJSON(error_message, String::new(), String::new())
                    }
                }
                _ => StructError::InvalidJSON("Invalid JSON format".to_string(), String::new(), String::new()),
            }
        }
    } else {
        StructError::InvalidJSON("Invalid JSON format".to_string(), String::new(), String::new())
    };

    let (status_code, error_response) = error.to_response();
    log!(Level::Error, "Error: {:?}", error);

    InternalError::from_response("", HttpResponse::build(status_code).content_type("application/json").body(error_response.to_string())).into()
}
