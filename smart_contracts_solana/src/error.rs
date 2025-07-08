use thiserror::Error;
use solana_program::msg;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("User already registered")]
    AlreadyRegistered,
    #[error("Arithmetic overflow")]
    MathOverflow,
}

impl From<RegistryError> for solana_program::program_error::ProgramError {
    fn from(e: RegistryError) -> Self {
        msg!("{}", e.to_string());
        solana_program::program_error::ProgramError::Custom(e as u32)
    }
}
