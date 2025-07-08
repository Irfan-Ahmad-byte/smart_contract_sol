use actix_web::http::StatusCode;
use bigdecimal::{BigDecimal, ToPrimitive};
use serde_json::{Value, json};

use crate::entities::configs::Configs;
use crate::entities::deposits::Deposits;
use crate::entities::users::Users;
use crate::entities::users_wallets::UsersWallets;
use crate::entities::withdrawals::Withdrawals;
use crate::structs::coins::CoinInfo;
use crate::structs::deposits::DepositsDetails;
// Add this import

#[allow(dead_code, unused)]
#[derive(Debug)]
pub enum SuccessMessages {
    // user
    UserCreationFailed,
    CreatedUser {
        user_id: i64,
    },
    FoundUser {
        user_id: i64,
        users_list: Option<Vec<Users>>,
    },
    DeletedUser {
        user_id: i64,
    },

    // coin
    CreatedCoin {
        coin_id: i16,
        coin_name: String,
    },
    FoundCoin {
        coin_id: i16,
        coins_list: Option<Vec<CoinInfo>>,
    },
    UpdatedCoin {
        coin_id: i16,
    },
    DeletedCoin {
        coin_id: i16,
    },
    AddressGenerated {
        coin: i16,
        address: String,
    },
    AddressValidated {
        is_valid: bool,
        coin: i16,
        address: String,
    },
    UnconfirmedBalance {
        address: String,
        unconfirmed_balance: f64,
        confirmations: i64,
    },
    TransactionInformation {
        txid: String,
        data: Vec<Value>,
        confirmations: i64,
    },
    NodeBalance {
        crypto_balance: BigDecimal,
        usd_balance: BigDecimal,
    },
    TransferSuccessful {
        hash: String,
        amount: f64,
    },

    // rates
    ConversionRate {
        coin_id: i16,
        symbol: String,
        rate: f64,
    },

    // wallets
    CreatedWallet {
        user_id: i64,
        wallet: UsersWallets,
    },
    FoundWallet {
        user_id: i64,
        wallets_list: Vec<UsersWallets>,
    },
    UpdatedWallet {
        wallet_id: i64,
    },
    DeletedWallet {
        user_id: i64,
    },

    // withdrawals
    WithdrawalFailed,
    CreatedWithdrawal {
        withdrawal_id: i64,
        hash: Option<String>,
        withdrawal: Withdrawals,
        rate: BigDecimal,
    },
    FoundWithdrawal {
        withdrawal_id: i64,
        hash: Option<String>,
        withdrawal_list: Option<Vec<Withdrawals>>,
    },
    UpdatedWithdrawal {
        withdrawal_id: i64,
    },
    FoundWithdrawalsHistory {
        user_id: i64,
        page: i32,
        per_page: i32,
        total_results: i32,
        total_pages: i32,
        data: Vec<Withdrawals>,
    },

    // invoice
    CreatedInvoice {
        invoice_id: i64,
    }, // Add these

    // transaction
    TransactionFailed, // Add these
    CreatedTransaction {
        transaction_id: i64,
        transaction: Deposits,
    }, // Add these
    FoundTransaction {
        transaction_id: i64,
        transaction: Option<Deposits>,
    },
    FoundTransactions {
        transactions_list: Option<Vec<Deposits>>,
    },
    UpdatedTransaction {
        transaction_id: i64,
    },
    FoundTransactionsHistory {
        user_id: i64,
        page: i32,
        per_page: i32,
        total_results: i32,
        total_pages: i32,
        data: Vec<Deposits>,
    },
    FoundBulkTransactionsHistory {
        data: Vec<DepositsDetails>,
    },

    FoundBulkWithdrawalsHistory {
        data: Vec<Withdrawals>,
    },

    // config
    CreatedConfig {
        config_id: i16,
    },
    FoundConfig {
        config_id: i16,
        configs_list: Option<Vec<Configs>>,
    },
    UpdatedConfig {
        config_id: i16,
        configs_list: Option<Vec<Configs>>,
    },
    DeletedConfig {
        config: String,
    },
    DeletedAllConfigs,

    // payments
    PaymentProcessed {
        tx_id: String,
        details: Vec<Value>,
        confirmations: i64,
    },
}

