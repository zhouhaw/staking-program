pub mod processor;
pub mod instruction;
pub mod state;
pub mod error;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

pub const ADD_SEED_LIST_OF_POOLS: &str = "LIST_OF_POOLS_test_2"; 
pub const BUMP_SEED_LIST_OF_POOLS: u8 = 254; 

pub const ADD_SEED_TOKEN_ACCOUNT_AUTHORITY: &str = "TOKEN_ACCOUNT_AUTHORITY";

pub const ADD_SEED_STATE_POOL: &str = "STATE_POOL";
pub const ADD_SEED_WALLET_POOL: &str = "WALLET_POOL"; // PDA with SOL for creating PDA UserInfo

solana_program::declare_id!("3TFhUrwaAdkraCgaapGcqYZfA9agmCMZjSNc1zQBfvnc"); 