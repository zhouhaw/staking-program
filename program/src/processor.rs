use solana_program::{
    account_info::{
        next_account_info,
        AccountInfo
    },
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

use crate::instruction::StakingInstruction;

pub struct Processor;
impl Processor {
    pun fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[8]
    ) -> ProgramResult{
        let instruction = StakingInstruction::unpack(instruction_data)?;

        match instruction {
            StakingInstruction::Stake { amount } => {
                msg!("Instruction: Stake");
                Self::process_stake(accounts, amount, program_id)
            }
        }
    }

    fn process_stake(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {

    }
}