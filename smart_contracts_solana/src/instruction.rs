use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum RegistryInstruction {
    /// 0: Register user
    /// Accounts:
    ///   [signer] payer
    ///   [writable] user_account (PDA)
    RegisterUser { bump: u8 },

    /// 1: Transfer SOL
    /// Accounts:
    ///   [signer] from
    ///   [writable] to
    TransferSol { amount: u64 },

    /// 2: Transfer SPL token
    /// Accounts:
    ///   [signer] payer
    ///   [] token_program
    ///   [writable] from_ata
    ///   [writable] to_ata
    ///   [] mint
    TransferSpl { amount: u64 },

    /// 3: Validate Transaction (compare pre/post balances)
    /// Accounts:
    ///   [] system_program
    ///   [writable] account_to_check
    ValidateTxn { pre_balance: u64 },
}

impl RegistryInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, solana_program::program_error::ProgramError> {
        Self::try_from_slice(input)
            .map_err(|_| solana_program::program_error::ProgramError::InvalidInstructionData)
    }
}
