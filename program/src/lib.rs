pub mod processor;
pub mod instruction;
pub mod state;
pub mod error;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

solana_program::declare_id!("4tsfpuJ34UA4QmM9ukbXC8ChEDJLGe9eVyseQyG6bNfX");
