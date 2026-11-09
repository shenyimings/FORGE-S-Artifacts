#![cfg(feature = "test-bpf")]

use bytemuck::Zeroable;
use std::str::FromStr;
use utils::USDC_DECIMALS;

use price_proxy::state::utils::str_to_array;
use solana_program::pubkey::Pubkey;
use solana_program_test::ProgramTestBanksClientExt;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use super_lendy::instruction::RefreshReserve;
use super_lendy::pda::{find_liquidity_supply, find_lp_token_mint, find_program_authority};
use super_lendy::state::reserve::Reserve;
use super_lendy::state::SLOTS_PER_YEAR;
use texture_common::account::PodAccount;
use texture_common::math::{CheckedMul, Decimal};
use tracing::info;
use vault::instruction::Deposit;
use vault::pda::{find_vault_liquidity_buffer, find_vault_lp_token_storage};
use vault::state::position::Position;
use vault::state::vault::{DistributionItem, Vault, VaultParams, VAULT_DISTRIBUTION_MAX_ITEMS};

use crate::utils::setup::{setup_env, setup_vault, EnvKeys};
use crate::utils::vault_executor::{create_position, deposit};
use crate::utils::{
    add_curve_acc, add_price_feed_acc, admin_keypair, borrow_keypair,
    create_associated_token_account, get_account, get_token_account, init_program_test,
    init_token_accounts, price_feed_authority, superlendy_executor, texture_config_keypair,
    update_prices, Runner, LAMPORTS, LAMPORTS_PER_USDC,
};

