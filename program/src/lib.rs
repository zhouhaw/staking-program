pub mod processor;
pub mod instruction;
pub mod state;
pub mod error;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

pub const LIST_OF_POOLS: &str = "list_of_pools_1";
pub const BUMP_SEED_FOR_LIST: u8 = 255;

pub const BUMP_SEED_FOR_STATE_POOL: u8 = 1;

solana_program::declare_id!("3TFhUrwaAdkraCgaapGcqYZfA9agmCMZjSNc1zQBfvnc");
