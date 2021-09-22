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
    /// 0. '[writable, signer]' owner of the token-account with reward. Initializer
    /// 1. '[writable]' PDA for state StakePool
    /// 2. '[writable]' PDA for wallet stake pool
    /// 3. '[writable]' PDA for vec of pools
    /// 4. '[]' this program
    /// 5. '[]' token mint
    /// 6. '[]' rent
    /// 7. '[]' system-program 
    /// 8. '[]' token-program
    /// 9. '[writable]' token-account with tokens for reward. Tokens will be relocated to the pool token-account
    /// 10. '[writable]' PDA authority for the token-account 
    /// 11. '[writable]' PDA token-account for the stake pool
    Initialize {
        n_reward_tokens: u64, // Number of reward tokens
        reward_amount: u64,
        pool_name: [u8; 31],
        start_block: u64,
        end_block: u64,
    },
    /// Deposit staked tokens and collect reward tokens (if any)
    ///
    /// Accounts expected:
    ///
    /// 0. '[signer]' owner of the token-account with deposit
    /// 1. '[writable]' token-account with tokens for deposit. Tokens will be relocated to the PDA token-account
    /// 2. '[]' token mint for staked token
    /// 3. '[writable]' PDA for state StakePool. Should be created prior to this instruction
    /// 4. '[]' PDA authority for the token-account. Should be created prior to this instruction
    /// 5. '[writable]' PDA token-account for the pool. Should be created prior to this instruction
    /// 6. '[writable]' PDA wallet stake pool. Should be created prior to this instruction
    /// 7. '[writable]' PDA for state UserInfo
    /// 8. '[]' rent
    /// 9. '[]' clock
    /// 10. '[]' system-program
    /// 11. '[]' token-program
    Deposit {
        amount: u64,
    },
    /// Withdraw staked tokens and collect reward tokens 
    ///
    /// Accounts expected:
    ///
    /// 0. '[writable]' token-account for staked tokens
    /// 1. '[writable]' PDA for state StakePool. Should be created prior to this instruction
    /// 2. '[]' PDA authority for the token-account. Should be created prior to this instruction
    /// 3. '[writable]' PDA token-account for the pool. Should be created prior to this instruction
    /// 4. '[writable]' PDA for state UserInfo. Should be created prior to this instruction
    /// 5. '[]' clock
    /// 6. '[]' token-program
    Withdraw {
        amount: u64,
    },
    /// Withdraw staked tokens without caring about rewards 
    ///
    /// Accounts expected:
    ///
    /// 0. '[writable]' token-account for staked tokens
    /// 1. '[]' PDA for state StakePool. Should be created prior to this instruction
    /// 2. '[]' PDA authority for the token-account. Should be created prior to this instruction
    /// 3. '[wirtable]' PDA token-account for the pool. Should be created prior to this instruction
    /// 4. '[writable]' PDA for state UserInfo. Should be created prior to this instruction 
    /// 5. '[]' token-program
    EmergencyWithdraw,
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
