use crate::cache::coins::get_all_coins_from_cache;
use crate::cache::conversion_rates::{get_conversion_rate_from_cache, set_conversion_rate_cache};
use crate::entities::conversion_rates::ConversionRates;
use crate::responses::error_msgs::Error;
use crate::responses::success_msgs::SuccessMessages;
use crate::services::coins::{get_a_coin_from_db_by_id, get_a_coin_from_db_by_symbol, update_all_coins_cache};
use crate::services::configs::get_a_config;
use crate::structs::conversion_rates::ConversionRateAdd;
use crate::utils::time::TimeHandler;
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use crypsol_logger::log;
use deadpool_redis::Pool;
use log::Level;
use serde_json::Value;
use sqlx::{PgPool, query, query_as};
use std::collections::HashMap;

pub async fn add_rate(pool: &PgPool, redis_pool: &Pool, rate: ConversionRateAdd) -> Result<SuccessMessages, Error> {
    let coin = get_a_coin_from_db_by_symbol(pool, &rate.coin_symbol.clone()).await?;

    let time_handler = TimeHandler::new();
    let now = time_handler.get_current_time().naive_utc();

    let result = query!(
        r#"
        INSERT INTO conversion_rates (coin_id, conversion_rate, created_at, updated_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id, coin_id
        "#,
        coin.id,
        rate.conversion_rate,
        now,
        now
    )
    .fetch_one(pool)
    .await;

    let new_coin = match result {
        Ok(record) => ConversionRates { id: record.id, coin_id: coin.id, conversion_rate: rate.conversion_rate, created_at: now, updated_at: now },
        Err(e) => {
            log!(Level::Error, "Database error on insert of conversion rate adding request: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };

    set_conversion_rate_cache(redis_pool, &new_coin).await?;

    Ok(SuccessMessages::ConversionRate { coin_id: new_coin.coin_id, symbol: rate.coin_symbol, rate: new_coin.conversion_rate.to_f64().unwrap_or(0.0) })
}

pub async fn get_rate(pool: &PgPool, redis_pool: &Pool, symbol: String) -> Result<SuccessMessages, Error> {
    let coin = get_a_coin_from_db_by_symbol(pool, &symbol.clone()).await?;

    let result = get_conversion_rate_from_cache(redis_pool, coin.id).await;
    if let Some(rate) = result {
        return Ok(SuccessMessages::ConversionRate { coin_id: coin.id, symbol, rate: rate.conversion_rate.to_f64().unwrap_or(0.0) });
    }

    let result = query_as!(
        ConversionRates,
        r#"
        SELECT * FROM conversion_rates WHERE coin_id = $1 ORDER BY created_at DESC LIMIT 1
        "#,
        coin.id
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(rate) => Ok(SuccessMessages::ConversionRate { coin_id: coin.id, symbol: coin.symbol, rate: rate.conversion_rate.to_f64().unwrap_or(0.0) }),
        Err(sqlx::Error::RowNotFound) => {
            log!(Level::Error, "Conversion rate not found for coin : {}", symbol);
            Err(Error::NotFound("Conversion rate not found".to_string()))
        }
        Err(e) => {
            log!(Level::Error, "Database error while getting conversion rate by coin symbol: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn get_rate_by_coin_id(pool: &PgPool, redis_pool: &Pool, coin_id: i16) -> Result<BigDecimal, Error> {
    let result = get_conversion_rate_from_cache(redis_pool, coin_id).await;
    if let Some(rate) = result {
        return Ok(rate.conversion_rate);
    }

    let result = query_as!(
        ConversionRates,
        r#"
        SELECT * FROM conversion_rates WHERE coin_id = $1 ORDER BY created_at DESC
        "#,
        coin_id
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(rate) => Ok(rate.conversion_rate),
        Err(sqlx::Error::RowNotFound) => {
            let api_key = get_a_config(pool, redis_pool, "COIN_MARKET_CAP_API_KEY".to_string()).await.unwrap_or("".to_string());
            let rate = fetch_and_store_single_coin_rate(pool.clone(), redis_pool.clone(), &api_key, Some(coin_id), None).await?;
            Ok(rate)
        }
        Err(e) => {
            log!(Level::Error, "Database error while getting conversion rate by coin id: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

async fn fetch_coin_rate_from_api(pool: &PgPool, redis_pool: &Pool, api_key: &str, symbol: &str) -> Result<BigDecimal, Error> {
    let coin_market_base_url = get_a_config(pool, redis_pool, "COIN_MARKET_CAP_BASE_URL".to_string()).await.unwrap_or("".to_string());
    let url = format!("{coin_market_base_url}/v1/tools/price-conversion?amount=1&symbol={symbol}&convert=USD");

    let client = reqwest::Client::new();
    let response = match client.get(&url).header("X-CMC_PRO_API_KEY", api_key).send().await {
        Ok(response) => response,
        Err(e) => {
            log!(Level::Error, "Failed to fetch coin rate from API: {}", e);
            return Err(Error::TechnicalIssue);
        }
    };

    let api_response: HashMap<String, Value> = response.json().await.map_err(|e| {
        log!(Level::Error, "Failed to parse API response from Coin Market Cap API request: {}", e);
        Error::TechnicalIssue
    })?;

    let price = match api_response
        .get("data")
        .and_then(|data| data.get("quote"))
        .and_then(|quote| quote.get("USD"))
        .and_then(|usd| usd.get("price"))
        .and_then(|price| price.as_f64())
    {
        Some(price) => price,
        None => {
            log!(Level::Error, "Failed to extract price from API response");
            0.0
        }
    };
    let precise_price = BigDecimal::from_f64(price).unwrap().round(8);
    Ok(precise_price)
}

async fn fetch_and_store_single_coin_rate(pool: PgPool, redis_pool: Pool, api_key: &str, coin_id: Option<i16>, coin_symbol: Option<String>) -> Result<BigDecimal, Error> {
    if let Some(coin_id) = coin_id {
        let coin = get_a_coin_from_db_by_id(&pool, coin_id).await?;
        let data = fetch_coin_rate_from_api(&pool, &redis_pool, api_key, &coin.symbol).await?;
        let rate_data = ConversionRateAdd { coin_symbol: coin.symbol, conversion_rate: data.clone() };
        if let Err(e) = add_rate(&pool, &redis_pool, rate_data).await {
            log!(Level::Error, "Error while storing single coin rate: {:?}", e);
            return Err(e);
        }
        Ok(data)
    } else if let Some(coin_symbol) = coin_symbol {
        let data = fetch_coin_rate_from_api(&pool, &redis_pool, api_key, &coin_symbol).await?;
        let rate_data = ConversionRateAdd { coin_symbol, conversion_rate: data.clone() };
        if let Err(e) = add_rate(&pool, &redis_pool, rate_data).await {
            log!(Level::Error, "Error while storing single coin rate: {:?}", e);
            return Err(e);
        }
        return Ok(data);
    } else {
        log!(Level::Error, "Coin ID or Symbol not provided while fetching coin rate");
        return Err(Error::TechnicalIssue);
    }
}

pub async fn fetch_and_store_coin_rates(pool: PgPool, redis_pool: Pool) -> Result<(), Error> {
    let api_key = get_a_config(&pool, &redis_pool, "COIN_MARKET_CAP_API_KEY".to_string()).await.unwrap_or("".to_string());

    let active_coins = match get_all_coins_from_cache(&redis_pool).await {
        Some(coins) => coins,
        _ => update_all_coins_cache(&pool, &redis_pool).await?,
    };

    // let mut tasks = Vec::new();

    for coin in active_coins {
        if coin.symbol.trim().to_lowercase().contains("usd") {
            // log!(Level::Info, "Skipping USD coin as it is not needed for conversion rates");
            continue;
        }
        let api_key_clone = api_key.clone();
        let pool_clone = pool.clone();
        let redis_clone = redis_pool.clone();
        // let task = tokio::spawn(async move { fetch_and_store_single_coin_rate(pool_clone, redis_clone, &api_key_clone, None, Some(coin.symbol)).await });
        // tasks.push(task);

        let update_try = fetch_and_store_single_coin_rate(pool_clone, redis_clone, &api_key_clone, None, Some(coin.symbol.clone())).await;
        match update_try {
            Ok(_rate) => {}
            Err(e) => {
                log!(Level::Error, "Error in fetching/storing rate: {:?}", e);
            }
        }
    }

    Ok(())
}
