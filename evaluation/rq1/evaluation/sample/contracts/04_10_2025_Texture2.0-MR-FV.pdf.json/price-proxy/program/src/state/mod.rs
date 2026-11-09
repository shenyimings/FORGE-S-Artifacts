pub use texture_common::account as texture_account;

pub mod price_feed;

pub mod stake_pool;
pub mod utils;

pub const PRICE_FEED_DISCRIMINATOR: &[u8; 8] = b"PRICEEED";
