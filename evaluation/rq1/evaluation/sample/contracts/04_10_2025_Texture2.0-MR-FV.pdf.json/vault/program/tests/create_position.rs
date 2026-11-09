#![cfg(feature = "test-bpf")]

use std::str::FromStr;
use std::time::Duration;

use price_proxy::state::utils::str_to_array;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction::create_account;
use solana_program_test::ProgramTestBanksClientExt;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use texture_common::account::PodAccount;
use tracing::info;

use vault::state::vault::{Vault, VaultParams, VAULT_DISTRIBUTION_MAX_ITEMS};
use vault::VAULT_ID;

use crate::utils::setup::{setup_env, setup_vault, EnvKeys};
use crate::utils::vault_executor::create_position;
use crate::utils::{
    add_curve_acc, add_price_feed_acc, admin_keypair, init_program_test, init_token_accounts,
    texture_config_keypair, Runner, LAMPORTS,
};

pub mod utils;

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#c2dec47ecfdf4a0ba087287499f50ed3
#[tokio::test]
async fn create_with_incorrect_vault() {
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
        vault_keypair,
        vaults_authority_keypair,
        position_keypair,
        ..
    } = env_keys;

    // CREATE ACCOUNT OWNED BY VAULT

    let blockhash = ctx
        .banks_client
        .get_new_latest_blockhash(&ctx.last_blockhash)
        .await
        .expect("get latest blockhash");

    let rent = ctx.banks_client.get_rent().await.expect("get rent");
    let vault_lamports = rent.minimum_balance(Vault::SIZE);

    let create_ix = create_account(
        &ctx.payer.pubkey(),
        &vault_keypair.pubkey(),
        vault_lamports,
        Vault::SIZE as u64,
        &VAULT_ID,
    );

    let tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &vault_keypair],
        blockhash,
    );

    info!("create account owned by vault");
    ctx.banks_client
        .process_transaction(tx)
        .await
        .expect("create_falsified_vault");

    // CREATE POSITION WITH INCORRECT VAULT ACCOUNT

    let result = create_position(
        &mut ctx,
        &position_keypair,
        vault_keypair.pubkey(),
        &vaults_authority_keypair,
    )
    .await;
    assert!(result.is_err())
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#1875c2c31cb54e37be194b902bf0b217
#[tokio::test]
async fn create_and_reset_position() {
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
        vault_keypair,
        curator_keypair,
        vaults_authority_keypair,
        position_keypair,
        ..
    } = env_keys;

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 10000;
    let params = VaultParams {
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
        vec![main_pool_reserve_usdc_keypair.pubkey()],
    )
    .await
    .expect("create_vault");

    // CREATE POSITION

    info!("create position");
    create_position(
        &mut ctx,
        &position_keypair,
        vault_keypair.pubkey(),
        &vaults_authority_keypair,
    )
    .await
    .expect("create_position");

    // CREATE POSITION REPEATED

    info!("create position repeated");
    tokio::time::sleep(Duration::from_secs(1)).await; // because of race condition in solana-program-test
    let result = create_position(
        &mut ctx,
        &position_keypair,
        vault_keypair.pubkey(),
        &vaults_authority_keypair,
    )
    .await;
    assert!(result.is_err())
}
