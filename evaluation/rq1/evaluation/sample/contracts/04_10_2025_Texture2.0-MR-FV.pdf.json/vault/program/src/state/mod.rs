pub mod last_update;
pub mod position;
pub mod reward;
pub mod vault;

pub const POSITION_DISCRIMINATOR: &[u8; 8] = b"POSITION";
pub const VAULT_DISCRIMINATOR: &[u8; 8] = b"VAULT___";
