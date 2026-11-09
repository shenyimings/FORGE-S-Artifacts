pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

solana_program::declare_id!("CurvyNqr6HXwWUHk7MfDk7rqL3a4Kodkz8BkdBhep7ed");

pub type CurvyResult<T> = std::result::Result<T, error::CurvyError>;
