use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversionRateAdd {
    pub coin_symbol: String,
    #[serde(with = "bigdecimal::serde::json_num")]
    pub conversion_rate: BigDecimal,
}
