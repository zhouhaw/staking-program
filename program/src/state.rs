use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
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
 
 #[derive(Debug)]
pub struct StakePool {
   pub pool_owner: Pubkey,
   pub is_initialized: u16,
}
 
impl Sealed for StakePool {}

impl IsInitialized for StakePool {
   fn is_initialized(&self) -> bool {
       self.is_initialized != 0
   }
}

impl Pack for StakePool {
   const LEN: usize = 34; // actually 49184

   fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
       let src = array_ref![src, 0, 34];
       let (pool_owner, is_initialized) = array_refs![src, 32, 2];

       Ok(
          StakePool {
             pool_owner: Pubkey::new_from_array(*pool_owner),
             is_initialized: u16::from_le_bytes(*is_initialized),
          }
       )
   }

   fn pack_into_slice(&self, dst: &mut [u8]) {
      let dst = array_mut_ref![dst, 0, 34];
      let (pool_owner_dst, is_initialized_dst) = mut_array_refs![dst, 32, 2];

      let &StakePool {
         ref pool_owner,
         is_initialized,
      } = self;

      pool_owner_dst.copy_from_slice(pool_owner.as_ref());
      *is_initialized_dst = is_initialized.to_le_bytes();
   }
}

#[derive(Debug, Copy, Clone)]
pub struct UserInfo {
   pub user_id: Pubkey,
   pub amount: u64,
   pub reward_debt: u64,
}

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