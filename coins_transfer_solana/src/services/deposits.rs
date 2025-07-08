use crypsol_logger::log;
use deadpool_redis::Pool;
use log::Level;
use sqlx::{PgPool, Postgres, Transaction, query, query_as};
use std::cmp::max;

use crate::cache::deposits::{get_transactions_from_cache_by_user_id, set_transaction_cache_by_id, set_transactions_cache_by_user_id};
use crate::entities::deposits::Deposits;
use crate::responses::error_msgs::Error;
use crate::responses::success_msgs::SuccessMessages;
use crate::services::coins::get_a_coin_from_db_by_id;
use crate::services::users::get_a_user_from_db;
use crate::structs::deposits::{DepositCreate, DepositUpdate};
use crate::structs::history::{TransactionsHistoryRequest, TransactionsHistoryResult};
use crate::utils::time::TimeHandler;

pub async fn create_user_transaction(pool: &PgPool, redis_pool: &Pool, transaction: DepositCreate) -> Result<SuccessMessages, Error> {
    // Validate user and coin existence
    get_a_user_from_db(pool, transaction.user_id).await?;
    get_a_coin_from_db_by_id(pool, transaction.coin_id).await?;

    // Start transaction
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log!(Level::Error, "Database error on transaction start for create user transaction: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };

    // Add new user transaction
    let new_transaction = add_user_transaction(&mut tx, &transaction).await?;

    // Commit transaction
    match tx.commit().await {
        Ok(_) => {}
        Err(e) => {
            log!(Level::Error, "Database error on commit for create user transaction: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };

    // Cache the new transaction
    let _ = set_transaction_cache_by_id(redis_pool, &new_transaction).await;
    let _ = set_transactions_cache_by_user_id(redis_pool, new_transaction.user_id, &[new_transaction.clone()]).await;

    Ok(SuccessMessages::CreatedTransaction { transaction_id: new_transaction.id, transaction: new_transaction })
}

pub async fn add_user_transaction(tx: &mut Transaction<'_, Postgres>, transaction: &DepositCreate) -> Result<Deposits, Error> {
    let result = query_as!(
        Deposits,
        r#"
        INSERT INTO deposits (
            user_id, coin_id, amount, fiat_amount, user_address_id, transaction_hash, status, created_at, updated_at, event_id, event_status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW(), $8, 1)
        RETURNING id, user_id, coin_id, amount, fiat_amount, user_address_id, transaction_hash, status, created_at, updated_at
        "#,
        transaction.user_id,
        transaction.coin_id,
        transaction.amount,
        transaction.fiat_amount,
        transaction.address_id,
        transaction.transaction_hash,
        transaction.status,
        transaction.event_id
    )
    .fetch_one(&mut **tx)
    .await;

    match result {
        Ok(record) => Ok(Deposits {
            id: record.id,
            user_id: record.user_id,
            coin_id: record.coin_id,
            amount: record.amount,
            fiat_amount: record.fiat_amount,
            user_address_id: record.user_address_id,
            transaction_hash: record.transaction_hash,
            status: record.status,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }),
        Err(e) => {
            log!(Level::Error, "Database error on insert of user transaction: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn get_deposit_by_hash(pool: &PgPool, redis_pool: &Pool, transaction_hash: String) -> Result<Option<Deposits>, Error> {
    let result = query_as!(
        Deposits,
        r#"
        SELECT id, user_id, coin_id, amount, fiat_amount, user_address_id, transaction_hash, status, created_at, updated_at FROM deposits WHERE transaction_hash = $1 AND status = true AND event_status = 1
        "#,
        transaction_hash
    )
    .fetch_optional(pool)
    .await;

    match result {
        Ok(transaction) => Ok(transaction),
        Err(e) => {
            log!(Level::Error, "Database error while getting transaction by hash from DB: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn get_transactions_history(pool: &PgPool, redis_pool: &Pool, query: TransactionsHistoryRequest) -> Result<SuccessMessages, Error> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);

    let time_handler = TimeHandler::new();
    let from_date_default = time_handler.get_current_time().naive_utc().date() - chrono::Duration::days(30);
    let to_date_default = time_handler.get_current_time().naive_utc().date() + chrono::Duration::days(1);

    let from_date = query.from_date.unwrap_or(from_date_default);
    let to_date = query.to_date.unwrap_or(to_date_default);

    let status = query.status;

    let request = TransactionsHistoryRequest { user_id: query.user_id, page: Some(page), per_page: Some(per_page), status, from_date: Some(from_date), to_date: Some(to_date) };

    let history = match get_paginated_user_transactions_from_cache(redis_pool, query.user_id, request.clone()).await {
        Ok(Some(history)) => history,
        _ => match get_paginated_transaction_history_from_db(pool, query.user_id, request).await {
            Ok(history) => {
                history
            }
            Err(e) => return Err(e),
        },
    };

    Ok(SuccessMessages::FoundTransactionsHistory { user_id: query.user_id, page: history.page, per_page: history.per_page, total_results: history.total_results, total_pages: history.total_pages, data: history.data })
}

pub async fn get_paginated_transaction_history_from_db(pool: &PgPool, user_id: i64, history_query: TransactionsHistoryRequest) -> Result<TransactionsHistoryResult, Error> {
    let mut query = String::from("SELECT id, user_id, coin_id, amount, fiat_amount, user_address_id, transaction_hash, status, created_at, updated_at from deposits where user_id = $1");
    let mut count_query = String::from("SELECT COUNT(*) from deposits where user_id = $1");

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
        Ok(total_results) => total_results,
        Err(sqlx::Error::RowNotFound) => {
            log!(Level::Error, "No transaction history found for user: {}", user_id);
            return Ok(TransactionsHistoryResult { page, per_page, total_results: 0, total_pages: 0, data: vec![] });
        }
        Err(e) => {
            log!(Level::Error, "Error getting total results: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };
    let total_pages = (total_results as f64 / per_page as f64).ceil() as i32;
    let offset = (page - 1) * per_page;

    // Data query with pagination
    let mut data_query = sqlx::query_as::<_, Deposits>(&query).bind(user_id);

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
        Ok(history) => {
            if history.is_empty() {
                log!(Level::Error, "No transaction history found for user: {}", user_id);
                return Ok(TransactionsHistoryResult { page, per_page, total_results: total_results as i32, total_pages, data: vec![] });
            }
            history
        }
        Err(sqlx::Error::RowNotFound) => {
            log!(Level::Error, "No transaction history found for user: {}", user_id);
            return Ok(TransactionsHistoryResult { page, per_page, total_results: total_results as i32, total_pages, data: vec![] });
        }
        Err(e) => {
            log!(Level::Error, "Error getting transaction history: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };

    Ok(TransactionsHistoryResult { page, per_page, total_results: total_results as i32, total_pages, data: history })
}

async fn get_paginated_user_transactions_from_cache(redis_pool: &Pool, user_id: i64, history_query: TransactionsHistoryRequest) -> Result<Option<TransactionsHistoryResult>, Error> {
    // Fetch history from cache
    let mut history = match get_transactions_from_cache_by_user_id(redis_pool, user_id).await {
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
    let result = TransactionsHistoryResult { page: history_query.page.unwrap(), per_page: history_query.page.unwrap(), total_results, total_pages, data: paginated_data };

    Ok(Some(result))
}
