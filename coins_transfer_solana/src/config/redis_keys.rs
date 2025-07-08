use crate::config::constants::MODULE_NAME;

#[derive(Debug)]
pub enum RedisKeys {
    // users
    User { user_id: i64 },
    AllUsers,
    // configs
    Config { name: String },
    AllConfigs,
    // coins
    CoinById { id: i16 },
    CoinByName { name: String },
    CoinBySymbol { symbol: String },
    AllCoins,
    ConversionRate { coin_id: i16 },
    // user transactions
    Deposits { id: i64 },
    UserDeposits { user_id: i64 },
    // user wallets
    UserWallet { id: i64 },
    WalletsByUserId { user_id: i64 },
    WalletsByCoinId { coin_id: i16 },
    AllWallets,
    // withdrawals
    Withdrawal { id: i64 },
    WithdrawalsByUserId { user_id: i64 },
}

impl RedisKeys {
    pub fn to_string(&self) -> String {
        match self {
            // users
            RedisKeys::User { user_id } => format!("{MODULE_NAME}:users:{user_id}"),
            RedisKeys::AllUsers => format!("{MODULE_NAME}:users:all"),
            // configs
            RedisKeys::Config { name } => format!("{MODULE_NAME}:configs:{name}"),
            RedisKeys::AllConfigs => format!("{MODULE_NAME}:configs:all"),
            // coins
            RedisKeys::CoinById { id } => format!("{MODULE_NAME}:coins:id:{id}"),
            RedisKeys::CoinByName { name } => format!("{MODULE_NAME}:coins:name:{name}"),
            RedisKeys::CoinBySymbol { symbol } => format!("{MODULE_NAME}:coins:symbol:{symbol}"),
            RedisKeys::AllCoins => format!("{MODULE_NAME}:coins:all"),
            RedisKeys::ConversionRate { coin_id } => format!("{MODULE_NAME}:conversion_rate:coin_id:{coin_id}"),
            // cash transactions
            RedisKeys::Deposits { id } => format!("{MODULE_NAME}:deposits:{id}"),
            RedisKeys::UserDeposits { user_id } => format!("{MODULE_NAME}:deposits:user_id:{user_id}"),
            // user wallets
            RedisKeys::UserWallet { id } => format!("{MODULE_NAME}:wallets:{id}"),
            RedisKeys::WalletsByUserId { user_id } => format!("{MODULE_NAME}:wallets:user_id:{user_id}"),
            RedisKeys::WalletsByCoinId { coin_id } => format!("{MODULE_NAME}:wallets:coin_id:{coin_id}"),
            RedisKeys::AllWallets => format!("{MODULE_NAME}:wallets:all"),
            // withdrawals
            RedisKeys::Withdrawal { id } => format!("{MODULE_NAME}:withdrawals:id:{id}"),
            RedisKeys::WithdrawalsByUserId { user_id } => format!("{MODULE_NAME}:withdrawals:user_id:{user_id}"),
        }
    }
}
