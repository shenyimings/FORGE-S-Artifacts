#![cfg(feature = "test-bpf")]

use bytemuck::Zeroable;
use std::str::FromStr;

use price_proxy::state::utils::str_to_array;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address;
use super_lendy::pda::{find_lp_token_mint, find_program_authority};
use super_lendy::state::reserve::{ReserveConfig, ReserveFeesConfig, RESERVE_TYPE_NORMAL};
use texture_common::account::PodAccount;
use tracing::info;
use vault::pda::{find_vault_liquidity_buffer, find_vault_lp_token_storage};

use vault::state::vault::{Vault, VaultParams, VAULT_DISTRIBUTION_MAX_ITEMS};

use crate::utils::setup::{setup_env, setup_vault, EnvKeys};
use crate::utils::superlendy_executor::create_reserve;
use crate::utils::vault_executor::{
    add_distribution_item, alter_vault, create_position, deposit, rebalance,
    remove_distribution_item, withdraw,
};
use crate::utils::{
    add_curve_acc, add_price_feed_acc, admin_keypair, borrow_keypair, get_account,
    init_program_test, init_token_accounts, texture_config_keypair, Runner, LAMPORTS,
    LAMPORTS_PER_USDC,
};

pub mod utils;

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#10eb5734a22080d28da5e0c986fa6330
#[tokio::test]
async fn load_vault_memory_success() {
    let mut runner = init_program_test();

    let admin_keypair = admin_keypair();
    let texture_config_keypair = texture_config_keypair();
    let pool_authority_keypair = Keypair::new();
    let texture_owner_keypair = Keypair::new();
    let owner_keypair = borrow_keypair();

    runner.add_native_wallet(admin_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(texture_owner_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(pool_authority_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(owner_keypair.pubkey(), LAMPORTS);

    // 1 SOL = 100 USD
    add_price_feed_acc(&mut runner, "sol-usd").await;
    // 1 USDC = 1 USD
    let usdc_price_feed = add_price_feed_acc(&mut runner, "usdc-usd").await;
    // 1 JUP = 0.7 USD
    add_price_feed_acc(&mut runner, "jup-usd").await;
    // 1 BONK = 0.00001 USD
    add_price_feed_acc(&mut runner, "bonk-usd").await;

    let irm = add_curve_acc(&mut runner, "const-40-pct-acc").await;

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
        main_pool,
        vault_keypair,
        curator_keypair,
        vaults_authority_keypair,
        risky3_pool_reserve_usdc_keypair,
        risky4_pool_reserve_usdc_keypair,
        position_keypair,
        ..
    } = env_keys;

    let mut reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
        risky3_pool_reserve_usdc_keypair.pubkey(),
        risky4_pool_reserve_usdc_keypair.pubkey(),
    ];

    const NUM_RESERVES: usize = 16;

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    let mut excess_distribution_bps = 10000;
    for item in &mut distribution_bps[0..NUM_RESERVES] {
        const DISTRIBUTION_PER_ITEM_BPS: u16 = 100;
        *item = DISTRIBUTION_PER_ITEM_BPS;
        excess_distribution_bps -= DISTRIBUTION_PER_ITEM_BPS;
    }
    distribution_bps[0] += excess_distribution_bps; // make 100% share

    let fees_config = ReserveFeesConfig {
        curator_borrow_fee_rate_bps: 0,
        curator_performance_fee_rate_bps: 0,
        _padding: Zeroable::zeroed(),
    };
    let reserve_config = ReserveConfig {
        market_price_feed: usdc_price_feed,
        irm,
        liquidation_bonus_bps: 200,
        max_borrow_ltv_bps: 9998,
        partly_unhealthy_ltv_bps: 9999,
        fully_unhealthy_ltv_bps: 10000,
        partial_liquidation_factor_bps: 2000,
        _padding: Zeroable::zeroed(),
        fees: fees_config,
        max_total_liquidity: 10_000 * LAMPORTS_PER_SOL,
        price_stale_threshold_sec: 20,
        max_borrow_utilization_bps: 10000,
        max_withdraw_utilization_bps: 10000,
    };

    for _ in 0..NUM_RESERVES - reserves.len() {
        let reserve_keypair = Keypair::new();
        create_reserve(
            &mut ctx,
            &reserve_keypair,
            main_pool.pubkey(),
            &pool_authority_keypair,
            curator_keypair.pubkey(),
            liquidity_usdc_mint,
            usdc_price_feed,
            reserve_config,
            RESERVE_TYPE_NORMAL,
        )
        .await
        .expect("create main_pool_reserve_usdc");
        reserves.push(reserve_keypair.pubkey());
    }
    let mut params = VaultParams {
        name: str_to_array("Vault"),
        description: str_to_array("Vault description"),
        lut: Default::default(),
        distribution_manager: Pubkey::new_unique(),
        distribution_bps,
        periodic_fee_rate_bps: 0,
    };

    setup_vault(
        &mut ctx,
        &vault_keypair,
        curator_keypair.pubkey(),
        params.clone(),
        &vaults_authority_keypair,
        reserves.clone(),
    )
    .await
    .expect("create_vault");

    let vault_acc = get_account(&mut ctx.banks_client, vault_keypair.pubkey())
        .await
        .expect("get pool");
    let vault = Vault::try_from_bytes(&vault_acc.data).expect("cast vault data");
    assert_eq!(vault.distribution.effective_items().count(), NUM_RESERVES); // max available items to create vault

    // CREATE POSITION

    create_position(
        &mut ctx,
        &position_keypair,
        vault_keypair.pubkey(),
        &owner_keypair,
    )
    .await
    .expect("create_position");

    // DEPOSIT

    let source_liquidity_wallet =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_usdc_mint);
    let superlendy_program_authority = find_program_authority().0;

    let amount = 100 * LAMPORTS_PER_USDC;

    info!("deposit {} USDC", amount);
    deposit(
        &mut ctx,
        position_keypair.pubkey(),
        vault_keypair.pubkey(),
        &owner_keypair,
        liquidity_usdc_mint,
        source_liquidity_wallet,
        superlendy_program_authority,
        amount,
    )
    .await
    .expect("deposit");

    // ALTER DISTRIBUTION SHARE

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    for item in &mut distribution_bps[0..NUM_RESERVES] {
        *item = 600;
    }
    distribution_bps[0] += 400; // make 100% share

    params.distribution_bps = distribution_bps;

    info!("alter share");
    alter_vault(
        &mut ctx,
        vault_keypair.pubkey(),
        curator_keypair.pubkey(),
        params.clone(),
        &vaults_authority_keypair,
        params.distribution_manager,
    )
    .await
    .expect("alter_vault");

    // REBALANCE

    let liquidity_buffer = find_vault_liquidity_buffer(&vault_keypair.pubkey()).0;
    let superlendy_program_authority = find_program_authority().0;

    info!("rebalance");
    rebalance(
        &mut ctx,
        vault_keypair.pubkey(),
        liquidity_usdc_mint,
        liquidity_buffer,
        superlendy_program_authority,
    )
    .await
    .expect("rebalance");

    // WITHDRAW

    info!("withdraw {} USDC", amount);
    withdraw(
        &mut ctx,
        position_keypair.pubkey(),
        vault_keypair.pubkey(),
        &owner_keypair,
        liquidity_usdc_mint,
        source_liquidity_wallet,
        superlendy_program_authority,
        amount,
    )
    .await
    .expect("deposit");
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#10db5734a22080bf9b4bfe103b40decb
#[tokio::test]
async fn add_by_incorrect_authority() {
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
        risky3_pool_reserve_usdc_keypair,
        ..
    } = env_keys;

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 1000;
    distribution_bps[1] = 4000;
    distribution_bps[2] = 5000;
    let params = VaultParams {
        name: str_to_array("Vault"),
        description: str_to_array("Vault description"),
        lut: Default::default(),
        distribution_manager: texture_owner_keypair.pubkey(),
        distribution_bps,
        periodic_fee_rate_bps: 0,
    };

    setup_vault(
        &mut ctx,
        &vault_keypair,
        curator_keypair.pubkey(),
        params.clone(),
        &vaults_authority_keypair,
        reserves.clone(),
    )
    .await
    .expect("create_vault");

    // TRY TO ADD DISTRIBUTION ITEM BY DISTRIBUTION MANAGER

    let (lp_token_mint, _) = find_lp_token_mint(&risky3_pool_reserve_usdc_keypair.pubkey());
    let (lp_token_storage, _) =
        find_vault_lp_token_storage(&vault_keypair.pubkey(), &lp_token_mint);

    info!("try to add distribution item by distribution manager");
    let result = add_distribution_item(
        &mut ctx,
        vault_keypair.pubkey(),
        curator_keypair.pubkey(),
        &texture_owner_keypair,
        lp_token_storage,
        lp_token_mint,
        0,
    )
    .await;
    assert!(result.is_err())
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#10db5734a2208017b4e0c342a15eb366
#[tokio::test]
async fn alter_vacant_item_share() {
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
        risky3_pool_reserve_usdc_keypair,
        ..
    } = env_keys;

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
        risky3_pool_reserve_usdc_keypair.pubkey(),
    ];

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 1000;
    distribution_bps[1] = 0;
    distribution_bps[2] = 4000;
    distribution_bps[3] = 5000;
    let mut params = VaultParams {
        name: str_to_array("Vault"),
        description: str_to_array("Vault description"),
        lut: Default::default(),
        distribution_manager: Pubkey::new_unique(),
        distribution_bps,
        periodic_fee_rate_bps: 0,
    };

    setup_vault(
        &mut ctx,
        &vault_keypair,
        curator_keypair.pubkey(),
        params.clone(),
        &vaults_authority_keypair,
        reserves.clone(),
    )
    .await
    .expect("create_vault");

    // REMOVE RISKY1.USDC RESERVE DISTRIBUTION ITEM

    let (lp_token_mint, _) = find_lp_token_mint(&risky1_pool_reserve_usdc_keypair.pubkey());
    let (lp_token_storage, _) =
        find_vault_lp_token_storage(&vault_keypair.pubkey(), &lp_token_mint);

    info!("remove risky1.usdc reserve distribution item");
    remove_distribution_item(
        &mut ctx,
        vault_keypair.pubkey(),
        curator_keypair.pubkey(),
        &vaults_authority_keypair,
        lp_token_storage,
        risky1_pool_reserve_usdc_keypair.pubkey(),
    )
    .await
    .expect("remove_distribution_item");

    // ALTER SHARE OF VACANT ITEM

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 1000;
    distribution_bps[1] = 1000;
    distribution_bps[2] = 4000;
    distribution_bps[3] = 4000;
    params.distribution_bps = distribution_bps;

    info!("alter share of vacant item");
    let result = alter_vault(
        &mut ctx,
        vault_keypair.pubkey(),
        curator_keypair.pubkey(),
        params.clone(),
        &vaults_authority_keypair,
        params.distribution_manager,
    )
    .await;
    assert!(result.is_err())
}
