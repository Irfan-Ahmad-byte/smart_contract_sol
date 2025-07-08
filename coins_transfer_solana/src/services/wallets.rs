use crypsol_logger::log;
use deadpool_redis::Pool;
use log::Level;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, query, query_as};

use crate::cache::wallets::{
    drop_a_wallet_from_all_coin_id_wallets, drop_a_wallet_from_all_user_id_wallets, drop_a_wallet_from_all_wallets, get_all_wallets_from_cache,
    get_wallet_from_cache_by_id, get_wallets_from_cache_for_coin_id, get_wallets_from_cache_for_user_id, increment_all_wallets_cache, increment_wallet_cache_for_coin_id, increment_wallets_cache_for_user_id, set_wallet_cache_for_id,
};
use crate::entities::coins::Coins;
use crate::entities::users_wallets::UsersWallets;
use crate::responses::error_msgs::Error;
use crate::responses::success_msgs::SuccessMessages;
use crate::services::coins::get_a_coin_from_db_by_id;
use crate::services::rpc_client::RpcClient;
use crate::services::solana_client::get_solana_wallet_keypair;
use crate::services::users::get_a_user_from_db;
use crate::structs::wallets::{WalletCreate, WalletQuery, WalletUpdate};
use crate::utils::time::TimeHandler;

