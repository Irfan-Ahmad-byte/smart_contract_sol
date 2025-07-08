use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ConversionRates {
    pub id: i64,
    pub coin_id: i16,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub conversion_rate: BigDecimal,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
