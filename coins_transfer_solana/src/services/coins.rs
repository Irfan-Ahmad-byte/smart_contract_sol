use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use crypsol_logger::log;
use deadpool_redis::Pool;
use log::Level;
use sqlx::{PgPool, query, query_as};

use crate::cache::coins::{
    get_all_coins_from_cache, get_coin_from_cache_by_id,
    increment_all_coins_cache, set_all_coins_cache, set_coin_cache_by_id, set_coin_cache_by_name, set_coin_cache_by_symbol,
};
use crate::entities::coins::Coins;
use crate::responses::error_msgs::Error;
use crate::responses::success_msgs::SuccessMessages;
use crate::services::conversion_rates::get_rate_by_coin_id;
use crate::services::rpc_client::RpcClient;
use crate::services::users::fetch_a_user;
use crate::services::wallets::{get_user_address, get_wallet_by_address, get_wallet_by_user_and_coin, validate_user_address};
use crate::services::withdrawals::{get_user_withdrawal_details, process_withdrawal};
use crate::structs::coins::{AddressGenerationRequest, AddressValidationRequest, CoinCreate, CoinInfo};
use crate::structs::withdrawals::WithdrawalCreate;

pub async fn create_coin(pool: &PgPool, redis_pool: &Pool, coin: CoinCreate) -> Result<SuccessMessages, Error> {
    let result = query!(
        r#"
        INSERT INTO coins (coin_name, symbol, status, created_at, updated_at)
        VALUES ($1, $2, $3, NOW(), NOW())
        RETURNING id, coin_name, symbol, status, created_at, updated_at
        "#,
        coin.coin_name.clone(),
        coin.symbol.clone(),
        coin.status
    )
    .fetch_one(pool)
    .await;

    let new_coin = match result {
        Ok(record) => Coins { id: record.id, coin_name: record.coin_name, symbol: record.symbol, status: record.status, created_at: record.created_at, updated_at: record.updated_at },
        Err(sqlx::Error::Database(e)) => {
            return if e.is_unique_violation() {
                if e.constraint().filter(|c| *c == "coins_coin_name_key").is_some() {
                    log!(Level::Error, "Duplicate Entry: for coin name {:?}", coin.coin_name);
                } else {
                    log!(Level::Error, "Duplicate Entry: for coin symbol {:?}", coin.symbol);
                }
                Err(Error::DuplicateEntry)
            } else {
                log!(Level::Error, "Database error on insert: {:?}", e);
                Err(Error::DatabaseIssue)
            };
        }
        Err(e) => {
            log!(Level::Error, "Database error on insert: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };
    let _ = set_coin_cache_by_id(redis_pool, &new_coin).await;
    let _ = set_coin_cache_by_name(redis_pool, &new_coin).await;
    let _ = set_coin_cache_by_symbol(redis_pool, &new_coin).await;
    let _ = increment_all_coins_cache(redis_pool, &new_coin).await;

    Ok(SuccessMessages::CreatedCoin { coin_id: new_coin.id, coin_name: new_coin.coin_name })
}

pub async fn fetch_a_coin_by_id(pool: &PgPool, redis_pool: &Pool, coin_id: i16) -> Result<Coins, Error> {
    if let Some(coin) = get_coin_from_cache_by_id(redis_pool, coin_id).await {
        return Ok(coin);
    }

    let coin = get_a_coin_from_db_by_id(pool, coin_id).await?;

    let _ = set_coin_cache_by_id(redis_pool, &coin).await;
    let _ = set_coin_cache_by_name(redis_pool, &coin).await;
    let _ = set_coin_cache_by_symbol(redis_pool, &coin).await;
    let _ = increment_all_coins_cache(redis_pool, &coin).await;

    Ok(coin)
}
pub async fn get_all_coins(pool: &PgPool, redis_pool: &Pool) -> Result<SuccessMessages, Error> {
    let coins = match get_all_coins_from_cache(redis_pool).await {
        Some(coins) => coins.into_iter().filter(|coin| coin.status).collect(),
        _ => update_all_coins_cache(pool, redis_pool).await?,
    };

    let mut coins_info = Vec::new();

    for coin in coins {
        let rate = get_rate_by_coin_id(pool, redis_pool, coin.id).await?;
        coins_info.push(CoinInfo { id: coin.id, coin_name: coin.coin_name.clone(), symbol: coin.symbol.clone(), status: coin.status, current_rate: rate });
    }

    Ok(SuccessMessages::FoundCoin { coin_id: 0, coins_list: Some(coins_info) })
}

pub async fn get_a_coin_from_db_by_id(pool: &PgPool, coin_id: i16) -> Result<Coins, Error> {
    let result = query_as!(
        Coins,
        r#"
        SELECT * FROM coins WHERE id = $1
        "#,
        coin_id
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(coin) => Ok(coin),
        Err(sqlx::Error::RowNotFound) => {
            log!(Level::Error, "Coin not found for id: {}", coin_id);
            Err(Error::NotFound("Coin not found".to_string()))
        }
        Err(e) => {
            log!(Level::Error, "Database error: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn get_a_coin_from_db_by_symbol(pool: &PgPool, symbol: &str) -> Result<Coins, Error> {
    let result = query_as!(
        Coins,
        r#"
        SELECT * FROM coins WHERE symbol = $1
        "#,
        symbol
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(coin) => Ok(coin),
        Err(sqlx::Error::RowNotFound) => {
            log!(Level::Error, "Coin not found for symbol: {}", symbol);
            Err(Error::NotFound("Coin not found".to_string()))
        }
        Err(e) => {
            log!(Level::Error, "Database error: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn update_all_coins_cache(pool: &PgPool, redis_pool: &Pool) -> Result<Vec<Coins>, Error> {
    let result = query_as!(
        Coins,
        r#"
        SELECT id, coin_name, symbol, status, created_at, updated_at FROM coins WHERE status = true
        "#
    )
    .fetch_all(pool)
    .await;
    match result {
        Ok(coins) => {
            let _ = set_all_coins_cache(redis_pool, &coins).await;
            Ok(coins)
        }
        Err(e) => {
            log!(Level::Error, "Database error in update_all_coins_cache: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn get_unconfirmed_user_balance(pool: &PgPool, redis_pool: &Pool, address: String) -> Result<SuccessMessages, Error> {
    let coin = get_wallet_by_address(pool, redis_pool, address.clone()).await?;
    let coin = fetch_a_coin_by_id(pool, redis_pool, coin.coin_id).await?;
    let rpc_client = RpcClient::new(pool, redis_pool, &coin.coin_name.to_uppercase()).await?;
    let (unconfirmed_balance, confirmations) = rpc_client.get_unconfirmed_balance(&address).await?;

    Ok(SuccessMessages::UnconfirmedBalance { address, unconfirmed_balance, confirmations })
}

pub async fn generate_address(pool: &PgPool, redis_pool: &Pool, address_request: AddressGenerationRequest) -> Result<SuccessMessages, Error> {
    let _user = fetch_a_user(pool, redis_pool, address_request.user_id).await?;

    let coin = fetch_a_coin_by_id(pool, redis_pool, address_request.coin_id).await?;

    let wallet = match get_wallet_by_user_and_coin(pool, redis_pool, address_request.user_id, address_request.coin_id).await {
        Ok(SuccessMessages::CreatedWallet { user_id, wallet }) => {
            let _ = user_id;
            wallet
        }
        Ok(SuccessMessages::FoundWallet { user_id, wallets_list }) => {
            let _ = user_id;
            wallets_list[0].clone()
        }
        Ok(_res) => {
            return Err(Error::NotFound("Wallet not found".to_string()));
        }
        Err(e) => return Err(e),
    };

    if wallet.address.is_some() {
        return Ok(SuccessMessages::AddressGenerated { coin: coin.id, address: wallet.address.unwrap() });
    }

    get_user_address(pool, redis_pool, wallet, coin).await
}

pub async fn validate_address(pool: &PgPool, redis_pool: &Pool, address_request: AddressValidationRequest) -> Result<SuccessMessages, Error> {
    // get_coin_by_id(pool, redis_pool, address_request.coin_id).await?;

    match validate_user_address(pool, redis_pool, address_request.address.clone()).await {
        Ok(is_valid) => Ok(SuccessMessages::AddressValidated { is_valid, coin: address_request.coin_id, address: address_request.address }),
        Err(e) => Err(e),
    }
}

pub async fn get_node_balance(pool: &PgPool, redis_pool: &Pool, coin: i16) -> Result<SuccessMessages, Error> {
    let coin = fetch_a_coin_by_id(pool, redis_pool, coin).await?;
    let rpc_client = RpcClient::new(pool, redis_pool, &coin.coin_name.to_uppercase()).await?;
    let balance = rpc_client.get_balance().await?;
    let current_rate = get_rate_by_coin_id(pool, redis_pool, coin.id).await?;
    let usd_balance = BigDecimal::from_f64(balance).unwrap_or(BigDecimal::from(0)) * current_rate;
    Ok(SuccessMessages::NodeBalance { crypto_balance: BigDecimal::from_f64(balance).unwrap_or(BigDecimal::from(0)).round(4), usd_balance: usd_balance.round(4) })
}

pub async fn transfer_amount(pool: &PgPool, redis_pool: &Pool, transfer: WithdrawalCreate) -> Result<SuccessMessages, Error> {
    let coin = fetch_a_coin_by_id(pool, redis_pool, transfer.coin_id).await?;
    let mut trx = match pool.begin().await {
        Ok(trx) => trx,
        Err(e) => {
            log!(Level::Error, "Database error while starting transaction for amount transfer: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };
    let rate = get_rate_by_coin_id(pool, redis_pool, coin.id).await?.to_f64().map_or(0.0, |rate| rate);
    let coin_amount = transfer.usd_amount.clone().to_f64().unwrap() / rate;

    let rpc_client = RpcClient::new(pool, redis_pool, &coin.coin_name.to_uppercase()).await?;
    let hash = rpc_client.send_to_address(&transfer.address, transfer.usd_amount.clone().to_f64().unwrap()).await?;

    let coin_amount = BigDecimal::from_f64(coin_amount).unwrap();
    let network_fee = get_user_withdrawal_details(rpc_client, &hash, coin_amount.clone()).await?;
    let network_fee_usd = network_fee.clone() * BigDecimal::from_f64(rate).unwrap();

    let withdrawal = match process_withdrawal(&mut trx, redis_pool, transfer.clone(), coin.clone(), transfer.usd_amount, hash.clone(), network_fee, network_fee_usd).await {
        Ok(withdrawal) => withdrawal,
        Err(e) => {
            let _ = trx.rollback().await.map_err(|e| {
                log!(Level::Error, "Database error while rolling back transaction for amount transfer: {:?}", e);
                Error::DatabaseIssue
            });
            return Err(e);
        }
    };

    let _ = trx.commit().await.map_err(|e| {
        log!(Level::Error, "Database error while committing transaction for amount transfer: {:?}", e);
        Error::DatabaseIssue
    });

    Ok(SuccessMessages::TransferSuccessful { hash, amount: withdrawal.coin_amount.to_f64().unwrap() })
}
