use serde::Deserialize;
use strum::Display;

pub const MODULE_NAME: &str = "coins_transfer_solana";
pub const USER_CACHE_EXPIRATION: usize = 3600;
pub const COIN_CACHE_EXPIRATION: usize = 36000;
pub const CONFIGS_CACHE_EXPIRATION: usize = 36000;
pub const WITHDRAWALS_CACHE_EXPIRATION: usize = 36000;
pub const WALLETS_CACHE_EXPIRATION: usize = 36000;

pub const INVOICE_CACHE_EXPIRATION: usize = 36000;

pub const USER_TRANSACTION_CACHE_EXPIRATION: usize = 36000;
pub const CONVERSION_RATES_CACHE_EXPIRATION: usize = 36000;
pub const _WEBHOOKS_CACHE_EXPIRATION: usize = 3600;

/// Default database connection limits.
// Hard limits (boundaries) for the number of connections allowed via environment configuration.
pub const DB_MAX_CONNECTIONS_CAP: usize = 60; // Upper boundary for maximum connections.
pub const DB_MIN_CONNECTIONS_FLOOR: usize = 10; // Lower boundary for minimum connections.

// Default connection counts if environment variables are not set.
pub const DB_MAX_CONNECTIONS_DEFAULT: usize = 60; // Default maximum connections for pool.
pub const DB_MIN_CONNECTIONS_DEFAULT: usize = 4; // Default minimum connections for pool.

/// Default timeout values in seconds.
pub const ACQUIRE_TIMEOUT_SECS: u64 = 5; // Default acquire timeout (in seconds)
pub const IDLE_TIMEOUT_SECS: u64 = 10; // Default idle timeout (in seconds)
pub const MAX_LIFETIME_SECS: u64 = 30; // Default maximum lifetime for a connection (in seconds)

/// Default Redis connection configuration.
pub const REDIS_ENABLED_DEFAULT: bool = false;
pub const REDIS_URL_DEFAULT: &str = "redis://127.0.0.1:6379";

#[derive(Debug, Display, Deserialize)]
pub enum Coin {
    Litecoin=1,
    USDT=2,
    USDC=3,
    Solana=4,
}

impl Coin {
    pub fn to_string(&self) -> &str {
        match self {
            Coin::Litecoin => "litecoin",
            Coin::USDT => "usdt",
            Coin::USDC => "usdc",
            Coin::Solana => "solana",
        }
    }

    pub fn from_i16(i: i16) -> Option<Self> {
        match i {
            1 => Some(Coin::Litecoin),
            2 => Some(Coin::USDT),
            3 => Some(Coin::USDC),
            4 => Some(Coin::Solana),
            _ => None,
        }
    }

    pub fn to_i16(&self) -> i16 {
        match self {
            Coin::Litecoin => 1,
            Coin::USDT => 2,
            Coin::USDC => 3,
            Coin::Solana => 4,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "litecoin" => Some(Coin::Litecoin),
            "usdt" => Some(Coin::USDT),
            "usdc" => Some(Coin::USDC),
            "solana" => Some(Coin::Solana),
            _ => None,
        }
    }

    pub fn mint(&self) -> &'static str {
        match self {
            Coin::Litecoin => "ltc",
            Coin::USDT => "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",
            Coin::USDC => "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            Coin::Solana => "sol",
        }
    }

    pub fn from_mint(mint: &str) -> Option<Self> {
        match mint {
            "ltc" => Some(Coin::Litecoin),
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => Some(Coin::USDT),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => Some(Coin::USDC),
            "sol" => Some(Coin::Solana),
            _ => None,
        }
    }

    /// Number of decimal places for each token
    pub fn decimals(&self) -> u8 {
        match self {
            Coin::Litecoin => 8,
            Coin::USDT       => 6,
            Coin::USDC       => 6,
            Coin::Solana     => 9,
        }
    }
}
