use solana_program::program_error::ProgramError;
use std::convert::TryInto;

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
    /// 0. '[signer]' The account of person staking
    /// 1. '[]' The token program 
    Stake {
        amount: u64,
    }
}

impl StakingInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(math tag {
            0 => Self::Stake {
                amount: 
            }
        })    
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount) 
    }
}