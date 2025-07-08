use std::collections::HashMap;

use actix_web::web;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;

use crate::structs::request_validation::{FieldError, FieldValidationErrors, ValidationErrors, ValidationRule};

fn parse_float(string: &str) -> bool {
    string.parse::<f64>().is_ok()
}

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z]+$").unwrap();
    static ref SPECIAL_CHAR_REGEX: Regex = Regex::new(r"[^\w\s]").unwrap();
    static ref ALPHANUMERIC_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9]+$").unwrap();
    static ref URL_REGEX: Regex = Regex::new(r"^(http|https)://[^\s/$.?#].[^\s]*$").unwrap();
}

fn is_valid_email(email: &str) -> bool {
    EMAIL_REGEX.is_match(email)
}

fn contains_special_chars(string: &str) -> bool {
    SPECIAL_CHAR_REGEX.is_match(string)
}

fn is_alphanumeric(string: &str) -> bool {
    ALPHANUMERIC_REGEX.is_match(string)
}

fn is_valid_url(url: &str) -> bool {
    URL_REGEX.is_match(url)
}

fn parse_date(date_str: &str) -> bool {
    DateTime::parse_from_rfc3339(date_str).is_ok()
}

fn parse_offset_datetime(datetime_str: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc3339(datetime_str).map(|dt| dt.with_timezone(&Utc))
}

