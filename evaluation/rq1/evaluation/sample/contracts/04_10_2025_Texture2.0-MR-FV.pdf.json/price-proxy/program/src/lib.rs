pub mod error;
pub mod instruction;
#[cfg(feature = "with-processor")]
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

solana_program::declare_id!("priceEvKXX3KERsitDpmvujXfPFYesmEspw4kiC3ryF");

pub type PriceProxyResult<T> = std::result::Result<T, error::PriceProxyError>;
