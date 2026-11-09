#![cfg(feature = "test-bpf")]

use bytemuck::Zeroable;
use std::str::FromStr;

use price_proxy::state::utils::str_to_array;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use super_lendy::pda::find_lp_token_mint;
use texture_common::account::PodAccount;
use tracing::info;

use vault::pda::{find_vault_liquidity_buffer, find_vault_lp_token_storage};
use vault::state::vault::{DistributionItem, Vault, VaultParams, VAULT_DISTRIBUTION_MAX_ITEMS};

use crate::utils::setup::{setup_env, EnvKeys};
use crate::utils::vault_executor::create_vault;
use crate::utils::{
    add_curve_acc, add_price_feed_acc, admin_keypair, get_account, init_program_test,
    init_token_accounts, texture_config_keypair, Runner, LAMPORTS,
};

pub mod utils;

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#6f436900c2b04f848bf38980ce4f2372
#[tokio::test]
async fn create_success() {
    let mut runner = init_program_test();

    let admin_keypair = admin_keypair();
    let texture_config_keypair = texture_config_keypair();
    let pool_authority_keypair = Keypair::new();
    let texture_owner_keypair = Keypair::new();

    runner.add_native_wallet(admin_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(texture_owner_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(pool_authority_keypair.pubkey(), LAMPORTS);

    // 1 SOL = 100 USD
    add_price_feed_acc(&mut runner, "sol-usd").await;
    // 1 USDC = 1 USD
    add_price_feed_acc(&mut runner, "usdc-usd").await;
    // 1 JUP = 0.7 USD
    add_price_feed_acc(&mut runner, "jup-usd").await;
    // 1 BONK = 0.00001 USD
    add_price_feed_acc(&mut runner, "bonk-usd").await;

    add_curve_acc(&mut runner, "const-40-pct-acc").await;

    let liquidity_sol_mint =
        Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let liquidity_usdc_mint =
        Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let liquidity_jup_mint =
        Pubkey::from_str("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN").unwrap();
    let liquidity_bonk_mint =
        Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap();
    init_token_accounts(&mut runner, &liquidity_sol_mint);
    init_token_accounts(&mut runner, &liquidity_usdc_mint);
    init_token_accounts(&mut runner, &liquidity_jup_mint);
    init_token_accounts(&mut runner, &liquidity_bonk_mint);

    let mut ctx = runner.start_with_context().await;

    let env_keys = setup_env(
        &mut ctx,
        &admin_keypair,
        &texture_owner_keypair,
        &texture_config_keypair,
        &pool_authority_keypair,
    )
    .await;

    let EnvKeys {
        main_pool_reserves: (_, main_pool_reserve_usdc_keypair),
        risky1_pool_reserves: (_, risky1_pool_reserve_usdc_keypair),
        risky2_pool_reserves: (_, risky2_pool_reserve_usdc_keypair),
        vault_keypair,
        curator_keypair,
        vaults_authority_keypair,
        ..
    } = env_keys;

    // CREATE VAULT

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];
    let lp_token_mints = reserves
        .clone()
        .iter()
        .map(|reserve| find_lp_token_mint(reserve).0)
        .collect::<Vec<_>>();
    let lp_token_storages = lp_token_mints
        .clone()
        .iter()
        .map(|lp_mint| find_vault_lp_token_storage(&vault_keypair.pubkey(), lp_mint).0)
        .collect::<Vec<_>>();

    let mut distribution_items = reserves
        .clone()
        .into_iter()
        .map(|reserve| DistributionItem {
            reserve,
            share_bps: 3300,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    distribution_items[2].share_bps = 3400;
    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    for (idx, item) in distribution_items.iter().enumerate() {
        distribution_bps[idx] = item.share_bps();
    }
    let params = VaultParams {
        name: str_to_array("Vault"),
        description: str_to_array("Vault description"),
        lut: Default::default(),
        distribution_manager: Pubkey::new_unique(),
        distribution_bps,
        periodic_fee_rate_bps: 0,
    };

    macro_rules! validate {
        ($this:ident, $params:ident) => {
            assert_eq!($this.name, $params.name);
            assert_eq!($this.description, $params.description);
            assert_eq!($this.lut, $params.lut);
            assert_eq!($this.distribution_manager, $params.distribution_manager);
        };
    }

    info!("create vault");
    let liquidity_buffer = find_vault_liquidity_buffer(&vault_keypair.pubkey()).0;
    create_vault(
        &mut ctx,
        &vault_keypair,
        curator_keypair.pubkey(),
        params.clone(),
        &vaults_authority_keypair,
        liquidity_usdc_mint,
        liquidity_buffer,
        reserves,
        lp_token_mints,
        lp_token_storages,
    )
    .await
    .expect("create_vault");

    let vault_acc = get_account(&mut ctx.banks_client, vault_keypair.pubkey())
        .await
        .expect("get pool");
    let vault = Vault::try_from_bytes(&vault_acc.data).expect("cast vault data");
    validate!(vault, params);
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#6d6823e8c4b64c75927040537ddeeb6d
#[tokio::test]
async fn create_with_incorrect_authority() {
    let mut runner = init_program_test();

    let admin_keypair = admin_keypair();
    let texture_config_keypair = texture_config_keypair();
    let pool_authority_keypair = Keypair::new();
    let texture_owner_keypair = Keypair::new();

    runner.add_native_wallet(admin_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(texture_owner_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(pool_authority_keypair.pubkey(), LAMPORTS);

    // 1 SOL = 100 USD
    add_price_feed_acc(&mut runner, "sol-usd").await;
    // 1 USDC = 1 USD
    add_price_feed_acc(&mut runner, "usdc-usd").await;
    // 1 JUP = 0.7 USD
    add_price_feed_acc(&mut runner, "jup-usd").await;
    // 1 BONK = 0.00001 USD
    add_price_feed_acc(&mut runner, "bonk-usd").await;

    add_curve_acc(&mut runner, "const-40-pct-acc").await;

    let liquidity_sol_mint =
        Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let liquidity_usdc_mint =
        Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let liquidity_jup_mint =
        Pubkey::from_str("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN").unwrap();
    let liquidity_bonk_mint =
        Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap();
    init_token_accounts(&mut runner, &liquidity_sol_mint);
    init_token_accounts(&mut runner, &liquidity_usdc_mint);
    init_token_accounts(&mut runner, &liquidity_jup_mint);
    init_token_accounts(&mut runner, &liquidity_bonk_mint);

    let mut ctx = runner.start_with_context().await;

    let env_keys = setup_env(
        &mut ctx,
        &admin_keypair,
        &texture_owner_keypair,
        &texture_config_keypair,
        &pool_authority_keypair,
    )
    .await;

    let EnvKeys {
        main_pool_reserves: (_, main_pool_reserve_usdc_keypair),
        risky1_pool_reserves: (_, risky1_pool_reserve_usdc_keypair),
        risky2_pool_reserves: (_, risky2_pool_reserve_usdc_keypair),
        vault_keypair,
        curator_keypair,
        vaults_authority_keypair: _,
        ..
    } = env_keys;

    // CREATE VAULT WITH INCORRECT VAULTS AUTHORITY

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];
    let lp_token_mints = reserves
        .clone()
        .iter()
        .map(|reserve| find_lp_token_mint(reserve).0)
        .collect::<Vec<_>>();
    let lp_token_storages = lp_token_mints
        .clone()
        .iter()
        .map(|lp_mint| find_vault_lp_token_storage(&vault_keypair.pubkey(), lp_mint).0)
        .collect::<Vec<_>>();
    let distribution_items = reserves
        .clone()
        .into_iter()
        .map(|reserve| DistributionItem {
            reserve,
            share_bps: 100,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    for (idx, item) in distribution_items.iter().enumerate() {
        distribution_bps[idx] = item.share_bps();
    }
    let params = VaultParams {
        name: str_to_array("Vault"),
        description: str_to_array("Vault description"),
        lut: Default::default(),
        distribution_manager: Pubkey::new_unique(),
        distribution_bps,
        periodic_fee_rate_bps: 0,
    };

    info!("create vault with incorrect vaults authority");
    let liquidity_buffer = find_vault_liquidity_buffer(&vault_keypair.pubkey()).0;
    let result = create_vault(
        &mut ctx,
        &vault_keypair,
        curator_keypair.pubkey(),
        params.clone(),
        &admin_keypair,
        liquidity_usdc_mint,
        liquidity_buffer,
        reserves.clone(),
        lp_token_mints.clone(),
        lp_token_storages.clone(),
    )
    .await;
    assert!(result.is_err());

    // CREATE VAULT WITH INCORRECT CURATOR ACCOUNT

    info!("create vault with incorrect curator account");
    let liquidity_buffer = find_vault_liquidity_buffer(&vault_keypair.pubkey()).0;
    let result = create_vault(
        &mut ctx,
        &vault_keypair,
        main_pool_reserve_usdc_keypair.pubkey(),
        params.clone(),
        &admin_keypair,
        liquidity_usdc_mint,
        liquidity_buffer,
        reserves,
        lp_token_mints,
        lp_token_storages,
    )
    .await;
    assert!(result.is_err());
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#d07c56d1dc5743cd93d07617dfd7b974
#[tokio::test]
async fn create_with_different_currencies() {
    let mut runner = init_program_test();

    let admin_keypair = admin_keypair();
    let texture_config_keypair = texture_config_keypair();
    let pool_authority_keypair = Keypair::new();
    let texture_owner_keypair = Keypair::new();

    runner.add_native_wallet(admin_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(texture_owner_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(pool_authority_keypair.pubkey(), LAMPORTS);

    // 1 SOL = 100 USD
    add_price_feed_acc(&mut runner, "sol-usd").await;
    // 1 USDC = 1 USD
    add_price_feed_acc(&mut runner, "usdc-usd").await;
    // 1 JUP = 0.7 USD
    add_price_feed_acc(&mut runner, "jup-usd").await;
    // 1 BONK = 0.00001 USD
    add_price_feed_acc(&mut runner, "bonk-usd").await;

    add_curve_acc(&mut runner, "const-40-pct-acc").await;

    let liquidity_sol_mint =
        Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let liquidity_usdc_mint =
        Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let liquidity_jup_mint =
        Pubkey::from_str("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN").unwrap();
    let liquidity_bonk_mint =
        Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap();
    init_token_accounts(&mut runner, &liquidity_sol_mint);
    init_token_accounts(&mut runner, &liquidity_usdc_mint);
    init_token_accounts(&mut runner, &liquidity_jup_mint);
    init_token_accounts(&mut runner, &liquidity_bonk_mint);

    let mut ctx = runner.start_with_context().await;

    let env_keys = setup_env(
        &mut ctx,
        &admin_keypair,
        &texture_owner_keypair,
        &texture_config_keypair,
        &pool_authority_keypair,
    )
    .await;

    let EnvKeys {
        main_pool_reserves: (_, main_pool_reserve_usdc_keypair),
        risky1_pool_reserves: (_, risky1_pool_reserve_usdc_keypair),
        risky2_pool_reserves: (risky2_pool_reserve_bonk_keypair, _),
        vault_keypair,
        curator_keypair,
        vaults_authority_keypair,
        ..
    } = env_keys;

    // CREATE VAULT WITH DIFFERENT CURRENCIES

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_bonk_keypair.pubkey(),
    ];
    let lp_token_mints = reserves
        .clone()
        .iter()
        .map(|reserve| find_lp_token_mint(reserve).0)
        .collect::<Vec<_>>();
    let lp_token_storages = lp_token_mints
        .clone()
        .iter()
        .map(|lp_mint| find_vault_lp_token_storage(&vault_keypair.pubkey(), lp_mint).0)
        .collect::<Vec<_>>();
    let distribution_items = reserves
        .clone()
        .into_iter()
        .map(|reserve| DistributionItem {
            reserve,
            share_bps: 100,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    for (idx, item) in distribution_items.iter().enumerate() {
        distribution_bps[idx] = item.share_bps();
    }
    let params = VaultParams {
        name: str_to_array("Vault"),
        description: str_to_array("Vault description"),
        lut: Default::default(),
        distribution_manager: Pubkey::new_unique(),
        distribution_bps,
        periodic_fee_rate_bps: 0,
    };

    info!("create vault with different currencies");
    let liquidity_buffer = find_vault_liquidity_buffer(&vault_keypair.pubkey()).0;
    let result = create_vault(
        &mut ctx,
        &vault_keypair,
        curator_keypair.pubkey(),
        params.clone(),
        &vaults_authority_keypair,
        liquidity_usdc_mint,
        liquidity_buffer,
        reserves.clone(),
        lp_token_mints.clone(),
        lp_token_storages.clone(),
    )
    .await;
    assert!(result.is_err());
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#d07c56d1dc5743cd93d07617dfd7b974
#[tokio::test]
async fn create_with_different_lp_mints() {
    let mut runner = init_program_test();

    let admin_keypair = admin_keypair();
    let texture_config_keypair = texture_config_keypair();
    let pool_authority_keypair = Keypair::new();
    let texture_owner_keypair = Keypair::new();

    runner.add_native_wallet(admin_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(texture_owner_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(pool_authority_keypair.pubkey(), LAMPORTS);

    // 1 SOL = 100 USD
    add_price_feed_acc(&mut runner, "sol-usd").await;
    // 1 USDC = 1 USD
    add_price_feed_acc(&mut runner, "usdc-usd").await;
    // 1 JUP = 0.7 USD
    add_price_feed_acc(&mut runner, "jup-usd").await;
    // 1 BONK = 0.00001 USD
    add_price_feed_acc(&mut runner, "bonk-usd").await;

    add_curve_acc(&mut runner, "const-40-pct-acc").await;

    let liquidity_sol_mint =
        Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let liquidity_usdc_mint =
        Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let liquidity_jup_mint =
        Pubkey::from_str("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN").unwrap();
    let liquidity_bonk_mint =
        Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap();
    init_token_accounts(&mut runner, &liquidity_sol_mint);
    init_token_accounts(&mut runner, &liquidity_usdc_mint);
    init_token_accounts(&mut runner, &liquidity_jup_mint);
    init_token_accounts(&mut runner, &liquidity_bonk_mint);

    let mut ctx = runner.start_with_context().await;

    let env_keys = setup_env(
        &mut ctx,
        &admin_keypair,
        &texture_owner_keypair,
        &texture_config_keypair,
        &pool_authority_keypair,
    )
    .await;

    let EnvKeys {
        main_pool_reserves: (_, main_pool_reserve_usdc_keypair),
        risky1_pool_reserves: (_, risky1_pool_reserve_usdc_keypair),
        risky2_pool_reserves: (_, risky2_pool_reserve_usdc_keypair),
        vault_keypair,
        curator_keypair,
        vaults_authority_keypair,
        ..
    } = env_keys;

    // CREATE VAULT WITH DIFFERENT LP MINTS

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
    ];

    let lp_token_mints = vec![
        find_lp_token_mint(&main_pool_reserve_usdc_keypair.pubkey()).0,
        find_lp_token_mint(&risky2_pool_reserve_usdc_keypair.pubkey()).0, // not associated with risky1_pool_reserve_usdc
    ];

    let lp_token_storages = lp_token_mints
        .clone()
        .iter()
        .map(|lp_mint| find_vault_lp_token_storage(&vault_keypair.pubkey(), lp_mint).0)
        .collect::<Vec<_>>();
    let distribution_items = reserves
        .clone()
        .into_iter()
        .map(|reserve| DistributionItem {
            reserve,
            share_bps: 100,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    for (idx, item) in distribution_items.iter().enumerate() {
        distribution_bps[idx] = item.share_bps();
    }
    let params = VaultParams {
        name: str_to_array("Vault"),
        description: str_to_array("Vault description"),
        lut: Default::default(),
        distribution_manager: Pubkey::new_unique(),
        distribution_bps,
        periodic_fee_rate_bps: 0,
    };

    info!("create vault with different lp mints");
    let liquidity_buffer = find_vault_liquidity_buffer(&vault_keypair.pubkey()).0;
    let result = create_vault(
        &mut ctx,
        &vault_keypair,
        curator_keypair.pubkey(),
        params.clone(),
        &vaults_authority_keypair,
        liquidity_usdc_mint,
        liquidity_buffer,
        reserves.clone(),
        lp_token_mints.clone(),
        lp_token_storages.clone(),
    )
    .await;
    assert!(result.is_err());
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#2be7436815aa49d2bcad4c2186be70a8
#[tokio::test]
async fn create_with_sharing() {
    let mut runner = init_program_test();

    let admin_keypair = admin_keypair();
    let texture_config_keypair = texture_config_keypair();
    let pool_authority_keypair = Keypair::new();
    let texture_owner_keypair = Keypair::new();

    runner.add_native_wallet(admin_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(texture_owner_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(pool_authority_keypair.pubkey(), LAMPORTS);

    // 1 SOL = 100 USD
    add_price_feed_acc(&mut runner, "sol-usd").await;
    // 1 USDC = 1 USD
    add_price_feed_acc(&mut runner, "usdc-usd").await;
    // 1 JUP = 0.7 USD
    add_price_feed_acc(&mut runner, "jup-usd").await;
    // 1 BONK = 0.00001 USD
    add_price_feed_acc(&mut runner, "bonk-usd").await;

    add_curve_acc(&mut runner, "const-40-pct-acc").await;

    let liquidity_sol_mint =
        Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let liquidity_usdc_mint =
        Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let liquidity_jup_mint =
        Pubkey::from_str("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN").unwrap();
    let liquidity_bonk_mint =
        Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap();
    init_token_accounts(&mut runner, &liquidity_sol_mint);
    init_token_accounts(&mut runner, &liquidity_usdc_mint);
    init_token_accounts(&mut runner, &liquidity_jup_mint);
    init_token_accounts(&mut runner, &liquidity_bonk_mint);

    let mut ctx = runner.start_with_context().await;

    let env_keys = setup_env(
        &mut ctx,
        &admin_keypair,
        &texture_owner_keypair,
        &texture_config_keypair,
        &pool_authority_keypair,
    )
    .await;

    let EnvKeys {
        main_pool_reserves: (_, main_pool_reserve_usdc_keypair),
        risky1_pool_reserves: (_, risky1_pool_reserve_usdc_keypair),
        risky2_pool_reserves: _,
        vault_keypair,
        curator_keypair,
        vaults_authority_keypair,
        ..
    } = env_keys;

    // CREATE VAULT WITH INSUFFICIENT SHARING

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
    ];
    let lp_token_mints = reserves
        .clone()
        .iter()
        .map(|reserve| find_lp_token_mint(reserve).0)
        .collect::<Vec<_>>();
    let lp_token_storages = lp_token_mints
        .clone()
        .iter()
        .map(|lp_mint| find_vault_lp_token_storage(&vault_keypair.pubkey(), lp_mint).0)
        .collect::<Vec<_>>();

    let distribution_items = reserves
        .clone()
        .into_iter()
        .map(|reserve| DistributionItem {
            reserve,
            share_bps: 3300,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    for (idx, item) in distribution_items.iter().enumerate() {
        distribution_bps[idx] = item.share_bps();
    }
    let params = VaultParams {
        name: str_to_array("Vault"),
        description: str_to_array("Vault description"),
        lut: Default::default(),
        distribution_manager: Pubkey::new_unique(),
        distribution_bps,
        periodic_fee_rate_bps: 0,
    };

    info!("create vault with insufficient sharing");
    let liquidity_buffer = find_vault_liquidity_buffer(&vault_keypair.pubkey()).0;
    let result = create_vault(
        &mut ctx,
        &vault_keypair,
        curator_keypair.pubkey(),
        params.clone(),
        &vaults_authority_keypair,
        liquidity_usdc_mint,
        liquidity_buffer,
        reserves.clone(),
        lp_token_mints.clone(),
        lp_token_storages.clone(),
    )
    .await;
    assert!(result.is_err());

    // CREATE VAULT WITH EXCEEDING SHARING

    distribution_bps[0] = 13300;

    info!("create vault with exceeding sharing");
    let liquidity_buffer = find_vault_liquidity_buffer(&vault_keypair.pubkey()).0;
    let result = create_vault(
        &mut ctx,
        &vault_keypair,
        curator_keypair.pubkey(),
        params.clone(),
        &vaults_authority_keypair,
        liquidity_usdc_mint,
        liquidity_buffer,
        reserves.clone(),
        lp_token_mints.clone(),
        lp_token_storages.clone(),
    )
    .await;
    assert!(result.is_err());

    // CREATE VAULT WITH INCORRECT DistributionItem.reserve

    let mut distribution_items = reserves
        .clone()
        .into_iter()
        .map(|reserve| DistributionItem {
            reserve,
            share_bps: 3300,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    distribution_items[1].share_bps = 6700;
    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    for (idx, item) in distribution_items.iter().enumerate() {
        distribution_bps[idx] = item.share_bps();
    }
    let params = VaultParams {
        name: str_to_array("Vault"),
        description: str_to_array("Vault description"),
        lut: Default::default(),
        distribution_manager: Pubkey::new_unique(),
        distribution_bps,
        periodic_fee_rate_bps: 0,
    };

    let mut reserves = reserves;
    reserves[0] = curator_keypair.pubkey();

    info!("create vault with incorrect DistributionItem.reserve");
    let liquidity_buffer = find_vault_liquidity_buffer(&vault_keypair.pubkey()).0;
    let result = create_vault(
        &mut ctx,
        &vault_keypair,
        curator_keypair.pubkey(),
        params.clone(),
        &vaults_authority_keypair,
        liquidity_usdc_mint,
        liquidity_buffer,
        reserves,
        lp_token_mints,
        lp_token_storages,
    )
    .await;
    assert!(result.is_err())
}
