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
    program_option::COption,
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
        UserInfo,
        USER_INFO_LEN,
    },
    error::StakingError, 
    instruction::StakingInstruction,
    id as this_program_id,
    ADD_SEED_LIST_OF_POOLS,
    BUMP_SEED_LIST_OF_POOLS,
    ADD_SEED_TOKEN_ACCOUNT_AUTHORITY,
    BUMP_SEED_TOKEN_ACCOUNT_AUTHORITY,
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
                start_block,
                end_block,
                pool_name, 
                project_link,
                theme_id,
            } => {
                msg!("Instruction: Initialize stake pool");
                Self::process_initialize(
                    accounts,
                    n_reward_tokens,
                    reward_amount,
                    start_block,
                    end_block,
                    pool_name,
                    project_link,
                    theme_id,
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
                msg!("Instruction: Emergency Withdraw");
                Self::process_emergency_withdraw(
                    accounts,
                )
            },
            StakingInstruction::UpdateProjectInfo {
                pool_name,
                project_link,
                theme_id,
            }
            => {
                msg!("Instruction: Update Project Info");
                Self::process_update_project_info(
                    accounts,
                    pool_name,
                    project_link,
                    theme_id,
                )
            }
            StakingInstruction::SetBonusTime{
                bonus_multiplier,
                bonus_start_block,
                bonus_end_block,
            } => {
                msg!("Instruction: Set Bonus Time");
                Self::process_set_bonus_time(
                    accounts,
                    bonus_multiplier,
                    bonus_start_block,
                    bonus_end_block,
                )
            }
            StakingInstruction::UpdateEndBlock{
                end_block,
            } => {
                msg!("Instruction: Update End Block");
                Self::process_update_end_block(
                    accounts,
                    end_block,
                )
            }
            StakingInstruction::CreateAuthority
            => {
                msg!("Instruction: Create authority");
                Self::process_create_authority(
                    accounts,
                )
            },
        }
    }

    fn process_initialize(
        accounts: &[AccountInfo],
        n_reward_tokens: u8,
        reward_amount: u64,
        start_block: u64,
        end_block: u64,
        pool_name: [u8; 32],
        project_link: [u8; 128],
        theme_id: u8,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let owner_account_info = next_account_info(account_info_iter)?; // 0
        if !owner_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        
        let pda_stake_pool_info = next_account_info(account_info_iter)?; // 1
        let pda_wallet_for_create_user_info = next_account_info(account_info_iter)?; // 2

        let this_program_info = next_account_info(account_info_iter)?; // 3
        if *this_program_info.key != this_program_id(){
            return Err(ProgramError::IncorrectProgramId);
        }

        let mint_info = next_account_info(account_info_iter)?; // 4
        let mint = TokenMint::unpack_unchecked(&mint_info.data.borrow())?;

        let rent_info = next_account_info(account_info_iter)?; // 5
        let rent = &Rent::from_account_info(rent_info)?; 

        let system_program_info = next_account_info(account_info_iter)?; // 6
        let token_program_info = next_account_info(account_info_iter)?; // 7
        let token_account_info = next_account_info(account_info_iter)?; // 8

        if *token_account_info.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let token_account = TokenAccount::unpack_unchecked(
            &token_account_info.data.borrow(),
        )?;

        if token_account.owner != *owner_account_info.key {
            return Err(TokenError::OwnerMismatch.into());
        }

        let pda_pool_token_account_authority_info = next_account_info(account_info_iter)?; // 9

        // TODO: Add validate for token-account
        let pda_pool_token_account_info = next_account_info(account_info_iter)?; // 10

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

        let min_balance_stake_pool = rent.minimum_balance(StakePool::LEN);

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
            StakePool::LEN as u64,
            this_program_info.key,
        );

        invoke_signed( 
            &instruction_create_account_for_stake_pool, 
            &[owner_account_info.clone(), pda_stake_pool_info.clone(), system_program_info.clone()],
            &[&sign_seeds_pda_state_pool],
        )?;

        assert!(mint.decimals < 21, "Token decimals must be inferior to 21");

        let precision_factor_rank = 21_u8
            .checked_sub(mint.decimals as u8)
            .ok_or(StakingError::Overflow)?;

        let reward_per_block = reward_amount
            .checked_div(
                end_block
                .checked_sub(start_block)
                .ok_or(StakingError::Overflow)?)
            .ok_or(StakingError::Overflow)?;

        let stake_pool = StakePool {
            n_reward_tokens,
            owner: *owner_account_info.key,
            mint: *mint_info.key,  
            is_initialized: 1, 
            precision_factor_rank,
            bonus_multiplier: COption::Some(1),
            bonus_start_block: COption::None,
            bonus_end_block: COption::None,
            last_reward_block: 0,
            start_block,
            end_block,
            reward_amount,
            reward_per_block,
            accrued_token_per_share: 0,
            pool_name,
            project_link,
            theme_id,
        };

        StakePool::pack(stake_pool, &mut pda_stake_pool_info.data.borrow_mut())
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
        
        let mint_info = next_account_info(account_info_iter)?; // 2
        let pda_stake_pool_info = next_account_info(account_info_iter)?; // 3
        let mut stake_pool = StakePool::unpack(&pda_stake_pool_info.data.borrow_mut())
            .expect("Failed to deserialie StakePool");

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
        // TODO: add loop
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
            let pending = get_pending(
                current_amount,
                stake_pool.accrued_token_per_share,
                stake_pool.precision_factor_rank,
                user_data.reward_debt,
            )
            .expect("Unable to get pending value");

            // TODO: Check reward_amount > pending
            if pending > 0 {
                let sign_seeds_pda_pool_token_account_authority: &[&[_]] = 
                    &[
                    ADD_SEED_TOKEN_ACCOUNT_AUTHORITY.as_bytes(),
                    &[BUMP_SEED_TOKEN_ACCOUNT_AUTHORITY],
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
            get_reward_debt(
                user_data.amount,
                stake_pool.accrued_token_per_share,
                stake_pool.precision_factor_rank,
            )?
        );

        user_data.serialize(&mut &mut pda_user_state_info.data.borrow_mut()[..])?;

        msg!("stake_pool after deposit is {:#?}", stake_pool);
        StakePool::pack(stake_pool, &mut pda_stake_pool_info.data.borrow_mut())?;

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

        let owner_info = next_account_info(account_info_iter)?; // 0
        let token_account_info = next_account_info(account_info_iter)?; // 1

        let token_account = TokenAccount::unpack_unchecked(
            &token_account_info.data.borrow(),
        )?;

        if token_account.owner != *owner_info.key {
            return Err(TokenError::OwnerMismatch.into());
        }
        if !owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let pda_stake_pool_info = next_account_info(account_info_iter)?; // 2
        let pda_pool_token_account_authority_info = next_account_info(account_info_iter)?; // 3
        let pda_pool_token_account_info = next_account_info(account_info_iter)?; // 4
        let pda_user_state_info = next_account_info(account_info_iter)?; // 5

        let clock_program_info = next_account_info(account_info_iter)?; // 6
        let clock = &Clock::from_account_info(clock_program_info)?;

        let token_program_info = next_account_info(account_info_iter)?; // 7

        let pda_pool_token_account = TokenAccount::unpack_unchecked( 
            &pda_pool_token_account_info.data.borrow(),
        )?;

        let sign_seeds_pda_pool_token_account_authority: &[&[_]] = 
            &[
            ADD_SEED_TOKEN_ACCOUNT_AUTHORITY.as_bytes(),
            &[BUMP_SEED_TOKEN_ACCOUNT_AUTHORITY],
            ];

        let mut stake_pool = StakePool::unpack(&pda_stake_pool_info.data.borrow_mut())
            .expect("Failed to deserialie StakePool");

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

        let pending = get_pending(
            current_amount,
            stake_pool.accrued_token_per_share,
            stake_pool.precision_factor_rank,
            user_data.reward_debt,
        )
        .expect("Unable to get pending value");
        
        // TODO: Check reward_amount > pending
        // TODO: add loop for reward tokens
            if pending > 0 {

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
                get_reward_debt(
                    user_data.amount,
                    stake_pool.accrued_token_per_share,
                    stake_pool.precision_factor_rank,
                )?
            );

        user_data.serialize(&mut &mut pda_user_state_info.data.borrow_mut()[..])?;

        stake_pool.reward_amount = stake_pool 
                .reward_amount
                .checked_sub(pending)
                .ok_or(StakingError::Overflow)?;

        msg!("stake_pool after deposit is {:#?}", stake_pool);
        StakePool::pack(stake_pool, &mut pda_stake_pool_info.data.borrow_mut())?;

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

        let owner_info = next_account_info(account_info_iter)?; // 0
        if !owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let token_account_info = next_account_info(account_info_iter)?; // 1
        let token_account = TokenAccount::unpack_unchecked(
            &token_account_info.data.borrow(),
        )?;
        if token_account.owner != *owner_info.key {
            return Err(TokenError::OwnerMismatch.into());
        }

        let pda_stake_pool_info = next_account_info(account_info_iter)?; // 2
        let pda_pool_token_account_authority_info = next_account_info(account_info_iter)?; // 3
        let pda_pool_token_account_info = next_account_info(account_info_iter)?; // 4
        let pda_user_state_info = next_account_info(account_info_iter)?; // 5
        let token_program_info = next_account_info(account_info_iter)?; // 6

        let pda_pool_token_account = TokenAccount::unpack_unchecked( 
            &pda_pool_token_account_info.data.borrow(),
        )?;

        // TODO: maybe need check for reward amount
        // TODO: maybe stake_pool isn't needed
        let stake_pool = StakePool::unpack(&pda_stake_pool_info.data.borrow())
            .expect("Failed to deserialie StakePool");

        let mut user_data = UserInfo::try_from_slice(&pda_user_state_info.data.borrow_mut())
            .expect("Failed to deserialize UserData");

        let amount_to_transfer = user_data.amount;

        // TODO: Stakers--;
        if amount_to_transfer > 0 {
            user_data.amount = user_data
                .amount
                .checked_sub(amount_to_transfer)
                .ok_or(StakingError::Overflow)?;

            let sign_seeds_pda_pool_token_account_authority: &[&[_]] = 
                &[
                ADD_SEED_TOKEN_ACCOUNT_AUTHORITY.as_bytes(),
                &[BUMP_SEED_TOKEN_ACCOUNT_AUTHORITY],
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
        }

        //debug
        msg!("user_data after emergency-withdraw is {:#?}", user_data);
        //
        user_data.serialize(&mut &mut pda_user_state_info.data.borrow_mut()[..])?;

        Ok(())
    }

    pub fn process_update_project_info(
        accounts: &[AccountInfo],
        pool_name: [u8; 32],
        project_link: [u8; 128],
        theme_id: u8,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let pool_owner_info = next_account_info(account_info_iter)?; // 0
        if !pool_owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mint_info = next_account_info(account_info_iter)?; // 1
        let pda_stake_pool_info = next_account_info(account_info_iter)?; // 2
        let mut stake_pool = StakePool::unpack(&pda_stake_pool_info.data.borrow_mut())
            .expect("Failed to deserialie StakePool");

        validate_stake_pool(
            &stake_pool,
            pool_owner_info.key,
            mint_info.key,
        )?;

        stake_pool.update_project_info(
            pool_name,
            project_link,
            theme_id,
        );

        msg!("stake_pool after update_project_info is {:#?}", stake_pool);
        StakePool::pack(stake_pool, &mut pda_stake_pool_info.data.borrow_mut())?;        

        Ok(())
    }

    pub fn process_set_bonus_time(
        accounts: &[AccountInfo],
        bonus_multiplier: u8,
        bonus_start_block: u64,
        bonus_end_block: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let pool_owner_info = next_account_info(account_info_iter)?; // 0
        if !pool_owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mint_info = next_account_info(account_info_iter)?; // 1
        let pda_stake_pool_info = next_account_info(account_info_iter)?; // 2 
        let mut stake_pool = StakePool::unpack(&pda_stake_pool_info.data.borrow_mut())
            .expect("Failed to deserialie StakePool");

        validate_stake_pool(
            &stake_pool,
            pool_owner_info.key,
            mint_info.key,
        )?;

        let pda_pool_token_account_info = next_account_info(account_info_iter)?; // 3
        let pda_pool_token_account = TokenAccount::unpack_unchecked(
            &pda_pool_token_account_info.data.borrow(),
        )?;

        let pda_pool_token_account_authority_pubkey = Pubkey::create_program_address(
            &[ADD_SEED_TOKEN_ACCOUNT_AUTHORITY.as_bytes(), &[BUMP_SEED_TOKEN_ACCOUNT_AUTHORITY]],
            &this_program_id(),
        )?;
        if pda_pool_token_account.owner != pda_pool_token_account_authority_pubkey {
            return Err(TokenError::OwnerMismatch.into());
        }

        let clock_info = next_account_info(account_info_iter)?; // 4
        let clock = &Clock::from_account_info(clock_info)?;

        assert!(bonus_start_block < bonus_end_block);
        assert!(bonus_start_block >= stake_pool.start_block, "Cant set early than start time");

        stake_pool.update_pool(
            &pda_pool_token_account,
            &clock,
        )?;

        assert!(stake_pool.bonus_end_block == COption::None, "Can't start another Bonus time");

        let end_block = stake_pool.end_block
            .checked_sub(
                (bonus_end_block - bonus_start_block) * (bonus_multiplier as u64 - 1))
            .ok_or(StakingError::Overflow)?;

        assert!(end_block > clock.slot && end_block > stake_pool.start_block, "Not enough rewards for Bonus");

        if end_block < bonus_end_block {
            stake_pool.set_bonus_end_block(end_block);
        }
        else {
            stake_pool.set_bonus_end_block(bonus_end_block);
        }
        stake_pool.set_bonus_multiplier(bonus_multiplier);
        stake_pool.set_bonus_start_block(bonus_start_block);
        stake_pool.set_end_block(end_block);

        msg!("stake_pool after set_bonus_time is {:#?}", stake_pool);
        StakePool::pack(stake_pool, &mut pda_stake_pool_info.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_update_end_block(
        accounts: &[AccountInfo],
        end_block: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let pool_owner_info = next_account_info(account_info_iter)?; // 0
        let mint_info = next_account_info(account_info_iter)?; // 1
        let pda_stake_pool_info = next_account_info(account_info_iter)?; // 2
        
        let clock_info = next_account_info(account_info_iter)?; // 3
        let clock = &Clock::from_account_info(clock_info)?;

        let token_program_info = next_account_info(account_info_iter)?; // 4

        let reward_token_account_info = next_account_info(account_info_iter)?; // 5
        let pda_pool_token_account_info = next_account_info(account_info_iter)?; // 6

        let mut stake_pool = StakePool::unpack(&pda_stake_pool_info.data.borrow_mut())
            .expect("Failed to deserialie StakePool");

        validate_stake_pool(
            &stake_pool,
            pool_owner_info.key,
            mint_info.key,
        )?;

        let current_block = clock.slot;

        assert!(stake_pool.end_block > current_block, "Pool already finished");
        assert!(end_block > stake_pool.end_block, "Cannot shorten");

        let blocks_added = end_block - stake_pool.end_block;

        // TODO: add loop for reward tokens
            let to_transfer = blocks_added * stake_pool.reward_per_block;

            let instruction_transfer_tokens = spl_token::instruction::transfer(
                &spl_token::id(),
                reward_token_account_info.key,
                pda_pool_token_account_info.key,
                pool_owner_info.key,
                &[pool_owner_info.key],
                to_transfer,
            )?;

            invoke(
                &instruction_transfer_tokens,
                &[
                reward_token_account_info.clone(),
                pda_pool_token_account_info.clone(),
                pool_owner_info.clone(),
                token_program_info.clone(),
                ],
            )?;

        stake_pool.set_reward_amount(
            stake_pool
            .reward_amount
            .checked_add(to_transfer)
            .ok_or(StakingError::Overflow)?
        );
        stake_pool.set_end_block(end_block);

        //debug
        msg!("StakePool after instruction is \n{:#?}", stake_pool); 
        //
        StakePool::pack(stake_pool, &mut pda_stake_pool_info.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_create_authority( 
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let payer_info = next_account_info(account_info_iter)?; // 0

        let pda_pool_token_account_authority_info = next_account_info(account_info_iter)?; // 1

        let this_program_info = next_account_info(account_info_iter)?; // 2
        if *this_program_info.key != this_program_id(){
            return Err(ProgramError::IncorrectProgramId);
        }

        let rent_info = next_account_info(account_info_iter)?; // 3
        let rent = &Rent::from_account_info(rent_info)?;

        let system_program_info = next_account_info(account_info_iter)?; // 4

        let sign_seeds_pda_token_account_authority: &[&[_]] = 
            &[
            ADD_SEED_TOKEN_ACCOUNT_AUTHORITY.as_bytes(),
            &[BUMP_SEED_TOKEN_ACCOUNT_AUTHORITY],
            ];

        let instruction_create_token_account_authority = system_instruction::create_account(
            payer_info.key,
            pda_pool_token_account_authority_info.key,
            0,
            0,
            this_program_info.key,
        );

        invoke_signed(
            &instruction_create_token_account_authority,
            &[payer_info.clone(), pda_pool_token_account_authority_info.clone(), system_program_info.clone()],
            &[&sign_seeds_pda_token_account_authority],
        )?;
 
        Ok(())
    }
}

pub fn validate_stake_pool(
    stake_pool: &StakePool,
    owner_key: &Pubkey,
    mint_key: &Pubkey,
) -> ProgramResult {
    if stake_pool.owner != *owner_key || 
       stake_pool.mint != *mint_key {
            return Err(StakingError::StakePoolMissmatch.into());
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