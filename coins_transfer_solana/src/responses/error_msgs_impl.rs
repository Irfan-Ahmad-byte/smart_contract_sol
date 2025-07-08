use actix_web::http::StatusCode;
use serde_json::{Value, json};

use crate::responses::error_msgs::{Error, StructError};

impl Error {
    pub fn to_response(&self) -> (&str, String, Value, StatusCode) {
        let message = self.to_string();
        let data = Value::String(message.clone());

        match self {
            Error::NotFound(m) => {
                let message = m.clone();
                ("not_found", message, data, StatusCode::NOT_FOUND)
            }
            Error::DuplicateEntry => ("duplicate_entry", message, data, StatusCode::CONFLICT),
            Error::MissingField(field) => ("missing_field", message, json!({ "field": field }), StatusCode::BAD_REQUEST),
            Error::RpcIssue => ("rpc_issue", message, data, StatusCode::INTERNAL_SERVER_ERROR),
            Error::InvalidAmount => ("invalid_amount", message, data, StatusCode::BAD_REQUEST),
            Error::TransactionInProgress => ("transaction_in_progress", message, data, StatusCode::BAD_REQUEST),
            Error::TransactionNotConfirmed => ("transaction_not_confirmed", message, data, StatusCode::BAD_REQUEST),
            Error::GreaterThanMaximumTransfer => ("greater_than_maximum_transfer", message, data, StatusCode::BAD_REQUEST),
            Error::InsufficientBalance => ("insufficient_balance", message, data, StatusCode::BAD_REQUEST),
            Error::LessThanMinimumTransfer => ("less_than_minimum_transfer", message, data, StatusCode::BAD_REQUEST),
            Error::InvalidAddress => ("invalid_address", message, data, StatusCode::BAD_REQUEST),
            Error::DepositAlreadyRecorded => ("deposit_already_recorded", message, data, StatusCode::BAD_REQUEST),
            Error::UserIdMismatch => ("user_id_mismatch", message, data, StatusCode::BAD_REQUEST),
            _ => {
                let status_code = StatusCode::INTERNAL_SERVER_ERROR;
                let error_key = match self {
                    Error::EnvVarMissing(_) => "env_var_missing",
                    Error::InvalidConfiguration => "invalid_configuration",
                    Error::RedisIssue | Error::DatabaseIssue | Error::TechnicalIssue | Error::SerializationIssue => "system_issue",
                    _ => "system_issue",
                };
                (error_key, message, data, status_code)
            }
        }
    }
}

impl StructError {
    pub fn to_response(&self) -> (StatusCode, Value) {
        match self {
            StructError::InvalidJSON(field, expected, found) => {
                (StatusCode::BAD_REQUEST, json!({"message": format!("You entered \"{}\" for \"{}\", which is of incorrect data type. Expected data type was \"{}\".", found, field, expected), "message_key": "invalid_json"}))
            }
            StructError::EmptyData => (StatusCode::BAD_REQUEST, json!({"message": "No data provided", "message_key": "empty_data"})),
            StructError::DeserializationError(field) => (StatusCode::BAD_REQUEST, json!({"message": format!("{} is required", field), "message_key": "required", "data": { "field": field }})),
        }
    }
}
