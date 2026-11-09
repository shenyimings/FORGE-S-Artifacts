#![cfg(feature = "test-bpf")]

use bytemuck::Zeroable;
use solana_program_test::BanksClientError;
use solana_sdk::instruction::InstructionError;
use solana_sdk::transaction::TransactionError;
use std::str::FromStr;
use vault::error::VaultErrorDiscriminants;

use price_proxy::state::utils::str_to_array;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address;
use super_lendy::pda::{find_lp_token_mint, find_program_authority};
use super_lendy::state::reserve::{Reserve, RESERVE_MODE_NORMAL, RESERVE_MODE_RETAIN_LIQUIDITY};
use super_lendy::state::SLOTS_PER_YEAR;
use texture_common::account::PodAccount;
use texture_common::math::Decimal;
use tracing::info;
use vault::pda::{find_vault_liquidity_buffer, find_vault_lp_token_storage};
use vault::state::position::Position;

use vault::state::vault::{DistributionItem, VaultParams, VAULT_DISTRIBUTION_MAX_ITEMS};

use crate::utils::setup::{setup_env, setup_vault, EnvKeys};
use crate::utils::superlendy_executor::alter_reserve;
use crate::utils::vault_executor::{
    alter_vault, create_position, deposit, rebalance, remove_distribution_item,
};
use crate::utils::{
    add_curve_acc, add_price_feed_acc, admin_keypair, borrow_keypair,
    create_associated_token_account, get_account, init_program_test, init_token_accounts,
    price_feed_authority, superlendy_executor, texture_config_keypair, Runner, LAMPORTS,
    LAMPORTS_PER_USDC,
};

