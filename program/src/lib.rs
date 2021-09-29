pub mod processor;
pub mod instruction;
pub mod state;
pub mod error;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

pub const ADD_SEED_TOKEN_ACCOUNT_AUTHORITY: &str = "TOKEN_ACCOUNT_AUTHORITY_test_4"; 
pub const BUMP_SEED_TOKEN_ACCOUNT_AUTHORITY: u8 = 251;

pub const ADD_SEED_STATE_POOL: &str = "STATE_POOL";
pub const ADD_SEED_WALLET_POOL: &str = "WALLET_POOL"; // PDA with SOL for creating PDA UserInfo

solana_program::declare_id!("3TFhUrwaAdkraCgaapGcqYZfA9agmCMZjSNc1zQBfvnc"); 