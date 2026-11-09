pub mod distributor;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

pub mod pda;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

use std::time::Duration;

pub use vault_id::ID as VAULT_ID;
mod vault_id {
    solana_program::declare_id!("Vau3BGKpqWi24LHwVQnVvd31ATGLQ3YmpUJ1DYTdgQs");
}

pub type VaultResult<T> = std::result::Result<T, error::VaultError>;

pub const MIN_FEE_CLAIM_INTERVAL_SEC: Duration = Duration::from_secs(7 * 24 * 60 * 60); // a week
