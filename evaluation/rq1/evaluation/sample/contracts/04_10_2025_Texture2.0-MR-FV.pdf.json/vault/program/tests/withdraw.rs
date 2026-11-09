#![cfg(feature = "test-bpf")]

use bytemuck::Zeroable;
use chrono::Utc;
use std::str::FromStr;
use texture_common::dec;
use utils::USDC_DECIMALS;

use price_proxy::state::utils::str_to_array;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::pubkey::Pubkey;
use solana_program_test::BanksClientError;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address;
use super_lendy::pda::{find_lp_token_mint, find_program_authority};
use super_lendy::state::reserve::{Reserve, RESERVE_MODE_RETAIN_LIQUIDITY};
use super_lendy::state::SLOTS_PER_YEAR;
use super_lendy::MAX_AMOUNT;
use texture_common::account::PodAccount;
use texture_common::math::{CheckedMul, Decimal};
use tracing::info;
use vault::state::position::Position;

use vault::state::vault::{DistributionItem, VaultParams, VAULT_DISTRIBUTION_MAX_ITEMS};

use crate::utils::setup::{setup_env, setup_vault, EnvKeys};
use crate::utils::superlendy_executor::{alter_reserve, repay};
use crate::utils::vault_executor::{alter_vault, create_position, deposit, withdraw};
use crate::utils::{
    add_curve_acc, add_price_feed_acc, admin_keypair, borrow_keypair,
    create_associated_token_account, get_account, get_token_account, init_program_test,
    init_token_accounts, price_feed_authority, superlendy_executor, texture_config_keypair,
    write_price, Runner, LAMPORTS, LAMPORTS_PER_USDC,
};

