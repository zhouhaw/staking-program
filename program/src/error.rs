use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug)]
pub enum StakingError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Unable to add new pool to the list")]
    UnableToAddPool,
}

impl From<StakingError> for ProgramError {
    fn from(e: StakingError) -> Self {
        ProgramError::Custom(e as u32)
    }
}