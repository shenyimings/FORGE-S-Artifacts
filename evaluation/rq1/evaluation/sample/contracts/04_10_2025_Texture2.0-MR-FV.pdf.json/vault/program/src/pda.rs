use solana_program::pubkey::Pubkey;

use crate::VAULT_ID;

pub const LP_STORAGE_TOKEN_SEED: &[u8] = b"LP_STORAGE";
pub const POSITION_SEED: &[u8] = b"POSITION";

pub const LIQUIDITY_BUFFER_SEED: &[u8] = b"LIQUIDITY_BUFFER";

pub const AUTHORITY_SEED: &[u8] = b"AUTHORITY";

pub const REWARD_SUPPLY_SEED: &[u8] = b"REWARD_SUPPLY";

/// Account for storing LP tokens received from underlying pools
pub fn find_vault_lp_token_storage(vault: &Pubkey, lp_token_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            vault.as_ref(),
            lp_token_mint.as_ref(),
            LP_STORAGE_TOKEN_SEED,
        ],
        &VAULT_ID,
    )
}

/// Temporary storage of incoming liquidity
pub fn find_vault_liquidity_buffer(vault: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[vault.as_ref(), LIQUIDITY_BUFFER_SEED], &VAULT_ID)
}

/// Program authority is the owner of SPL Token mints used to mint VLP tokens.
pub fn find_program_authority() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[AUTHORITY_SEED], &VAULT_ID)
}

/// Reward token supply - is the wallet where Rewards tokens stored by the contract.
/// Reward wallets are global for the Vault.
pub fn find_reward_supply(vault: &Pubkey, reward_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[vault.as_ref(), reward_mint.as_ref(), REWARD_SUPPLY_SEED],
        &VAULT_ID,
    )
}

/// Program (contract) authority used in rewards operations (e.g. to transfer rewards to users).
pub fn find_rewards_program_authority(vault: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[vault.as_ref(), AUTHORITY_SEED], &VAULT_ID)
}

/// Finds user's position address. There could be just one position per Vault
pub fn find_position(vault: &Pubkey, owner: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[vault.as_ref(), owner.as_ref(), POSITION_SEED], &VAULT_ID)
}
