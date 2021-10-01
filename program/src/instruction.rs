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
    /// 1. '[writable]' PDA master-staking
    /// 2. '[writable]' PDA for state StakePool
    /// 3. '[writable]' PDA for wallet stake pool
    /// 4. '[]' this program
    /// 5. '[]' token mint
    /// 6. '[]' rent
    /// 7. '[]' system-program 
    /// 8. '[]' token-program
    /// 9. '[writable]' token-account with tokens for reward. Tokens will be relocated to the pool token-account
    /// 10. '[writable]' PDA authority for the token-account 
    /// 11. '[writable]' PDA token-account for the staked tokens
    /// 12. '[writable]' PDA token-account for the reward tokens
    Initialize {
        n_reward_tokens: u8, // Number of reward tokens
        reward_amount: u64,
        start_block: u64,
        end_block: u64,
        pool_name: [u8; 32],
        project_link: [u8; 128],
        theme_id: u8,
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
    /// 5. '[writable]' PDA token-account for staked tokens. Should be created prior to this instruction
    /// 6. '[writable]' PDA token-account for reward tokens. Should be created prior to this instruction 
    /// 7. '[writable]' PDA wallet stake pool. Should be created prior to this instruction
    /// 8. '[writable]' PDA for state UserInfo
    /// 9. '[]' rent
    /// 10. '[]' clock
    /// 11. '[]' system-program
    /// 12. '[]' token-program
    Deposit {
        amount: u64,
    },
    /// Withdraw staked tokens and collect reward tokens 
    ///
    /// Accounts expected:
    ///
    /// 0. '[signer]' owner of the token-account.
    /// 1. '[writable]' token-account for staked tokens
    /// 2. '[writable]' PDA for state StakePool. Should be created prior to this instruction
    /// 3. '[]' PDA authority for the token-account. Should be created prior to this instruction
    /// 4. '[writable]' PDA token-account for staked tokens. Should be created prior to this instruction
    /// 5. '[writable]' PDA token-account for reward tokens. Should be created prior to this instruction
    /// 6. '[writable]' PDA for state UserInfo. Should be created prior to this instruction
    /// 7. '[]' clock
    /// 8. '[]' token-program
    Withdraw {
        amount: u64,
    },
    /// Withdraw staked tokens without caring about rewards 
    ///
    /// Accounts expected:
    ///
    /// 0. '[signer]' owner of the token-account
    /// 1. '[writable]' token-account for staked tokens
    /// 2. '[]' PDA authority for the token-account. Should be created prior to this instruction
    /// 3. '[wirtable]' PDA token-account for staked tokens. Should be created prior to this instruction
    /// 4. '[writable]' PDA for state UserInfo. Should be created prior to this instruction 
    /// 5. '[]' PDA for state StakePool. Should be created prior to this instruction
    /// 6. '[]' token-program
    EmergencyWithdraw,
    /// Update project info
    ///
    /// Accounts expected:
    ///
    /// 0. '[signer]' Pool owner
    /// 1. '[]' mint of the reward token
    /// 2. '[writable]' PDA for state StakePool. Shoud be created prior to this instruction
    UpdateProjectInfo {
        pool_name: [u8; 32],
        project_link: [u8; 128],
        theme_id: u8,
    },
    /// Set bonus time
    ///
    /// Accounts expected:
    ///
    /// 0. '[signer]' Pool owner
    /// 1. '[]' mint of the reward token 
    /// 2. '[writable]' PDA for state StakePool. Should be created prior to this instruction
    /// 3. '[]' PDA token-account for staked tokens. Should be created prior to this instruction
    /// 4. '[]' clock
    SetBonusTime {
        bonus_multiplier: u8,
        bonus_start_block: u64,
        bonus_end_block: u64,
    },
    /// Change time of end pool
    ///
    /// Accounts expected:
    ///
    /// 0. '[signer]' owner of the token-account with reward. Pool owner
    /// 1. '[]' mint of the reward token
    /// 2. '[writable]' PDA for state StakePool. Should be created prior to this instruction
    /// 3. '[]' clock
    /// 4. '[]' token-program
    /// 5. '[writable]' token-account with reward
    /// 6. '[writable]' PDA token-account for reward
    UpdateEndBlock {
        end_block: u64,
    },
    /// Initialize a PDA for vec of pools
    ///
    /// Accounts expected:
    ///
    /// 0. '[signer]' payer
    /// 1. '[writable]' PDA token-account authority 
    /// 2. '[writable]' PDA master-staking
    /// 3. '[]' this program
    /// 4. '[]' rent
    /// 5. '[]' system-program
    CreateMasterAndAuthority,
}
