use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationRule {
    pub field: String,
    pub rules: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldValidationErrors {
    pub field: String,
    pub errors: Vec<FieldError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldError {
    pub message: String,
    pub message_key: String,
    pub params: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationErrors {
    pub errors: Vec<FieldValidationErrors>,
}

#[allow(dead_code)]
impl ValidationRule {
    pub fn new(field: String, rules: String) -> Self {
        Self { field, rules }
    }
}
