use bigdecimal::BigDecimal;

// custom function to limit precision to specified number of decimal places with truncation
pub fn truncate_decimal(value: &BigDecimal, precision: u32) -> BigDecimal {
    // Create a scale based on the specified precision
    // let scale = 10u32.pow(precision);
    // Truncate the value to the specified scale
    value.with_scale(precision as i64)
}
