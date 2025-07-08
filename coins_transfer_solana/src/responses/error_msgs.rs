use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;
use std::io::Error as StdIoError;
use strum::EnumIter;
use thiserror::Error;

#[allow(dead_code, unused)]
#[derive(Debug, Error, Serialize, Deserialize, EnumIter)]
pub enum Error {
    #[error("Environment Variable Missing: {0}")]
    EnvVarMissing(String),

    #[error("Invalid Configuration")]
    InvalidConfiguration,

    #[error("System error detected. We're on it. Please retry later.")]
    RedisIssue,

    #[error("{0}.")]
    NotFound(String),

    #[error("Duplicate Entry")]
    DuplicateEntry,

    #[error("System error detected. We're on it. Please retry later.")]
    DatabaseIssue,

    #[error("System error detected. We're on it. Please retry later.")]
    TechnicalIssue,

    #[error("A transaction is already in progress.")]
    TransactionInProgress,

    #[error("Error making transaction.")]
    RpcIssue,

    #[error("Error making transaction: {0} is required.")]
    MissingField(String),

    #[error("Error making transaction: amount is invalid.")]
    InvalidAmount,

    #[error("System error detected. We're on it. Please retry later.")]
    SerializationIssue,

    #[error("Transaction is not yet confirmed.")]
    TransactionNotConfirmed,

    #[error("The requested transfer amount is greater than the maximum allowed.")]
    GreaterThanMaximumTransfer,

    #[error("You do not have sufficient balance to complete this transaction.")]
    InsufficientBalance,

    #[error("The requested transfer amount is less than the minimum allowed.")]
    LessThanMinimumTransfer,

    #[error("Invalid Address")]
    InvalidAddress,

    #[error("This deposit has already been processed.")]
    DepositAlreadyRecorded,

    #[error("User can not request deposit for other users. Though, you can transfer funds to other users.")]
    UserIdMismatch,
}

#[derive(Debug, Error, Serialize, Deserialize, EnumIter)]
pub enum StructError {
    #[error("Invalid JSON: {0}, Expected: {1}, Found: {2}")]
    InvalidJSON(String, String, String),

    #[error("Empty Data")]
    EmptyData,

    #[error("Deserialization Error: {0}")]
    DeserializationError(String),
}

#[derive(Debug, Error)]
pub enum ModuleError {
    #[error("Database Error: {0}")]
    Database(#[from] SqlxError),

    #[error("std Error: {0}")]
    Std(#[from] StdIoError),

    #[error("Application Error: {0}")]
    App(#[from] Error),

    #[error("Environment Variable Missing: {0}")]
    EnvVarMissing(String),

    #[error("Invalid Configuration")]
    InvalidConfiguration,

    #[error("System error detected. We're on it. Please retry later.")]
    DatabaseIssue,

    #[error("System error detected. We're on it. Please retry later.")]
    RedisIssue,
}