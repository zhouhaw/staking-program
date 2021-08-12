use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug)]
pub enum StakingError {
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,

    #[error("Initialize: not enough tokens")]
    InitializeNotEnoughTokens,
}

impl From<StakingError> for ProgramError {
    fn from(e: StakingError) -> Self {
        ProgramError::Custom(e as u32)
    }
}