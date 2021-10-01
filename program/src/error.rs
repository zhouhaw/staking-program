use std::num::TryFromIntError;
use num_derive::FromPrimitive;
use thiserror::Error;
use solana_program::{
    program_error::{
        PrintProgramError, 
        ProgramError
    },
    decode_error::DecodeError,
    msg,
};

#[derive(Error, Debug, FromPrimitive)]
pub enum StakingError {
    #[error("Operation overflowed")] 
    RewardOverflow,
    #[error("Operation overflowed")]
    RewardMulPrecisionOverflow,
    #[error("Operation overflowed")]
    RewardMulPrecisionDivSupplyOverflow,
    #[error("Operation overflowed")]
    AccuredTokenPerShareOverflow,
    #[error("Pool counter overflow")]
    PoolCounterOverflow,
    #[error("Operation overflowed")] 
    Overflow,

    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Unable to deserializse MasterStaking")]
    InvalidMasterStaking,
    #[error("Unable to deserialize UserInfo")]
    InvalidUserInfo,
    #[error("Unable to add new pool to the list")]
    UnableToAddPool,

    #[error("Pool Owner or pool Mint missmatch")]
    StakePoolMissmatch,
    #[error("Pool Token Account missmatch")]
    PoolTokenAccountMissmatch,
    #[error("User Info missmatch")]
    UserInfoMissmatch,
}

impl PrintProgramError for StakingError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl<T> DecodeError<T> for StakingError {
    fn type_of() -> &'static str {
        "Staking Error"
    }
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