#[allow(dead_code)]
pub async fn validate_request(req: web::Json<HashMap<String, Value>>, rules: &[ValidationRule]) -> Result<web::Json<HashMap<String, Value>>, ValidationErrors> {
    let mut validation_errors = ValidationErrors { errors: vec![] };
    let mut is_error = false;

    let _request = req.clone();

    for rule in rules {
        let field_value = req.get(&rule.field);
        let mut _is_required = false;
        let mut _is_given = false;

        let mut field_value_string = String::new();
        let mut _value = Value::Null;

        if let Some(val) = field_value {
            _value = val.clone();
            field_value_string = match &_value {
                Value::String(val) => val.clone(),
                Value::Number(num) => num.to_string(),
                _ => String::new(),
            };
        }

        let field_name = rule.field.clone();
        let rules_list = rule.rules.split('|');
        let mut field_error = FieldValidationErrors { field: field_name.clone(), errors: vec![] };

        for rule_string in rules_list {
            let (rule_name, rule_value, _rule_params) = if let Some(_pos) = rule_string.find(':') {
                let parts: Vec<&str> = rule_string.splitn(2, ':').collect();
                let params: HashMap<String, String> = parts[1]
                    .split(',')
                    .collect::<Vec<&str>>()
                    .chunks(2)
                    .filter_map(|chunk| if chunk.len() == 2 { Some((chunk[0].to_string(), chunk[1].to_string())) } else { None })
                    .collect();
                (parts[0], parts[1], params)
            } else {
                (rule_string, "", HashMap::new())
            };

            match rule_name {
                "required" => {
                    _is_required = true;
                    if field_value.is_none() || field_value_string.is_empty() {
                        is_error = true;
                        let mut error_detail = HashMap::new();
                        error_detail.insert("field".to_string(), Value::String(field_name.clone()));

                        field_error.errors.push(FieldError { message: format!("{field_name} is required"), message_key: "required".to_string(), params: error_detail });
                    } else {
                        _is_given = true;
                    }
                }
                "numeric" if field_value.is_some() => {
                    if field_value.unwrap().as_u64().is_none() {
                        is_error = true;
                        let mut error_detail = HashMap::new();
                        error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                        error_detail.insert("be".to_string(), Value::String("numeric".to_string()));

                        field_error.errors.push(FieldError { message: format!("{field_name} must be a number"), message_key: "must_be".to_string(), params: error_detail });
                    }
                }
                "email" if field_value.is_some() => {
                    if !is_valid_email(&field_value_string) {
                        is_error = true;
                        let mut error_detail = HashMap::new();
                        error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                        error_detail.insert("be".to_string(), Value::String("valid email".to_string()));

                        field_error
                            .errors
                            .push(FieldError { message: format!("{field_name} must be a valid email"), message_key: "must_be".to_string(), params: error_detail });
                    }
                }
                "date" if field_value.is_some() => {
                    if let Some(date_str) = field_value.unwrap().as_str() {
                        if !parse_date(date_str) {
                            is_error = true;
                            let mut error_detail = HashMap::new();
                            error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                            error_detail.insert("be".to_string(), Value::String("valid date".to_string()));

                            field_error
                                .errors
                                .push(FieldError { message: format!("{field_name} must be a valid date"), message_key: "must_be".to_string(), params: error_detail });
                        }
                    }
                }
                "float" if field_value.is_some() => {
                    if !parse_float(&field_value_string) {
                        is_error = true;
                        let mut error_detail = HashMap::new();
                        error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                        error_detail.insert("be".to_string(), Value::String("float value".to_string()));

                        field_error
                            .errors
                            .push(FieldError { message: format!("{field_name} must be a float value"), message_key: "must_be".to_string(), params: error_detail });
                    }
                }
                "regex" if field_value.is_some() => {
                    if !Regex::new(rule_value).is_ok_and(|re| re.is_match(&field_value_string)) {
                        is_error = true;
                        let mut error_detail = HashMap::new();
                        error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                        error_detail.insert("match".to_string(), Value::String(rule_value.to_string()));

                        field_error
                            .errors
                            .push(FieldError { message: format!("{field_name} must match {rule_value}"), message_key: "must_match".to_string(), params: error_detail });
                    }
                }
                "min" if field_value.is_some() => {
                    if let Ok(min_value) = rule_value.parse::<f64>() {
                        if field_value.unwrap().as_f64().unwrap_or(0.0) < min_value {
                            is_error = true;
                            let mut error_detail = HashMap::new();
                            error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                            error_detail.insert("value".to_string(), Value::String(rule_value.to_string()));

                            field_error
                                .errors
                                .push(FieldError { message: format!("{field_name} must be at least {rule_value}"), message_key: "at_least".to_string(), params: error_detail });
                        }
                    }
                }
                "max" if field_value.is_some() => {
                    if let Ok(max_value) = rule_value.parse::<f64>() {
                        if field_value.unwrap().as_f64().unwrap_or(0.0) > max_value {
                            is_error = true;
                            let mut error_detail = HashMap::new();
                            error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                            error_detail.insert("value".to_string(), Value::String(rule_value.to_string()));

                            field_error
                                .errors
                                .push(FieldError { message: format!("{field_name} must be less than {rule_value}"), message_key: "max".to_string(), params: error_detail });
                        }
                    }
                }
                "length" | "len" if field_value.is_some() => {
                    if let Ok(length) = rule_value.parse::<usize>() {
                        if field_value.unwrap().as_str().unwrap_or("").len() != length {
                            is_error = true;
                            let mut error_detail = HashMap::new();
                            error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                            error_detail.insert("value".to_string(), Value::String(rule_value.to_string()));

                            field_error
                                .errors
                                .push(FieldError { message: format!("{field_name} must be exactly {rule_value}"), message_key: "must_be_exactly".to_string(), params: error_detail });
                        }
                    }
                }
                "min_len" if field_value.is_some() => {
                    if let Ok(min_length) = rule_value.parse::<usize>() {
                        if field_value_string.len() < min_length {
                            is_error = true;
                            let mut error_detail = HashMap::new();
                            error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                            error_detail.insert("value".to_string(), Value::String(rule_value.to_string()));

                            field_error
                                .errors
                                .push(FieldError { message: format!("{field_name} must be at least {rule_value}"), message_key: "min_length".to_string(), params: error_detail });
                        }
                    }
                }
                "in" if field_value.is_some() => {
                    // Check if the field value is in the specified list
                    if !rule_value.split(',').any(|val| val == field_value_string) {
                        is_error = true;
                        let mut error_detail = HashMap::new();
                        error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                        error_detail.insert("in".to_string(), Value::String(rule_value.to_string()));

                        field_error
                            .errors
                            .push(FieldError { message: format!("{field_name} must be one of {rule_value}"), message_key: "must_be_in".to_string(), params: error_detail });
                    }
                }
                "gt" if field_value.is_some() => {
                    // Check if the field value is greater than the specified value
                    if let Ok(gt_value) = rule_value.parse::<f64>() {
                        if field_value.unwrap().as_f64().unwrap_or(0.0) <= gt_value {
                            is_error = true;
                            let mut error_detail = HashMap::new();
                            error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                            error_detail.insert("value".to_string(), Value::String(rule_value.to_string()));

                            field_error
                                .errors
                                .push(FieldError { message: format!("{field_name} must be greater than {rule_value}"), message_key: "greater_than".to_string(), params: error_detail });
                        }
                    } else if let Ok(gt_date) = parse_offset_datetime(rule_value) {
                        if let Ok(date) = parse_offset_datetime(&field_value_string) {
                            if date < gt_date {
                                is_error = true;
                                let mut error_detail = HashMap::new();
                                error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                                error_detail.insert("value".to_string(), Value::String(rule_value.to_string()));

                                field_error
                                    .errors
                                    .push(FieldError { message: format!("{field_name} must be greater than {rule_value}"), message_key: "greater_than".to_string(), params: error_detail });
                            }
                        }
                    }
                }
                "lt" if field_value.is_some() => {
                    // Check if the field value is less than the specified value
                    if let Ok(lt_value) = rule_value.parse::<f64>() {
                        if field_value.unwrap().as_f64().unwrap_or(0.0) >= lt_value {
                            is_error = true;
                            let mut error_detail = HashMap::new();
                            error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                            error_detail.insert("value".to_string(), Value::String(rule_value.to_string()));

                            field_error
                                .errors
                                .push(FieldError { message: format!("{field_name} must be less than {rule_value}"), message_key: "less_than".to_string(), params: error_detail });
                        }
                    } else if let Ok(lt_date) = parse_offset_datetime(rule_value) {
                        if let Ok(date) = parse_offset_datetime(&field_value_string) {
                            if date >= lt_date {
                                is_error = true;
                                let mut error_detail = HashMap::new();
                                error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                                error_detail.insert("value".to_string(), Value::String(rule_value.to_string()));

                                field_error
                                    .errors
                                    .push(FieldError { message: format!("{field_name} must be less than {rule_value}"), message_key: "less_than".to_string(), params: error_detail });
                            }
                        }
                    }
                }
                "bool" | "boolean" if field_value.is_some() => {
                    // Check if the field value is a boolean
                    if !field_value.unwrap().is_boolean() {
                        is_error = true;
                        let mut error_detail = HashMap::new();
                        error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                        error_detail.insert("value".to_string(), Value::String(rule_value.to_string()));

                        field_error
                            .errors
                            .push(FieldError { message: format!("{field_name} must be a boolean"), message_key: "must_be_a_boolean".to_string(), params: error_detail });
                    }
                }
                "same_as" if field_value.is_some() => {
                    // Check if the field value is the same as the specified field's value
                    if let Some(same_as_value) = req.get(rule_value) {
                        if field_value_string != same_as_value.as_str().unwrap_or("") {
                            is_error = true;
                            let mut error_detail = HashMap::new();
                            error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                            error_detail.insert("value".to_string(), Value::String(rule_value.to_string()));

                            field_error
                                .errors
                                .push(FieldError { message: format!("{field_name} must be the same as {rule_value}"), message_key: "same_as".to_string(), params: error_detail });
                        }
                    }
                }
                "no_special_chars" if field_value.is_some() => {
                    // Check if the field value contains special characters
                    if contains_special_chars(&field_value_string) {
                        is_error = true;
                        let mut error_detail = HashMap::new();
                        error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                        error_detail.insert("value".to_string(), Value::String(rule_value.to_string()));

                        field_error
                            .errors
                            .push(FieldError { message: format!("{field_name} must not contain special characters"), message_key: "no_special_chars".to_string(), params: error_detail });
                    }
                }
                "alphanumeric" if field_value.is_some() => {
                    if !is_alphanumeric(&field_value_string) {
                        is_error = true;
                        let mut error_detail = HashMap::new();
                        error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                        error_detail.insert("value".to_string(), Value::String(rule_value.to_string()));

                        field_error
                            .errors
                            .push(FieldError { message: format!("{field_name} must be alphanumeric"), message_key: "must_be_alphanumeric".to_string(), params: error_detail });
                    }
                }
                "url" if field_value.is_some() => {
                    if !is_valid_url(&field_value_string) {
                        is_error = true;
                        let mut error_detail = HashMap::new();
                        error_detail.insert("field".to_string(), Value::String(field_name.clone()));
                        error_detail.insert("value".to_string(), Value::String(rule_value.to_string()));

                        field_error
                            .errors
                            .push(FieldError { message: format!("{field_name} must be a valid URL"), message_key: "must_be_valid_url".to_string(), params: error_detail });
                    }
                }
                _ => {}
            }
        }

        if !field_error.errors.is_empty() {
            validation_errors.errors.push(field_error);
        }
    }

    if is_error { Err(validation_errors) } else { Ok(req) }
}
