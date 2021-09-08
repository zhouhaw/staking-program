use solana_program::{
    instruction::{
        AccountMeta,
        Instruction,
    },
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::convert::TryInto;
use borsh::{
    BorshSerialize,     
    BorshDeserialize,
    BorshSchema,
};

#[derive(BorshSchema, BorshSerialize, BorshDeserialize)]
pub enum StakingInstruction {
    /// Intitializes a new pool 
    ///
    /// Accounts expected: 
    ///
    /// 0. '[signer]' owner of the token-account with reward. Initializer
    /// 1. '[writable]' token-account with tokens for reward. Tokens will be relocated to the pool token-account
    /// 2. '[writable]' PDA token-account for the pool. Should be created prior to this instruction 
    /// 3. '[writable]' PDA for state StakePool. Should be created prior to this instruction
    /// 4. '[writable]' PDA for vec of pools
    /// 5. '[]' this program
    /// 6. '[]' token
    /// 7. '[]' rent
    /// 8. '[]' system-program 
    /// 9. '[]' token-program
    Initialize {
        amount_reward: u64,
        pool_name: [u8; 31],
        bump_seed: [u8; 2],
    },
    /// Deposit staked tokens and collect reward tokens (if any)
    ///
    /// Accounts expected:
    ///
    /// 0. '[signer]' owner of the token-account with deposit
    /// 1. '[writable]' token-account with tokens for deposit. Tokens will be relocated to the pool token-account
    /// 2. '[]' PDA state pool
    /// 3. '[writable]' PDA token-account for the pool
    /// 4. '[writable]' PDA for state UserInfo. Should be created prior to this instruction
    /// 5. '[]' this program
    /// 6. '[]' rent
    /// 7. '[]' system-program
    /// 8. '[]' token-program
    Deposit {
        amount: u64,
    },
    /// Initialize a PDA for vec of pools
    ///
    /// Accounts expected:
    ///
    /// 0. '[signer]' payer
    /// 1. '[writable]' PDA for vec of pools
    /// 2. '[]' this program
    /// 3. '[]' rent
    /// 4. '[]' system-program
    CreateVecOfPools,
}
