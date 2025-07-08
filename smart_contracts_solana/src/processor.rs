use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::{next_account_info, AccountInfo}, entrypoint::ProgramResult, msg, program::invoke, program_error::ProgramError, pubkey::Pubkey, sysvar::clock::Clock, sysvar::Sysvar};
use solana_system_interface::instruction as system_instruction;
use spl_token::instruction as token_instruction;
use crate::{
    error::RegistryError,
    instruction::RegistryInstruction,
    state::UserAccount,
};

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: &[u8],
    ) -> ProgramResult {
        let ix = RegistryInstruction::unpack(data)?;
        match ix {
            RegistryInstruction::RegisterUser { bump } => {
                Self::register_user(program_id, accounts, bump)
            }
            RegistryInstruction::TransferSol { amount } => {
                Self::transfer_sol(accounts, amount)
            }
            RegistryInstruction::TransferSpl { amount } => {
                Self::transfer_spl(accounts, amount)
            }
            RegistryInstruction::ValidateTxn { pre_balance } => {
                Self::validate_txn(accounts, pre_balance)
            }
        }
    }

    fn register_user(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        bump: u8,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let user_account = next_account_info(account_info_iter)?;

        // Derive PDA
        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"user", payer.key.as_ref()], program_id);
        if pda != *user_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        // Initialize account
        let mut data = user_account.try_borrow_mut_data()?;
        let mut state = UserAccount::try_from_slice(&data)?;
        if state.is_initialized {
            return Err(RegistryError::AlreadyRegistered.into());
        }
        state.is_initialized = true;
        state.owner = *payer.key;
        let clock = Clock::get()?;
        state.created_at = clock.unix_timestamp as u64;
        state.serialize(&mut *data)?;

        msg!("User registered: {}", payer.key);
        Ok(())
    }

    fn transfer_sol(
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let from = next_account_info(account_info_iter)?;
        let to = next_account_info(account_info_iter)?;
        // invoke system transfer
        let ix = system_instruction::transfer(from.key, to.key, amount);
        invoke(&ix, &[from.clone(), to.clone()])?;
        msg!("Transferred {} lamports", amount);
        Ok(())
    }

    fn transfer_spl(
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;
        let from_ata = next_account_info(account_info_iter)?;
        let to_ata = next_account_info(account_info_iter)?;
        let mint = next_account_info(account_info_iter)?;

        let ix = token_instruction::transfer(
            token_program.key,
            from_ata.key,
            to_ata.key,
            payer.key,
            &[],
            amount,
        )?;
        invoke(&ix, &[
            from_ata.clone(),
            to_ata.clone(),
            payer.clone(),
            token_program.clone(),
        ])?;
        msg!("Transferred {} tokens", amount);
        Ok(())
    }

    fn validate_txn(
        accounts: &[AccountInfo],
        pre_balance: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let acct = next_account_info(account_info_iter)?;
        let lamports = acct.lamports();
        if lamports < pre_balance {
            msg!("Balance decreased: {} -> {}", pre_balance, lamports);
        } else {
            msg!("Balance increased or same: {} -> {}", pre_balance, lamports);
        }
        Ok(())
    }
}
