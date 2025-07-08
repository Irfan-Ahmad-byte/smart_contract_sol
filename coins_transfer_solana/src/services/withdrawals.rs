use crate::cache::withdrawals::{get_withdrawals_from_cache_by_user_id, set_withdrawal_cache_by_id, set_withdrawals_cache_by_user_id};
use crate::entities::coins::Coins;
use crate::entities::withdrawals::Withdrawals;
use crate::responses::error_msgs::Error;
use crate::responses::success_msgs::SuccessMessages;
use crate::services::coins::get_a_coin_from_db_by_id;
use crate::services::configs::get_a_config;
use crate::services::conversion_rates::get_rate_by_coin_id;
use crate::services::rpc_client::RpcClient;
use crate::services::users::get_a_user_from_db;
use crate::structs::history::{WithdrawalsHistoryRequest, WithdrawalsHistoryResult};
use crate::structs::withdrawals::{WithdrawalCreate, WithdrawalUpdate};
use crate::utils::decimal_functions::truncate_decimal;
use crate::utils::time::TimeHandler;
use crate::utils::transactions_lock::TransactionsLocks;
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use crypsol_logger::log;
use deadpool_redis::Pool;
use log::Level;
use sqlx::{PgPool, Postgres, Transaction, query, query_as};
use std::cmp::max;
use std::str::FromStr;
use solana_sdk::pubkey::Pubkey;
use crate::config::constants::Coin;
use crate::services::solana_client::{get_solana_wallet_keypair, SolanaClient};
use crate::services::wallets::get_wallet_keys_by_user_id;

pub async fn create_withdrawal(pool: &PgPool, redis_pool: &Pool, withdrawal: WithdrawalCreate) -> Result<SuccessMessages, Error> {
    let _user = get_a_user_from_db(pool, withdrawal.user_id).await?;

    let minimum_withdrawal = get_a_config(pool, redis_pool, "WITHDRAWAL_MINIMUM".to_string()).await?;
    let minimum_withdrawal = minimum_withdrawal.parse::<BigDecimal>().unwrap();
    let maximum_withdrawal = get_a_config(pool, redis_pool, "WITHDRAWAL_MAXIMUM".to_string()).await?;
    let maximum_transfer = maximum_withdrawal.parse::<BigDecimal>().unwrap();

    let is_locked = TransactionsLocks::is_locked(withdrawal.user_id);
    if is_locked {
        log!(Level::Error, "Withdrawal Transaction is already in progress for this user: {:?}", withdrawal.user_id);
        return Err(Error::TransactionInProgress);
    }

    TransactionsLocks::add_lock(withdrawal.user_id);

    let coin = get_a_coin_from_db_by_id(pool, withdrawal.coin_id).await.inspect_err(|_e| {
        TransactionsLocks::remove_lock(withdrawal.user_id);
    })?;
    let rate = get_rate_by_coin_id(pool, redis_pool, coin.id).await.inspect_err(|_e| {
        TransactionsLocks::remove_lock(withdrawal.user_id);
    })?;

    let coin_amount = withdrawal.usd_amount.clone() / rate.clone();
    if withdrawal.usd_amount < minimum_withdrawal {
        TransactionsLocks::remove_lock(withdrawal.user_id);
        return Err(Error::LessThanMinimumTransfer);
    }
    if withdrawal.usd_amount > maximum_transfer {
        TransactionsLocks::remove_lock(withdrawal.user_id);
        return Err(Error::GreaterThanMaximumTransfer);
    }
    // round to 4 decimal points
    let coin_amount = truncate_decimal(&coin_amount, 8);

    // match rpc_client.validate_address(&withdrawal.address).await {
    //     Ok(_) => {}
    //     Err(e) => {
    //         TransactionsLocks::remove_lock(withdrawal.user_id);
    //         log!(Level::Error, "Error validating withdrawal address: {:?}", e);
    //         return Err(Error::InvalidAddress);
    //     }
    // }

    let (tx_id, network_fee) = match process_chain_withdrawal(pool, redis_pool, coin.clone(), coin_amount.clone(), withdrawal.user_id, &withdrawal.address).await {
        Ok((tx_id, network_fee)) => (tx_id, network_fee),
        Err(e) => {
            TransactionsLocks::remove_lock(withdrawal.user_id);
            log!(Level::Error, "Error processing withdrawal: {:?}", e);
            return Err(Error::TechnicalIssue);
        }
    };
    let network_fee = truncate_decimal(&network_fee, 8);
    let network_fee_usd = network_fee.clone() * rate.clone();
    let network_fee_usd = truncate_decimal(&network_fee_usd, 8);

    let mut trx = pool.begin().await.map_err(|e| {
        log!(Level::Error, "Error beginning transaction for withdrawal request: {:?}", e);
        TransactionsLocks::remove_lock(withdrawal.user_id);
        Error::DatabaseIssue
    })?;

    let new_withdrawal = match process_withdrawal(&mut trx, redis_pool, withdrawal.clone(), coin, coin_amount, tx_id, network_fee, network_fee_usd).await {
        Ok(w) => {
            TransactionsLocks::remove_lock(withdrawal.user_id);
            trx.commit().await.map_err(|e| {
                log!(Level::Error, "Error commiting transaction for withdrawal request: {:?}", e);
                Error::DatabaseIssue
            })?;
            w
        }
        Err(e) => {
            TransactionsLocks::remove_lock(withdrawal.user_id);
            log!(Level::Error, "Error in process adding withdrawal transaction: {:?}", e);
            trx.rollback().await.map_err(|e| {
                log!(Level::Error, "Error rollingback transaction for withdrawal request: {:?}", e);
                Error::DatabaseIssue
            })?;
            return Err(Error::DatabaseIssue);
        }
    };

    Ok(SuccessMessages::CreatedWithdrawal { withdrawal_id: new_withdrawal.id, hash: new_withdrawal.transaction_hash.clone(), withdrawal: new_withdrawal, rate })
}

