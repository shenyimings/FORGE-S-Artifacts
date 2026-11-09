#![cfg(feature = "test-bpf")]

use std::str::FromStr;

use price_proxy::state::utils::str_to_array;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address;
use super_lendy::pda::{find_lp_token_mint, find_program_authority};
use texture_common::account::PodAccount;
use tracing::info;
use vault::pda::{find_vault_liquidity_buffer, find_vault_lp_token_storage};

use vault::state::vault::{Vault, VaultParams, VAULT_DISTRIBUTION_MAX_ITEMS};

use crate::utils::setup::{setup_env, setup_vault, EnvKeys};
use crate::utils::vault_executor::{
    alter_vault, create_position, deposit, rebalance, remove_distribution_item,
};
use crate::utils::{
    add_curve_acc, add_price_feed_acc, admin_keypair, borrow_keypair, get_account,
    init_program_test, init_token_accounts, texture_config_keypair, Runner, LAMPORTS,
    LAMPORTS_PER_USDC,
};

pub mod utils;

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#10db5734a22080a8a7b8d6c2c9665797
#[tokio::test]
async fn remove_success() {
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

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 1000;
    distribution_bps[1] = 4000;
    distribution_bps[2] = 5000;
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

    // ALTER SHARE

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 1000;
    distribution_bps[1] = 0;
    distribution_bps[2] = 9000;
    params.distribution_bps = distribution_bps;

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

    // REMOVE DISTRIBUTION ITEM

    let (lp_token_mint, _) = find_lp_token_mint(&risky1_pool_reserve_usdc_keypair.pubkey());
    let (lp_token_storage, _) =
        find_vault_lp_token_storage(&vault_keypair.pubkey(), &lp_token_mint);

    info!("remove distribution item");
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

    // CHECK ITEMS

    let vault_acc = get_account(&mut ctx.banks_client, vault_keypair.pubkey())
        .await
        .expect("get pool");
    let vault = Vault::try_from_bytes(&vault_acc.data).expect("cast vault data");
    assert_eq!(
        vault.distribution.0[0].reserve,
        main_pool_reserve_usdc_keypair.pubkey()
    );
    assert_eq!(vault.distribution.0[0].share_bps, 1000);
    assert_eq!(vault.distribution.0[1].reserve, Pubkey::default());
    assert_eq!(vault.distribution.0[1].share_bps, 0);
    assert_eq!(
        vault.distribution.0[2].reserve,
        risky2_pool_reserve_usdc_keypair.pubkey()
    );
    assert_eq!(vault.distribution.0[2].share_bps, 9000);
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#10db5734a2208046b9c6de131e786066
#[tokio::test]
async fn remove_incorrect_authority() {
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

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 1000;
    distribution_bps[1] = 4000;
    distribution_bps[2] = 5000;
    let mut params = VaultParams {
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

    // ALTER SHARE

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 1000;
    distribution_bps[1] = 0;
    distribution_bps[2] = 9000;
    params.distribution_bps = distribution_bps;

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

    // REMOVE DISTRIBUTION ITEM

    let (lp_token_mint, _) = find_lp_token_mint(&risky1_pool_reserve_usdc_keypair.pubkey());
    let (lp_token_storage, _) =
        find_vault_lp_token_storage(&vault_keypair.pubkey(), &lp_token_mint);

    info!("remove distribution item");
    let result = remove_distribution_item(
        &mut ctx,
        vault_keypair.pubkey(),
        curator_keypair.pubkey(),
        &texture_owner_keypair, // remove from distribution_manager
        lp_token_storage,
        risky1_pool_reserve_usdc_keypair.pubkey(),
    )
    .await;

    assert!(result.is_err())
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#10db5734a22080ab9b17c17e6b5069cf
#[tokio::test]
async fn remove_non_vacant_item() {
    let mut runner = init_program_test();

    let admin_keypair = admin_keypair();
    let owner_keypair = borrow_keypair();
    let texture_config_keypair = texture_config_keypair();
    let pool_authority_keypair = Keypair::new();
    let texture_owner_keypair = Keypair::new();

    runner.add_native_wallet(admin_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(owner_keypair.pubkey(), LAMPORTS);
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

    let rent = ctx.banks_client.get_rent().await.expect("get rent");
    let lp_token_storage_lamports = rent.minimum_balance(spl_token::state::Account::LEN);

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
        position_keypair,
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

    // TRY TO REMOVE NON-VACANT DISTRIBUTION ITEM

    let (lp_token_mint, _) = find_lp_token_mint(&main_pool_reserve_usdc_keypair.pubkey());
    let (lp_token_storage, _) =
        find_vault_lp_token_storage(&vault_keypair.pubkey(), &lp_token_mint);

    info!("try to remove non-vacant distribution item");
    let result = remove_distribution_item(
        &mut ctx,
        vault_keypair.pubkey(),
        curator_keypair.pubkey(),
        &vaults_authority_keypair,
        lp_token_storage,
        main_pool_reserve_usdc_keypair.pubkey(),
    )
    .await;
    assert!(result.is_err());

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

    // ALTER SHARE

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 1000;
    distribution_bps[1] = 0;
    distribution_bps[2] = 9000;
    params.distribution_bps = distribution_bps;

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

    // TRY TO REMOVE DISTRIBUTION ITEM WITH NON-ZERO LP TOKEN STORAGE

    let (lp_token_mint, _) = find_lp_token_mint(&risky1_pool_reserve_usdc_keypair.pubkey());
    let (lp_token_storage, _) =
        find_vault_lp_token_storage(&vault_keypair.pubkey(), &lp_token_mint);

    info!("try to remove distribution item with non-zero lp token storage");
    let result = remove_distribution_item(
        &mut ctx,
        vault_keypair.pubkey(),
        curator_keypair.pubkey(),
        &vaults_authority_keypair,
        lp_token_storage,
        risky1_pool_reserve_usdc_keypair.pubkey(),
    )
    .await;
    assert!(result.is_err());

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

    // REMOVE DISTRIBUTION ITEM

    let vaults_authority_acc0 =
        get_account(&mut ctx.banks_client, vaults_authority_keypair.pubkey())
            .await
            .expect("get account");

    info!("remove distribution item");
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

    let vaults_authority_acc1 =
        get_account(&mut ctx.banks_client, vaults_authority_keypair.pubkey())
            .await
            .expect("get account");

    // CHECK ITEMS

    let vault_acc = get_account(&mut ctx.banks_client, vault_keypair.pubkey())
        .await
        .expect("get pool");
    let vault = Vault::try_from_bytes(&vault_acc.data).expect("cast vault data");
    assert_eq!(
        vault.distribution.0[0].reserve,
        main_pool_reserve_usdc_keypair.pubkey()
    );
    assert_eq!(vault.distribution.0[0].share_bps, 1000);
    assert_eq!(vault.distribution.0[1].reserve, Pubkey::default());
    assert_eq!(vault.distribution.0[1].share_bps, 0);
    assert_eq!(
        vault.distribution.0[2].reserve,
        risky2_pool_reserve_usdc_keypair.pubkey()
    );
    assert_eq!(vault.distribution.0[2].share_bps, 9000);

    // CHECK RENT FOR LP TOKEN STORAGE HAS BEEN RETURNED

    assert_eq!(
        vaults_authority_acc1.lamports,
        vaults_authority_acc0.lamports + lp_token_storage_lamports
    )
}
