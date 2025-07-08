use crypsol_logger::log;
use crate::responses::error_msgs::Error;
use deadpool_redis::Pool;
use log::Level;
use sqlx::PgPool;
use crate::responses::success_msgs::SuccessMessages;
use crate::services::conversion_rates::get_rate_by_coin_id;
use crate::services::deposits::{create_user_transaction, get_deposit_by_hash};
use crate::services::solana_client::SolanaClient;
use crate::services::wallets::get_wallet_by_address;
use crate::structs::deposits::DepositCreate;


pub async fn validate_usdt_deposit( pool: &PgPool, redis_pool: &Pool, tx_id: &str, user_id: i64) -> Result<(), Error> {
    // 1. Skip if already recorded
    if get_deposit_by_hash(pool, redis_pool, tx_id.to_string()).await?.is_some() {
        log!(Level::Error, "[Validate Deposit] Already exists for tx: {}", tx_id);
        return Err(Error::DepositAlreadyRecorded);
    }

    // 2. Fetch transaction info and metadata
    let client = SolanaClient::new(pool, redis_pool).await?;
    let info = client.get_transaction_info(tx_id).await?;
    let coin_id = info.coin.to_i16();

    // 5. Validate recipient wallet
    let wallet = get_wallet_by_address(pool, redis_pool, info.recipient.clone()).await?;
    if wallet.user_id != user_id {
        log!(Level::Error, "[Validate Deposit] User ID mismatch: expected {}, got {}", user_id, wallet.user_id);
        return Err(Error::UserIdMismatch);
    }

    // 6. Calculate fiat amount
    let rate = get_rate_by_coin_id(pool, redis_pool, coin_id.into()).await?;
    let fiat = (info.amount.clone() * rate).with_scale(4);

    // 7. Create deposit record
    let deposit = DepositCreate {
        user_id: wallet.user_id,
        coin_id: coin_id.into(),
        amount: info.amount.clone(),
        fiat_amount: fiat.clone(),
        address_id: wallet.id,
        transaction_hash: tx_id.to_string(),
        status: true,
        event_id: Some(1),
    };
    let SuccessMessages::CreatedTransaction { transaction_id, .. } = create_user_transaction(pool, redis_pool, deposit).await? else {
        return Err(Error::DatabaseIssue);
    };
    let _ = transaction_id;

    Ok(())
}