pub async fn process_chain_withdrawal(pool: &PgPool, redis_pool: &Pool, coin:Coins, coin_amount: BigDecimal, from_user_id: i64, address: &str) -> Result<(String, BigDecimal), Error> {
    if coin.coin_name.to_lowercase().contains(Coin::Litecoin.to_string()) {
        let rpc_client = RpcClient::new(pool, redis_pool, &coin.coin_name.to_uppercase()).await?;
        let tx_id = rpc_client.send_to_address(&address, coin_amount.to_f64().unwrap()).await?;
        let network_fee = get_user_withdrawal_details(rpc_client, &tx_id, coin_amount.clone()).await?;
        Ok((tx_id, network_fee))
    } else {
        let rpc_client = SolanaClient::new(pool, redis_pool).await?;
        let wallet = get_wallet_keys_by_user_id(pool, redis_pool, from_user_id, coin.id).await?;
        let to_address = Pubkey::from_str(address).map_err(
            |e| {
                log!(Level::Error, "Failed to parse recipient address: {}", e);
                Error::InvalidAddress
            }
        )?;
        let (_,_, sender_keypair) = get_solana_wallet_keypair(pool, redis_pool, wallet.wallet_index.unwrap() as u32).await?;
        let tx_id = if coin.coin_name.to_lowercase().contains(Coin::Solana.to_string()) {
            rpc_client.transfer_sol(&sender_keypair, &to_address, coin_amount.to_f64().unwrap()).await?
        } else {
            rpc_client.transfer_token(&sender_keypair, &to_address, Coin::from_str(&coin.coin_name.to_uppercase()).unwrap(), coin_amount.to_f64().unwrap()).await?
        };
        let tx_info = rpc_client.get_transaction_info(&tx_id).await?;
        Ok((tx_id, BigDecimal::from_f64(tx_info.network_fee).unwrap()))
    }
}

