use solana_program::{
    account_info::{
        next_account_info,
        AccountInfo
    }, 
    program_error::{
        ProgramError,
    },
    entrypoint::ProgramResult, 
    msg, 
    program_pack::Pack, 
    pubkey::Pubkey,
};
use spl_token::{
    state::{
        Account,
    },
    instruction::{
        transfer,
    },
    error::{
        TokenError,
    },
};
use borsh::{
    BorshDeserialize,
    BorshSerialize,
    BorshSchema,
};
use crate::{
    error::StakingError, 
    instruction::StakingInstruction
};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult{
        let instruction = StakingInstruction::try_from_slice(instruction_data)?;

        match instruction {
            StakingInstruction::Initialize { 
                amount_reward,
                pool_name, 
            } => {
                msg!("Instruction: Initialize stake pool");
                Self::process_initialize(
                    program_id,
                    accounts,
                    amount_reward,
                    pool_name,
                )
            }
        }
    }

    fn process_initialize(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount_reward: u64,
        pool_name: [u8; 32],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let owner_token_account_info = next_account_info(account_info_iter)?;
        let token_account_info = next_account_info(account_info_iter)?;

        if *token_account_info.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let token_account = spl_token::state::Account::unpack_unchecked(
            &token_account_info.data.borrow(),
        )?;

        Self::validate_owner(
            &token_account.owner,
            owner_token_account_info,
        )?;

        if token_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }

        if token_account.amount < amount_reward {
            return Err(TokenError::InsufficientFunds.into());
        }

        let pool_token_account_info = next_account_info(account_info_iter)?;

        let token_info = next_account_info(account_info_iter)?;

        msg!(
            "Token account {} has {} tokens\n
            Args: amount_reward: {}, pool_name: {:?}",
            token_account_info.key,
            token_account.amount,
            amount_reward,
            pool_name,
        );

        Ok(())
    }

    pub fn validate_owner(
        expected_owner: &Pubkey,
        owner_account_info: &AccountInfo,
    ) -> ProgramResult {
        if expected_owner != owner_account_info.key {
            return Err(TokenError::OwnerMismatch.into());
        }

        if !owner_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature.into());
        }

        Ok(())
    }
}