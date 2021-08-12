use solana_program::{
    account_info::{
        next_account_info,
        AccountInfo
    },
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};
use borsh::{
    BorshDeserialize,
    BorshSerialize,
    BorshSchema,
};
use crate::instruction::StakingInstruction;

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
        amount_reward: u16,
        pool_name: [u8; 32],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let token_account_info = next_account_info(account_info_iter)?;
        let token_info = next_account_info(account_info_iter)?;

        msg!(
            "Token account {} has {} tokens\n
            Args: amount_reward: {}, pool_name: {:?}",
            token_account_info.key,
            token_account_info.lamports(),
            amount_reward,
            pool_name,
        );

        Ok(())
    }
}