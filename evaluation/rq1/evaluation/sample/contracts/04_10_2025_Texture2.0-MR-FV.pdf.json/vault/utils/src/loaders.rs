use anyhow::{Error, Result};
use std::collections::HashMap;
use vault::state::position::Position;
use vault::state::vault::Vault;
use vault::state::{POSITION_DISCRIMINATOR, VAULT_DISCRIMINATOR};
use vault::VAULT_ID;

use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{Memcmp, RpcFilterType};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;

use texture_common::account::PodAccount;

pub async fn load_vaults(rpc: &RpcClient) -> Result<HashMap<Pubkey, Vault>> {
    let filter = RpcFilterType::Memcmp(Memcmp::new_raw_bytes(0, VAULT_DISCRIMINATOR.to_vec()));

    let account_config = RpcAccountInfoConfig {
        encoding: Some(UiAccountEncoding::Base64),
        data_slice: None,
        commitment: Some(CommitmentConfig::confirmed()),
        min_context_slot: None,
    };

    let config = RpcProgramAccountsConfig {
        filters: Some(vec![filter]),
        account_config,
        with_context: None,
    };

    let accounts = rpc
        .get_program_accounts_with_config(&VAULT_ID, config)
        .await?;

    let mut vaults = HashMap::new();
    for (key, account) in &accounts {
        match Vault::try_from_bytes(&account.data) {
            Ok(vault) => {
                vaults.insert(*key, *vault);
            }
            Err(err) => {
                return Err(Error::from(err));
            }
        }
    }

    Ok(vaults)
}

pub async fn load_positions(rpc: &RpcClient) -> Result<HashMap<Pubkey, Position>> {
    let filter = RpcFilterType::Memcmp(Memcmp::new_raw_bytes(0, POSITION_DISCRIMINATOR.to_vec()));

    let account_config = RpcAccountInfoConfig {
        encoding: Some(UiAccountEncoding::Base64),
        data_slice: None,
        commitment: Some(CommitmentConfig::confirmed()),
        min_context_slot: None,
    };

    let config = RpcProgramAccountsConfig {
        filters: Some(vec![filter]),
        account_config,
        with_context: None,
    };

    let accounts = rpc
        .get_program_accounts_with_config(&VAULT_ID, config)
        .await?;

    let mut positions = HashMap::new();
    for (key, account) in &accounts {
        match Position::try_from_bytes(&account.data) {
            Ok(position) => {
                positions.insert(*key, *position);
            }
            Err(err) => {
                return Err(Error::from(err));
            }
        }
    }

    Ok(positions)
}

pub async fn load_vault(rpc: &RpcClient, vault: Pubkey) -> Result<Vault> {
    let account = rpc.get_account(&vault).await?;

    let vault = Vault::try_from_bytes(&account.data).copied()?;
    Ok(vault)
}

pub async fn load_position(rpc: &RpcClient, position: Pubkey) -> Result<Position> {
    let account = rpc.get_account(&position).await?;

    let position = Position::try_from_bytes(&account.data).copied()?;
    Ok(position)
}