pub async fn process_withdrawal(trx: &mut Transaction<'_, Postgres>, redis_pool: &Pool, withdrawal: WithdrawalCreate, coin: Coins, coin_amount: BigDecimal, tx_id: String, fee_coin: BigDecimal, fee_usd: BigDecimal) -> Result<Withdrawals, Error> {
    let usd_amount = truncate_decimal(&withdrawal.usd_amount, 8);
    let result = query!(
        r#"
        INSERT INTO withdrawals (user_id, coin_id, usd_amount, coin_amount, transaction_hash, address, status, created_at, updated_at, event_id, event_status, fee_usd_amount, fee_coin_amount)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW(), $8, 1, $9, $10)
        RETURNING id, user_id, coin_id, usd_amount, coin_amount, fee_usd_amount, fee_coin_amount, transaction_hash, address, status, created_at, updated_at
        "#,
        withdrawal.user_id,
        coin.id,
        usd_amount,
        coin_amount.clone(),
        tx_id.clone(),
        withdrawal.address.clone(),
        withdrawal.status,
        withdrawal.event_id,
        fee_usd,
        fee_coin
    )
    .fetch_one(&mut **trx)
    .await;

    let new_withdrawal = match result {
        Ok(record) => Withdrawals {
            id: record.id,
            user_id: record.user_id,
            coin_id: record.coin_id,
            usd_amount,
            coin_amount: record.coin_amount,
            fee_usd_amount: record.fee_usd_amount,
            fee_coin_amount: record.fee_coin_amount,
            transaction_hash: record.transaction_hash,
            address: record.address,
            status: record.status,
            created_at: record.created_at,
            updated_at: record.updated_at,
        },
        Err(e) => {
            log!(Level::Error, "Database error on insert of withdrawal adding request: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };
    let _ = set_withdrawal_cache_by_id(redis_pool, &new_withdrawal).await;
    let _ = set_withdrawals_cache_by_user_id(redis_pool, withdrawal.user_id, &[new_withdrawal.clone()]).await;

    Ok(new_withdrawal)
}

pub async fn rollback_withdrawal_request(pool: &PgPool, _redis_pool: &Pool, event_id: i64) -> Result<SuccessMessages, Error> {
    let result = query!(
        r#"
        UPDATE withdrawals
        SET event_status = 2
        WHERE event_id = $1
        RETURNING id
        "#,
        event_id
    )
    .fetch_one(pool)
    .await;

    let _record = match result {
        Ok(users) => users,
        Err(e) => {
            log!(Level::Error, "Database error on rollback of withdrawal request: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };

    Ok(SuccessMessages::WithdrawalFailed)
}

pub async fn update_withdrawals_cache(pool: &PgPool, redis_pool: &Pool, user_id: i64) -> Result<Vec<Withdrawals>, Error> {
    let result = query_as!(
        Withdrawals,
        r#"
        SELECT id, user_id, coin_id, usd_amount, coin_amount, fee_usd_amount, fee_coin_amount, transaction_hash, address, status, created_at, updated_at FROM withdrawals WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(withdrawals) => {
            let _ = set_withdrawals_cache_by_user_id(redis_pool, user_id, &withdrawals).await;
            Ok(withdrawals)
        }
        Err(e) => {
            log!(Level::Error, "Database error while getting withdrawals by user id from db: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn update_withdrawal_with_trx(trx: &mut Transaction<'_, Postgres>, redis_pool: &Pool, withdrawal: Withdrawals, withdrawal_update: WithdrawalUpdate) -> Result<SuccessMessages, Error> {
    let update_result = query!(
        r#"
        UPDATE withdrawals
        SET status = COALESCE($1, status),
            updated_at = NOW()
        WHERE id = $2
        RETURNING updated_at
        "#,
        withdrawal_update.status,
        withdrawal.id
    )
    .fetch_one(&mut **trx)
    .await;

    let updated_withdrawal = match update_result {
        Ok(r) => Withdrawals {
            id: withdrawal.id,
            user_id: withdrawal.user_id,
            coin_id: withdrawal.coin_id,
            usd_amount: withdrawal.usd_amount.clone(),
            coin_amount: withdrawal.coin_amount.clone(),
            fee_usd_amount: withdrawal.fee_usd_amount.clone(),
            fee_coin_amount: withdrawal.fee_coin_amount.clone(),
            transaction_hash: withdrawal.transaction_hash.clone(),
            address: withdrawal.address.clone(),
            status: withdrawal_update.status,
            created_at: withdrawal.created_at,
            updated_at: r.updated_at,
        },
        Err(e) => {
            log!(Level::Error, "Database error while updating withdrawal: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };
    let _ = set_withdrawal_cache_by_id(redis_pool, &updated_withdrawal).await;

    Ok(SuccessMessages::UpdatedWithdrawal { withdrawal_id: updated_withdrawal.id })
}

pub async fn get_withdrawals_history(pool: &PgPool, redis_pool: &Pool, query: WithdrawalsHistoryRequest) -> Result<SuccessMessages, Error> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);

    let time_handler = TimeHandler::new();
    let from_date_default = time_handler.get_current_time().naive_utc().date() - chrono::Duration::days(30);
    let to_date_default = time_handler.get_current_time().naive_utc().date() + chrono::Duration::days(1);

    let from_date = query.from_date.unwrap_or(from_date_default);
    let to_date = query.to_date.unwrap_or(to_date_default);

    let status = query.status;

    let request = WithdrawalsHistoryRequest { user_id: query.user_id, page: Some(page), per_page: Some(per_page), status, from_date: Some(from_date), to_date: Some(to_date) };

    let history = match get_paginated_withdrawals_from_cache(redis_pool, query.user_id, request.clone()).await {
        Ok(Some(history)) => history,
        _ => match get_paginated_withdrawals_from_db(pool, query.user_id, request).await {
            Ok(history) => {
                let _ = update_withdrawals_cache(pool, redis_pool, query.user_id).await;
                history
            }
            Err(e) => return Err(e),
        },
    };

    Ok(SuccessMessages::FoundWithdrawalsHistory { user_id: query.user_id, page: history.page, per_page: history.per_page, total_results: history.total_results, total_pages: history.total_pages, data: history.data })
}

pub async fn get_paginated_withdrawals_from_db(pool: &PgPool, user_id: i64, history_query: WithdrawalsHistoryRequest) -> Result<WithdrawalsHistoryResult, Error> {
    let mut query =
        String::from("SELECT id, user_id, coin_id, usd_amount, coin_amount, fee_usd_amount, fee_coin_amount, transaction_hash, address, status, created_at, updated_at from withdrawals where user_id = $1 AND status= true AND event_status=1");
    let mut count_query = String::from("SELECT COUNT(*) from withdrawals where user_id = $1");

    let mut idx = 2;

    // Dynamic filters
    if history_query.status.is_some() {
        query.push_str(&format!(" AND status = ${idx}"));
        count_query.push_str(&format!(" AND status = ${idx}"));
        idx += 1;
    }

    if history_query.from_date.is_some() {
        query.push_str(&format!(" AND DATE(created_at) >= ${idx}"));
        count_query.push_str(&format!(" AND DATE(created_at) >= ${idx}"));
        idx += 1;
    }

    if history_query.to_date.is_some() {
        query.push_str(&format!(" AND DATE(created_at) <= ${idx}"));
        count_query.push_str(&format!(" AND DATE(created_at) <= ${idx}"));
        idx += 1;
    }

    query.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", idx, idx + 1));

    // Total results calculation
    let mut total_results_query = sqlx::query_scalar::<_, i64>(&count_query).bind(user_id);

    if let Some(status) = history_query.status {
        total_results_query = total_results_query.bind(status);
    }

    if let Some(from_date) = history_query.from_date {
        total_results_query = total_results_query.bind(from_date);
    }

    if let Some(to_date) = history_query.to_date {
        total_results_query = total_results_query.bind(to_date);
    }

    let per_page = history_query.per_page.unwrap();
    let page = history_query.page.unwrap();

    let total_results: i64 = match total_results_query.fetch_one(pool).await {
        Ok(total) => total,
        Err(sqlx::Error::RowNotFound) => {
            log!(Level::Error, "No withdrawal history found for user: {}", user_id);
            return Ok(WithdrawalsHistoryResult { page, per_page, total_results: 0, total_pages: 0, data: vec![] });
        }
        Err(e) => {
            log!(Level::Error, "Error getting withdrawal history: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };
    let total_pages = (total_results as f64 / per_page as f64).ceil() as i32;
    let offset = (page - 1) * per_page;

    // Data query with pagination
    let mut data_query = sqlx::query_as::<_, Withdrawals>(&query).bind(user_id);

    if let Some(status) = history_query.status {
        data_query = data_query.bind(status);
    }

    if let Some(from_date) = history_query.from_date {
        data_query = data_query.bind(from_date);
    }

    if let Some(to_date) = history_query.to_date {
        data_query = data_query.bind(to_date);
    }

    data_query = data_query.bind(per_page).bind(offset);

    let history = data_query.fetch_all(pool).await;

    let history = match history {
        Ok(w_history) => {
            if w_history.is_empty() {
                log!(Level::Error, "No withdrawal history found for user: {}", user_id);
                return Ok(WithdrawalsHistoryResult { page, per_page, total_results: total_results as i32, total_pages, data: vec![] });
            }
            w_history
        }
        Err(sqlx::Error::RowNotFound) => {
            log!(Level::Error, "No withdrawal history found for user: {}", user_id);
            return Ok(WithdrawalsHistoryResult { page, per_page, total_results: total_results as i32, total_pages, data: vec![] });
        }
        Err(e) => {
            log!(Level::Error, "Error getting withdrawal history: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };

    Ok(WithdrawalsHistoryResult { page, per_page, total_results: total_results as i32, total_pages, data: history })
}

async fn get_paginated_withdrawals_from_cache(redis_pool: &Pool, user_id: i64, history_query: WithdrawalsHistoryRequest) -> Result<Option<WithdrawalsHistoryResult>, Error> {
    // Fetch history from cache
    let mut history = match get_withdrawals_from_cache_by_user_id(redis_pool, user_id).await {
        Some(history) => history,
        None => {
            return Ok(None);
        }
    };

    // Filter by status if provided
    if let Some(status) = history_query.status {
        history.retain(|record| record.status == status);
    }

    // Filter by date range if provided
    if let Some(from) = history_query.from_date {
        history.retain(|record| record.created_at >= from.into());
    }
    if let Some(to) = history_query.to_date {
        history.retain(|record| record.created_at <= to.into());
    }

    // Sort by update_time in descending order
    history.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Calculate total results and pages
    let page = history_query.page.unwrap();
    let per_page = history_query.per_page.unwrap();

    let total_results = history.len() as i32;
    let total_pages = max((total_results + per_page - 1) / per_page, 1);

    // Paginate the results
    let start: usize = (page.saturating_sub(1) * per_page) as usize;
    let end: usize = page.saturating_mul(per_page) as usize;
    let paginated_data = history.into_iter().skip(start).take(end).collect::<Vec<_>>();

    // Return the paginated result
    let result = WithdrawalsHistoryResult { page: history_query.page.unwrap(), per_page: history_query.page.unwrap(), total_results, total_pages, data: paginated_data };

    Ok(Some(result))
}

pub async fn get_user_withdrawal_details(rpc_client: RpcClient, tx_id: &str, user_amount: BigDecimal) -> Result<BigDecimal, Error> {
    let (details, _) = rpc_client.get_transaction(tx_id).await?;
    for detail in details.iter().filter(|d| d["category"] == "send") {
        let amount = detail["amount"].as_f64().unwrap();
        if amount == user_amount.to_f64().unwrap() {
            let fee = detail["fee"].as_f64().unwrap();
            return Ok(BigDecimal::from_f64(fee).unwrap());
        }
    }
    Ok(BigDecimal::from(0))
}
