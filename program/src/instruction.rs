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
    /// Token Geyser
    /// A smart-contract based mechanism to distribute tokens over time, inspired loosely by
    /// Compound and Uniswap.
    ///
    /// Distribution tokens are added to a locked pool in the contract and become unlocked over time
    /// according to a once-configurable unlock schedule. Once unlocked, they are available to be
    /// claimed by users.
    ///
    /// A user may deposit tokens to accrue ownership share over the unlocked pool. This owner share
    /// is a function of the number of tokens deposited as well as the length of time deposited.
    /// Specifically, a user's share of the currently-unlocked pool equals their "deposit-seconds"
    /// divided by the global "deposit-seconds". This aligns the new token distribution with long
    /// term supporters of the project, addressing one of the major drawbacks of simple airdrops.
    ///
    /// More background and motivation available at:
    /// https://github.com/ampleforth/RFCs/blob/master/RFCs/rfc-1.md
    /// 
    /// 
    /// Accounts excepted: 
    /// 0. '[signer]' owner of the token-account with reward. Initializer
    /// 1. '[]' token-account with tokens for reward. Tokens will be relocated to the pool token-account
    /// 2. '[]' mint-account 
    /// 3. '[writable]' token-account for the pool. Should be created prior to this instruction and 
    ///                 owned by this program
    /// 4. '[]' this program
    /// 5. '[]' token
    /// 6. '[]' rent
    /// 7. '[]' system-program 
    /// 8. '[]' token-program
    Initialize {
        amount_reward: u64,
        pool_name: String,
        bump_seed: u8,
    },
}
