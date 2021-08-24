use solana_program::{
    account_info::{
        next_account_info,
        AccountInfo
    }, 
    program_error::{
        ProgramError,
    }, 
    rent::{
        Rent,
    }, 
    system_program,
    program::invoke_signed,
    entrypoint::ProgramResult, 
    msg, 
    program_pack::Pack, 
    pubkey::Pubkey, 
    system_instruction, 
    sysvar::Sysvar,
};
use spl_token::{
    state::{
        Account as TokenAccount,
    },
    instruction::{
        transfer,
        initialize_account,
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
    instruction::StakingInstruction,
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
                bump_seed,
            } => {
                msg!(
                    "Instruction: Initialize stake pool.
                    Args: pool_name = {}, bump_seed = {}",
                    pool_name, bump_seed,
                );
                Self::process_initialize(
                    program_id,
                    accounts,
                    amount_reward,
                    pool_name,
                    bump_seed,
                )
            }
        }
    }

    fn process_initialize(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount_reward: u64,
        pool_name: String,
        bump_seed: u8,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let owner_token_account_info = next_account_info(account_info_iter)?; // 0
        let token_account_info = next_account_info(account_info_iter)?; // 1

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

        let mint_account_info = next_account_info(account_info_iter)?; // 2
        let pool_token_account_info = next_account_info(account_info_iter)?; // 3
        let this_program_info = next_account_info(account_info_iter)?; // 4
        let token_info = next_account_info(account_info_iter)?; // 5

        let rent_info = next_account_info(account_info_iter)?; // 6
        let rent = &Rent::from_account_info(rent_info)?; 

        let system_program_info = next_account_info(account_info_iter)?; // 7
        let token_program_info = next_account_info(account_info_iter)?; // 8

        let minimum_balance = rent.minimum_balance(TokenAccount::LEN);
        let signers_seeds: &[&[_]] = &[pool_name.as_bytes(), &[bump_seed]];

        let instruction_create_account = system_instruction::create_account(
            owner_token_account_info.key,
            pool_token_account_info.key,
            minimum_balance,
            TokenAccount::LEN as u64,
            &spl_token::id(),
        );

        msg!(
            "invoke_signed (create_account). token_info.key is {}",
            token_info.key
        );

        invoke_signed(
            &instruction_create_account,
            &[owner_token_account_info.clone(), pool_token_account_info.clone(), system_program_info.clone()],
            &[&signers_seeds],
        )?;

        let instruction_initialize_account = spl_token::instruction::initialize_account(
            &spl_token::id(),
            pool_token_account_info.key,
            token_info.key,
            this_program_info.key,
        )?;

        msg!("invoke_signed (initialize_account)");

        invoke_signed(
            &instruction_initialize_account,
            &[
            pool_token_account_info.clone(), 
            token_info.clone(), 
            this_program_info.clone(),
            rent_info.clone(),
            token_program_info.clone(),
            ],
            &[&signers_seeds],
        )?;

        msg!("Successfully!");

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