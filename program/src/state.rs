use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
    program_memory::sol_memcpy,
    program_pack::{
       IsInitialized,
       Pack,
       Sealed,
    },
 };
use arrayref::{
   array_ref,
   array_refs,
   array_mut_ref,
   mut_array_refs,
};
use std::{
   str::FromStr,
   collections::BTreeMap,
   error::Error,
};
use borsh::{
   BorshDeserialize,
   BorshSerialize,
   BorshSchema,
};
use crate::{
   error::StakingError,
};

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
 
pub const STAKE_POOL_LEN: usize = 64;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, BorshSchema)]
pub struct StakePool {
   pub pool_owner: Pubkey,
   pub is_initialized: u8,
   pub pool_name: [u8; 31],
}
 
impl Sealed for StakePool {}

impl IsInitialized for StakePool {
   fn is_initialized(&self) -> bool {
      self.is_initialized != 0
   }
}
/* 
impl Sealed for StakePool {}

impl IsInitialized for StakePool {
   fn is_initialized(&self) -> bool {
      self.is_initialized != 0
   }
}

impl Pack for StakePool {
   const LEN: usize = 40; 

   fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
      let src = array_ref![src, 0, 40];
      let (pool_owner, is_initialized) = array_refs![src, 32, 8];

      Ok(
         StakePool {
            pool_owner: Pubkey::new_from_array(*pool_owner),
            is_initialized: u64::from_le_bytes(*is_initialized),
         }
      ) 
   }

   fn pack_into_slice(&self, dst: &mut [u8]) {
      let dst = array_mut_ref![dst, 0, 40];
      let (pool_owner_dst, is_initialized_dst) = mut_array_refs![dst, 32, 8];

      let &StakePool {
         ref pool_owner,
         is_initialized, 
      } = self;

      pool_owner_dst.copy_from_slice(pool_owner.as_ref());
      *is_initialized_dst = is_initialized.to_le_bytes();
   }
}
*/

pub const USER_INFO_LEN: usize = 48;

#[derive(Debug, Copy, Clone, BorshSerialize, BorshDeserialize, BorshSchema)]
pub struct UserInfo {
   pub token_account_id: Pubkey,
   pub amount: u64,
   pub reward_debt: u64,
}

/* 
impl Default for UserInfo {
   fn default() -> Self {
      UserInfo {
         user_id: Pubkey::new_unique(), 
         amount: 0, 
         reward_debt: 0, 
      }
   }
}
 
impl Sealed for UserInfo {}

impl IsInitialized for UserInfo {
   fn is_initialized(&self) -> bool {
       true
   }
}

impl Pack for UserInfo {
   const LEN: usize = 48;

   fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
      let src = array_ref![src, 0, 48];
      let (
         user_id,
         amount,
         reward_debt,
      ) = array_refs![src, 32, 8, 8];

      Ok( 
         UserInfo {
            user_id: Pubkey::new_from_array(*user_id),
            amount: u64::from_le_bytes(*amount),
            reward_debt: u64::from_le_bytes(*reward_debt),
         }         
      )
   }

   fn pack_into_slice(&self, dst: &mut [u8]) {
      let dst = array_mut_ref![dst, 0, 48];
      let (
         user_id_dst,
         amount_dst,
         reward_debt_dst,
      ) = mut_array_refs![dst, 32, 8, 8];

      let &UserInfo {
         ref user_id,
         amount,
         reward_debt,
      } = self;

      user_id_dst.copy_from_slice(user_id.as_ref());
      *amount_dst = amount.to_le_bytes();
      *reward_debt_dst = reward_debt.to_le_bytes();
   }
}
*/