pub async fn create_wallet(pool: &PgPool, redis_pool: &Pool, wallet: WalletCreate) -> Result<SuccessMessages, Error> {
    // Duplication check
    get_a_user_from_db(pool, wallet.user_id).await?;
    get_a_coin_from_db_by_id(pool, wallet.coin_id).await?;

    let duplicate_check = get_wallets_from_db_by_coin_id_and_user(pool, wallet.coin_id, wallet.user_id).await?;

    if let Some(_wallets) = duplicate_check {
        log!(Level::Error, "Duplicate wallet for user ID: {} and coin ID: {}", wallet.user_id, wallet.coin_id);
        return Err(Error::DuplicateEntry);
    }

    let time_handler = TimeHandler::new();
    let now = time_handler.get_current_time().naive_utc();

    let result = query!(
        r#"
        INSERT INTO users_wallets (user_id, coin_id, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id
        "#,
        wallet.user_id,
        wallet.coin_id,
        true,
        now,
        now
    )
    .fetch_one(pool)
    .await;

    let new_wallet = match result {
        Ok(record) => UsersWallets { id: record.id, user_id: wallet.user_id, coin_id: wallet.coin_id, address: None, status: true, created_at: now, updated_at: now },
        Err(e) => {
            log!(Level::Error, "Database error on inserting new wallet: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };

    let _ = set_wallet_cache_for_id(redis_pool, &new_wallet).await;
    let _ = increment_wallets_cache_for_user_id(redis_pool, &[new_wallet.clone()]).await;
    let _ = increment_wallet_cache_for_coin_id(redis_pool, &[new_wallet.clone()]).await;
    let _ = increment_all_wallets_cache(redis_pool, &[new_wallet.clone()]).await;

    Ok(SuccessMessages::CreatedWallet { user_id: new_wallet.user_id, wallet: new_wallet })
}

pub async fn get_wallets(pool: &PgPool, redis_pool: &Pool, query: WalletQuery) -> Result<SuccessMessages, Error> {
    match (query.user_id, query.wallet_id, query.coin_id) {
        (None, None, None) => get_all_wallets(pool, redis_pool).await,

        (Some(user_id), None, None) => get_wallets_by_user_id(pool, redis_pool, user_id).await,

        (None, Some(wallet_id), None) => get_wallet_by_id(pool, redis_pool, wallet_id).await,

        (None, None, Some(coin_id)) => get_wallets_by_coin_id(pool, redis_pool, coin_id).await,

        (Some(user_id), Some(_wallet_id), None) => get_wallets_by_user_id(pool, redis_pool, user_id).await,

        (None, Some(wallet_id), Some(_coin_id)) => get_wallet_by_id(pool, redis_pool, wallet_id).await,

        (Some(user_id), Some(_wallet_id), Some(_coin_id)) => get_wallets_by_user_id(pool, redis_pool, user_id).await,

        (Some(user_id), None, Some(coin_id)) => {
            let wallets = get_wallets_by_user_id(pool, redis_pool, user_id).await?;
            let filtered_wallets: Vec<_> = match wallets {
                SuccessMessages::FoundWallet { wallets_list, .. } => wallets_list.into_iter().filter(|wallet| wallet.coin_id == coin_id).collect(),
                _ => Vec::new(),
            };
            if !filtered_wallets.is_empty() {
                Ok(SuccessMessages::FoundWallet { user_id, wallets_list: filtered_wallets })
            } else {
                Err(Error::NotFound("No user wallet found".to_string()))
            }
        }
    }
}

pub async fn get_wallet_by_user_and_coin(pool: &PgPool, redis_pool: &Pool, user_id: i64, coin_id: i16) -> Result<SuccessMessages, Error> {
    let result = query_as!(
        UsersWallets,
        r#"
        SELECT id, user_id, coin_id, address, status, created_at, updated_at FROM users_wallets WHERE user_id = $1 AND coin_id = $2
        "#,
        user_id,
        coin_id
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(wallet) => Ok(SuccessMessages::FoundWallet { user_id: wallet.user_id, wallets_list: vec![wallet] }),
        Err(sqlx::Error::RowNotFound) => create_wallet(pool, redis_pool, WalletCreate { user_id, coin_id }).await,
        Err(e) => {
            log!(Level::Error, "Database error: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn get_wallets_by_user_id(pool: &PgPool, redis_pool: &Pool, user_id: i64) -> Result<SuccessMessages, Error> {
    if let Some(wallets) = get_wallets_from_cache_for_user_id(redis_pool, &user_id).await {
        return Ok(SuccessMessages::FoundWallet { user_id: wallets[0].user_id, wallets_list: wallets });
    }

    let wallets = get_wallets_from_db_by_user_id(pool, user_id).await?;

    let _ = increment_wallets_cache_for_user_id(redis_pool, &wallets).await;

    Ok(SuccessMessages::FoundWallet { user_id: wallets[0].user_id, wallets_list: wallets })
}

pub async fn get_wallet_by_id(pool: &PgPool, redis_pool: &Pool, wallet_id: i64) -> Result<SuccessMessages, Error> {
    if let Some(wallet) = get_wallet_from_cache_by_id(redis_pool, &wallet_id).await {
        return Ok(SuccessMessages::FoundWallet { user_id: wallet.user_id, wallets_list: vec![wallet] });
    }

    let wallet = get_a_wallet_from_db_by_id(pool, wallet_id).await?;

    let _ = set_wallet_cache_for_id(redis_pool, &wallet).await;

    Ok(SuccessMessages::FoundWallet { user_id: wallet.user_id, wallets_list: vec![wallet] })
}

pub async fn get_wallets_by_coin_id(pool: &PgPool, redis_pool: &Pool, coin_id: i16) -> Result<SuccessMessages, Error> {
    if let Some(wallets) = get_wallets_from_cache_for_coin_id(redis_pool, &coin_id).await {
        return Ok(SuccessMessages::FoundWallet { user_id: wallets[0].user_id, wallets_list: wallets });
    }

    let wallets = query_as!(
        UsersWallets,
        r#"
        SELECT id, user_id, coin_id, address, status, created_at, updated_at FROM users_wallets WHERE coin_id = $1
        "#,
        coin_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        log!(Level::Error, "Database error: {:?}", e);
        Error::DatabaseIssue
    })?;

    if wallets.is_empty() {
        log!(Level::Warn, "No wallets found for coin ID: {}", coin_id);
        return Err(Error::NotFound("No user wallet found".to_string()));
    }
    let _ = increment_wallet_cache_for_coin_id(redis_pool, &wallets).await;

    Ok(SuccessMessages::FoundWallet { user_id: wallets[0].user_id, wallets_list: wallets })
}

async fn get_wallets_from_db_by_user_id(pool: &PgPool, user_id: i64) -> Result<Vec<UsersWallets>, Error> {
    let result = query_as!(
        UsersWallets,
        r#"
        SELECT id, user_id, coin_id, address, status, created_at, updated_at FROM users_wallets WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(wallet) => {
            if wallet.is_empty() {
                log!(Level::Warn, "No wallets found for user ID: {}", user_id);
                return Err(Error::NotFound("No user wallet found".to_string()));
            }
            Ok(wallet)
        }
        Err(e) => {
            log!(Level::Error, "Database error while getting all user wallts: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

async fn get_a_wallet_from_db_by_id(pool: &PgPool, id: i64) -> Result<UsersWallets, Error> {
    let result = query_as!(
        UsersWallets,
        r#"
        SELECT id, user_id, coin_id, address, status, created_at, updated_at FROM users_wallets WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(wallet) => Ok(wallet),
        Err(sqlx::Error::RowNotFound) => {
            log!(Level::Error, "wallet not found with wallet ID: {}", id);
            Err(Error::NotFound("Wallet not found".to_string()))
        }
        Err(e) => {
            log!(Level::Error, "Database error: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn get_wallets_from_db_by_coin_id_and_user(pool: &PgPool, coin_id: i16, user_id: i64) -> Result<Option<UsersWallets>, Error> {
    let result = query_as!(
        UsersWallets,
        r#"
        SELECT id, user_id, coin_id, address, status, created_at, updated_at FROM users_wallets WHERE coin_id = $1 AND user_id = $2
        "#,
        coin_id,
        user_id
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(wallet) => Ok(Some(wallet)),
        Err(sqlx::Error::RowNotFound) => Ok(None),
        Err(e) => {
            log!(Level::Error, "Database error: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn get_wallet_by_address(pool: &PgPool, _redis_pool: &Pool, address: String) -> Result<UsersWallets, Error> {
    let result = query_as!(
        UsersWallets,
        r#"
        SELECT id, user_id, coin_id, address, status, created_at, updated_at FROM users_wallets WHERE address = $1
        "#,
        address
    )
        .fetch_one(pool)
        .await;

    match result {
        Ok(wallet) => Ok(wallet),
        Err(sqlx::Error::RowNotFound) => {
            log!(Level::Error, "wallet not found with address: {}", address);
            Err(Error::NotFound("Wallet not found".to_string()))
        }
        Err(e) => {
            log!(Level::Error, "Database error: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainWallet {
    pub public_key: Option<String>,
    pub private_key: Option<String>,
    pub wallet_index: Option<i64>,
}
pub async fn get_wallet_keys_by_user_id(pool: &PgPool, _redis_pool: &Pool, user_id: i64, coin_id: i16) -> Result<ChainWallet, Error> {
    let result = query_as!(
        ChainWallet,
        r#"
        SELECT address as public_key, private_key, wallet_index FROM users_wallets WHERE user_id = $1 AND coin_id = $2
        "#,
        user_id,
        coin_id
    )
        .fetch_one(pool)
        .await;

    match result {
        Ok(wallet) => Ok(wallet),
        Err(sqlx::Error::RowNotFound) => {
            Err(Error::NotFound("Wallet not found".to_string()))
        }
        Err(e) => {
            log!(Level::Error, "Database error: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn get_all_wallets(pool: &PgPool, redis_pool: &Pool) -> Result<SuccessMessages, Error> {
    if let Some(wallets) = get_all_wallets_from_cache(redis_pool).await {
        return Ok(SuccessMessages::FoundWallet {
            user_id: 0, // use 0 to indicate a list, as there is no specific ID
            wallets_list: wallets,
        });
    }

    let wallets = update_all_wallets_cache(pool, redis_pool).await?;

    Ok(SuccessMessages::FoundWallet {
        user_id: 0, // use 0 to indicate a list, as there is no specific ID
        wallets_list: wallets,
    })
}

async fn update_all_wallets_cache(pool: &PgPool, redis_pool: &Pool) -> Result<Vec<UsersWallets>, Error> {
    let result = query_as!(
        UsersWallets,
        r#"
        SELECT id, user_id, coin_id, address, status, created_at, updated_at FROM users_wallets
        "#
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(wallets) => {
            if wallets.is_empty() {
                log!(Level::Warn, "No wallets found in the database");
                return Err(Error::NotFound("Wallets not found".to_string()));
            }
            let _ = increment_all_wallets_cache(redis_pool, &wallets).await;
            Ok(wallets)
        }
        Err(e) => {
            log!(Level::Error, "Database error: {:?}", e);
            Err(Error::DatabaseIssue)
        }
    }
}

pub async fn update_wallet(pool: &PgPool, redis_pool: &Pool, wallet_id: i64, wallet: Option<&UsersWallets>, wallet_update: WalletUpdate) -> Result<SuccessMessages, Error> {
    let wallet = if let Some(w) = wallet { w } else { &get_a_wallet_from_db_by_id(pool, wallet_id).await? };

    let time_handler = TimeHandler::new();
    let now = time_handler.get_current_time().naive_utc();

    let update_result = query_as!(
        UsersWallets,
        r#"
        UPDATE users_wallets
        SET coin_id = COALESCE($1, coin_id),
            address = COALESCE($2, address),
            status = COALESCE($3, status),
            private_key = COALESCE($6, private_key),
            wallet_index = COALESCE($7, wallet_index),
            updated_at = $4
        WHERE id = $5
        RETURNING id, user_id, coin_id, address, status, created_at, updated_at
        "#,
        wallet_update.coin_id,
        wallet_update.address.clone(),
        wallet_update.status,
        now,
        wallet_id,
        wallet_update.private_key,
        wallet_update.wallet_index
    )
    .fetch_one(pool)
    .await;

    let updated_wallet = match update_result {
        Ok(w) => w,
        Err(e) => {
            log!(Level::Error, "Database error: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };
    let _ = set_wallet_cache_for_id(redis_pool, &updated_wallet).await;
    let _ = drop_a_wallet_from_all_user_id_wallets(redis_pool, wallet).await;
    let _ = increment_wallets_cache_for_user_id(redis_pool, &[updated_wallet.clone()]).await;
    let _ = drop_a_wallet_from_all_coin_id_wallets(redis_pool, wallet).await;
    let _ = increment_wallet_cache_for_coin_id(redis_pool, &[updated_wallet.clone()]).await;
    let _ = drop_a_wallet_from_all_wallets(redis_pool, wallet).await;
    let _ = increment_all_wallets_cache(redis_pool, &[updated_wallet.clone()]).await;

    Ok(SuccessMessages::UpdatedWallet { wallet_id: updated_wallet.id })
}

pub async fn get_user_address(pool: &PgPool, redis_pool: &Pool, wallet: UsersWallets, coin: Coins) -> Result<SuccessMessages, Error> {
    let label = format!("user_{}", wallet.user_id);
    // let coin = fetch_a_coin_by_id(pool, redis_pool, wallet.coin_id).await?;
    let mut wallet_index = None;
    let (mut _address, mut secret) = ("".to_string(), None);
    if coin.coin_name.to_lowercase().contains("usd") || coin.coin_name.to_lowercase().contains("solana") {
        wallet_index = Some(get_current_highest_wallet_index(pool, redis_pool).await?+1);
        let (addr, sec, _) = get_solana_wallet_keypair(pool, redis_pool, wallet_index.unwrap() as u32).await?;
        (_address, secret) = (addr, Some(sec));
    } else {
        let rpc_client = RpcClient::new(pool, redis_pool, &coin.coin_name.to_uppercase()).await?;
        _address = rpc_client.generate_new_address(&label).await?;
    }

    update_wallet(pool, redis_pool, wallet.id, Some(&wallet), WalletUpdate { user_id: wallet.user_id, coin_id: Some(wallet.coin_id), address: Some(_address.clone()), private_key: secret, wallet_index, status: Some(wallet.status) }).await?;

    Ok(SuccessMessages::AddressGenerated { coin: wallet.coin_id, address: _address })
}

pub async fn get_current_highest_wallet_index(pool: &PgPool, redis_pool: &Pool) -> Result<i64, Error> {
    let result = query!(
        r#"
        SELECT MAX(wallet_index) AS max_index FROM users_wallets
        "#
    )
    .fetch_one(pool)
    .await;

    let index = match result {
        Ok(record) => {
            record.max_index
        },
        Err(e) => {
            log!(Level::Error, "Database error while getting highest wallet index: {:?}", e);
            return Err(Error::DatabaseIssue)
        }
    };
    if index.is_some() {
        return Ok(index.unwrap());
    }
    let (root_wallet, wallet_exists) = match query!(
        r#"
        SELECT * FROM users_wallets WHERE user_id = 1 AND coin_id=2
        "#
    )
        .fetch_one(pool)
        .await {
        Ok(record) => (record.wallet_index, true),
        Err(sqlx::error::Error::RowNotFound) => {
            (None, false)
        }
        Err(e) => {
            log!(Level::Error, "Database error while getting root wallet: {:?}", e);
            return Err(Error::DatabaseIssue);
        }
    };
    
    if root_wallet.is_some() {
        return Ok(root_wallet.unwrap());
    }
    
    let index = if wallet_exists {
        let (addr, sec) = generate_root_sol_wallet(pool, redis_pool).await?;
        query!(
            r#"
            UPDATE users_wallets SET wallet_index=0, address=$1, private_key=$2
            WHERE user_id =1 AND coin_id=2
            "#,
            addr,
            sec
        )
        .execute(pool)
        .await.map_err(|e| {
            log!(Level::Error, "Database error while inserting root wallet: {:?}", e);
            Error::DatabaseIssue
        })?;
        0
    } else {
        let (addr, sec) = generate_root_sol_wallet(pool, redis_pool).await?;
        query!(
            r#"
            INSERT INTO users_wallets (user_id, coin_id, status, wallet_index, address, private_key)
            VALUES (1, 2, true, 0, $1, $2)
            "#,
            addr,
            sec
        )
            .execute(pool)
            .await.map_err(|e| {
            log!(Level::Error, "Database error while inserting root wallet: {:?}", e);
            Error::DatabaseIssue
        })?;
        0
    };
    
    Ok(index)
}

pub async fn generate_root_sol_wallet(pool: &PgPool, redis_pool: &Pool) -> Result<(String, String), Error> {
    let (address, secret, _) = get_solana_wallet_keypair(pool, redis_pool, 0).await?;
    Ok((address, secret))
}

pub async fn validate_user_address(pool: &PgPool, redis_pool: &Pool, address: String) -> Result<bool, Error> {
    // let wallet = get_wallet_by_address(pool, redis_pool, address.clone()).await?;
    // let coin = fetch_a_coin_by_id(pool, redis_pool, wallet.coin_id).await?;
    let rpc_client = RpcClient::new(pool, redis_pool, "LITECOIN").await?;
    rpc_client.validate_address(&address).await
}