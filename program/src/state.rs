use solana_program::{
   program_pack::{
      IsInitialized,
      Sealed,
      Pack,
   },
   program_option::COption,
   program_error::ProgramError,
   entrypoint::ProgramResult,
   pubkey::Pubkey,
   clock::Clock,
   msg,
};
use derivative::*;
use spl_token::state::Account as TokenAccount;
use arrayref::{
   array_refs,
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
use crate::processor::get_precision_factor;

#[derive(Derivative, Clone, Copy)]
#[derivative(Debug)]
pub struct StakePool {
   pub n_reward_tokens: u8, 
   pub owner: Pubkey, 
   pub mint: Pubkey, 
   pub is_initialized: u8, 
   pub precision_factor_rank: u8,
   pub bonus_multiplier: COption<u8>, 
   pub bonus_start_block: COption<u64>, 
   pub bonus_end_block: COption<u64>,
   pub last_reward_block: u64, 
   pub start_block: u64,
   pub end_block: u64,
   pub reward_amount: u64,
   pub reward_per_block: u64,
   pub accrued_token_per_share: u128, 
   #[derivative(Debug="ignore")]
   pub pool_name: [u8; 32],
   #[derivative(Debug="ignore")]
   pub project_link: [u8; 128],
   #[derivative(Debug="ignore")]
   pub theme_id: u8,
}
 
impl Sealed for StakePool {}
impl IsInitialized for StakePool {
   fn is_initialized(&self) -> bool {
      self.is_initialized != 0
   }
}
impl Pack for StakePool {
   const LEN: usize = 313;
   fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
      let src = array_ref![src, 0, 313];
      let (
         n_reward_tokens,
         owner, 
         mint, 
         is_initialized, 
         precision_factor_rank,
         bonus_multiplier,
         bonus_start_block,
         bonus_end_block,
         last_reward_block,
         start_block,
         end_block,
         reward_amount,
         reward_per_block,
         accrued_token_per_share,
         pool_name,
         project_link,
         theme_id,
      ) = array_refs![src, 1, 32, 32, 1, 1, 5, 12, 12, 8, 8, 8, 8, 8, 16, 32, 128, 1];
      Ok(StakePool {
         n_reward_tokens: u8::from_le_bytes(*n_reward_tokens),
         owner: Pubkey::new_from_array(*owner),
         mint: Pubkey::new_from_array(*mint),
         is_initialized: u8::from_le_bytes(*is_initialized),
         precision_factor_rank: u8::from_le_bytes(*precision_factor_rank),
         bonus_multiplier: unpack_coption_u8(bonus_multiplier)?,
         bonus_start_block: unpack_coption_u64(bonus_start_block)?,
         bonus_end_block: unpack_coption_u64(bonus_end_block)?,
         last_reward_block: u64::from_le_bytes(*last_reward_block),
         start_block: u64::from_le_bytes(*start_block),
         end_block: u64::from_le_bytes(*end_block),
         reward_amount: u64::from_le_bytes(*reward_amount),
         reward_per_block: u64::from_le_bytes(*reward_per_block),
         accrued_token_per_share: u128::from_le_bytes(*accrued_token_per_share),
         pool_name: *pool_name,
         project_link: *project_link,
         theme_id: u8::from_le_bytes(*theme_id),
      })
   }
   fn pack_into_slice(&self, dst: &mut [u8]) {
       let dst = array_mut_ref![dst, 0, 313];
       let (
         n_reward_tokens_dst,
         owner_dst, 
         mint_dst, 
         is_initialized_dst, 
         precision_factor_rank_dst,
         bonus_multiplier_dst,
         bonus_start_block_dst,
         bonus_end_block_dst,
         last_reward_block_dst,
         start_block_dst,
         end_block_dst,
         reward_amount_dst,
         reward_per_block_dst,
         accrued_token_per_share_dst,
         pool_name_dst,
         project_link_dst,
         theme_id_dst,
      ) = mut_array_refs![dst, 1, 32, 32, 1, 1, 5, 12, 12, 8, 8, 8, 8, 8, 16, 32, 128, 1];
      let &StakePool {
         n_reward_tokens,
         ref owner,
         ref mint,
         is_initialized,
         precision_factor_rank,
         ref bonus_multiplier,
         ref bonus_start_block,
         ref bonus_end_block,
         last_reward_block,
         start_block,
         end_block,
         reward_amount,
         reward_per_block,
         accrued_token_per_share,
         pool_name,
         project_link,
         theme_id,
      } = self;
      *n_reward_tokens_dst = n_reward_tokens.to_le_bytes();
      owner_dst.copy_from_slice(owner.as_ref());
      mint_dst.copy_from_slice(mint.as_ref());
      *is_initialized_dst = is_initialized.to_le_bytes();
      *precision_factor_rank_dst = precision_factor_rank.to_le_bytes();
      pack_coption_u8(bonus_multiplier, bonus_multiplier_dst);
      pack_coption_u64(bonus_start_block, bonus_start_block_dst);
      pack_coption_u64(bonus_end_block, bonus_end_block_dst);
      *last_reward_block_dst = last_reward_block.to_le_bytes();
      *start_block_dst = start_block.to_le_bytes();
      *end_block_dst = end_block.to_le_bytes();
      *reward_amount_dst = reward_amount.to_le_bytes();
      *reward_per_block_dst = reward_per_block.to_le_bytes();
      *accrued_token_per_share_dst = accrued_token_per_share.to_le_bytes();
      pool_name_dst.copy_from_slice(&pool_name);
      project_link_dst.copy_from_slice(&project_link);
      *theme_id_dst = theme_id.to_le_bytes();
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

      let precision_factor = crate::processor::get_precision_factor(
         self.precision_factor_rank,
      )?;

      self.accrued_token_per_share = self
         .accrued_token_per_share
         .checked_add(
            (reward as u128)
            .checked_mul(precision_factor as u128)
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

      if let COption::Some(v) = self.bonus_end_block {
         if v != 0 && current_block > v {
            self.bonus_start_block = COption::None;
            self.bonus_end_block = COption::None;
            self.set_bonus_multiplier(1);
         }
      }

      Ok(())
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

      let multiplier: u64 = self.bonus_multiplier.unwrap().into();
      let start = match self.bonus_start_block {
         COption::Some(v) => v,
         COption::None => 0,
      };
      let end = match self.bonus_end_block {
         COption::Some(v) => v,
         COption::None => 0,
      };

      if from < start && to > end {
         return start - from + to - end + (end - start) * multiplier; 
      }
      else if from < start && to > start {
         return start - from + (to - start) * multiplier;
      }
      else if from < end && to > end {
         return to - end + (end - from) * multiplier;
      }
      else if from >= start && to <= end {
         return (to - from) * multiplier;
      }
      else {
         return to - from;
      }
   }

   fn set_last_reward_block(
      &mut self,
      block: u64,
   ) {
      self.last_reward_block = block;
   }

   pub fn set_end_block(
      &mut self,
      block: u64,
   ) {
      self.end_block = block;
   }

   pub fn set_bonus_multiplier(
      &mut self,
      multiplier: u8,
   ) {
      self.bonus_multiplier = COption::Some(multiplier);
   }

   pub fn set_bonus_start_block(
      &mut self,
      block: u64,
   ) {
      self.bonus_start_block = COption::Some(block);
   }

   pub fn set_bonus_end_block(
      &mut self,
      block: u64,
   ) {
      self.bonus_end_block = COption::Some(block);
   }

   pub fn set_reward_amount(
      &mut self,
      amount: u64,
   ) {
      self.reward_amount = amount;
   }

   pub fn update_project_info(
      &mut self,
      pool_name: [u8; 32],
      project_link: [u8; 128],
      theme_id: u8,
   ) {
      self.pool_name = pool_name;
      self.project_link = project_link;
      self.theme_id = theme_id;
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

fn unpack_coption_u8(src: &[u8; 5]) -> Result<COption<u8>, ProgramError> {
   let (tag, body) = array_refs![src, 4, 1];
   match *tag {
      [0, 0, 0, 0] => Ok(COption::None),
      [1, 0, 0, 0] => Ok(COption::Some(u8::from_le_bytes(*body))),
      _ => Err(ProgramError::InvalidAccountData),
   }
}
fn pack_coption_u8(src: &COption<u8>, dst: &mut [u8; 5]) {
   let (tag, body) = mut_array_refs![dst, 4, 1];
   match src {
      COption::Some(amount) => {
         *tag = [1, 0, 0, 0];
         *body = amount.to_le_bytes();
      }
      COption::None => {
         *tag = [0; 4];
      }
   }
}

fn unpack_coption_u64(src: &[u8; 12]) -> Result<COption<u64>, ProgramError> {
   let (tag, body) = array_refs![src, 4, 8];
   match *tag {
      [0, 0, 0, 0] => Ok(COption::None),
      [1, 0, 0, 0] => Ok(COption::Some(u64::from_le_bytes(*body))),
      _ => Err(ProgramError::InvalidAccountData),
   }
}
fn pack_coption_u64(src: &COption<u64>, dst: &mut [u8; 12]) {
   let (tag, body) = mut_array_refs![dst, 4, 8];
   match src {
      COption::Some(amount) => {
         *tag = [1, 0, 0, 0];
         *body = amount.to_le_bytes();
      }
      COption::None => {
         *tag = [0; 4];
      }
   }
}