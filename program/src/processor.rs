use std::convert::TryFrom;
use solana_program::{
    account_info::{
        next_account_info,
        AccountInfo
    }, 
    program::{
        invoke_signed,
        invoke,
    },
    program_error::ProgramError,
    entrypoint::ProgramResult, 
    program_pack::Pack, 
    pubkey::Pubkey, 
    system_instruction, 
    sysvar::Sysvar,
    clock::Clock,
    rent::Rent,
    msg, 
};
use spl_token::{
    state::Account as TokenAccount,
    state::Mint as TokenMint,
    error::TokenError,
};
use borsh::{
    BorshDeserialize,
    BorshSerialize,
};
use crate::{
    state::{
        VEC_STATE_SPACE,
        unpack_from_slice,
        pack_into_slice,
        StakePool,
        STAKE_POOL_LEN,
        UserInfo,
        USER_INFO_LEN,
    },
    error::StakingError, 
    instruction::StakingInstruction,
    id as this_program_id,
    ADD_SEED_LIST_OF_POOLS,
    BUMP_SEED_LIST_OF_POOLS,
    ADD_SEED_TOKEN_ACCOUNT_AUTHORITY,
    ADD_SEED_STATE_POOL,
    ADD_SEED_WALLET_POOL,
};

pub struct Processor;
impl Processor {
    pub fn process(
        _program_id: &Pubkey, 
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult{
        let instruction = StakingInstruction::try_from_slice(instruction_data)?; 

        match instruction {
            StakingInstruction::Initialize {  
                n_reward_tokens,
                reward_amount, 
                pool_name, 
                start_block,
                end_block,
            } => {
                msg!("Instruction: Initialize stake pool");
                Self::process_initialize(
                    accounts,
                    n_reward_tokens,
                    reward_amount,
                    pool_name,
                    start_block,
                    end_block,
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
            StakingInstruction::Withdraw {
                amount,
            } => {
                msg!("Instruction: Withdraw");
                Self::process_withdraw(
                    accounts,
                    amount,
                )
            },
            StakingInstruction::EmergencyWithdraw 
            => {
                msg!("Instruiction: Emergency Withdraw");
                Self::process_emergency_withdraw(
                    accounts,
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
        n_reward_tokens: u64,
        reward_amount: u64,
        pool_name: [u8; 31],
        start_block: u64,
        end_block: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let owner_account_info = next_account_info(account_info_iter)?; // 0
        let pda_stake_pool_info = next_account_info(account_info_iter)?; // 1
        let pda_wallet_for_create_user_info = next_account_info(account_info_iter)?; // 2
        let pda_list_of_pools_info = next_account_info(account_info_iter)?; // 3

        Self::validate_pda_vec_of_pools(pda_list_of_pools_info)?;

        let this_program_info = next_account_info(account_info_iter)?; // 4
        if *this_program_info.key != this_program_id(){
            return Err(ProgramError::IncorrectProgramId);
        }

        let mint_info = next_account_info(account_info_iter)?; // 5
        let mint = TokenMint::unpack_unchecked(&mint_info.data.borrow())?;

        let rent_info = next_account_info(account_info_iter)?; // 6
        let rent = &Rent::from_account_info(rent_info)?; 

        let system_program_info = next_account_info(account_info_iter)?; // 7
        let token_program_info = next_account_info(account_info_iter)?; // 8
        let token_account_info = next_account_info(account_info_iter)?; // 9

        if *token_account_info.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let token_account = TokenAccount::unpack_unchecked(
            &token_account_info.data.borrow(),
        )?;

        if token_account.owner != *owner_account_info.key {
            return Err(TokenError::OwnerMismatch.into());
        }

        if !owner_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if token_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }

        if token_account.amount < reward_amount {
            return Err(TokenError::InsufficientFunds.into()); 
        }

        let pda_pool_token_account_authority_info = next_account_info(account_info_iter)?; // 10

        // TODO: Add validate for token-account
        let pda_pool_token_account_info = next_account_info(account_info_iter)?; // 11

        let (_pda_token_account_authority_pubkey, bump_seed_token_account_authority) = Pubkey::find_program_address(
            &[owner_account_info.key.as_ref(), mint_info.key.as_ref(), ADD_SEED_TOKEN_ACCOUNT_AUTHORITY.as_bytes()],
            &this_program_info.key,
        );
        let sign_seeds_pda_token_account_authority: &[&[_]] = 
            &[
            owner_account_info.key.as_ref(),
            mint_info.key.as_ref(),
            ADD_SEED_TOKEN_ACCOUNT_AUTHORITY.as_bytes(),
            &[bump_seed_token_account_authority],
            ];

        let instruction_create_token_account_authority = system_instruction::create_account(
            owner_account_info.key,
            pda_pool_token_account_authority_info.key,
            0,
            0,
            this_program_info.key,
        );

        invoke_signed(
            &instruction_create_token_account_authority,
            &[owner_account_info.clone(), pda_pool_token_account_authority_info.clone(), system_program_info.clone()],
            &[&sign_seeds_pda_token_account_authority],
        )?;

        let minimum_balance_token_acc = rent.minimum_balance(TokenAccount::LEN);

        let (_pda_token_account_pubkey, bump_seed_pda_token_account) = Pubkey::find_program_address(
            &[owner_account_info.key.as_ref(), mint_info.key.as_ref()],
            &this_program_info.key
        );
        let sign_seeds_pda_token_account: &[&[_]] = 
            &[
            owner_account_info.key.as_ref(), 
            mint_info.key.as_ref(),
            &[bump_seed_pda_token_account],
            ];

        let instruction_create_token_account = system_instruction::create_account(
            owner_account_info.key,
            pda_pool_token_account_info.key,
            minimum_balance_token_acc,
            TokenAccount::LEN as u64,
            &spl_token::id(),
        );

        invoke_signed(
            &instruction_create_token_account,
            &[owner_account_info.clone(), pda_pool_token_account_info.clone(), system_program_info.clone()],
            &[&sign_seeds_pda_token_account],
        )?;                                                             

        let instruction_initialize_account = spl_token::instruction::initialize_account(
            &spl_token::id(),
            pda_pool_token_account_info.key,
            mint_info.key,
            pda_pool_token_account_authority_info.key,
        )?;

        invoke_signed(
            &instruction_initialize_account,
            &[
            pda_pool_token_account_info.clone(), 
            mint_info.clone(), 
            pda_pool_token_account_authority_info.clone(),
            rent_info.clone(),
            token_program_info.clone(),
            ],
            &[&sign_seeds_pda_token_account],
        )?;

        let instruction_transfer_tokens = spl_token::instruction::transfer(
            &spl_token::id(),
            token_account_info.key,
            pda_pool_token_account_info.key,
            owner_account_info.key,
            &[owner_account_info.key],
            reward_amount,
        )?;

        invoke(
            &instruction_transfer_tokens,
            &[
            token_account_info.clone(), 
            pda_pool_token_account_info.clone(), 
            owner_account_info.clone(),
            token_program_info.clone(),
            ],
        )?;

        let min_balance_wallet_pool = rent.minimum_balance(USER_INFO_LEN) * 5; 

        let (_pda_wallet_for_create_user_pubkey, bump_seed_wallet_for_create_user) = Pubkey::find_program_address(
            &[owner_account_info.key.as_ref(), mint_info.key.as_ref(), ADD_SEED_WALLET_POOL.as_bytes()],
            &this_program_info.key,
        );
        let sign_seeds_pda_wallet_pool: &[&[_]] = 
            &[
            owner_account_info.key.as_ref(),
            mint_info.key.as_ref(),
            ADD_SEED_WALLET_POOL.as_bytes(),
            &[bump_seed_wallet_for_create_user],
            ];

        let instruction_create_account_for_wallet_pool = system_instruction::create_account(
            owner_account_info.key,
            pda_wallet_for_create_user_info.key,
            min_balance_wallet_pool,
            0,
            system_program_info.key,
        );

        invoke_signed(
            &instruction_create_account_for_wallet_pool,
            &[owner_account_info.clone(), pda_wallet_for_create_user_info.clone(), system_program_info.clone()],
            &[&sign_seeds_pda_wallet_pool],
        )?;

        let min_balance_stake_pool = rent.minimum_balance(STAKE_POOL_LEN);

        let (_pda_state_pool_pubkey, bump_seed_state_pool) = Pubkey::find_program_address(
            &[owner_account_info.key.as_ref(), mint_info.key.as_ref(), ADD_SEED_STATE_POOL.as_bytes()],
            &this_program_info.key,
        );
        let sign_seeds_pda_state_pool: &[&[_]] = 
            &[
            owner_account_info.key.as_ref(), 
            mint_info.key.as_ref(),
            ADD_SEED_STATE_POOL.as_bytes(),
            &[bump_seed_state_pool]
            ];

        let instruction_create_account_for_stake_pool = system_instruction::create_account(
            owner_account_info.key,
            pda_stake_pool_info.key, 
            min_balance_stake_pool,
            STAKE_POOL_LEN as u64,
            this_program_info.key,
        );

        invoke_signed( 
            &instruction_create_account_for_stake_pool, 
            &[owner_account_info.clone(), pda_stake_pool_info.clone(), system_program_info.clone()],
            &[&sign_seeds_pda_state_pool],
        )?;

        assert!(mint.decimals < 21, "Token decimals must be inferior to 21");

        let precision_factor = 10_u64
            .checked_pow(21_u32 - mint.decimals as u32)
            .ok_or(StakingError::Overflow)?;

        let reward_per_block = reward_amount
            .checked_div(
                end_block
                .checked_sub(start_block)
                .ok_or(StakingError::Overflow)?)
            .ok_or(StakingError::Overflow)?;

        let stake_pool = StakePool {
            owner: *owner_account_info.key,
            mint: *mint_info.key,  
            precision_factor,
            is_initialized: 1, 
            pool_name,
            last_reward_block: 0,
            start_block,
            end_block,
            reward_amount,
            reward_per_block,
            accrued_token_per_share: 0,
        };

        let mut vec_of_pools = unpack_from_slice(&pda_list_of_pools_info.data.borrow())
            .expect("Failed to deserialize vec_of_pools");

        vec_of_pools.push(*pda_stake_pool_info.key);

        pack_into_slice(&vec_of_pools, &mut pda_list_of_pools_info.data.borrow_mut());

        stake_pool.serialize(&mut &mut pda_stake_pool_info.data.borrow_mut()[..])
            .expect("Failed to serialize StakePool");
        
        Ok(())
    }

    pub fn process_deposit(
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        
        let owner_token_account_info = next_account_info(account_info_iter)?; // 0
        let token_account_info = next_account_info(account_info_iter)?; // 1
        
        if *token_account_info.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }
        
        let token_account = TokenAccount::unpack_unchecked(
            &token_account_info.data.borrow(),
        )?;
        
        if token_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }
        
        if token_account.amount < amount || amount == 0 {
            return Err(TokenError::InsufficientFunds.into());
        };
        
        let mint_info = next_account_info(account_info_iter)?; // 2
        let pda_stake_pool_info = next_account_info(account_info_iter)?; // 3
        let mut stake_pool = StakePool::try_from_slice(&pda_stake_pool_info.data.borrow())
            .expect("Failed to deserialize StakePool");

        let pda_pool_token_account_authority_info = next_account_info(account_info_iter)?; // 4
        let pda_pool_token_account_info = next_account_info(account_info_iter)?; // 5
        let pda_wallet_for_create_user_info = next_account_info(account_info_iter)?; // 6
        let pda_user_state_info = next_account_info(account_info_iter)?; // 7

        let pda_pool_token_account = TokenAccount::unpack_unchecked( 
            &pda_pool_token_account_info.data.borrow(),
        )?;
 
        if pda_pool_token_account.owner != *pda_pool_token_account_authority_info.key {
            return Err(ProgramError::IllegalOwner);
        }

        let rent_info = next_account_info(account_info_iter)?; // 8
        let rent = &Rent::from_account_info(rent_info)?;

        let clock_program_info = next_account_info(account_info_iter)?; // 9
        let clock = &Clock::from_account_info(clock_program_info)?;

        let system_program_info = next_account_info(account_info_iter)?; // 10
        let token_program_info = next_account_info(account_info_iter)?; // 11
        
        if pda_user_state_info.data_is_empty() {
            msg!("Creating account for UserInfo");

            let (_pda_wallet_pubkey, bump_seed_wallet) = Pubkey::find_program_address(
                &[stake_pool.owner.as_ref(), mint_info.key.as_ref(), ADD_SEED_WALLET_POOL.as_bytes()],
                &this_program_id(), 
            );
    
            let (_pda_user_state_pubkey, bump_seed_user_state) = Pubkey::find_program_address(
                &[pda_stake_pool_info.key.as_ref(), token_account_info.key.as_ref()],
                &this_program_id(),
            );
            
            let signers_seeds_pda_wallet: &[&[_]] = 
                &[
                stake_pool.owner.as_ref(), 
                mint_info.key.as_ref(), 
                ADD_SEED_WALLET_POOL.as_bytes(),
                &[bump_seed_wallet],
                ];
            
            let signers_seeds_pda_user_state: &[&[_]] = 
                &[
                pda_stake_pool_info.key.as_ref(),
                token_account_info.key.as_ref(),
                &[bump_seed_user_state],
                ]; 
            
            let min_balance_user_info = rent.minimum_balance(USER_INFO_LEN);
    
            let instruction_create_account_for_user_info = system_instruction::create_account(
                pda_wallet_for_create_user_info.key, // account "from" for transfer instruction must not carry data
                pda_user_state_info.key,
                min_balance_user_info,
                USER_INFO_LEN as u64,
                &this_program_id(),
            );

            invoke_signed( 
                &instruction_create_account_for_user_info,
                &[pda_wallet_for_create_user_info.clone(), pda_user_state_info.clone(), system_program_info.clone()],
                &[&signers_seeds_pda_wallet, &signers_seeds_pda_user_state],
            )?;
    
            let user_data = UserInfo {
                token_account_id: *token_account_info.key, 
                amount: 0,
                reward_debt: 0,
            };
    
            user_data.serialize(&mut &mut pda_user_state_info.data.borrow_mut()[..])?;
        } 
        
        let instruction_transfer_tokens_to_pool = spl_token::instruction::transfer(
            &spl_token::id(),
            token_account_info.key,
            pda_pool_token_account_info.key,
            owner_token_account_info.key,
            &[owner_token_account_info.key],
            amount,
        )?;

        invoke(
            &instruction_transfer_tokens_to_pool,
            &[
            token_account_info.clone(),
            pda_pool_token_account_info.clone(),
            owner_token_account_info.clone(),
            token_program_info.clone()
            ],
        )?;

        // TODO: make transfer instruction after update_pool
        // TODO: stakers++
        stake_pool.update_pool(
            &pda_pool_token_account,
            clock
        )
        .expect("Unable to update pool");  

        let mut user_data = UserInfo::try_from_slice(&pda_user_state_info.data.borrow())?;  
        let current_amount = user_data.amount;

        user_data.amount = user_data
            .amount
            .checked_add(amount)
            .ok_or(StakingError::Overflow)?;
 
        if current_amount > 0 {
            let pending = Self::get_pending(
                current_amount,
                stake_pool.accrued_token_per_share,
                stake_pool.precision_factor,
                user_data.reward_debt,
            )
            .expect("Unable to get pending value");

            // TODO: Check reward_amount > pending
            if pending > 0 {
                let (_pda_token_acc_auth_pubkey, bump_seed_pda_token_account_authority) = Pubkey::find_program_address(
                    &[stake_pool.owner.as_ref(), mint_info.key.as_ref(), ADD_SEED_TOKEN_ACCOUNT_AUTHORITY.as_bytes()],
                    &this_program_id(),
                );
                let sign_seeds_pda_pool_token_account_authority: &[&[_]] = 
                    &[
                    stake_pool.owner.as_ref(), 
                    mint_info.key.as_ref(),
                    ADD_SEED_TOKEN_ACCOUNT_AUTHORITY.as_bytes(),
                    &[bump_seed_pda_token_account_authority],
                    ];

                let instruction_transfer_pending_tokens = spl_token::instruction::transfer(
                    &spl_token::id(),
                    pda_pool_token_account_info.key,
                    token_account_info.key,
                    pda_pool_token_account_authority_info.key,
                    &[pda_pool_token_account_authority_info.key],
                    pending,
                )?;

                invoke_signed(
                    &instruction_transfer_pending_tokens, 
                    &[
                    pda_pool_token_account_info.clone(),
                    token_account_info.clone(),
                    pda_pool_token_account_authority_info.clone(), 
                    token_program_info.clone(),
                    ],
                    &[&sign_seeds_pda_pool_token_account_authority]
                )?;

                stake_pool.reward_amount = stake_pool 
                    .reward_amount
                    .checked_sub(pending)
                    .ok_or(StakingError::Overflow)?;
            }
        }
        user_data.set_reward_debt(
            (user_data.amount as u128)
            .checked_mul(stake_pool.accrued_token_per_share)
            .ok_or(StakingError::Overflow)?
            .checked_div(stake_pool.precision_factor as u128)
            .ok_or(StakingError::Overflow)? as u64
        );

        user_data.serialize(&mut &mut pda_user_state_info.data.borrow_mut()[..])?;

        msg!("stake_pool after deposit is {:#?}", stake_pool);
        stake_pool.serialize(&mut &mut pda_stake_pool_info.data.borrow_mut()[..])?;

        // debug
        let unpacked_user_data = UserInfo::try_from_slice(&pda_user_state_info.data.borrow())?; 
        msg!("unpacked_user_data is {:#?}", unpacked_user_data);
        // 
        
        Ok(())
    }

    pub fn process_withdraw(
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let token_account_info = next_account_info(account_info_iter)?; // 0
        let pda_stake_pool_info = next_account_info(account_info_iter)?; // 1
        let pda_pool_token_account_authority_info = next_account_info(account_info_iter)?; // 2
        let pda_pool_token_account_info = next_account_info(account_info_iter)?; // 3
        let pda_user_state_info = next_account_info(account_info_iter)?; // 4

        let clock_program_info = next_account_info(account_info_iter)?; // 5
        let clock = &Clock::from_account_info(clock_program_info)?;

        let token_program_info = next_account_info(account_info_iter)?; // 6

        let pda_pool_token_account = TokenAccount::unpack_unchecked( 
            &pda_pool_token_account_info.data.borrow(),
        )?;

        let mut stake_pool = StakePool::try_from_slice(&pda_stake_pool_info.data.borrow_mut())
            .expect("Failed to deserialize StakePool");

        let mut user_data = UserInfo::try_from_slice(&pda_user_state_info.data.borrow_mut())
            .expect("Failed to deserialize UserData");
        
        assert!(user_data.amount >= amount, "Amount to withdraw too high");

        stake_pool.update_pool(
            &pda_pool_token_account,
            &clock,
        )?;

        let current_amount = user_data.amount;

        if amount > 0 {
            user_data.amount = user_data
                .amount
                .checked_sub(amount)
                .ok_or(StakingError::Overflow)?;

            let (_pda_token_acc_auth_pubkey, bump_seed_pda_token_account_authority) = Pubkey::find_program_address(
                &[stake_pool.owner.as_ref(), stake_pool.mint.as_ref(), ADD_SEED_TOKEN_ACCOUNT_AUTHORITY.as_bytes()],
                &this_program_id(),
            );

            let sign_seeds_pda_pool_token_account_authority: &[&[_]] =
                &[
                stake_pool.owner.as_ref(),
                stake_pool.mint.as_ref(),
                ADD_SEED_TOKEN_ACCOUNT_AUTHORITY.as_bytes(),
                &[bump_seed_pda_token_account_authority],
                ];
            
            let instruction_transfer_tokens = spl_token::instruction::transfer(
                &spl_token::id(),
                pda_pool_token_account_info.key,
                token_account_info.key,
                pda_pool_token_account_authority_info.key,
                &[pda_pool_token_account_authority_info.key],
                amount,
            )?;

            invoke_signed(
                &instruction_transfer_tokens,
                &[
                pda_pool_token_account_info.clone(),
                token_account_info.clone(),
                pda_pool_token_account_authority_info.clone(),
                token_program_info.clone(),
                ],
                &[&sign_seeds_pda_pool_token_account_authority]
            )?;
        }

        let pending = Self::get_pending(
            current_amount,
            stake_pool.accrued_token_per_share,
            stake_pool.precision_factor,
            user_data.reward_debt,
        )
        .expect("Unable to get pending value");
        
        // TODO: Check reward_amount > pending
        if pending > 0 {
            let (_pda_token_acc_auth_pubkey, bump_seed_pda_token_account_authority) = Pubkey::find_program_address(
                &[stake_pool.owner.as_ref(), stake_pool.mint.as_ref(), ADD_SEED_TOKEN_ACCOUNT_AUTHORITY.as_bytes()],
                &this_program_id(),
            );
            let sign_seeds_pda_pool_token_account_authority: &[&[_]] = 
                &[
                stake_pool.owner.as_ref(), 
                stake_pool.mint.as_ref(),
                ADD_SEED_TOKEN_ACCOUNT_AUTHORITY.as_bytes(),
                &[bump_seed_pda_token_account_authority],
                ];

            let instruction_transfer_pending_tokens = spl_token::instruction::transfer(
                &spl_token::id(),
                pda_pool_token_account_info.key,
                token_account_info.key,
                pda_pool_token_account_authority_info.key,
                &[pda_pool_token_account_authority_info.key],
                pending,
            )?;

            invoke_signed(
                &instruction_transfer_pending_tokens, 
                &[
                pda_pool_token_account_info.clone(),
                token_account_info.clone(),
                pda_pool_token_account_authority_info.clone(), 
                token_program_info.clone(),
                ],
                &[&sign_seeds_pda_pool_token_account_authority]
            )?;
        }

        user_data.set_reward_debt(
            (user_data.amount as u128)
            .checked_mul(stake_pool.accrued_token_per_share)
            .ok_or(StakingError::Overflow)?
            .checked_div(stake_pool.precision_factor as u128)
            .ok_or(StakingError::Overflow)? as u64
        );

        user_data.serialize(&mut &mut pda_user_state_info.data.borrow_mut()[..])?;

        stake_pool.reward_amount = stake_pool 
                .reward_amount
                .checked_sub(pending)
                .ok_or(StakingError::Overflow)?;

        msg!("stake_pool after deposit is {:#?}", stake_pool);
        stake_pool.serialize(&mut &mut pda_stake_pool_info.data.borrow_mut()[..])?;

        // debug
        let unpacked_user_data = UserInfo::try_from_slice(&pda_user_state_info.data.borrow())?; 
        msg!("unpacked_user_data is {:#?}", unpacked_user_data);
        // 

        // TODO: Need to delete UserInfo, but it can't possible.
        // TODO: stakers--; 
        
        Ok(())
    }

    pub fn process_emergency_withdraw(
        accounts: &[AccountInfo]
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let token_account_info = next_account_info(account_info_iter)?; // 0 
        let pda_stake_pool_info = next_account_info(account_info_iter)?; // 1
        let pda_pool_token_account_authority_info = next_account_info(account_info_iter)?; // 2
        let pda_pool_token_account_info = next_account_info(account_info_iter)?; // 3
        let pda_user_state_info = next_account_info(account_info_iter)?; // 4
        let token_program_info = next_account_info(account_info_iter)?; // 5

        let pda_pool_token_account = TokenAccount::unpack_unchecked( 
            &pda_pool_token_account_info.data.borrow(),
        )?;

        let stake_pool = StakePool::try_from_slice(&pda_stake_pool_info.data.borrow())
            .expect("Failed to deserialize StakePool");

        let mut user_data = UserInfo::try_from_slice(&pda_user_state_info.data.borrow_mut())
            .expect("Failed to deserialize UserData");

        let amount_to_transfer = user_data.amount;

        // TODO: Stakers--;
        if amount_to_transfer > 0 {
            let (_pda_token_acc_auth_pubkey, bump_seed_pda_token_account_authority) = Pubkey::find_program_address(
                &[stake_pool.owner.as_ref(), stake_pool.mint.as_ref(), ADD_SEED_TOKEN_ACCOUNT_AUTHORITY.as_bytes()],
                &this_program_id(),
            );
            let sign_seeds_pda_pool_token_account_authority: &[&[_]] = 
                &[
                stake_pool.owner.as_ref(), 
                stake_pool.mint.as_ref(),
                ADD_SEED_TOKEN_ACCOUNT_AUTHORITY.as_bytes(),
                &[bump_seed_pda_token_account_authority],
                ];

            let instruction_transfer_pending_tokens = spl_token::instruction::transfer(
                &spl_token::id(),
                pda_pool_token_account_info.key,
                token_account_info.key,
                pda_pool_token_account_authority_info.key,
                &[pda_pool_token_account_authority_info.key],
                amount_to_transfer,
            )?;

            invoke_signed(
                &instruction_transfer_pending_tokens, 
                &[
                pda_pool_token_account_info.clone(),
                token_account_info.clone(),
                pda_pool_token_account_authority_info.clone(), 
                token_program_info.clone(),
                ],
                &[&sign_seeds_pda_pool_token_account_authority]
            )?;

            //TODO: user_data.amount -= amount
        }

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

        if *this_program_info.key != this_program_id(){
            return Err(ProgramError::IncorrectProgramId);
        }

        let rent_info = next_account_info(account_info_iter)?; // 3
        let rent = &Rent::from_account_info(rent_info)?;

        let system_program_info = next_account_info(account_info_iter)?; // 4

        msg!("Creating the account for vec of pools");

        let min_balance = rent.minimum_balance(VEC_STATE_SPACE);
        let signers_seeds_list_pda: &[&[_]] = &[ADD_SEED_LIST_OF_POOLS.as_bytes(), &[BUMP_SEED_LIST_OF_POOLS]];

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

    fn validate_pda_vec_of_pools(
        vec_of_pools_info: &AccountInfo,
    ) -> ProgramResult {
        let list_of_pools_key = Pubkey::create_program_address(
            &[ADD_SEED_LIST_OF_POOLS.as_bytes(), &[BUMP_SEED_LIST_OF_POOLS]],
            &this_program_id(),
        )?;

        if *vec_of_pools_info.key != list_of_pools_key {
            return Err(ProgramError::IncorrectProgramId);
        }

        Ok(())
    }

    fn get_pending(
        current_amount: u64,
        accrued_token_per_share: u128,
        precision_factor: u64,
        reward_debt: u64,
    ) -> Result<u64, StakingError> {
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
}