impl SuccessMessages {
    pub fn to_response(&self) -> (&str, String, Value, StatusCode) {
        match self {
            // user
            SuccessMessages::UserCreationFailed => ("UserCreationFailed", "User creation failed".to_string(), json!({}), StatusCode::INTERNAL_SERVER_ERROR),
            SuccessMessages::CreatedUser { user_id } => {
                let message = format!("User with ID {user_id} is created successfully");
                let data = json!({
                    "user_id": user_id
                });
                ("UserCreated", message, data, StatusCode::CREATED)
            }
            SuccessMessages::FoundUser { user_id, users_list } => {
                let (message_key, message) = match users_list {
                    Some(users) if !users.is_empty() => ("UsersFound", "Users found successfully".to_string()),
                    _ => ("UserFound", format!("User with ID {user_id} found successfully")),
                };
                let data = match users_list {
                    Some(users) => json!({ "users": users }),
                    None => json!({ "user_id": user_id }),
                };
                (message_key, message, data, StatusCode::OK)
            }
            SuccessMessages::DeletedUser { user_id } => ("UserDeleted", format!("User with ID {user_id} deleted successfully"), json!({ "user_id": user_id }), StatusCode::OK),

            // coin
            SuccessMessages::CreatedCoin { coin_id, coin_name } => {
                let message = format!("Coin with ID {coin_id} and name {coin_name} is created successfully");
                let data = json!({
                    "coin_id": coin_id,
                    "coin_name": coin_name
                });
                ("CoinCreated", message, data, StatusCode::CREATED)
            }
            SuccessMessages::FoundCoin { coin_id, coins_list } => {
                let (message_key, message) = match coins_list {
                    Some(coins) if !coins.is_empty() => ("CoinsFound", "Coins found successfully".to_string()),
                    _ => ("CoinFound", format!("Coin with ID {coin_id} found successfully")),
                };
                let data = match coins_list {
                    Some(coins) => json!({ "coins": coins }),
                    None => json!({ "coin_id": coin_id }),
                };
                (message_key, message, data, StatusCode::OK)
            }
            SuccessMessages::UpdatedCoin { coin_id } => ("CoinUpdated", format!("Coin with ID {coin_id} updated successfully"), json!({ "coin_id": coin_id }), StatusCode::OK),
            SuccessMessages::DeletedCoin { coin_id } => ("CoinDeleted", format!("Coin with ID {coin_id} deleted successfully"), json!({ "coin_id": coin_id }), StatusCode::OK),
            SuccessMessages::AddressGenerated { coin, address } => {
                let message = format!("Address generated for coin with ID {coin} is {address}");
                let data = json!({
                    "coin_id": coin,
                    "address": address
                });
                ("AddressGenerated", message, data, StatusCode::OK)
            }
            SuccessMessages::AddressValidated { is_valid, coin, address } => {
                let message = format!("Address for coin with ID {coin} is {address} and is valid = {is_valid}");
                let data = json!({
                    "is_valid": is_valid,
                    "coin_id": coin,
                    "address": address,
                });
                ("AddressValidated", message, data, StatusCode::OK)
            }
            SuccessMessages::UnconfirmedBalance { address, unconfirmed_balance, confirmations } => {
                let message = format!("Unconfirmed balance for address {address} is {unconfirmed_balance} with {confirmations} confirmations");
                let data = json!({
                    "address": address,
                    "unconfirmed_balance": unconfirmed_balance,
                    "confirmations": confirmations
                });
                ("UnconfirmedBalance", message, data, StatusCode::OK)
            }
            SuccessMessages::TransactionInformation { txid, data, confirmations } => {
                let message = format!("Transaction information for address {txid} is {data:?}");
                let data = json!({
                    "address": txid,
                    "data": data,
                    "confirmation": confirmations
                });
                ("TransactionInformation", message, data, StatusCode::OK)
            }
            SuccessMessages::NodeBalance { crypto_balance, usd_balance } => {
                let message = format!("Node balance for crypto is {crypto_balance} and USD is {usd_balance}");
                let data = json!({
                    "crypto_balance": crypto_balance,
                    "usd_balance": usd_balance
                });
                ("NodeBalance", message, data, StatusCode::OK)
            }
            SuccessMessages::TransferSuccessful { hash, amount } => {
                let message = format!("Transfer successful for amount {amount} with hash {hash}");
                let data = json!({
                    "amount": amount,
                    "hash": hash
                });
                ("TransferSuccessful", message, data, StatusCode::OK)
            }

            // rates
            SuccessMessages::ConversionRate { coin_id, symbol, rate } => {
                let message = format!("Conversion rate for coin with ID {coin_id} and symbol {symbol} is {rate}");
                let data = json!({
                    "coin_id": coin_id,
                    "symbol": symbol,
                    "rate": rate
                });
                ("ConversionRate", message, data, StatusCode::OK)
            }

            // wallets
            SuccessMessages::CreatedWallet { user_id, wallet } => {
                let message = format!("Wallet for user with ID {user_id} is created successfully");
                let data = json!({
                    "user_id": user_id,
                    "wallet": wallet
                });
                ("WalletCreated", message, data, StatusCode::CREATED)
            }
            SuccessMessages::FoundWallet { user_id, wallets_list } => {
                let (message_key, message) = ("WalletFound", "found wallets data".to_string());
                let data = json!({ "user_id": user_id, "wallets": wallets_list });
                (message_key, message, data, StatusCode::OK)
            }
            SuccessMessages::UpdatedWallet { wallet_id } => ("WalletUpdated", format!("Wallet with ID {wallet_id} updated successfully"), json!({ "wallet_id": wallet_id }), StatusCode::OK),
            SuccessMessages::DeletedWallet { user_id } => ("WalletDeleted", format!("Wallet with ID {user_id} deleted successfully"), json!({ "user_id": user_id }), StatusCode::OK),

            // withdrawals
            SuccessMessages::WithdrawalFailed => ("WithdrawalFailed", "Withdrawal failed".to_string(), json!({}), StatusCode::INTERNAL_SERVER_ERROR),
            SuccessMessages::CreatedWithdrawal { withdrawal_id, hash, withdrawal, rate } => {
                let message = format!("Withdrawal with ID {withdrawal_id} is created successfully");
                let data = json!({
                    "withdrawal_id": withdrawal_id,
                    "hash": hash,
                    "withdrawal": withdrawal,
                    "rate":rate.to_f64().unwrap()
                });
                ("WithdrawalCreated", message, data, StatusCode::CREATED)
            }
            SuccessMessages::FoundWithdrawal { withdrawal_id, hash, withdrawal_list } => {
                let (message_key, message) = match withdrawal_list {
                    Some(withdrawals) if !withdrawals.is_empty() => ("WithdrawalsFound", "Withdrawals found successfully".to_string()),
                    _ => ("WithdrawalFound", format!("Withdrawal with ID {withdrawal_id} found successfully")),
                };
                let data = match withdrawal_list {
                    Some(withdrawals) => json!({ "withdrawals": withdrawals }),
                    None => json!({ "withdrawal_id": withdrawal_id, "hash": hash }),
                };
                (message_key, message, data, StatusCode::OK)
            }
            SuccessMessages::UpdatedWithdrawal { withdrawal_id } => ("WithdrawalUpdated", format!("Withdrawal with ID {withdrawal_id} updated successfully"), json!({ "withdrawal_id": withdrawal_id }), StatusCode::OK),
            SuccessMessages::FoundWithdrawalsHistory { user_id, page, per_page, total_results, total_pages, data } => {
                let message = format!("Withdrawals history for user with ID {user_id} found successfully");
                let data = json!({
                    "user_id": user_id,
                    "page": page,
                    "per_page": per_page,
                    "total_results": total_results,
                    "total_pages": total_pages,
                    "data": data
                });
                ("WithdrawalsHistoryFetched", message, data, StatusCode::OK)
            }

            // invoice
            SuccessMessages::CreatedInvoice { invoice_id } => {
                let message = format!("Invoice with ID {invoice_id} is created successfully");
                let data = json!({
                    "invoice_id": invoice_id
                });
                ("InvoiceCreated", message, data, StatusCode::CREATED)
            }
            // transaction
            SuccessMessages::TransactionFailed => ("TransactionFailed", "Transaction failed".to_string(), json!({}), StatusCode::INTERNAL_SERVER_ERROR),
            SuccessMessages::CreatedTransaction { transaction_id, transaction } => {
                let message = format!("Transaction with ID {transaction_id} is created successfully");
                let data = json!({
                    "transaction_id": transaction_id,
                    "transaction": transaction
                });
                ("TransactionCreated", message, data, StatusCode::CREATED)
            }
            SuccessMessages::FoundTransaction { transaction_id, transaction } => {
                let message = format!("Transaction with ID {transaction_id} found successfully");
                let data = json!({
                    "transaction_id": transaction_id,
                    "transaction": transaction
                });
                ("TransactionFound", message, data, StatusCode::OK)
            }
            SuccessMessages::FoundTransactions { transactions_list } => {
                let message = "Transactions found successfully".to_string();
                let data = json!({ "transactions": transactions_list });
                ("TransactionsFound", message, data, StatusCode::OK)
            }
            SuccessMessages::UpdatedTransaction { transaction_id } => ("TransactionUpdated", format!("Transaction with ID {transaction_id} updated successfully"), json!({ "transaction_id": transaction_id }), StatusCode::OK),
            SuccessMessages::FoundTransactionsHistory { user_id, page, per_page, total_results, total_pages, data } => {
                let message = format!("Transaction history for user with ID {user_id} found successfully");
                let data = json!({
                    "user_id": user_id,
                    "page": page,
                    "per_page": per_page,
                    "total_results": total_results,
                    "total_pages": total_pages,
                    "data": data
                });
                ("TransactionsHistoryFetched", message, data, StatusCode::OK)
            }
            SuccessMessages::FoundBulkTransactionsHistory { data } => {
                let message = "Bulk transaction history found successfully".to_string();
                let data = json!({ "data": data });
                ("BulkTransactionsHistoryFetched", message, data, StatusCode::OK)
            }
            SuccessMessages::FoundBulkWithdrawalsHistory { data } => {
                let message = "Bulk withdrawals history found successfully".to_string();
                let data = json!({ "data": data });
                ("BulkWithdrawalsHistoryFetched", message, data, StatusCode::OK)
            }

            // config
            SuccessMessages::CreatedConfig { config_id } => {
                let message = format!("Config with ID {config_id} is created successfully");
                let data = json!({
                    "config_id": config_id
                });
                ("ConfigCreated", message, data, StatusCode::CREATED)
            }
            SuccessMessages::FoundConfig { config_id, configs_list } => {
                let (message_key, message) = match configs_list {
                    Some(configs) if !configs.is_empty() => ("ConfigsFound", "Configs found successfully".to_string()),
                    _ => ("ConfigFound", format!("Config with ID {config_id} found successfully")),
                };
                let data = match configs_list {
                    Some(configs) => json!({ "configs": configs }),
                    None => json!({ "config_id": config_id }),
                };
                (message_key, message, data, StatusCode::OK)
            }
            SuccessMessages::UpdatedConfig { config_id, configs_list } => {
                let (message_key, message) = match configs_list {
                    Some(configs) if !configs.is_empty() => ("ConfigsUpdated", "Configs updated successfully".to_string()),
                    _ => ("ConfigUpdated", format!("Config with ID {config_id} updated successfully")),
                };
                let data = match configs_list {
                    Some(configs) => json!({ "configs": configs }),
                    None => json!({ "config_id": config_id }),
                };
                (message_key, message, data, StatusCode::OK)
            }
            SuccessMessages::DeletedConfig { config } => ("ConfigDeleted", format!("Config with name {config} deleted successfully"), json!({ "config": config }), StatusCode::OK),
            SuccessMessages::DeletedAllConfigs => ("ConfigsDeleted", "All configs deleted successfully".to_string(), json!({}), StatusCode::OK),

            // webhooks
            // payments
            SuccessMessages::PaymentProcessed { tx_id, details, confirmations } => {
                ("PaymentProcessed", format!("Transaction with with ID {} processed successfully", tx_id.clone()), json!({ "transaction_id": tx_id, "details": details, "confirmations": confirmations }), StatusCode::OK)
            }
        }
    }
}
