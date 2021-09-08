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
    program::{
        invoke_signed,
        invoke,
    },
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
use std::{
    str::FromStr,
};
use crate::{
    error::StakingError, 
    instruction::StakingInstruction,
    state::{
        VEC_STATE_SPACE,
        unpack_from_slice,
        pack_into_slice,
        StakePool,
        STAKE_POOL_LEN,
        UserInfo,
        USER_INFO_LEN,
    },
    id as this_program_id,
    LIST_OF_POOLS,
    BUMP_SEED_FOR_LIST,
    BUMP_SEED_FOR_STATE_POOL,
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
                msg!("Instruction: Initialize stake pool");
                Self::process_initialize(
                    accounts,
                    amount_reward,
                    pool_name,
                    bump_seed,
                )
            },
            StakingInstruction::Deposit {
                amount,
            } => {
                msg!("Instruction: Deposit");
                Self::process_deposit(
                    accounts,
                    amount,
                )
            },
            StakingInstruction::CreateVecOfPools
            => {
                msg!("Instruction: Create vec of pools");
                Self::process_create_vec_of_pools(
                    accounts,
                )
            },
        }
    }

    fn process_initialize(
        accounts: &[AccountInfo],
        amount_reward: u64,
        pool_name: [u8; 31],
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
        let pda_list_of_pools_info = next_account_info(account_info_iter)?; // 4

        Self::validate_pda_vec_of_pools(pda_list_of_pools_info)?;

        let this_program_info = next_account_info(account_info_iter)?; // 5

        if (*this_program_info.key != this_program_id()){
            return Err(ProgramError::IncorrectProgramId);
        }

        let token_info = next_account_info(account_info_iter)?; // 6

        let rent_info = next_account_info(account_info_iter)?; // 7
        let rent = &Rent::from_account_info(rent_info)?; 

        let system_program_info = next_account_info(account_info_iter)?; // 8
        let token_program_info = next_account_info(account_info_iter)?; // 9

        let minimum_balance_token_acc = rent.minimum_balance(TokenAccount::LEN);
        let signers_seeds_token_pda: &[&[_]] = 
            &[
            owner_account_info.key.as_ref(), 
            token_info.key.as_ref(),
            &[bump_seed[0]],
            ];

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

        let instruction_transfer_tokens = spl_token::instruction::transfer(
            &spl_token::id(),
            token_account_info.key,
            pool_token_account_info.key,
            owner_account_info.key,
            &[owner_account_info.key],
            amount_reward,
        )?;

        invoke(
            &instruction_transfer_tokens,
            &[
            token_account_info.clone(), 
            pool_token_account_info.clone(), 
            owner_account_info.clone(),
            token_program_info.clone(),
            ],
        )?;

        let min_balance_pool = rent.minimum_balance(STAKE_POOL_LEN);
        let min_balance_user_info = rent.minimum_balance(USER_INFO_LEN);
        let min_balance_stake_pool = min_balance_pool + min_balance_user_info * 5;

        let signers_seeds_state_pda: &[&[_]] = 
            &[
            owner_account_info.key.as_ref(), 
            token_info.key.as_ref(),
            &[BUMP_SEED_FOR_STATE_POOL],
            &[bump_seed[1]]
            ];

        let instruction_create_account_for_stake_pool = system_instruction::create_account(
            owner_account_info.key,
            pda_state_info.key,
            min_balance_stake_pool,
            STAKE_POOL_LEN as u64,
            this_program_info.key,
        );

        invoke_signed( 
            &instruction_create_account_for_stake_pool,
            &[owner_account_info.clone(), pda_state_info.clone(), system_program_info.clone()],
            &[&signers_seeds_state_pda],
        )?;

        let stake_pool = StakePool {
            pool_owner: *owner_account_info.key,  
            is_initialized: 1,
            pool_name: pool_name,
        };

        stake_pool.serialize(&mut &mut pda_state_info.data.borrow_mut()[..])?;

        let unpacked_stake = StakePool::try_from_slice(&pda_state_info.data.borrow())?;

        msg!("unpacked_stake is {:#?}", unpacked_stake);

        let list_of_pools_key = Pubkey::create_program_address(
            &[LIST_OF_POOLS.as_bytes(), &[BUMP_SEED_FOR_LIST]],
            this_program_info.key,
        )?;
        
        let mut vec_of_pools = unpack_from_slice(&pda_list_of_pools_info.data.borrow()).unwrap();
        vec_of_pools.push(*pda_state_info.key);
        pack_into_slice(&vec_of_pools, &mut pda_list_of_pools_info.data.borrow_mut());

        let vec_of_pools_unpacked = unpack_from_slice(&pda_list_of_pools_info.data.borrow()).unwrap();

        msg!("unppacked_vec_of_pools {:#?}", vec_of_pools_unpacked);

        Ok(())
    }

    pub fn process_deposit(
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        
        let owner_info = next_account_info(account_info_iter)?; // 0
        let token_account_info = next_account_info(account_info_iter)?; // 1
        
        if *token_account_info.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }
        
        let token_account = spl_token::state::Account::unpack_unchecked(
            &token_account_info.data.borrow(),
        )?;
        
        if token_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }
        
        if token_account.amount < amount || amount == 0 {
            return Err(TokenError::InsufficientFunds.into());
        };

        let mint_pubkey = token_account.mint;
        
        let pda_pool_info = next_account_info(account_info_iter)?; // 2
        let pda_pool_token_account_info = next_account_info(account_info_iter)?; // 3
        let pda_user_state_info = next_account_info(account_info_iter)?; // 4
        let this_program_info = next_account_info(account_info_iter)?; // 5 
        
        let pda_pool_token_account = spl_token::state::Account::unpack_unchecked(
            &pda_pool_token_account_info.data.borrow(),
        )?;
 
        if pda_pool_token_account.owner != *this_program_info.key {
            return Err(ProgramError::IllegalOwner);
        }

        let rent_info = next_account_info(account_info_iter)?; // 6
        let rent = &Rent::from_account_info(rent_info)?;

        let system_program_info = next_account_info(account_info_iter)?; // 7

        let token_program_info = next_account_info(account_info_iter)?; // 8
        
        if pda_user_state_info.data_is_empty() {
            msg!("Creating account for UserInfo");

            let stake_pool = StakePool::try_from_slice(&pda_pool_info.data.borrow())?;

            let (pda_pool_pubkey, bump_seed_pool_state) = Pubkey::find_program_address(
                &[stake_pool.pool_owner.as_ref(), mint_pubkey.as_ref(), &[BUMP_SEED_FOR_STATE_POOL]],
                &this_program_info.key, 
            );

            let (pda_user_state_pubkey, bump_seed_user_state) = Pubkey::find_program_address(
                &[pda_pool_pubkey.as_ref(), token_account_info.key.as_ref()],
                &this_program_info.key,
            );
            
            let signers_seeds_pda_pool: &[&[_]] = 
                &[
                stake_pool.pool_owner.as_ref(), 
                mint_pubkey.as_ref(), 
                &[BUMP_SEED_FOR_STATE_POOL],
                &[bump_seed_pool_state],
                ];
            
            let signers_seeds_pda_user_state: &[&[_]] = 
                &[
                pda_pool_pubkey.as_ref(),
                token_account_info.key.as_ref(),
                &[bump_seed_user_state],
                ]; 
            
            let min_balance_user_info = rent.minimum_balance(USER_INFO_LEN);

            let instruction_create_account_for_user_info = system_instruction::create_account(
                owner_info.key, // account for transfer "from" must not carry data
                pda_user_state_info.key,
                min_balance_user_info,
                USER_INFO_LEN as u64,
                this_program_info.key,
            );

            invoke_signed( 
                &instruction_create_account_for_user_info,
                &[owner_info.clone(), pda_user_state_info.clone(), system_program_info.clone()],
                &[signers_seeds_pda_user_state],
                //&[&signers_seeds_pda_pool, &signers_seeds_pda_user_state],
            )?;

            let user_data = UserInfo {
                token_account_id: *token_account_info.key, 
                amount: 0,
                reward_debt: 0,
            };

            user_data.serialize(&mut &mut pda_user_state_info.data.borrow_mut()[..])?;
            
        } 
        
        let mut user_data = UserInfo::try_from_slice(&pda_user_state_info.data.borrow())?;  

        user_data.amount += amount;

        user_data.serialize(&mut &mut pda_user_state_info.data.borrow_mut()[..])?;
        
        Ok(())
    }

    pub fn process_create_vec_of_pools(
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let payer_info = next_account_info(account_info_iter)?; // 0
        let list_of_pools_info = next_account_info(account_info_iter)?; // 1

        Self::validate_pda_vec_of_pools(list_of_pools_info)?;

        let this_program_info = next_account_info(account_info_iter)?; // 2

        if (*this_program_info.key != this_program_id()){
            return Err(ProgramError::IncorrectProgramId);
        }

        let rent_info = next_account_info(account_info_iter)?; // 3
        let rent = &Rent::from_account_info(rent_info)?;

        let system_program_info = next_account_info(account_info_iter)?; // 4

        msg!("Creating the account for vec of pools");

        let min_balance = rent.minimum_balance(VEC_STATE_SPACE);
        let signers_seeds_list_pda: &[&[_]] = &[LIST_OF_POOLS.as_bytes(), &[BUMP_SEED_FOR_LIST]];

        let instruction_create_account_for_list = system_instruction::create_account(
            payer_info.key,
            list_of_pools_info.key,
            min_balance,
            VEC_STATE_SPACE as u64,
            this_program_info.key, 
        );
        invoke_signed(
            &instruction_create_account_for_list,
            &[payer_info.clone(), list_of_pools_info.clone(), system_program_info.clone()],
            &[&signers_seeds_list_pda],
        )?;

        let vec_of_pools: Vec<Pubkey> = Vec::new();
        pack_into_slice(&vec_of_pools, &mut list_of_pools_info.data.borrow_mut());
 
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

    pub fn validate_pda_vec_of_pools(
        vec_of_pools_info: &AccountInfo,
    ) -> ProgramResult {
        let list_of_pools_key = Pubkey::create_program_address(
            &[LIST_OF_POOLS.as_bytes(), &[BUMP_SEED_FOR_LIST]],
            &this_program_id(),
        )?;

        if *vec_of_pools_info.key != list_of_pools_key {
            return Err(ProgramError::IncorrectProgramId);
        }

        Ok(())
    }
}