use solana_program::{
   program_pack::{
      IsInitialized,
      Sealed,
   },
   entrypoint::ProgramResult,
   pubkey::Pubkey,
   clock::Clock,
   msg,
};
use spl_token::state::Account as TokenAccount;
use arrayref::{
   array_ref,
   array_mut_ref,
   mut_array_refs,
};
use std::error::Error;
use borsh::{
   BorshDeserialize,
   BorshSerialize,
   BorshSchema,
};
use crate::error::StakingError;

pub const VEC_LEN: usize = 4;
pub const VEC_STORAGE: usize = 160;
pub const VEC_STATE_SPACE: usize = VEC_LEN + VEC_STORAGE;

pub fn unpack_from_slice(src: &[u8]) -> Result<Vec<Pubkey>, Box<dyn Error>> {
   let src = array_ref![src, 0, VEC_STATE_SPACE];
   let data_len_src = array_ref![src, 0, VEC_LEN];

   let data_len = u32::from_le_bytes(*data_len_src) as usize;
   let data_len_bytes = data_len * 32;

   if data_len == 0 {
      Ok(Vec::<Pubkey>::new())
   } else {
      let data_dser = Vec::<Pubkey>::try_from_slice(&src[0..data_len_bytes + 4]).unwrap();
      Ok(data_dser)
   }
}

pub fn pack_into_slice(
   vec: &Vec<Pubkey>,
   dst: &mut [u8],
) {
   let dst = array_mut_ref![dst, 0, VEC_STATE_SPACE];
   let (len_dst, data_dst) = mut_array_refs![dst, VEC_LEN, VEC_STORAGE];

   let data_len = vec.len();
   let data_len_bytes = data_len * 32;
   
   len_dst[..].copy_from_slice(&(data_len as u32).to_le_bytes());

   if data_len_bytes <= VEC_STORAGE {
      let mut iter = vec.iter();
      for i in 0..data_len {
         let temp_pubkey = iter.next().unwrap();
         data_dst[i*32..i*32+32].copy_from_slice(temp_pubkey.as_ref());
      }
   } else {
      panic!();
   }
}
 
pub const STAKE_POOL_LEN: usize = 160;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, BorshSchema)]
pub struct StakePool {
   pub owner: Pubkey,
   pub mint: Pubkey,
   pub precision_factor: u64,
   pub is_initialized: u8,
   pub pool_name: [u8; 31],
   pub last_reward_block: u64,
   pub start_block: u64,
   pub end_block: u64,
   pub reward_amount: u64,
   pub reward_per_block: u64,
   pub accrued_token_per_share: u128,
}
 
impl Sealed for StakePool {}

impl IsInitialized for StakePool {
   fn is_initialized(&self) -> bool {
      self.is_initialized != 0
   }
}

impl StakePool {
   pub fn update_pool(
      &mut self,
      pda_pool_token_account: &TokenAccount,
      clock: &Clock, 
   ) -> ProgramResult {
      let current_block = clock.slot;
      if current_block <= self.last_reward_block {
         return Ok(());
      }

      let staked_token_supply = pda_pool_token_account
         .amount
         .checked_sub(self.reward_amount)
         .ok_or(StakingError::StakedTokenSupplyOverflow)?;

      if staked_token_supply == 0 {
         self.set_last_reward_block(current_block);
   
         return Ok(());
      }

      let multiplier = self.get_multiplier(self.last_reward_block, current_block);

      let reward = multiplier
         .checked_mul(self.reward_per_block)
         .ok_or(StakingError::RewardOverflow)?;

      self.accrued_token_per_share = self
         .accrued_token_per_share
         .checked_add(
            (reward as u128)
            .checked_mul(self.precision_factor as u128)
            .ok_or(StakingError::RewardMulPrecisionOverflow)?
            .checked_div(staked_token_supply as u128)
            .ok_or(StakingError::RewardMulPrecisionDivSupplyOverflow)?)
         .ok_or(StakingError::AccuredTokenPerShareOverflow)?;

      //debug
      msg!(
         "multiplier: {}\n
         reward: {}\n
         staked_token_supply: {}\n,
         accrued_toked: {}\n",
         multiplier,
         reward,
         staked_token_supply,
         self.accrued_token_per_share,
      );
      //

      if self.end_block > current_block {
         self.set_last_reward_block(current_block);
      } 
      else {
         self.set_last_reward_block(self.end_block);
      }

      Ok(())
      
      // TODO: add bonus block condition
   }

   fn get_multiplier(
      &self,
      mut from: u64,
      mut to: u64,
   ) -> u64 {
      if from < self.start_block {
         from = self.start_block;
      }
      if self.end_block < to {
         to = self.end_block;
      }

      // TODO: add bonus logic

      return to - from;
   }

   fn set_last_reward_block(
      &mut self,
      block: u64,
   ) {
      self.last_reward_block = block;
   }
}

pub const USER_INFO_LEN: usize = 48;

#[derive(Debug, Copy, Clone, BorshSerialize, BorshDeserialize, BorshSchema)]
pub struct UserInfo {
   pub token_account_id: Pubkey,
   pub amount: u64,
   pub reward_debt: u64,
}

impl UserInfo {
   pub fn set_reward_debt(
      &mut self,
      value: u64,
   ) {
      self.reward_debt = value;
   }
}