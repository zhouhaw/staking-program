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
    state::{
        StakePool,
        UserInfo,
    },
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
                pool_name_for_token_pda, 
                pool_name_for_state_pda,
                bump_seed,
            } => {
                msg!("Instruction: Initialize stake pool");
                Self::process_initialize(
                    program_id,
                    accounts,
                    amount_reward,
                    pool_name_for_token_pda,
                    pool_name_for_state_pda,
                    bump_seed,
                )
            }
        }
    }

    fn process_initialize(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount_reward: u64,
        pool_name_for_token_pda: String,
        pool_name_for_state_pda: String,
        bump_seed: [u8; 2],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let owner_account_info = next_account_info(account_info_iter)?; // 0
        let token_account_info = next_account_info(account_info_iter)?; // 1

        if *token_account_info.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let token_account = spl_token::state::Account::unpack_unchecked(
            &token_account_info.data.borrow(),
        )?;

        Self::validate_owner(
            &token_account.owner,
            owner_account_info,
        )?;

        if token_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }

        if token_account.amount < amount_reward {
            return Err(TokenError::InsufficientFunds.into());
        }

        let pool_token_account_info = next_account_info(account_info_iter)?; // 2
        let pda_state_info = next_account_info(account_info_iter)?; // 3
        let this_program_info = next_account_info(account_info_iter)?; // 4
        let token_info = next_account_info(account_info_iter)?; // 5

        let rent_info = next_account_info(account_info_iter)?; // 6
        let rent = &Rent::from_account_info(rent_info)?; 

        let system_program_info = next_account_info(account_info_iter)?; // 7
        let token_program_info = next_account_info(account_info_iter)?; // 8

        let minimum_balance_token_acc = rent.minimum_balance(TokenAccount::LEN);
        let signers_seeds_token_pda: &[&[_]] = &[pool_name_for_token_pda.as_bytes(), &[bump_seed[0]]];

        let instruction_create_token_account = system_instruction::create_account(
            owner_account_info.key,
            pool_token_account_info.key,
            minimum_balance_token_acc,
            TokenAccount::LEN as u64,
            &spl_token::id(),
        );

        invoke_signed(
            &instruction_create_token_account,
            &[owner_account_info.clone(), pool_token_account_info.clone(), system_program_info.clone()],
            &[&signers_seeds_token_pda],
        )?;

        let instruction_initialize_account = spl_token::instruction::initialize_account(
            &spl_token::id(),
            pool_token_account_info.key,
            token_info.key,
            this_program_info.key,
        )?;

        invoke_signed(
            &instruction_initialize_account,
            &[
            pool_token_account_info.clone(), 
            token_info.clone(), 
            this_program_info.clone(),
            rent_info.clone(),
            token_program_info.clone(),
            ],
            &[&signers_seeds_token_pda],
        )?;

        let min_balance_pool = rent.minimum_balance(StakePool::LEN);
        let min_balance_user_info = rent.minimum_balance(UserInfo::LEN);
        let min_balance = min_balance_pool + min_balance_user_info * 5;

        let signers_seeds_state_pda: &[&[_]] = &[pool_name_for_state_pda.as_bytes(), &[bump_seed[1]]];

        let instruction_create_account_for_stake_pool = system_instruction::create_account(
            owner_account_info.key,
            pda_state_info.key,
            min_balance,
            StakePool::LEN as u64,
            this_program_info.key,
        );

        invoke_signed(
            &instruction_create_account_for_stake_pool,
            &[owner_account_info.clone(), pda_state_info.clone(), system_program_info.clone()],
            &[&signers_seeds_state_pda],
        )?;

        let stake_pool_data = StakePool {
            pool_owner: *owner_account_info.key,
            is_initialized: 1,
        };

        StakePool::pack(stake_pool_data, &mut pda_state_info.data.borrow_mut())?;
        let data_stake_pool = StakePool::unpack(&pda_state_info.data.borrow())?;

        msg!("unpacked stake pool data: {:#?}", data_stake_pool);

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