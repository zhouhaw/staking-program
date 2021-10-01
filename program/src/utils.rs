use std::convert::TryFrom;
use solana_program::{
    account_info::{
        AccountInfo
    }, 
    program_error::{
        PrintProgramError,
    },
    entrypoint::ProgramResult, 
    program_pack::Pack, 
    pubkey::Pubkey, 
};
use spl_token::{
    state::Account as TokenAccount,
};
use crate::{
    state::StakePool,
    error::StakingError, 
    id as this_program_id,
    ADD_SEED_TOKEN_ACCOUNT_AUTHORITY,
    BUMP_SEED_TOKEN_ACCOUNT_AUTHORITY,
};

pub fn validate_stake_pool(
    stake_pool: &StakePool,
    owner_key: &Pubkey,
    mint_key: &Pubkey,
) -> ProgramResult {
    if stake_pool.owner != *owner_key || 
       stake_pool.mint != *mint_key {
            StakingError::StakePoolMissmatch.print::<StakingError>();
            return Err(StakingError::StakePoolMissmatch.into());
    }

    Ok(())
}

pub fn validate_pool_token_account(
    pool_token_account_info: &AccountInfo,
) -> ProgramResult {
    let pool_token_account = TokenAccount::unpack(
        &pool_token_account_info.data.borrow(),
    )?;
    let pool_token_account_authority_pubkey = Pubkey::create_program_address(
        &[ADD_SEED_TOKEN_ACCOUNT_AUTHORITY.as_bytes(), &[BUMP_SEED_TOKEN_ACCOUNT_AUTHORITY]],
        &this_program_id(),
    )?;

    if pool_token_account.owner != pool_token_account_authority_pubkey {
        StakingError::PoolTokenAccountMissmatch.print::<StakingError>();
        return Err(StakingError::PoolTokenAccountMissmatch.into());
    }

    Ok(())
}

pub fn validate_user_state(
    user_state_info: &AccountInfo,
    stake_pool_info: &AccountInfo,
    token_account_info: &AccountInfo,
) -> ProgramResult {
    let (user_state_pubkey, _) = Pubkey::find_program_address(
        &[stake_pool_info.key.as_ref(), token_account_info.key.as_ref()],
        &this_program_id(),
    );

    if user_state_pubkey != *user_state_info.key {
        StakingError::UserInfoMissmatch.print::<StakingError>();
        return Err(StakingError::UserInfoMissmatch.into());
    }

    Ok(())
}

pub fn get_pending(
    current_amount: u64,
    accrued_token_per_share: u128,
    precision_factor_rank: u8,
    reward_debt: u64,
) -> Result<u64, StakingError> {
    let precision_factor = get_precision_factor(precision_factor_rank)?;

    let pending = (current_amount as u128) 
        .checked_mul(accrued_token_per_share)
        .ok_or(StakingError::Overflow)?
        .checked_div(precision_factor as u128)
        .ok_or(StakingError::Overflow)?
        .checked_sub(reward_debt as u128)
        .ok_or(StakingError::Overflow)?;
    
    match u64::try_from(pending) {
        Ok(pending) => Ok(pending),
        Err(e) => Err(e.into()),
    }
}

pub fn get_reward_debt(
    user_amount: u64,
    accrued_token_per_share: u128,
    precision_factor_rank: u8,
) -> Result<u64, StakingError> {
    let precision_factor = get_precision_factor(precision_factor_rank)?;

    let reward_debt = (user_amount as u128)
        .checked_mul(accrued_token_per_share)
        .ok_or(StakingError::Overflow)?
        .checked_div(precision_factor as u128)
        .ok_or(StakingError::Overflow)? as u64;

    Ok(reward_debt)
}

pub fn get_precision_factor(
    precision_factor_rank: u8,
) -> Result<u64, StakingError> {
    let precision_factor = 10_u64
        .checked_pow(precision_factor_rank as u32)
        .ok_or(StakingError::Overflow)?;

    Ok(precision_factor)
}