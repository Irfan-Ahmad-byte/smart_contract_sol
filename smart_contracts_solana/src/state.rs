use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

// 1. User account data
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserAccount {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub created_at: u64, // Unix timestamp
}

impl UserAccount {
    pub const LEN: usize = 1 + 32 + 8;
}