pub mod utils;

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#2fe63b6d03eb4339ba1cbb3a6df6ae61
#[tokio::test]
async fn withdraw_success() {
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
    let sol_price_feed = add_price_feed_acc(&mut runner, "sol-usd").await;
    // 1 USDC = 1 USD
    let usdc_price_feed = add_price_feed_acc(&mut runner, "usdc-usd").await;
    // 1 JUP = 0.7 USD
    add_price_feed_acc(&mut runner, "jup-usd").await;
    // 1 BONK = 0.00001 USD
    add_price_feed_acc(&mut runner, "bonk-usd").await;

    let irm_40 = add_curve_acc(&mut runner, "const-40-pct-acc").await;
    let irm_200 = add_curve_acc(&mut runner, "const-200-pct-acc").await;

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
        main_pool_reserves: (main_pool_reserve_sol_keypair, main_pool_reserve_usdc_keypair),
        risky1_pool_reserves: (_, risky1_pool_reserve_usdc_keypair),
        risky2_pool_reserves: (_, risky2_pool_reserve_usdc_keypair),
        vault_keypair,
        curator_keypair,
        vaults_authority_keypair,
        position_keypair,
        main_pool,
        ..
    } = env_keys;

    // ALTER reserve.irm to const-200-pct

    let reserve_acc = get_account(
        &mut ctx.banks_client,
        main_pool_reserve_usdc_keypair.pubkey(),
    )
    .await
    .expect("get reserve");
    let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
    let mut params = reserve.config;
    params.irm = irm_200;
    alter_reserve(
        &mut ctx,
        main_pool_reserve_usdc_keypair.pubkey(),
        main_pool.pubkey(),
        &pool_authority_keypair,
        curator_keypair.pubkey(),
        params,
        0,
    )
    .await
    .expect("alter_reserve");

    // CREATE VAULT

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

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

    info!("deposit {} USDC lamports", amount);
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

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    assert_eq!(
        position.issued_vlp_amount().unwrap(),
        Decimal::from_i128_with_scale(100, 0).unwrap()
    );

    // DEPOSIT & LOCK COLLATERAL 1 SOL & BORROW 16,5 USDC FROM MAIN USDC RESERVE

    let superlendy_main_position_kp = Keypair::new();
    superlendy_executor::create_position(
        &mut ctx,
        &superlendy_main_position_kp,
        main_pool.pubkey(),
        &owner_keypair,
    )
    .await
    .expect("create_position");

    let source_liq_wallet_sol =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_sol_mint);
    let lp_mint = find_lp_token_mint(&main_pool_reserve_sol_keypair.pubkey()).0;
    let dest_lp_wallet_sol = create_associated_token_account(&mut ctx, &owner_keypair, &lp_mint)
        .await
        .expect("create lp ata");
    let deposit_amount = LAMPORTS_PER_SOL;

    info!("deposit {} SOL lamports", deposit_amount);
    superlendy_executor::deposit_liquidity(
        &mut ctx,
        main_pool_reserve_sol_keypair.pubkey(),
        sol_price_feed,
        irm_40,
        &owner_keypair,
        source_liq_wallet_sol,
        dest_lp_wallet_sol,
        deposit_amount,
    )
    .await
    .expect("deposit_liquidity");

    superlendy_executor::refresh_position(&mut ctx, superlendy_main_position_kp.pubkey())
        .await
        .expect("refresh position");

    info!("lock {} SOL lamports", deposit_amount);
    superlendy_executor::lock_collateral(
        &mut ctx,
        main_pool_reserve_sol_keypair.pubkey(),
        sol_price_feed,
        irm_40,
        superlendy_main_position_kp.pubkey(),
        &owner_keypair,
        dest_lp_wallet_sol,
        deposit_amount,
    )
    .await
    .expect("lock_collateral");

    let texture_fee_receiver =
        create_associated_token_account(&mut ctx, &texture_owner_keypair, &liquidity_usdc_mint)
            .await
            .expect("create texture fee receiver ata");
    let curator_fee_receiver =
        create_associated_token_account(&mut ctx, &pool_authority_keypair, &liquidity_usdc_mint)
            .await
            .expect("create curator fee receiver ata");
    let source_liq_wallet_usdc =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_usdc_mint);

    superlendy_executor::refresh_position(&mut ctx, superlendy_main_position_kp.pubkey())
        .await
        .expect("refresh position");

    let borrow_amount = Decimal::from_i128_with_scale(165, 1)
        .unwrap()
        .to_lamports_round(USDC_DECIMALS)
        .unwrap();
    info!("borrow {} USDC", borrow_amount);
    superlendy_executor::borrow(
        &mut ctx,
        superlendy_main_position_kp.pubkey(),
        main_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm_200,
        main_pool.pubkey(),
        &owner_keypair,
        curator_keypair.pubkey(),
        curator_fee_receiver,
        texture_fee_receiver,
        source_liq_wallet_usdc,
        borrow_amount,
        1,
    )
    .await
    .expect("borrow");

    // 1 SOLANA YEAR LATER

    info!("wrap to slot {}", SLOTS_PER_YEAR + 1);
    ctx.warp_to_slot(SLOTS_PER_YEAR + 1).expect("warp_to_slot"); // solana_year = 63072000 slots

    superlendy_executor::refresh_position(&mut ctx, superlendy_main_position_kp.pubkey())
        .await
        .expect("refresh position");

    // CHECK REPAYMENT

    let reserve_acc = get_account(
        &mut ctx.banks_client,
        main_pool_reserve_usdc_keypair.pubkey(),
    )
    .await
    .expect("get reserve");
    let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
    let borrowed_amount = reserve.liquidity.borrowed_amount().unwrap();
    let repayment = dec!(121.919421766316246193); // 121.9 USDC
    info!("repayment = borrow + interest = {}", borrowed_amount);
    assert_eq!(borrowed_amount, repayment);

    // REPAY

    let source_liquidity_acc0 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
        .await
        .expect("get token acc");

    repay(
        &mut ctx,
        superlendy_main_position_kp.pubkey(),
        main_pool_reserve_usdc_keypair.pubkey(),
        &owner_keypair,
        source_liq_wallet_usdc,
        MAX_AMOUNT,
    )
    .await
    .expect("repay");

    let source_liquidity_acc1 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
        .await
        .expect("get token acc");

    assert_eq!(
        source_liquidity_acc1.amount,
        source_liquidity_acc0.amount - repayment.to_lamports_round(USDC_DECIMALS).unwrap()
    );

    // WITHDRAW 100 VLP

    let source_liquidity_acc0 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
        .await
        .expect("get token acc");

    info!("withdraw {} VLP lamports", amount);
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
    .expect("withdraw");

    let source_liquidity_acc1 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
        .await
        .expect("get token acc");

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    assert_eq!(position.issued_vlp_amount().unwrap(), Decimal::ZERO);

    // CHECK LIQUIDITY WALLET BALANCE

    assert_eq!(
        source_liquidity_acc1.amount,
        source_liquidity_acc0.amount
            + (
            Decimal::from_i128_with_scale(165, 1).unwrap()
                .to_lamports_round(USDC_DECIMALS)
                .unwrap()
            + repayment.to_lamports_round(USDC_DECIMALS).unwrap()
        )       // 16,5 + 121,9 USDC from MainPool.USDC reserve
            + (33 * LAMPORTS_PER_USDC) // Risky1Pool.USDC reserve
            + (34 * LAMPORTS_PER_USDC) // Risky2Pool.USDC reserve
            - 5 // rounding
    )
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#6c654868e2e84720bc3cb6126f2a4e18
#[tokio::test]
async fn withdraw_incorrect_owner() {
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

    // CREATE VAULT

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

    let mut distribution_items = reserves
        .clone()
        .into_iter()
        .map(|reserve| DistributionItem {
            reserve,
            share_bps: 3300,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    distribution_items[2].share_bps = 3400; // make 100% complete sharing
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

    // TRY TO WITHDRAW BY INCORRECT OWNER
    info!("try to withdraw by incorrect owner");
    let result = withdraw(
        &mut ctx,
        position_keypair.pubkey(),
        vault_keypair.pubkey(),
        &admin_keypair,
        liquidity_usdc_mint,
        source_liquidity_wallet,
        superlendy_program_authority,
        amount,
    )
    .await;
    assert!(result.is_err());
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#628f9ff29e894b06893df8eed6dc4360
#[tokio::test]
async fn withdraw_from_incorrect_vault() {
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

    // CREATE VAULT1, VAULT2

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

    let mut distribution_items = reserves
        .clone()
        .into_iter()
        .map(|reserve| DistributionItem {
            reserve,
            share_bps: 3300,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    distribution_items[2].share_bps = 3400; // make 100% complete sharing
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

    let vault2_keypair = Keypair::new();
    setup_vault(
        &mut ctx,
        &vault2_keypair,
        curator_keypair.pubkey(),
        params.clone(),
        &vaults_authority_keypair,
        reserves.clone(),
    )
    .await
    .expect("create_vault2");

    // CREATE POSITION1, POSITION2

    create_position(
        &mut ctx,
        &position_keypair,
        vault_keypair.pubkey(),
        &owner_keypair,
    )
    .await
    .expect("create_position1");

    let position2_keypair = Keypair::new();
    create_position(
        &mut ctx,
        &position2_keypair,
        vault2_keypair.pubkey(),
        &owner_keypair,
    )
    .await
    .expect("create_position2");

    // DEPOSIT TO VAULT1

    let source_liquidity_wallet =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_usdc_mint);
    let superlendy_program_authority = find_program_authority().0;

    let amount = 100 * LAMPORTS_PER_USDC;

    info!("deposit {} USDC to vault1", amount);
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

    // DEPOSIT TO VAULT2

    info!("deposit {} USDC to vault2", amount);
    deposit(
        &mut ctx,
        position2_keypair.pubkey(),
        vault2_keypair.pubkey(),
        &owner_keypair,
        liquidity_usdc_mint,
        source_liquidity_wallet,
        superlendy_program_authority,
        amount,
    )
    .await
    .expect("deposit");

    // TRY TO WITHDRAW FROM VAULT1 WITH POSITION2
    info!("try to withdraw from vault1 with position2");
    let result = withdraw(
        &mut ctx,
        position2_keypair.pubkey(),
        vault_keypair.pubkey(),
        &owner_keypair,
        liquidity_usdc_mint,
        source_liquidity_wallet,
        superlendy_program_authority,
        amount,
    )
    .await;
    assert!(result.is_err());
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#82b5a506c47e44cabebd432ddef3da5a
#[tokio::test]
async fn withdraw_math_error() {
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

    // CREATE VAULT

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

    let mut distribution_items = reserves
        .clone()
        .into_iter()
        .map(|reserve| DistributionItem {
            reserve,
            share_bps: 3300,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    distribution_items[2].share_bps = 3400; // make 100% complete sharing
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
    .expect("withdraw");

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
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#106b5734a22080ca94d7f163dded7142
#[tokio::test]
async fn withdraw_disabled() {
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
        risky2_pool,
        ..
    } = env_keys;

    // ALTER reserve.mode to RESERVE_MODE_RETAIN_LIQUIDITY

    let reserve_acc = get_account(
        &mut ctx.banks_client,
        risky2_pool_reserve_usdc_keypair.pubkey(),
    )
    .await
    .expect("get reserve");
    let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
    alter_reserve(
        &mut ctx,
        risky2_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool.pubkey(),
        &pool_authority_keypair,
        curator_keypair.pubkey(),
        reserve.config,
        RESERVE_MODE_RETAIN_LIQUIDITY,
    )
    .await
    .expect("alter_reserve");

    // CREATE VAULT

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

    let mut distribution_items = reserves
        .clone()
        .into_iter()
        .map(|reserve| DistributionItem {
            reserve,
            share_bps: 3300,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    distribution_items[2].share_bps = 3400; // make 100% complete sharing
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

    info!("deposit {} USDC lamports", amount);
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

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    assert_eq!(
        position.issued_vlp_amount().unwrap(),
        Decimal::from_i128_with_scale(100, 0).unwrap()
    );

    let withdraw_amount = 66 * LAMPORTS_PER_USDC;
    info!("withdraw {} VLP lamports", amount);
    withdraw(
        &mut ctx,
        position_keypair.pubkey(),
        vault_keypair.pubkey(),
        &owner_keypair,
        liquidity_usdc_mint,
        source_liquidity_wallet,
        superlendy_program_authority,
        withdraw_amount,
    )
    .await
    .expect("withdraw");

    let mut unpacked_reserves = vec![];
    for reserve_key in reserves {
        let reserve_acc = get_account(&mut ctx.banks_client, reserve_key)
            .await
            .expect("get reserve");
        let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
        unpacked_reserves.push(*reserve);
    }

    // CHECK RESERVES LP VALUE

    assert_eq!(unpacked_reserves[0].liquidity.available_amount, 0);
    assert_eq!(unpacked_reserves[1].liquidity.available_amount, 0);
    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        34 * LAMPORTS_PER_USDC // cause withdraw disabled from Risky2.USDC reserve
    );
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#106b5734a220809ead9defe51b4cdf10
#[tokio::test]
async fn withdraw_utilization_exceeded() {
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
        vault_keypair,
        curator_keypair,
        vaults_authority_keypair,
        position_keypair,
        main_pool,
        ..
    } = env_keys;

    // ALTER reserve.max_withdraw_utilization to 50%

    let reserve_acc = get_account(
        &mut ctx.banks_client,
        main_pool_reserve_usdc_keypair.pubkey(),
    )
    .await
    .expect("get reserve");
    let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
    let mut params = reserve.config;
    params.max_withdraw_utilization_bps = 5000; // 50%
    alter_reserve(
        &mut ctx,
        main_pool_reserve_usdc_keypair.pubkey(),
        main_pool.pubkey(),
        &pool_authority_keypair,
        curator_keypair.pubkey(),
        params,
        0,
    )
    .await
    .expect("alter_reserve");

    // CREATE VAULT

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

    let mut distribution_items = reserves
        .clone()
        .into_iter()
        .map(|reserve| DistributionItem {
            reserve,
            share_bps: 3300,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    distribution_items[2].share_bps = 3400; // make 100% complete sharing
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

    info!("deposit {} USDC lamports", amount);
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

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    assert_eq!(
        position.issued_vlp_amount().unwrap(),
        Decimal::from_i128_with_scale(100, 0).unwrap()
    );

    // DEPOSIT & LOCK COLLATERAL & BORROW TO SET UTILIZATION TO 30%

    let superlendy_main_position_kp = Keypair::new();
    superlendy_executor::create_position(
        &mut ctx,
        &superlendy_main_position_kp,
        main_pool.pubkey(),
        &owner_keypair,
    )
    .await
    .expect("create_position");

    let source_liq_wallet_usdc =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_usdc_mint);
    let lp_mint = find_lp_token_mint(&main_pool_reserve_usdc_keypair.pubkey()).0;
    let dest_lp_wallet_usdc = create_associated_token_account(&mut ctx, &owner_keypair, &lp_mint)
        .await
        .expect("create lp ata");
    let deposit_amount = 14142857;

    info!("deposit {} USDC lamports", deposit_amount);
    superlendy_executor::deposit_liquidity(
        &mut ctx,
        main_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        &owner_keypair,
        source_liq_wallet_usdc,
        dest_lp_wallet_usdc,
        deposit_amount,
    )
    .await
    .expect("deposit_liquidity");

    superlendy_executor::refresh_position(&mut ctx, superlendy_main_position_kp.pubkey())
        .await
        .expect("refresh position");

    info!("lock {} USDC lamports", deposit_amount);
    superlendy_executor::lock_collateral(
        &mut ctx,
        main_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        superlendy_main_position_kp.pubkey(),
        &owner_keypair,
        dest_lp_wallet_usdc,
        deposit_amount,
    )
    .await
    .expect("lock_collateral");

    let texture_fee_receiver =
        create_associated_token_account(&mut ctx, &texture_owner_keypair, &liquidity_usdc_mint)
            .await
            .expect("create texture fee receiver ata");
    let curator_fee_receiver =
        create_associated_token_account(&mut ctx, &pool_authority_keypair, &liquidity_usdc_mint)
            .await
            .expect("create curator fee receiver ata");
    let source_liq_wallet_usdc =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_usdc_mint);

    superlendy_executor::refresh_position(&mut ctx, superlendy_main_position_kp.pubkey())
        .await
        .expect("refresh position");

    let borrow_amount = deposit_amount - LAMPORTS_PER_USDC; // -1 USD
    info!("borrow {} USDC", borrow_amount);
    superlendy_executor::borrow(
        &mut ctx,
        superlendy_main_position_kp.pubkey(),
        main_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        main_pool.pubkey(),
        &owner_keypair,
        curator_keypair.pubkey(),
        curator_fee_receiver,
        texture_fee_receiver,
        source_liq_wallet_usdc,
        borrow_amount,
        1,
    )
    .await
    .expect("borrow");

    let source_liquidity_acc0 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
        .await
        .expect("get token acc");

    // WITHDRAW TO SET Main.USDC RESERVE UTILIZATION TO 50%
    let withdraw_amount = 57575757;
    info!("withdraw {} VLP lamports", amount);
    withdraw(
        &mut ctx,
        position_keypair.pubkey(),
        vault_keypair.pubkey(),
        &owner_keypair,
        liquidity_usdc_mint,
        source_liquidity_wallet,
        superlendy_program_authority,
        withdraw_amount,
    )
    .await
    .expect("withdraw");

    let source_liquidity_acc1 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
        .await
        .expect("get token acc");

    let mut unpacked_reserves = vec![];
    for reserve_key in reserves {
        let reserve_acc = get_account(&mut ctx.banks_client, reserve_key)
            .await
            .expect("get reserve");
        let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
        unpacked_reserves.push(*reserve);
    }

    // CHECK LIQUIDITY WALLET BALANCE

    assert_eq!(
        source_liquidity_acc1.amount,
        source_liquidity_acc0.amount + withdraw_amount - 2 // -2 because of rounding
    );

    // CHECK RESERVES LP VALUE

    // FIXME: TODO: First reserve should reach it's lowest possible liquidity value because of utilization limits
    assert_eq!(unpacked_reserves[0].liquidity.available_amount, 15000001);
    assert_eq!(unpacked_reserves[1].liquidity.available_amount, 14000001);
    assert_eq!(unpacked_reserves[2].liquidity.available_amount, 14424243);
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#54ffc3d66ab34c6e9d38d630ba88ad5e
#[tokio::test]
async fn withdraw_insufficient_funds() {
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
    let sol_price_feed = add_price_feed_acc(&mut runner, "sol-usd").await;
    // 1 USDC = 1 USD
    let usdc_price_feed = add_price_feed_acc(&mut runner, "usdc-usd").await;
    // 1 JUP = 0.7 USD
    add_price_feed_acc(&mut runner, "jup-usd").await;
    // 1 BONK = 0.00001 USD
    add_price_feed_acc(&mut runner, "bonk-usd").await;

    let irm_40 = add_curve_acc(&mut runner, "const-40-pct-acc").await;
    let irm_200 = add_curve_acc(&mut runner, "const-200-pct-acc").await;

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
        main_pool_reserves: (main_pool_reserve_sol_keypair, main_pool_reserve_usdc_keypair),
        risky1_pool_reserves: (_, risky1_pool_reserve_usdc_keypair),
        risky2_pool_reserves: (_, risky2_pool_reserve_usdc_keypair),
        vault_keypair,
        curator_keypair,
        vaults_authority_keypair,
        position_keypair,
        main_pool,
        ..
    } = env_keys;

    // ALTER reserve.irm to const-200-pct

    let reserve_acc = get_account(
        &mut ctx.banks_client,
        main_pool_reserve_usdc_keypair.pubkey(),
    )
    .await
    .expect("get reserve");
    let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
    let mut params = reserve.config;
    params.irm = irm_200;
    alter_reserve(
        &mut ctx,
        main_pool_reserve_usdc_keypair.pubkey(),
        main_pool.pubkey(),
        &pool_authority_keypair,
        curator_keypair.pubkey(),
        params,
        0,
    )
    .await
    .expect("alter_reserve");

    // CREATE VAULT

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

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

    info!("deposit {} USDC lamports", amount);
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

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    assert_eq!(
        position.issued_vlp_amount().unwrap(),
        Decimal::from_i128_with_scale(100, 0).unwrap()
    );

    // DEPOSIT & LOCK COLLATERAL 1 SOL & BORROW 16,5 USDC FROM MAIN USDC RESERVE

    let superlendy_main_position_kp = Keypair::new();
    superlendy_executor::create_position(
        &mut ctx,
        &superlendy_main_position_kp,
        main_pool.pubkey(),
        &owner_keypair,
    )
    .await
    .expect("create_position");

    let source_liq_wallet_sol =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_sol_mint);
    let lp_mint = find_lp_token_mint(&main_pool_reserve_sol_keypair.pubkey()).0;
    let dest_lp_wallet_sol = create_associated_token_account(&mut ctx, &owner_keypair, &lp_mint)
        .await
        .expect("create lp ata");
    let deposit_amount = LAMPORTS_PER_SOL;

    info!("deposit {} SOL lamports", deposit_amount);
    superlendy_executor::deposit_liquidity(
        &mut ctx,
        main_pool_reserve_sol_keypair.pubkey(),
        sol_price_feed,
        irm_40,
        &owner_keypair,
        source_liq_wallet_sol,
        dest_lp_wallet_sol,
        deposit_amount,
    )
    .await
    .expect("deposit_liquidity");

    superlendy_executor::refresh_position(&mut ctx, superlendy_main_position_kp.pubkey())
        .await
        .expect("refresh position");

    info!("lock {} SOL lamports", deposit_amount);
    superlendy_executor::lock_collateral(
        &mut ctx,
        main_pool_reserve_sol_keypair.pubkey(),
        sol_price_feed,
        irm_40,
        superlendy_main_position_kp.pubkey(),
        &owner_keypair,
        dest_lp_wallet_sol,
        deposit_amount,
    )
    .await
    .expect("lock_collateral");

    let texture_fee_receiver =
        create_associated_token_account(&mut ctx, &texture_owner_keypair, &liquidity_usdc_mint)
            .await
            .expect("create texture fee receiver ata");
    let curator_fee_receiver =
        create_associated_token_account(&mut ctx, &pool_authority_keypair, &liquidity_usdc_mint)
            .await
            .expect("create curator fee receiver ata");
    let source_liq_wallet_usdc =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_usdc_mint);

    superlendy_executor::refresh_position(&mut ctx, superlendy_main_position_kp.pubkey())
        .await
        .expect("refresh position");

    let borrow_amount = Decimal::from_i128_with_scale(165, 1)
        .unwrap()
        .to_lamports_round(USDC_DECIMALS)
        .unwrap();
    info!("borrow {} USDC", borrow_amount);
    superlendy_executor::borrow(
        &mut ctx,
        superlendy_main_position_kp.pubkey(),
        main_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm_200,
        main_pool.pubkey(),
        &owner_keypair,
        curator_keypair.pubkey(),
        curator_fee_receiver,
        texture_fee_receiver,
        source_liq_wallet_usdc,
        borrow_amount,
        1,
    )
    .await
    .expect("borrow");

    // 1 SOLANA YEAR LATER

    info!("wrap to slot {}", SLOTS_PER_YEAR + 1);
    ctx.warp_to_slot(SLOTS_PER_YEAR + 1).expect("warp_to_slot"); // solana_year = 63072000 slots

    superlendy_executor::refresh_position(&mut ctx, superlendy_main_position_kp.pubkey())
        .await
        .expect("refresh position");

    // WITHDRAW 100 VLP

    info!("withdraw {} VLP lamports", amount);
    let result = withdraw(
        &mut ctx,
        position_keypair.pubkey(),
        vault_keypair.pubkey(),
        &owner_keypair,
        liquidity_usdc_mint,
        source_liquidity_wallet,
        superlendy_program_authority,
        amount, // amount=100, can only write off 83,5 VLP only
    )
    .await;

    assert!(result.is_err());

    // TODO: improve test when "withdraw maximum" feature will be added
    // let source_liquidity_acc0 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
    //     .await
    //     .expect("get token acc");

    // let source_liquidity_acc1 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
    //     .await
    //     .expect("get token acc");

    // // CHECK VAULT POSITION

    // let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
    //     .await
    //     .expect("get vault position");
    // let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    // assert_eq!(
    //     position.issued_vlp_amount().unwrap(),
    //     Decimal::from_i128_with_scale(59351459427731699353, 18).unwrap()
    // ); // TODO: Contact Sergey Ukolov for fix

    // // CHECK LIQUIDITY WALLET BALANCE

    // assert_eq!(
    //     source_liquidity_acc1.amount,
    //     source_liquidity_acc0.amount
    //         + 16_500_000               // MainPool.USDC reserve, debt wasn't repaid
    //         + (33 * LAMPORTS_PER_USDC) // Risky1Pool.USDC reserve
    //         + (34 * LAMPORTS_PER_USDC) // Risky2Pool.USDC reserve
    //         - 3 // rounding
    // )
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#127789cb00074f8595c7507997d81071
#[tokio::test]
async fn withdraw_unbalanced_vault_no_deficit() {
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
    add_curve_acc(&mut runner, "const-200-pct-acc").await;

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
        position_keypair,
        ..
    } = env_keys;

    // CREATE VAULT

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

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

    info!("deposit {} USDC lamports", amount);
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

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    assert_eq!(
        position.issued_vlp_amount().unwrap(),
        Decimal::from_i128_with_scale(100, 0).unwrap()
    );

    // ALTER VAULT.distribution

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 300;
    distribution_bps[1] = 6300;
    distribution_bps[2] = 3400;
    params.distribution_bps = distribution_bps;

    info!("alter_vault distribution");
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

    // WITHDRAW 1 VLP

    let source_liquidity_acc0 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
        .await
        .expect("get token acc");

    let withdraw_amount = LAMPORTS_PER_USDC;
    info!("withdraw {} VLP lamports", withdraw_amount);
    withdraw(
        &mut ctx,
        position_keypair.pubkey(),
        vault_keypair.pubkey(),
        &owner_keypair,
        liquidity_usdc_mint,
        source_liquidity_wallet,
        superlendy_program_authority,
        withdraw_amount,
    )
    .await
    .expect("withdraw");

    let source_liquidity_acc1 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
        .await
        .expect("get token acc");

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    assert_eq!(
        position.issued_vlp_amount().unwrap(),
        Decimal::from_i128_with_scale(99, 0).unwrap()
    );

    assert_eq!(
        source_liquidity_acc1.amount,
        source_liquidity_acc0.amount + withdraw_amount
    );

    let mut unpacked_reserves = vec![];
    for reserve_key in reserves {
        let reserve_acc = get_account(&mut ctx.banks_client, reserve_key)
            .await
            .expect("get reserve");
        let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
        info!(
            "reserve {}: {}",
            reserve_key, reserve.liquidity.available_amount
        );
        unpacked_reserves.push(*reserve);
    }

    // CHECK RESERVES LP VALUE

    assert_eq!(
        unpacked_reserves[0].liquidity.available_amount,
        99 * LAMPORTS_PER_USDC / 100 * 3
    );
    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount,
        99 * LAMPORTS_PER_USDC / 100 * 63
    );
    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        99 * LAMPORTS_PER_USDC / 100 * 34
    );
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#8f66bb6cafd04a2f9795a8334a7b260d
#[tokio::test]
async fn withdraw_unbalanced_vault_has_deficit() {
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
    add_curve_acc(&mut runner, "const-200-pct-acc").await;

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
        position_keypair,
        ..
    } = env_keys;

    // CREATE VAULT

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

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

    info!("deposit {} USDC lamports", amount);
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

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    assert_eq!(
        position.issued_vlp_amount().unwrap(),
        Decimal::from_i128_with_scale(100, 0).unwrap()
    );

    // ALTER VAULT.distribution

    let mut distribution_items = reserves
        .clone()
        .into_iter()
        .map(|reserve| DistributionItem {
            reserve,
            share_bps: 3300,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    distribution_items[0].share_bps = 300; // 3%
    distribution_items[1].share_bps = 6300; // 63%
    distribution_items[2].share_bps = 3400; // 34%
    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    for (idx, item) in distribution_items.iter().enumerate() {
        distribution_bps[idx] = item.share_bps();
    }

    params.distribution_bps = distribution_bps;

    info!("alter_vault distribution");
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

    // WITHDRAW 80 VLP

    let source_liquidity_acc0 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
        .await
        .expect("get token acc");

    let withdraw_amount = 80 * LAMPORTS_PER_USDC;
    info!("withdraw {} VLP lamports", withdraw_amount);
    withdraw(
        &mut ctx,
        position_keypair.pubkey(),
        vault_keypair.pubkey(),
        &owner_keypair,
        liquidity_usdc_mint,
        source_liquidity_wallet,
        superlendy_program_authority,
        withdraw_amount,
    )
    .await
    .expect("withdraw");

    let source_liquidity_acc1 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
        .await
        .expect("get token acc");

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    assert_eq!(
        position.issued_vlp_amount().unwrap(),
        Decimal::from_i128_with_scale(20, 0).unwrap()
    );

    assert_eq!(
        source_liquidity_acc1.amount,
        source_liquidity_acc0.amount + withdraw_amount
    );

    let mut unpacked_reserves = vec![];
    for reserve_key in reserves {
        let reserve_acc = get_account(&mut ctx.banks_client, reserve_key)
            .await
            .expect("get reserve");
        let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
        unpacked_reserves.push(*reserve);
    }

    // CHECK RESERVES LP VALUE

    assert_eq!(
        unpacked_reserves[0].liquidity.available_amount,
        600000 // 0.6, 3%
    );
    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount,
        12600000 // 12.6, 63%
    );
    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        6800000 // 6.8, 34%
    );
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#10bb5734a2208095b4ddcc1dd42b2aa7
#[tokio::test]
async fn withdraw_deficit_lp_diff_value() {
    let mut runner = init_program_test();

    let admin_keypair = admin_keypair();
    let owner_keypair = borrow_keypair();
    let texture_config_keypair = texture_config_keypair();
    let price_feed_authority_keypair = price_feed_authority();
    let pool_authority_keypair = Keypair::new();
    let texture_owner_keypair = Keypair::new();

    runner.add_native_wallet(admin_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(owner_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(texture_owner_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(price_feed_authority_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(pool_authority_keypair.pubkey(), LAMPORTS);

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
        risky1_pool,
        risky2_pool,
        main_pool_reserves: (_, main_pool_reserve_usdc_keypair),
        risky1_pool_reserves: (_, risky1_pool_reserve_usdc_keypair),
        risky2_pool_reserves: (_, risky2_pool_reserve_usdc_keypair),
        vault_keypair,
        curator_keypair,
        vaults_authority_keypair,
        position_keypair,
        ..
    } = env_keys;

    // SET USDC PRICE TO 1.2 USD

    let now = Utc::now().timestamp();
    write_price(
        &mut ctx,
        usdc_price_feed,
        &price_feed_authority(),
        Decimal::from_i128_with_scale(12, 1).unwrap(),
        now - 3,
    )
    .await
    .expect("update price feed");

    let superlendy_risky1_position_kp = Keypair::new();
    superlendy_executor::create_position(
        &mut ctx,
        &superlendy_risky1_position_kp,
        risky1_pool.pubkey(),
        &owner_keypair,
    )
    .await
    .expect("create_position");
    let superlendy_risky2_position_kp = Keypair::new();
    superlendy_executor::create_position(
        &mut ctx,
        &superlendy_risky2_position_kp,
        risky2_pool.pubkey(),
        &owner_keypair,
    )
    .await
    .expect("create_position");

    let source_liq_wallet_usdc =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_usdc_mint);

    let texture_fee_receiver =
        create_associated_token_account(&mut ctx, &texture_owner_keypair, &liquidity_usdc_mint)
            .await
            .expect("create texture fee receiver ata");
    let curator_fee_receiver =
        create_associated_token_account(&mut ctx, &pool_authority_keypair, &liquidity_usdc_mint)
            .await
            .expect("create curator fee receiver ata");

    // DEPOSIT 1000 USDC & LOCK COLLATERAL INTO RISKY2 USDC RESERVE

    let lp_mint = find_lp_token_mint(&risky2_pool_reserve_usdc_keypair.pubkey()).0;
    let dest_lp_wallet = create_associated_token_account(&mut ctx, &owner_keypair, &lp_mint)
        .await
        .expect("create lp ata");

    info!("deposit 1000 USDC & lock collateral into risky2 usdc reserve");
    let deposit_usdc_amount = 1_000 * LAMPORTS_PER_USDC;
    superlendy_executor::deposit_liquidity(
        &mut ctx,
        risky2_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        &owner_keypair,
        source_liq_wallet_usdc,
        dest_lp_wallet,
        deposit_usdc_amount,
    )
    .await
    .expect("deposit_liquidity");

    superlendy_executor::refresh_position(&mut ctx, superlendy_risky2_position_kp.pubkey())
        .await
        .expect("refresh position");

    superlendy_executor::lock_collateral(
        &mut ctx,
        risky2_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        superlendy_risky2_position_kp.pubkey(),
        &owner_keypair,
        dest_lp_wallet,
        deposit_usdc_amount,
    )
    .await
    .expect("lock_collateral");

    superlendy_executor::refresh_position(&mut ctx, superlendy_risky2_position_kp.pubkey())
        .await
        .expect("refresh position");

    // BORROW 999 USDC FROM RISKY1 RESERVE TO INCREASE LP EXCHANGE RATE

    let borrow_amount = 999 * LAMPORTS_PER_USDC;
    info!("borrow {} usdc from risky2 reserve", borrow_amount);
    superlendy_executor::borrow(
        &mut ctx,
        superlendy_risky2_position_kp.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        risky2_pool.pubkey(),
        &owner_keypair,
        curator_keypair.pubkey(),
        curator_fee_receiver,
        texture_fee_receiver,
        source_liq_wallet_usdc,
        borrow_amount,
        1,
    )
    .await
    .expect("borrow");

    // 1 SOLANA YEAR LATER

    let mut slot = SLOTS_PER_YEAR;
    info!("wrap to slot {}", slot);
    ctx.warp_to_slot(slot).expect("warp_to_slot"); // solana_year = 63072000 slots

    superlendy_executor::refresh_position(&mut ctx, superlendy_risky1_position_kp.pubkey())
        .await
        .expect("refresh position");
    superlendy_executor::refresh_position(&mut ctx, superlendy_risky2_position_kp.pubkey())
        .await
        .expect("refresh position");

    // DEPOSIT 1000 USDC & LOCK COLLATERAL INTO RISKY1 USDC RESERVE

    let lp_mint = find_lp_token_mint(&risky1_pool_reserve_usdc_keypair.pubkey()).0;
    let dest_lp_wallet = create_associated_token_account(&mut ctx, &owner_keypair, &lp_mint)
        .await
        .expect("create lp ata");

    info!(
        "deposit {} USDC & lock collateral into risky1 usdc reserve",
        deposit_usdc_amount
    );
    superlendy_executor::deposit_liquidity(
        &mut ctx,
        risky1_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        &owner_keypair,
        source_liq_wallet_usdc,
        dest_lp_wallet,
        deposit_usdc_amount,
    )
    .await
    .expect("deposit_liquidity");

    superlendy_executor::refresh_position(&mut ctx, superlendy_risky1_position_kp.pubkey())
        .await
        .expect("refresh position");

    superlendy_executor::lock_collateral(
        &mut ctx,
        risky1_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        superlendy_risky1_position_kp.pubkey(),
        &owner_keypair,
        dest_lp_wallet,
        deposit_usdc_amount,
    )
    .await
    .expect("lock_collateral");

    superlendy_executor::refresh_position(&mut ctx, superlendy_risky1_position_kp.pubkey())
        .await
        .expect("refresh position");

    // BORROW 1000 USDC FROM RISKY1 RESERVE TO INCREASE LP EXCHANGE RATE

    info!(
        "borrow {} usdc from risky1 reserve to increase lp exchange rate",
        borrow_amount
    );
    superlendy_executor::borrow(
        &mut ctx,
        superlendy_risky1_position_kp.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        risky1_pool.pubkey(),
        &owner_keypair,
        curator_keypair.pubkey(),
        curator_fee_receiver,
        texture_fee_receiver,
        source_liq_wallet_usdc,
        borrow_amount,
        1,
    )
    .await
    .expect("borrow");

    // 1 SOLANA YEAR LATER

    slot += SLOTS_PER_YEAR;
    info!("wrap to slot {}", slot);
    ctx.warp_to_slot(slot).expect("warp_to_slot");

    superlendy_executor::refresh_position(&mut ctx, superlendy_risky1_position_kp.pubkey())
        .await
        .expect("refresh position");
    superlendy_executor::refresh_position(&mut ctx, superlendy_risky2_position_kp.pubkey())
        .await
        .expect("refresh position");

    // CREATE VAULT

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

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

    let mut unpacked_reserves = vec![];
    for reserve_key in reserves.clone() {
        let reserve_acc = get_account(&mut ctx.banks_client, reserve_key)
            .await
            .expect("get reserve");
        let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
        unpacked_reserves.push(*reserve);
    }

    // CHECK RESERVES LP VALUE

    assert_eq!(
        unpacked_reserves[0].liquidity.available_amount,
        33 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[0].lp_exchange_rate().unwrap().0,
        Decimal::from_i128_with_scale(1, 0).unwrap()
    );
    assert_eq!(
        unpacked_reserves[0].lp_market_price().unwrap(),
        Decimal::from_i128_with_scale(12, 1).unwrap() // 1 LP = 1,2 USD
    );

    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount,
        34 * LAMPORTS_PER_USDC
    );
    let risky1_lp_exchange_rate =
        Decimal::from_i128_with_scale(6705411104162025157742242851, 28).unwrap(); // 0.67, 1 LP = 1,49 USDC
    assert_eq!(
        unpacked_reserves[1].lp_exchange_rate().unwrap().0,
        risky1_lp_exchange_rate
    );
    assert_eq!(
        unpacked_reserves[1].lp_market_price().unwrap(),
        Decimal::from_i128_with_scale(17895994464159911212207487280, 28).unwrap() // 1 LP = 1,79 USD
    );

    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        35 * LAMPORTS_PER_USDC
    );
    let risky2_lp_exchange_rate =
        Decimal::from_i128_with_scale(4495765367697067810515704660, 28).unwrap(); // 0.45, 1 LP = 2,22 USDC
    assert_eq!(
        unpacked_reserves[2].lp_exchange_rate().unwrap().0,
        risky2_lp_exchange_rate
    );
    assert_eq!(
        unpacked_reserves[2].lp_market_price().unwrap(),
        dec!(2.6691784420562270031651645545) // 1 LP = 2,67 USD
    );

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    assert_eq!(
        position.issued_vlp_amount().unwrap(),
        Decimal::from_i128_with_scale(100, 0).unwrap()
    );

    // ALTER VAULT.distribution

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 300;
    distribution_bps[1] = 6300;
    distribution_bps[2] = 3400;
    params.distribution_bps = distribution_bps;

    info!("alter_vault distribution");
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

    // WITHDRAW 80 VLP

    let source_liquidity_acc0 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
        .await
        .expect("get token acc");

    let withdraw_amount = 80 * LAMPORTS_PER_USDC;
    info!("withdraw {} VLP lamports", withdraw_amount);
    withdraw(
        &mut ctx,
        position_keypair.pubkey(),
        vault_keypair.pubkey(),
        &owner_keypair,
        liquidity_usdc_mint,
        source_liquidity_wallet,
        superlendy_program_authority,
        withdraw_amount,
    )
    .await
    .expect("withdraw");

    let source_liquidity_acc1 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
        .await
        .expect("get token acc");

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    assert_eq!(position.issued_vlp_amount().unwrap(), dec!(20));

    assert_eq!(
        source_liquidity_acc1.amount,
        source_liquidity_acc0.amount + withdraw_amount - 6 // rounding
    );

    let mut unpacked_reserves = vec![];
    for reserve_key in reserves {
        let reserve_acc = get_account(&mut ctx.banks_client, reserve_key)
            .await
            .expect("get reserve");
        let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
        unpacked_reserves.push(*reserve);
    }

    // CHECK RESERVES LP MARKET VALUE

    assert_eq!(
        Decimal::from_lamports(
            unpacked_reserves[0].liquidity.available_amount,
            USDC_DECIMALS
        )
        .unwrap()
        .checked_mul(unpacked_reserves[0].lp_market_price().unwrap())
        .unwrap(),
        Decimal::from_i128_with_scale(7200000 + 12, 7).unwrap() // 0.6 * 1.2 = 0.72 USD (also add rounding loss)
    );
    assert_eq!(
        Decimal::from_lamports(
            unpacked_reserves[1].liquidity.available_amount,
            USDC_DECIMALS
        )
        .unwrap()
        .checked_mul(unpacked_reserves[1].lp_market_price().unwrap())
        .unwrap(),
        dec!(24.338557848723362410155102321)
    );
    assert_eq!(
        Decimal::from_lamports(
            unpacked_reserves[2].liquidity.available_amount,
            USDC_DECIMALS
        )
        .unwrap()
        .checked_mul(unpacked_reserves[2].lp_market_price().unwrap())
        .unwrap(),
        dec!(20.819597188450666781980067826)
    );
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#10bb5734a22080b49e99fcd43bd40640
#[tokio::test]
async fn withdraw_deficit_lp_diff_value_insuff_funds() {
    let mut runner = init_program_test();

    let admin_keypair = admin_keypair();
    let owner_keypair = borrow_keypair();
    let texture_config_keypair = texture_config_keypair();
    let price_feed_authority_keypair = price_feed_authority();
    let pool_authority_keypair = Keypair::new();
    let texture_owner_keypair = Keypair::new();

    runner.add_native_wallet(admin_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(owner_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(texture_owner_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(price_feed_authority_keypair.pubkey(), LAMPORTS);
    runner.add_native_wallet(pool_authority_keypair.pubkey(), LAMPORTS);

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
        risky1_pool,
        risky2_pool,
        main_pool_reserves: (_, main_pool_reserve_usdc_keypair),
        risky1_pool_reserves: (_, risky1_pool_reserve_usdc_keypair),
        risky2_pool_reserves: (_, risky2_pool_reserve_usdc_keypair),
        vault_keypair,
        curator_keypair,
        vaults_authority_keypair,
        position_keypair,
        ..
    } = env_keys;

    // SET USDC PRICE TO 1.2 USD

    let now = Utc::now().timestamp();
    write_price(
        &mut ctx,
        usdc_price_feed,
        &price_feed_authority(),
        Decimal::from_i128_with_scale(12, 1).unwrap(),
        now - 3,
    )
    .await
    .expect("update price feed");

    // CREATE VAULT

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

    let mut distribution_items = reserves
        .clone()
        .into_iter()
        .map(|reserve| DistributionItem {
            reserve,
            share_bps: 3300,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    distribution_items[2].share_bps = 3400; // make 100% complete sharing
    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    for (idx, item) in distribution_items.iter().enumerate() {
        distribution_bps[idx] = item.share_bps();
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

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    assert_eq!(
        position.issued_vlp_amount().unwrap(),
        Decimal::from_i128_with_scale(100, 0).unwrap()
    );

    // DEPOSIT 1000 USDC & LOCK COLLATERAL INTO RISKY2 USDC RESERVE

    let superlendy_risky1_position_kp = Keypair::new();
    superlendy_executor::create_position(
        &mut ctx,
        &superlendy_risky1_position_kp,
        risky1_pool.pubkey(),
        &owner_keypair,
    )
    .await
    .expect("create_position");
    let superlendy_risky2_position_kp = Keypair::new();
    superlendy_executor::create_position(
        &mut ctx,
        &superlendy_risky2_position_kp,
        risky2_pool.pubkey(),
        &owner_keypair,
    )
    .await
    .expect("create_position");

    let source_liq_wallet_usdc =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_usdc_mint);

    let texture_fee_receiver =
        create_associated_token_account(&mut ctx, &texture_owner_keypair, &liquidity_usdc_mint)
            .await
            .expect("create texture fee receiver ata");
    let curator_fee_receiver =
        create_associated_token_account(&mut ctx, &pool_authority_keypair, &liquidity_usdc_mint)
            .await
            .expect("create curator fee receiver ata");

    let lp_mint = find_lp_token_mint(&risky2_pool_reserve_usdc_keypair.pubkey()).0;
    let dest_lp_wallet = create_associated_token_account(&mut ctx, &owner_keypair, &lp_mint)
        .await
        .expect("create lp ata");

    info!("deposit 1000 USDC & lock collateral into risky2 usdc reserve");
    let deposit_usdc_amount = 1_000 * LAMPORTS_PER_USDC;
    superlendy_executor::deposit_liquidity(
        &mut ctx,
        risky2_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        &owner_keypair,
        source_liq_wallet_usdc,
        dest_lp_wallet,
        deposit_usdc_amount,
    )
    .await
    .expect("deposit_liquidity");

    superlendy_executor::refresh_position(&mut ctx, superlendy_risky2_position_kp.pubkey())
        .await
        .expect("refresh position");

    superlendy_executor::lock_collateral(
        &mut ctx,
        risky2_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        superlendy_risky2_position_kp.pubkey(),
        &owner_keypair,
        dest_lp_wallet,
        deposit_usdc_amount,
    )
    .await
    .expect("lock_collateral");

    superlendy_executor::refresh_position(&mut ctx, superlendy_risky2_position_kp.pubkey())
        .await
        .expect("refresh position");

    // BORROW 999 USDC FROM RISKY1 RESERVE TO INCREASE LP EXCHANGE RATE

    let borrow_amount = 999 * LAMPORTS_PER_USDC;
    info!("borrow {} usdc from risky2 reserve", borrow_amount);
    superlendy_executor::borrow(
        &mut ctx,
        superlendy_risky2_position_kp.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        risky2_pool.pubkey(),
        &owner_keypair,
        curator_keypair.pubkey(),
        curator_fee_receiver,
        texture_fee_receiver,
        source_liq_wallet_usdc,
        borrow_amount,
        1,
    )
    .await
    .expect("borrow");

    // 1 SOLANA YEAR LATER

    let mut slot = SLOTS_PER_YEAR;
    info!("wrap to slot {}", slot);
    ctx.warp_to_slot(slot).expect("warp_to_slot"); // solana_year = 63072000 slots

    superlendy_executor::refresh_position(&mut ctx, superlendy_risky1_position_kp.pubkey())
        .await
        .expect("refresh position");
    superlendy_executor::refresh_position(&mut ctx, superlendy_risky2_position_kp.pubkey())
        .await
        .expect("refresh position");

    // DEPOSIT 1000 USDC & LOCK COLLATERAL INTO RISKY1 USDC RESERVE

    let lp_mint = find_lp_token_mint(&risky1_pool_reserve_usdc_keypair.pubkey()).0;
    let dest_lp_wallet = create_associated_token_account(&mut ctx, &owner_keypair, &lp_mint)
        .await
        .expect("create lp ata");

    info!(
        "deposit {} USDC & lock collateral into risky1 usdc reserve",
        deposit_usdc_amount
    );
    superlendy_executor::deposit_liquidity(
        &mut ctx,
        risky1_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        &owner_keypair,
        source_liq_wallet_usdc,
        dest_lp_wallet,
        deposit_usdc_amount,
    )
    .await
    .expect("deposit_liquidity");

    superlendy_executor::refresh_position(&mut ctx, superlendy_risky1_position_kp.pubkey())
        .await
        .expect("refresh position");

    superlendy_executor::lock_collateral(
        &mut ctx,
        risky1_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        superlendy_risky1_position_kp.pubkey(),
        &owner_keypair,
        dest_lp_wallet,
        deposit_usdc_amount,
    )
    .await
    .expect("lock_collateral");

    superlendy_executor::refresh_position(&mut ctx, superlendy_risky1_position_kp.pubkey())
        .await
        .expect("refresh position");

    // BORROW 1000 USDC FROM RISKY1 RESERVE TO INCREASE LP EXCHANGE RATE

    info!(
        "borrow {} usdc from risky1 reserve to increase lp exchange rate",
        borrow_amount
    );
    superlendy_executor::borrow(
        &mut ctx,
        superlendy_risky1_position_kp.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        risky1_pool.pubkey(),
        &owner_keypair,
        curator_keypair.pubkey(),
        curator_fee_receiver,
        texture_fee_receiver,
        source_liq_wallet_usdc,
        borrow_amount,
        1,
    )
    .await
    .expect("borrow");

    // 1 SOLANA YEAR LATER

    slot += SLOTS_PER_YEAR;
    info!("wrap to slot {}", slot);
    ctx.warp_to_slot(slot).expect("warp_to_slot");

    superlendy_executor::refresh_position(&mut ctx, superlendy_risky1_position_kp.pubkey())
        .await
        .expect("refresh position");
    superlendy_executor::refresh_position(&mut ctx, superlendy_risky2_position_kp.pubkey())
        .await
        .expect("refresh position");

    let mut unpacked_reserves = vec![];
    for reserve_key in reserves.clone() {
        let reserve_acc = get_account(&mut ctx.banks_client, reserve_key)
            .await
            .expect("get reserve");
        let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
        unpacked_reserves.push(*reserve);
    }

    // CHECK RESERVES LP EXCHANGE RATE

    assert_eq!(
        unpacked_reserves[0].liquidity.available_amount,
        33 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[0].lp_exchange_rate().unwrap().0,
        Decimal::from_i128_with_scale(1, 0).unwrap()
    );
    assert_eq!(
        unpacked_reserves[0].lp_market_price().unwrap(),
        Decimal::from_i128_with_scale(12, 1).unwrap() // 1 LP = 1,2 USD
    );

    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount,
        34 * LAMPORTS_PER_USDC
    );
    let risky1_lp_exchange_rate =
        Decimal::from_i128_with_scale(6776735053192183021717526565, 28).unwrap(); // 0.677, 1 LP = 1,477 USDC
    assert_eq!(
        unpacked_reserves[1].lp_exchange_rate().unwrap().0,
        risky1_lp_exchange_rate
    );
    assert_eq!(
        unpacked_reserves[1].lp_market_price().unwrap(),
        Decimal::from_i128_with_scale(17707642258121625255066795740, 28).unwrap() // 1 LP = 1,77 USD
    );

    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        35 * LAMPORTS_PER_USDC
    );
    let risky2_lp_exchange_rate =
        Decimal::from_i128_with_scale(4578634209961708997103692424, 28).unwrap(); // 0.45, 1 LP = 2,22 USDC
    assert_eq!(
        unpacked_reserves[2].lp_exchange_rate().unwrap().0,
        risky2_lp_exchange_rate
    );
    assert_eq!(
        unpacked_reserves[2].lp_market_price().unwrap(),
        dec!(2.6208688988283158131114119920) // 1 LP = 2,62 USD
    );

    // ALTER VAULT.distribution

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 300;
    distribution_bps[1] = 6300;
    distribution_bps[2] = 3400;
    params.distribution_bps = distribution_bps;

    info!("alter_vault distribution");
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

    // WITHDRAW 80 VLP

    let withdraw_amount = 80 * LAMPORTS_PER_USDC;
    info!("withdraw {} VLP lamports", withdraw_amount);
    let result = withdraw(
        &mut ctx,
        position_keypair.pubkey(),
        vault_keypair.pubkey(),
        &owner_keypair,
        liquidity_usdc_mint,
        source_liquidity_wallet,
        superlendy_program_authority,
        withdraw_amount,
    )
    .await;

    // Impossible to withdraw 80 VLP because of high exchange rate
    // Not enough liquidity
    assert!(result.is_err());

    // TODO: Improve test when "withdraw maximum" feature will be implemented
    // let source_liquidity_acc0 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
    //     .await
    //     .expect("get token acc");

    // let source_liquidity_acc1 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
    //     .await
    //     .expect("get token acc");

    // // CHECK VAULT POSITION

    // let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
    //     .await
    //     .expect("get vault position");
    // let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    // assert_eq!(
    //     position.issued_vlp_amount().unwrap(),
    //     Decimal::from_i128_with_scale(35901537788920594789, 18).unwrap()
    // ); // TODO: Contact Sergey Ukolov for fix

    // assert_eq!(
    //     source_liquidity_acc1.amount,
    //     source_liquidity_acc0.amount + (100 * LAMPORTS_PER_USDC) - 3 // rounding
    // );

    // let mut unpacked_reserves = vec![];
    // for reserve_key in reserves {
    //     let reserve_acc = get_account(&mut ctx.banks_client, reserve_key)
    //         .await
    //         .expect("get reserve");
    //     let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
    //     unpacked_reserves.push(*reserve);
    // }

    // // CHECK RESERVES LP MARKET VALUE

    // assert_eq!(unpacked_reserves[0].liquidity.available_amount, 0);
    // assert_eq!(unpacked_reserves[1].liquidity.available_amount, 1); // crypto dust
    // assert_eq!(unpacked_reserves[2].liquidity.available_amount, 2);
}

// TODO: add test description
#[tokio::test]
async fn withdraw_insufficient_vlp() {
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
    let irm_200 = add_curve_acc(&mut runner, "const-200-pct-acc").await;

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
        position_keypair,
        position_0_keypair,
        main_pool,
        ..
    } = env_keys;

    // ALTER reserve.irm to const-200-pct

    let reserve_acc = get_account(
        &mut ctx.banks_client,
        main_pool_reserve_usdc_keypair.pubkey(),
    )
    .await
    .expect("get reserve");
    let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
    let mut params = reserve.config;
    params.irm = irm_200;
    alter_reserve(
        &mut ctx,
        main_pool_reserve_usdc_keypair.pubkey(),
        main_pool.pubkey(),
        &pool_authority_keypair,
        curator_keypair.pubkey(),
        params,
        0,
    )
    .await
    .expect("alter_reserve");

    // CREATE VAULT

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

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

    // CREATE POSITION 0

    create_position(
        &mut ctx,
        &position_0_keypair,
        vault_keypair.pubkey(),
        &owner_keypair,
    )
    .await
    .expect("create_position");

    // DEPOSIT TO 0

    let source_liquidity_wallet =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_usdc_mint);
    let superlendy_program_authority = find_program_authority().0;

    let amount = 100 * LAMPORTS_PER_USDC;

    info!("deposit {} USDC lamports", amount);
    deposit(
        &mut ctx,
        position_0_keypair.pubkey(),
        vault_keypair.pubkey(),
        &owner_keypair,
        liquidity_usdc_mint,
        source_liquidity_wallet,
        superlendy_program_authority,
        amount,
    )
    .await
    .expect("deposit");

    // CREATE POSITION 1

    create_position(
        &mut ctx,
        &position_keypair,
        vault_keypair.pubkey(),
        &owner_keypair,
    )
    .await
    .expect("create_position");

    // DEPOSIT TO 1

    let source_liquidity_wallet =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_usdc_mint);
    let superlendy_program_authority = find_program_authority().0;

    let amount = 100 * LAMPORTS_PER_USDC;

    info!("deposit {} USDC lamports", amount);
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

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    assert_eq!(
        position.issued_vlp_amount().unwrap(),
        Decimal::from_i128_with_scale(100, 0).unwrap()
    );

    // WITHDRAW 200 VLP

    let amount = 200 * LAMPORTS_PER_USDC;

    info!("withdraw {} VLP lamports", amount);
    let result = withdraw(
        &mut ctx,
        position_keypair.pubkey(),
        vault_keypair.pubkey(),
        &owner_keypair,
        liquidity_usdc_mint,
        source_liquidity_wallet,
        superlendy_program_authority,
        amount, // amount=200, can write off 100 VLP only
    )
    .await;

    assert_matches::assert_matches!(result, Err(BanksClientError::TransactionError(_)));
}

// TODO: test description
#[tokio::test]
async fn withdraw_max() {
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
    let sol_price_feed = add_price_feed_acc(&mut runner, "sol-usd").await;
    // 1 USDC = 1 USD
    let usdc_price_feed = add_price_feed_acc(&mut runner, "usdc-usd").await;
    // 1 JUP = 0.7 USD
    add_price_feed_acc(&mut runner, "jup-usd").await;
    // 1 BONK = 0.00001 USD
    add_price_feed_acc(&mut runner, "bonk-usd").await;

    let irm_40 = add_curve_acc(&mut runner, "const-40-pct-acc").await;
    let irm_200 = add_curve_acc(&mut runner, "const-200-pct-acc").await;

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
        main_pool_reserves: (main_pool_reserve_sol_keypair, main_pool_reserve_usdc_keypair),
        risky1_pool_reserves: (_, risky1_pool_reserve_usdc_keypair),
        risky2_pool_reserves: (_, risky2_pool_reserve_usdc_keypair),
        vault_keypair,
        curator_keypair,
        vaults_authority_keypair,
        position_keypair,
        main_pool,
        ..
    } = env_keys;

    // ALTER reserve.irm to const-200-pct

    let reserve_acc = get_account(
        &mut ctx.banks_client,
        main_pool_reserve_usdc_keypair.pubkey(),
    )
    .await
    .expect("get reserve");
    let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
    let mut params = reserve.config;
    params.irm = irm_200;
    alter_reserve(
        &mut ctx,
        main_pool_reserve_usdc_keypair.pubkey(),
        main_pool.pubkey(),
        &pool_authority_keypair,
        curator_keypair.pubkey(),
        params,
        0,
    )
    .await
    .expect("alter_reserve");

    // CREATE VAULT

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

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

    info!("deposit {} USDC lamports", amount);
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

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    assert_eq!(
        position.issued_vlp_amount().unwrap(),
        Decimal::from_i128_with_scale(100, 0).unwrap()
    );

    // DEPOSIT & LOCK COLLATERAL 1 SOL & BORROW 16,5 USDC FROM MAIN USDC RESERVE

    let superlendy_main_position_kp = Keypair::new();
    superlendy_executor::create_position(
        &mut ctx,
        &superlendy_main_position_kp,
        main_pool.pubkey(),
        &owner_keypair,
    )
    .await
    .expect("create_position");

    let source_liq_wallet_sol =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_sol_mint);
    let lp_mint = find_lp_token_mint(&main_pool_reserve_sol_keypair.pubkey()).0;
    let dest_lp_wallet_sol = create_associated_token_account(&mut ctx, &owner_keypair, &lp_mint)
        .await
        .expect("create lp ata");
    let deposit_amount = LAMPORTS_PER_SOL;

    info!("deposit {} SOL lamports", deposit_amount);
    superlendy_executor::deposit_liquidity(
        &mut ctx,
        main_pool_reserve_sol_keypair.pubkey(),
        sol_price_feed,
        irm_40,
        &owner_keypair,
        source_liq_wallet_sol,
        dest_lp_wallet_sol,
        deposit_amount,
    )
    .await
    .expect("deposit_liquidity");

    superlendy_executor::refresh_position(&mut ctx, superlendy_main_position_kp.pubkey())
        .await
        .expect("refresh position");

    info!("lock {} SOL lamports", deposit_amount);
    superlendy_executor::lock_collateral(
        &mut ctx,
        main_pool_reserve_sol_keypair.pubkey(),
        sol_price_feed,
        irm_40,
        superlendy_main_position_kp.pubkey(),
        &owner_keypair,
        dest_lp_wallet_sol,
        deposit_amount,
    )
    .await
    .expect("lock_collateral");

    let texture_fee_receiver =
        create_associated_token_account(&mut ctx, &texture_owner_keypair, &liquidity_usdc_mint)
            .await
            .expect("create texture fee receiver ata");
    let curator_fee_receiver =
        create_associated_token_account(&mut ctx, &pool_authority_keypair, &liquidity_usdc_mint)
            .await
            .expect("create curator fee receiver ata");
    let source_liq_wallet_usdc =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_usdc_mint);

    superlendy_executor::refresh_position(&mut ctx, superlendy_main_position_kp.pubkey())
        .await
        .expect("refresh position");

    let borrow_amount = Decimal::from_i128_with_scale(165, 1)
        .unwrap()
        .to_lamports_round(USDC_DECIMALS)
        .unwrap();
    info!("borrow {} USDC", borrow_amount);
    superlendy_executor::borrow(
        &mut ctx,
        superlendy_main_position_kp.pubkey(),
        main_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm_200,
        main_pool.pubkey(),
        &owner_keypair,
        curator_keypair.pubkey(),
        curator_fee_receiver,
        texture_fee_receiver,
        source_liq_wallet_usdc,
        borrow_amount,
        1,
    )
    .await
    .expect("borrow");

    // 1 SOLANA YEAR LATER

    info!("wrap to slot {}", SLOTS_PER_YEAR + 1);
    ctx.warp_to_slot(SLOTS_PER_YEAR + 1).expect("warp_to_slot"); // solana_year = 63072000 slots

    superlendy_executor::refresh_position(&mut ctx, superlendy_main_position_kp.pubkey())
        .await
        .expect("refresh position");

    // CHECK REPAYMENT

    let reserve_acc = get_account(
        &mut ctx.banks_client,
        main_pool_reserve_usdc_keypair.pubkey(),
    )
    .await
    .expect("get reserve");
    let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
    let borrowed_amount = reserve.liquidity.borrowed_amount().unwrap();
    let repayment = dec!(121.919421766316246193); // 121.9 USDC
    info!("repayment = borrow + interest = {}", borrowed_amount);
    assert_eq!(borrowed_amount, repayment);

    // REPAY

    let source_liquidity_acc0 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
        .await
        .expect("get token acc");

    repay(
        &mut ctx,
        superlendy_main_position_kp.pubkey(),
        main_pool_reserve_usdc_keypair.pubkey(),
        &owner_keypair,
        source_liq_wallet_usdc,
        MAX_AMOUNT,
    )
    .await
    .expect("repay");

    let source_liquidity_acc1 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
        .await
        .expect("get token acc");

    assert_eq!(
        source_liquidity_acc1.amount,
        source_liquidity_acc0.amount - repayment.to_lamports_round(USDC_DECIMALS).unwrap()
    );

    // WITHDRAW MAX

    let source_liquidity_acc0 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
        .await
        .expect("get token acc");

    info!("withdraw maximum VLP");
    withdraw(
        &mut ctx,
        position_keypair.pubkey(),
        vault_keypair.pubkey(),
        &owner_keypair,
        liquidity_usdc_mint,
        source_liquidity_wallet,
        superlendy_program_authority,
        u64::MAX,
    )
    .await
    .expect("withdraw");

    let source_liquidity_acc1 = get_token_account(&mut ctx.banks_client, source_liquidity_wallet)
        .await
        .expect("get token acc");

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    assert_eq!(position.issued_vlp_amount().unwrap(), Decimal::ZERO);

    // CHECK LIQUIDITY WALLET BALANCE

    assert_eq!(
        source_liquidity_acc1.amount,
        source_liquidity_acc0.amount
            + (
            Decimal::from_i128_with_scale(165, 1)
                .unwrap()
                .to_lamports_round(USDC_DECIMALS)
                .unwrap()
            + repayment.to_lamports_round(USDC_DECIMALS).unwrap()
        )       // 16,5 + 121,9 USDC from MainPool.USDC reserve
            + (33 * LAMPORTS_PER_USDC) // Risky1Pool.USDC reserve
            + (34 * LAMPORTS_PER_USDC) // Risky2Pool.USDC reserve
            - 5 // rounding
    )
}
