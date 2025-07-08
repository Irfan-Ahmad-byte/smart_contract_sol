use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ConfigCreate {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ConfigUpdate {
    pub name: Option<String>,
    pub value: Option<String>,
}
