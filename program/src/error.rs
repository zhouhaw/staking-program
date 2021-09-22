use std::num::TryFromIntError;

use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug)]
pub enum StakingError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Unable to add new pool to the list")]
    UnableToAddPool,

    #[error("Operation overflowed")] // 0x2
    StakedTokenSupplyOverflow,
    #[error("Operation overflowed")] 
    RewardOverflow,
    #[error("Operation overflowed")]
    RewardMulPrecisionOverflow,
    #[error("Operation overflowed")]
    RewardMulPrecisionDivSupplyOverflow,
    #[error("Operation overflowed")]
    AccuredTokenPerShareOverflow,
    #[error("Operation overflowed")] // 0x7
    Overflow,
}

impl From<TryFromIntError> for StakingError{
    fn from(e: TryFromIntError) -> Self {
        StakingError::Overflow
    }
}

impl From<StakingError> for ProgramError {
    fn from(e: StakingError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
