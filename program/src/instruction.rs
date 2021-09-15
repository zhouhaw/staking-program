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
    /// 1. '[writable]' PDA for state StakePool. Pubkey should be created prior to this instruction
    /// 2. '[writable]' PDA for wallet stake pool. Pubkey should be created prior to this instruction
    /// 3. '[writable]' PDA for vec of pools
    /// 4. '[]' this program
    /// 5. '[]' token mint
    /// 6. '[]' rent
    /// 7. '[]' system-program 
    /// 8. '[]' token-program
    /// 9. '[writable]' token-account with tokens for reward. Tokens will be relocated to the pool token-account
    /// 10. '[writable]' PDA token-account for the pool. Pubkey should be created prior to this instruction 
    Initialize {
        n_reward_tokens: u64, // Number of reward tokens
        amount_reward: u64,
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
    /// 3. '[]' PDA state pool
    /// 4. '[writable]' PDA token-account for the pool
    /// 5. '[writable]' PDA wallet stake pool. Should be created prior to this instruction
    /// 6. '[writable]' PDA for state UserInfo. Pubkey should be created prior to this instruction
    /// 7. '[]' this program
    /// 8. '[]' rent
    /// 9. '[]' clock
    /// 10. '[]' system-program
    /// 11. '[]' token-program
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