pub mod utils;

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#44f78e2a7839421b894283a876f3a59d
#[tokio::test]
async fn deposit_lp_equal_value() {
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

    // CHECK RESERVES DEPOSITED VALUE

    let mut unpacked_reserves = vec![];
    for reserve in reserves {
        let reserve_acc = get_account(&mut ctx.banks_client, reserve)
            .await
            .expect("get reserve");
        let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
        unpacked_reserves.push(*reserve);
    }

    assert_eq!(
        unpacked_reserves[0].liquidity.available_amount,
        33 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[0].collateral.lp_total_supply,
        33 * LAMPORTS_PER_USDC
    );

    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount,
        33 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[1].collateral.lp_total_supply,
        33 * LAMPORTS_PER_USDC
    );

    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        34 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[2].collateral.lp_total_supply,
        34 * LAMPORTS_PER_USDC
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
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#33786196437d4dab9420a3ce8d7e8081
#[tokio::test]
async fn deposit_lp_diff_value() {
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

    // DEPOSIT 1000 USDC & LOCK COLLATERAL INTO RISKY1 USDC RESERVE

    let superlendy_risky1_position_kp = Keypair::new();
    superlendy_executor::create_position(
        &mut ctx,
        &superlendy_risky1_position_kp,
        risky1_pool.pubkey(),
        &owner_keypair,
    )
    .await
    .expect("create_position");

    let lp_mint = find_lp_token_mint(&risky1_pool_reserve_usdc_keypair.pubkey()).0;
    let dest_lp_wallet_sol = create_associated_token_account(&mut ctx, &owner_keypair, &lp_mint)
        .await
        .expect("create lp ata");
    let source_liq_wallet_usdc =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_usdc_mint);

    let deposit_usdc_amount = 1_000 * LAMPORTS_PER_USDC;

    info!("deposit 1000 USDC & lock collateral into risky1 usdc reserve");
    superlendy_executor::deposit_liquidity(
        &mut ctx,
        risky1_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        &owner_keypair,
        source_liq_wallet_usdc,
        dest_lp_wallet_sol,
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
        dest_lp_wallet_sol,
        deposit_usdc_amount,
    )
    .await
    .expect("lock_collateral");

    superlendy_executor::refresh_position(&mut ctx, superlendy_risky1_position_kp.pubkey())
        .await
        .expect("refresh position");

    let texture_fee_receiver =
        create_associated_token_account(&mut ctx, &texture_owner_keypair, &liquidity_usdc_mint)
            .await
            .expect("create texture fee receiver ata");
    let curator_fee_receiver =
        create_associated_token_account(&mut ctx, &pool_authority_keypair, &liquidity_usdc_mint)
            .await
            .expect("create curator fee receiver ata");
    let borrow_amount = 500 * LAMPORTS_PER_USDC;

    // BORROW 500 USDC FROM RISKY1 RESERVE TO INCREASE LP EXCHANGE RATE
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

    // DEPOSIT 1000 USDC & LOCK COLLATERAL INTO RISKY2 USDC RESERVE

    let superlendy_risky2_position_kp = Keypair::new();
    superlendy_executor::create_position(
        &mut ctx,
        &superlendy_risky2_position_kp,
        risky2_pool.pubkey(),
        &owner_keypair,
    )
    .await
    .expect("create_position");

    let lp_mint = find_lp_token_mint(&risky2_pool_reserve_usdc_keypair.pubkey()).0;
    let dest_lp_wallet_sol = create_associated_token_account(&mut ctx, &owner_keypair, &lp_mint)
        .await
        .expect("create lp ata");

    info!("deposit 1000 USDC & lock collateral into risky2 usdc reserve");
    superlendy_executor::deposit_liquidity(
        &mut ctx,
        risky2_pool_reserve_usdc_keypair.pubkey(),
        usdc_price_feed,
        irm,
        &owner_keypair,
        source_liq_wallet_usdc,
        dest_lp_wallet_sol,
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
        dest_lp_wallet_sol,
        deposit_usdc_amount,
    )
    .await
    .expect("lock_collateral");

    superlendy_executor::refresh_position(&mut ctx, superlendy_risky2_position_kp.pubkey())
        .await
        .expect("refresh position");

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

    info!("wrap to slot {}", SLOTS_PER_YEAR);
    ctx.warp_to_slot(SLOTS_PER_YEAR).expect("warp_to_slot"); // solana_year = 63072000 slots

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

    let mut lp_token_storages = vec![];
    let mut unpacked_reserves = vec![];
    for reserve_key in reserves {
        let reserve_acc = get_account(&mut ctx.banks_client, reserve_key)
            .await
            .expect("get reserve");
        let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
        unpacked_reserves.push(*reserve);

        let (lp_token_mint_key, _) = find_lp_token_mint(&reserve_key);
        let (lp_token_storage_key, _) =
            find_vault_lp_token_storage(&vault_keypair.pubkey(), &lp_token_mint_key);
        let lp_token_storage = get_token_account(&mut ctx.banks_client, lp_token_storage_key)
            .await
            .expect("get token acc");
        lp_token_storages.push(lp_token_storage);
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
        unpacked_reserves[1].liquidity.available_amount,
        (500 + 33) * LAMPORTS_PER_USDC // 500 USDC - initial deposit after borrow
    );
    let risky1_lp_exchange_rate =
        Decimal::from_i128_with_scale(8026246829994330706504214340, 28).unwrap(); // 0.8
    assert_eq!(
        unpacked_reserves[1].lp_exchange_rate().unwrap().0,
        risky1_lp_exchange_rate
    );

    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        35 * LAMPORTS_PER_USDC
    );
    let risky2_lp_exchange_rate =
        Decimal::from_i128_with_scale(6705411144987800058679291495, 28).unwrap(); // 0.67
    assert_eq!(
        unpacked_reserves[2].lp_exchange_rate().unwrap().0,
        risky2_lp_exchange_rate
    );

    // CHECK LP TOKEN STORAGES BALANCE

    assert_eq!(lp_token_storages[0].amount, 33 * LAMPORTS_PER_USDC);
    assert_eq!(
        lp_token_storages[1].amount,
        Decimal::from_i128_with_scale(33, 0)
            .unwrap()
            .checked_mul(risky1_lp_exchange_rate)
            .unwrap()
            .to_lamports_floor(USDC_DECIMALS)
            .unwrap()
    );
    assert_eq!(
        lp_token_storages[2].amount,
        Decimal::from_i128_with_scale(34, 0)
            .unwrap()
            .checked_mul(risky2_lp_exchange_rate)
            .unwrap()
            .to_lamports_floor(USDC_DECIMALS)
            .unwrap()
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
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#3df1e93e338245098dd8cd169d8fec7e
#[tokio::test]
async fn deposit_by_other_position_owner() {
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

    info!("deposit by other position owner");
    let result = deposit(
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
    assert!(result.is_err())
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#7fada32ea70349e781c6415ffba6aeb8
#[tokio::test]
async fn deposit_by_other_vault() {
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

    // CREATE Vault1, Vault2

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

    info!("deposit to Vault2 with position of Vault1");
    let result = deposit(
        &mut ctx,
        position_keypair.pubkey(),
        vault2_keypair.pubkey(),
        &owner_keypair,
        liquidity_usdc_mint,
        source_liquidity_wallet,
        superlendy_program_authority,
        amount,
    )
    .await;
    assert!(result.is_err())
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#15f0a6d731ca45f2a05d46832c309df9
#[tokio::test]
async fn deposit_insufficient_accounts_list() {
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

    info!("deposit with insufficient reserves count");

    let blockhash = ctx
        .banks_client
        .get_new_latest_blockhash(&ctx.last_blockhash)
        .await
        .expect("get latest blockhash");

    let vault_acc = get_account(&mut ctx.banks_client, vault_keypair.pubkey())
        .await
        .expect("getting Vault account");
    let unpacked_vault = Vault::try_from_bytes(&vault_acc.data).expect("unpacking Vault");

    let mut reserves = vec![];
    let mut reserve_liquidity_supplies = vec![];
    let mut lp_token_mints = vec![];
    let mut lp_token_storages = vec![];
    let mut ixs = vec![];

    for item in unpacked_vault.distribution.0 {
        if item.is_vacant() {
            break;
        }
        let reserve_key = item.reserve;
        let (lp_supply, _) = find_liquidity_supply(&reserve_key);
        let (lp_token_mint_key, _) = find_lp_token_mint(&reserve_key);
        let (lp_token_storage_key, _) =
            find_vault_lp_token_storage(&vault_keypair.pubkey(), &lp_token_mint_key);

        let reserve_acc = get_account(&mut ctx.banks_client, reserve_key)
            .await
            .expect("getting Reserve account");
        let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("unpacking Reserve");

        let refresh = RefreshReserve {
            reserve: reserve_key,
            market_price_feed: reserve.config.market_price_feed,
            irm: reserve.config.irm,
        }
        .into_instruction();

        ixs.push(refresh);

        reserves.push(reserve_key);
        reserve_liquidity_supplies.push(lp_supply);
        lp_token_mints.push(lp_token_mint_key);
        lp_token_storages.push(lp_token_storage_key);
    }

    let mut insufficient_reserves = reserves.clone();
    insufficient_reserves.pop();

    ixs.push(
        Deposit {
            position: position_keypair.pubkey(),
            vault: vault_keypair.pubkey(),
            owner: owner_keypair.pubkey(),
            liquidity_mint: liquidity_usdc_mint,
            liquidity_source: source_liquidity_wallet,
            liquidity_buffer: find_vault_liquidity_buffer(&vault_keypair.pubkey()).0,
            superlendy_program_authority,
            liquidity_token_program: spl_token::id(),
            reserves: insufficient_reserves,
            reserve_liquidity_supplies: reserve_liquidity_supplies.clone(),
            lp_token_mints: lp_token_mints.clone(),
            lp_token_storages: lp_token_mints.clone(),
            amount,
        }
        .into_instruction(),
    );

    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &owner_keypair],
        blockhash,
    );
    for reserve in reserves.clone() {
        update_prices(&mut ctx, &[reserve]).await;
    }
    let result = ctx.banks_client.process_transaction(tx).await;
    assert!(result.is_err());

    info!("deposit with insufficient liquidity_supplies count");

    let mut insufficient_liquidity_supplies = reserve_liquidity_supplies.clone();
    insufficient_liquidity_supplies.pop();

    ixs.push(
        Deposit {
            position: position_keypair.pubkey(),
            vault: vault_keypair.pubkey(),
            owner: owner_keypair.pubkey(),
            liquidity_mint: liquidity_usdc_mint,
            liquidity_source: source_liquidity_wallet,
            liquidity_buffer: find_vault_liquidity_buffer(&vault_keypair.pubkey()).0,
            superlendy_program_authority,
            liquidity_token_program: spl_token::id(),
            reserves: reserves.clone(),
            reserve_liquidity_supplies: insufficient_liquidity_supplies.clone(),
            lp_token_mints: lp_token_mints.clone(),
            lp_token_storages: lp_token_mints.clone(),
            amount,
        }
        .into_instruction(),
    );

    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &owner_keypair],
        blockhash,
    );
    for reserve in reserves.clone() {
        update_prices(&mut ctx, &[reserve]).await;
    }
    let result = ctx.banks_client.process_transaction(tx).await;
    assert!(result.is_err());

    info!("deposit with insufficient lp_token_mints count");

    let mut insufficient_lp_token_mints = lp_token_mints.clone();
    insufficient_lp_token_mints.pop();

    ixs.push(
        Deposit {
            position: position_keypair.pubkey(),
            vault: vault_keypair.pubkey(),
            owner: owner_keypair.pubkey(),
            liquidity_mint: liquidity_usdc_mint,
            liquidity_source: source_liquidity_wallet,
            liquidity_buffer: find_vault_liquidity_buffer(&vault_keypair.pubkey()).0,
            superlendy_program_authority,
            liquidity_token_program: spl_token::id(),
            reserves: reserves.clone(),
            reserve_liquidity_supplies: reserve_liquidity_supplies.clone(),
            lp_token_mints: insufficient_lp_token_mints.clone(),
            lp_token_storages: lp_token_mints.clone(),
            amount,
        }
        .into_instruction(),
    );

    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &owner_keypair],
        blockhash,
    );
    for reserve in reserves.clone() {
        update_prices(&mut ctx, &[reserve]).await;
    }
    let result = ctx.banks_client.process_transaction(tx).await;
    assert!(result.is_err());

    info!("deposit with insufficient lp_token_storages count");

    let mut insufficient_lp_token_storages = lp_token_storages.clone();
    insufficient_lp_token_storages.pop();

    ixs.push(
        Deposit {
            position: position_keypair.pubkey(),
            vault: vault_keypair.pubkey(),
            owner: owner_keypair.pubkey(),
            liquidity_mint: liquidity_usdc_mint,
            liquidity_source: source_liquidity_wallet,
            liquidity_buffer: find_vault_liquidity_buffer(&vault_keypair.pubkey()).0,
            superlendy_program_authority,
            liquidity_token_program: spl_token::id(),
            reserves: reserves.clone(),
            reserve_liquidity_supplies: reserve_liquidity_supplies.clone(),
            lp_token_mints: lp_token_mints.clone(),
            lp_token_storages: insufficient_lp_token_storages.clone(),
            amount,
        }
        .into_instruction(),
    );

    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &owner_keypair],
        blockhash,
    );
    for reserve in reserves.clone() {
        update_prices(&mut ctx, &[reserve]).await;
    }
    let result = ctx.banks_client.process_transaction(tx).await;
    assert!(result.is_err());
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#5e50d7380eb24b859124428aa00201d7
#[tokio::test]
async fn deposit_incorrect_accounts_sequence() {
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

    info!("deposit with incorrect reserves sequence");

    let blockhash = ctx
        .banks_client
        .get_new_latest_blockhash(&ctx.last_blockhash)
        .await
        .expect("get latest blockhash");

    let vault_acc = get_account(&mut ctx.banks_client, vault_keypair.pubkey())
        .await
        .expect("getting Vault account");
    let unpacked_vault = Vault::try_from_bytes(&vault_acc.data).expect("unpacking Vault");

    let mut reserves = vec![];
    let mut reserve_liquidity_supplies = vec![];
    let mut lp_token_mints = vec![];
    let mut lp_token_storages = vec![];
    let mut ixs = vec![];

    for item in unpacked_vault.distribution.0 {
        if item.is_vacant() {
            break;
        }
        let reserve_key = item.reserve;
        let (lp_supply, _) = find_liquidity_supply(&reserve_key);
        let (lp_token_mint_key, _) = find_lp_token_mint(&reserve_key);
        let (lp_token_storage_key, _) =
            find_vault_lp_token_storage(&vault_keypair.pubkey(), &lp_token_mint_key);

        let reserve_acc = get_account(&mut ctx.banks_client, reserve_key)
            .await
            .expect("getting Reserve account");
        let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("unpacking Reserve");

        let refresh = RefreshReserve {
            reserve: reserve_key,
            market_price_feed: reserve.config.market_price_feed,
            irm: reserve.config.irm,
        }
        .into_instruction();

        ixs.push(refresh);

        reserves.push(reserve_key);
        reserve_liquidity_supplies.push(lp_supply);
        lp_token_mints.push(lp_token_mint_key);
        lp_token_storages.push(lp_token_storage_key);
    }

    let mut incorrect_reserves = reserves.clone();
    let removed = incorrect_reserves.remove(0);
    incorrect_reserves.push(removed);

    ixs.push(
        Deposit {
            position: position_keypair.pubkey(),
            vault: vault_keypair.pubkey(),
            owner: owner_keypair.pubkey(),
            liquidity_mint: liquidity_usdc_mint,
            liquidity_source: source_liquidity_wallet,
            liquidity_buffer: find_vault_liquidity_buffer(&vault_keypair.pubkey()).0,
            superlendy_program_authority,
            liquidity_token_program: spl_token::id(),
            reserves: incorrect_reserves,
            reserve_liquidity_supplies: reserve_liquidity_supplies.clone(),
            lp_token_mints: lp_token_mints.clone(),
            lp_token_storages: lp_token_mints.clone(),
            amount,
        }
        .into_instruction(),
    );

    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &owner_keypair],
        blockhash,
    );
    for reserve in reserves.clone() {
        update_prices(&mut ctx, &[reserve]).await;
    }
    let result = ctx.banks_client.process_transaction(tx).await;
    assert!(result.is_err());

    info!("deposit with incorrect liquidity_supplies sequence");

    let mut incorrect_liquidity_supplies = reserve_liquidity_supplies.clone();
    let removed = incorrect_liquidity_supplies.remove(0);
    incorrect_liquidity_supplies.push(removed);

    ixs.push(
        Deposit {
            position: position_keypair.pubkey(),
            vault: vault_keypair.pubkey(),
            owner: owner_keypair.pubkey(),
            liquidity_mint: liquidity_usdc_mint,
            liquidity_source: source_liquidity_wallet,
            liquidity_buffer: find_vault_liquidity_buffer(&vault_keypair.pubkey()).0,
            superlendy_program_authority,
            liquidity_token_program: spl_token::id(),
            reserves: reserves.clone(),
            reserve_liquidity_supplies: incorrect_liquidity_supplies.clone(),
            lp_token_mints: lp_token_mints.clone(),
            lp_token_storages: lp_token_mints.clone(),
            amount,
        }
        .into_instruction(),
    );

    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &owner_keypair],
        blockhash,
    );
    for reserve in reserves.clone() {
        update_prices(&mut ctx, &[reserve]).await;
    }
    let result = ctx.banks_client.process_transaction(tx).await;
    assert!(result.is_err());

    info!("deposit with incorrect lp_token_mints sequence");

    let mut incorrect_lp_token_mints = lp_token_mints.clone();
    let removed = incorrect_lp_token_mints.remove(0);
    incorrect_lp_token_mints.push(removed);

    ixs.push(
        Deposit {
            position: position_keypair.pubkey(),
            vault: vault_keypair.pubkey(),
            owner: owner_keypair.pubkey(),
            liquidity_mint: liquidity_usdc_mint,
            liquidity_source: source_liquidity_wallet,
            liquidity_buffer: find_vault_liquidity_buffer(&vault_keypair.pubkey()).0,
            superlendy_program_authority,
            liquidity_token_program: spl_token::id(),
            reserves: reserves.clone(),
            reserve_liquidity_supplies: reserve_liquidity_supplies.clone(),
            lp_token_mints: incorrect_lp_token_mints.clone(),
            lp_token_storages: lp_token_mints.clone(),
            amount,
        }
        .into_instruction(),
    );

    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &owner_keypair],
        blockhash,
    );
    for reserve in reserves.clone() {
        update_prices(&mut ctx, &[reserve]).await;
    }
    let result = ctx.banks_client.process_transaction(tx).await;
    assert!(result.is_err());

    info!("deposit with incorrect lp_token_storages sequence");

    let mut incorrect_lp_token_storages = lp_token_storages.clone();
    let removed = incorrect_lp_token_storages.remove(0);
    incorrect_lp_token_storages.push(removed);

    ixs.push(
        Deposit {
            position: position_keypair.pubkey(),
            vault: vault_keypair.pubkey(),
            owner: owner_keypair.pubkey(),
            liquidity_mint: liquidity_usdc_mint,
            liquidity_source: source_liquidity_wallet,
            liquidity_buffer: find_vault_liquidity_buffer(&vault_keypair.pubkey()).0,
            superlendy_program_authority,
            liquidity_token_program: spl_token::id(),
            reserves: reserves.clone(),
            reserve_liquidity_supplies: reserve_liquidity_supplies.clone(),
            lp_token_mints: lp_token_mints.clone(),
            lp_token_storages: incorrect_lp_token_storages.clone(),
            amount,
        }
        .into_instruction(),
    );

    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer, &owner_keypair],
        blockhash,
    );
    for reserve in reserves.clone() {
        update_prices(&mut ctx, &[reserve]).await;
    }
    let result = ctx.banks_client.process_transaction(tx).await;
    assert!(result.is_err());
}