pub mod utils;

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#10bb5734a2208095b4ddcc1dd42b2aa7
#[tokio::test]
async fn rebalance_success() {
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

    // 2 SOLANA YEAR LATER

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

    // 2 SOLANA YEAR LATER

    slot += SLOTS_PER_YEAR * 2;
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
            share_bps: 0,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    distribution_items[0].share_bps = 5500;
    distribution_items[1].share_bps = 2700;
    distribution_items[2].share_bps = 1800;
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
        55 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[0].lp_exchange_rate().unwrap().0,
        Decimal::from_i128_with_scale(1, 0).unwrap()
    );

    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount,
        28 * LAMPORTS_PER_USDC
    );
    let risky1_lp_exchange_rate =
        Decimal::from_i128_with_scale(4495765338462073437899504865, 28).unwrap(); // 0.44
    assert_eq!(
        unpacked_reserves[1].lp_exchange_rate().unwrap().0,
        risky1_lp_exchange_rate
    );

    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        19 * LAMPORTS_PER_USDC
    );
    let risky2_lp_exchange_rate =
        Decimal::from_i128_with_scale(3014048383857686734901529917, 28).unwrap(); // 0.3, 1 LP = 3,33 USDC
    assert_eq!(
        unpacked_reserves[2].lp_exchange_rate().unwrap().0,
        risky2_lp_exchange_rate
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

    // ALTER SHARE

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 1000;
    distribution_bps[1] = 8000;
    distribution_bps[2] = 1000;
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

    rebalance(
        &mut ctx,
        vault_keypair.pubkey(),
        liquidity_usdc_mint,
        liquidity_buffer,
        superlendy_program_authority,
    )
    .await
    .expect("rebalance");

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
        10 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount,
        81 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        11 * LAMPORTS_PER_USDC
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

    // ALTER SHARE

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 0;
    distribution_bps[1] = 8000;
    distribution_bps[2] = 2000;
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

    rebalance(
        &mut ctx,
        vault_keypair.pubkey(),
        liquidity_usdc_mint,
        liquidity_buffer,
        superlendy_program_authority,
    )
    .await
    .expect("rebalance");

    let mut unpacked_reserves = vec![];
    for reserve_key in reserves.clone() {
        let reserve_acc = get_account(&mut ctx.banks_client, reserve_key)
            .await
            .expect("get reserve");
        let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
        unpacked_reserves.push(*reserve);
    }

    // CHECK RESERVES LP VALUE

    assert_eq!(unpacked_reserves[0].liquidity.available_amount, 0);
    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount,
        81 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        21 * LAMPORTS_PER_USDC
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
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#10bb5734a2208095b4ddcc1dd42b2aa7
#[tokio::test]
async fn rebalance_distribution_len() {
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

    let mut reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
    ];

    let mut distribution_items = reserves
        .clone()
        .into_iter()
        .map(|reserve| DistributionItem {
            reserve,
            share_bps: 0,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    distribution_items[0].share_bps = 5000;
    distribution_items[1].share_bps = 5000;
    distribution_items[2].share_bps = 0;
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
        50 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount,
        50 * LAMPORTS_PER_USDC
    );
    assert_eq!(unpacked_reserves[2].liquidity.available_amount, 0);

    // ALTER SHARE

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 5000;
    distribution_bps[1] = 3000;
    distribution_bps[2] = 2000;
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

    rebalance(
        &mut ctx,
        vault_keypair.pubkey(),
        liquidity_usdc_mint,
        liquidity_buffer,
        superlendy_program_authority,
    )
    .await
    .expect("rebalance");

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
        50 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount,
        30 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        20 * LAMPORTS_PER_USDC
    );

    // ALTER SHARE

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 10000;
    distribution_bps[1] = 0;
    distribution_bps[2] = 0;
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

    rebalance(
        &mut ctx,
        vault_keypair.pubkey(),
        liquidity_usdc_mint,
        liquidity_buffer,
        superlendy_program_authority,
    )
    .await
    .expect("rebalance");

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
        unpacked_reserves[0].liquidity.available_amount, // MainPool.USDC
        100 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount, // Risky1Pool.USDC
        0
    );
    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount, // Risky2Pool.USDC
        0
    );

    // REMOVE Risky1Pool.USDC DISTRIBUTION ITEM

    reserves.remove(1);
    let (lp_token_mint, _) = find_lp_token_mint(&risky1_pool_reserve_usdc_keypair.pubkey());
    let (lp_token_storage, _) =
        find_vault_lp_token_storage(&vault_keypair.pubkey(), &lp_token_mint);
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

    // REBALANCE

    let liquidity_buffer = find_vault_liquidity_buffer(&vault_keypair.pubkey()).0;
    let superlendy_program_authority = find_program_authority().0;

    let result = rebalance(
        &mut ctx,
        vault_keypair.pubkey(),
        liquidity_usdc_mint,
        liquidity_buffer,
        superlendy_program_authority,
    )
    .await;

    if let Err(BanksClientError::TransactionError(TransactionError::InstructionError(
        _,
        InstructionError::Custom(discriminant),
    ))) = result
    {
        assert_eq!(
            discriminant,
            VaultErrorDiscriminants::ImpossibleToImproveBalance as u32
        );
    } else {
        panic!("expected error")
    };

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
        unpacked_reserves[0].liquidity.available_amount, // MainPool.USDC
        100 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount, // Risky2Pool.USDC
        0
    );

    // ALTER SHARE

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 5000;
    distribution_bps[1] = 0;
    distribution_bps[2] = 5000;
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

    rebalance(
        &mut ctx,
        vault_keypair.pubkey(),
        liquidity_usdc_mint,
        liquidity_buffer,
        superlendy_program_authority,
    )
    .await
    .expect("rebalance");

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
        unpacked_reserves[0].liquidity.available_amount, // MainPool.USDC
        50 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount, // Risky2Pool.USDC
        50 * LAMPORTS_PER_USDC
    );
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#10bb5734a2208095b4ddcc1dd42b2aa7
#[tokio::test]
async fn rebalance_reserve_withdraw_disabled() {
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
        risky1_pool,
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
            share_bps: 0,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    distribution_items[0].share_bps = 5000;
    distribution_items[1].share_bps = 3000;
    distribution_items[2].share_bps = 2000;
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
        50 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount,
        30 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        20 * LAMPORTS_PER_USDC
    );

    // ALTER RESERVE MODE TO RETAIN_LIQUIDITY

    let reserve_acc = get_account(
        &mut ctx.banks_client,
        main_pool_reserve_usdc_keypair.pubkey(),
    )
    .await
    .expect("get reserve");
    let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
    alter_reserve(
        &mut ctx,
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool.pubkey(),
        &pool_authority_keypair,
        curator_keypair.pubkey(),
        reserve.config,
        RESERVE_MODE_RETAIN_LIQUIDITY,
    )
    .await
    .expect("alter_reserve");

    // ALTER SHARE

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 5000;
    distribution_bps[1] = 0;
    distribution_bps[2] = 5000;
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

    rebalance(
        &mut ctx,
        vault_keypair.pubkey(),
        liquidity_usdc_mint,
        liquidity_buffer,
        superlendy_program_authority,
    )
    .await
    .expect("rebalance");

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
        35 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount,
        30 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        35 * LAMPORTS_PER_USDC
    );

    // ALTER RESERVE MODE TO NORMAL

    let reserve_acc = get_account(
        &mut ctx.banks_client,
        main_pool_reserve_usdc_keypair.pubkey(),
    )
    .await
    .expect("get reserve");
    let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
    alter_reserve(
        &mut ctx,
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool.pubkey(),
        &pool_authority_keypair,
        curator_keypair.pubkey(),
        reserve.config,
        RESERVE_MODE_NORMAL,
    )
    .await
    .expect("alter_reserve");

    // REBALANCE

    let liquidity_buffer = find_vault_liquidity_buffer(&vault_keypair.pubkey()).0;
    let superlendy_program_authority = find_program_authority().0;

    rebalance(
        &mut ctx,
        vault_keypair.pubkey(),
        liquidity_usdc_mint,
        liquidity_buffer,
        superlendy_program_authority,
    )
    .await
    .expect("rebalance");

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
        50 * LAMPORTS_PER_USDC
    );
    assert_eq!(unpacked_reserves[1].liquidity.available_amount, 0);
    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        50 * LAMPORTS_PER_USDC
    );
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76#10eb5734a2208052b253d56e38333571
#[tokio::test]
async fn rebalance_reserves_withdraw_disabled() {
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
            share_bps: 0,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    distribution_items[0].share_bps = 5000;
    distribution_items[1].share_bps = 3000;
    distribution_items[2].share_bps = 2000;
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
        50 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount,
        30 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        20 * LAMPORTS_PER_USDC
    );

    // ALTER RESERVES MODE TO RETAIN_LIQUIDITY

    for reserve_key in reserves {
        let reserve_acc = get_account(&mut ctx.banks_client, reserve_key)
            .await
            .expect("get reserve");
        let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
        alter_reserve(
            &mut ctx,
            reserve_key,
            reserve.pool,
            &pool_authority_keypair,
            curator_keypair.pubkey(),
            reserve.config,
            RESERVE_MODE_RETAIN_LIQUIDITY,
        )
        .await
        .expect("alter_reserve");
    }

    // ALTER SHARE

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 5000;
    distribution_bps[1] = 0;
    distribution_bps[2] = 5000;
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

    // TRY TO REBALANCE WHEN RESERVES WITHDRAW LIQUIDITY DISABLED

    let liquidity_buffer = find_vault_liquidity_buffer(&vault_keypair.pubkey()).0;
    let superlendy_program_authority = find_program_authority().0;

    info!("try to rebalance when reserves withdraw liquidity disabled");
    let result = rebalance(
        &mut ctx,
        vault_keypair.pubkey(),
        liquidity_usdc_mint,
        liquidity_buffer,
        superlendy_program_authority,
    )
    .await;
    assert!(result.is_err())
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#10bb5734a2208095b4ddcc1dd42b2aa7
#[tokio::test]
async fn rebalance_utilization_exceeded() {
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
    let sol_price_feed = add_price_feed_acc(&mut runner, "sol-usd").await;
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

    // ALTER reserve.max_withdraw_utilization to 60%

    let reserve_acc = get_account(
        &mut ctx.banks_client,
        main_pool_reserve_usdc_keypair.pubkey(),
    )
    .await
    .expect("get reserve");
    let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
    let mut params = reserve.config;
    params.max_withdraw_utilization_bps = 6000; // 60%
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
            share_bps: 0,
            _padding: Zeroable::zeroed(),
        })
        .collect::<Vec<_>>();
    distribution_items[0].share_bps = 9000; // 90%
    distribution_items[1].share_bps = 100; // 1%
    distribution_items[2].share_bps = 900; // 9%
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
        90 * LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount,
        LAMPORTS_PER_USDC
    );
    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        9 * LAMPORTS_PER_USDC
    );

    // DEPOSIT 0.45 SOL & LOCK COLLATERAL INTO MAIN SOL RESERVE

    let superlendy_main_position_kp = Keypair::new();
    superlendy_executor::create_position(
        &mut ctx,
        &superlendy_main_position_kp,
        main_pool.pubkey(),
        &owner_keypair,
    )
    .await
    .expect("create_position");

    let lp_mint = find_lp_token_mint(&main_pool_reserve_sol_keypair.pubkey()).0;
    let dest_lp_wallet = create_associated_token_account(&mut ctx, &owner_keypair, &lp_mint)
        .await
        .expect("create lp ata");
    let source_liq_wallet_sol =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_sol_mint);

    info!("deposit 0.45 SOL & lock collateral into main sol reserve");
    let deposit_sol_amount = 45 * 10_000_000;
    superlendy_executor::deposit_liquidity(
        &mut ctx,
        main_pool_reserve_sol_keypair.pubkey(),
        sol_price_feed,
        irm,
        &owner_keypair,
        source_liq_wallet_sol,
        dest_lp_wallet,
        deposit_sol_amount,
    )
    .await
    .expect("deposit_liquidity");

    superlendy_executor::refresh_position(&mut ctx, superlendy_main_position_kp.pubkey())
        .await
        .expect("refresh position");

    superlendy_executor::lock_collateral(
        &mut ctx,
        main_pool_reserve_sol_keypair.pubkey(),
        sol_price_feed,
        irm,
        superlendy_main_position_kp.pubkey(),
        &owner_keypair,
        dest_lp_wallet,
        deposit_sol_amount,
    )
    .await
    .expect("lock_collateral");

    superlendy_executor::refresh_position(&mut ctx, superlendy_main_position_kp.pubkey())
        .await
        .expect("refresh position");

    // BORROW 22.5 USDC FROM MAIN RESERVE TO INCREASE UTILIZATION TO 25%

    let texture_fee_receiver =
        create_associated_token_account(&mut ctx, &texture_owner_keypair, &liquidity_usdc_mint)
            .await
            .expect("create texture fee receiver ata");
    let curator_fee_receiver =
        create_associated_token_account(&mut ctx, &pool_authority_keypair, &liquidity_usdc_mint)
            .await
            .expect("create curator fee receiver ata");
    let destination_usdc_wallet =
        get_associated_token_address(&owner_keypair.pubkey(), &liquidity_usdc_mint);

    let borrow_amount = 225 * LAMPORTS_PER_USDC / 10;

    info!("borrow {} usdc from main reserve", borrow_amount);
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
        destination_usdc_wallet,
        borrow_amount,
        1,
    )
    .await
    .expect("borrow");

    let reserve_acc = get_account(
        &mut ctx.banks_client,
        main_pool_reserve_usdc_keypair.pubkey(),
    )
    .await
    .expect("get reserve");
    let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("cast reserve data");
    assert_eq!(
        reserve.liquidity.utilization_rate().unwrap(),
        Decimal::from_i128_with_scale(25, 2).unwrap()
    );

    // ALTER SHARE

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 5000;
    distribution_bps[1] = 2500;
    distribution_bps[2] = 2500;
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

    rebalance(
        &mut ctx,
        vault_keypair.pubkey(),
        liquidity_usdc_mint,
        liquidity_buffer,
        superlendy_program_authority,
    )
    .await
    .expect("rebalance");

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
        275 * LAMPORTS_PER_USDC / 10 // 27.5 USDC
    );
    assert_eq!(
        unpacked_reserves[1].liquidity.available_amount,
        25 * LAMPORTS_PER_USDC // 25 USDC
    );
    assert_eq!(
        unpacked_reserves[2].liquidity.available_amount,
        25 * LAMPORTS_PER_USDC // 25 USDC
    );
}
