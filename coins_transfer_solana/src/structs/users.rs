use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCreate {
    pub user_id: i64,
    pub event_id: Option<i64>,
}
