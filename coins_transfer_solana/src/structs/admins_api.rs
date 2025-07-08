use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AdminApiCreate {
    pub name: String,
    pub status: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct AdminApiUpdate {
    pub name: Option<String>,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub status: Option<bool>,
}
