#![cfg(feature = "test-bpf")]

use std::str::FromStr;

use price_proxy::state::utils::str_to_array;
use solana_program::pubkey::Pubkey;
use solana_sdk::program_pack::Pack;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account;
use super_lendy::pda::find_program_authority;
use texture_common::account::PodAccount;
use texture_common::dec;
use texture_common::math::{CheckedAdd, Decimal};
use tracing::info;
use utils::create_associated_token_account;
use utils::setup::{get_slot, set_reward_rule};
use utils::vault_executor::{
    claim_reward, create_empty_mint_account, init_reward_supply, initialize_spl_mint, mint_tokens,
    withdraw,
};
use vault::pda::find_reward_supply;
use vault::state::position::Position;
use vault::state::vault::{DistributionItem, Vault, VaultParams, VAULT_DISTRIBUTION_MAX_ITEMS};

use crate::utils::setup::{setup_env, setup_vault, EnvKeys};
use crate::utils::vault_executor::{create_position, deposit};
use crate::utils::{
    add_curve_acc, add_price_feed_acc, admin_keypair, borrow_keypair, get_account,
    init_program_test, init_token_accounts, texture_config_keypair, Runner, LAMPORTS,
    LAMPORTS_PER_USDC,
};

pub mod utils;

/// TODO: test description
#[tokio::test]
async fn accrue_rewards() {
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
            _padding: Default::default(),
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

    // SETUP REWARD RULES

    set_reward_rule(
        &mut ctx,
        &vault_keypair,
        0,
        "Stingy".into(),
        Pubkey::new_unique(),
        Decimal::from_str("10.0").unwrap(),
        curator_keypair.pubkey(),
        &vaults_authority_keypair,
    )
    .await
    .expect("set_reward_rule");

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
    .expect("deposit 1");

    // 10 SLOTS LATER

    let slot = get_slot(&mut ctx).await.expect("get_slot");
    let target_slot = slot + 10;
    info!("wrap to slot {}", target_slot);
    ctx.warp_to_slot(target_slot).expect("warp_to_slot");

    // DEPOSIT 1 USDC TO TRIGGER REWARDS UPDATE

    let amount = LAMPORTS_PER_USDC;
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
    .expect("deposit 2");

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    let total_accrued_rewards_amount = position
        .rewards
        .rewards
        .iter()
        .map(|reward| reward.accrued_amount().unwrap())
        .try_fold(Decimal::ZERO, CheckedAdd::checked_add)
        .unwrap();

    // expect 100 VLP * 10 rewards * 10 slots = 10000 rewards
    assert_eq!(total_accrued_rewards_amount, dec!(10000));
}

/// TODO: test description
#[tokio::test]
async fn accrue_rewards_complex() {
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
            _padding: Default::default(),
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

    // SETUP REWARD RULES

    set_reward_rule(
        &mut ctx,
        &vault_keypair,
        0,
        "Stingy".into(),
        Pubkey::new_unique(),
        Decimal::from_str("10.0").unwrap(),
        curator_keypair.pubkey(),
        &vaults_authority_keypair,
    )
    .await
    .expect("set_reward_rule");

    set_reward_rule(
        &mut ctx,
        &vault_keypair,
        1,
        "Generous".into(),
        Pubkey::new_unique(),
        Decimal::from_str("15.0").unwrap(),
        curator_keypair.pubkey(),
        &vaults_authority_keypair,
    )
    .await
    .expect("set_reward_rule");

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
    .expect("deposit 1");

    // 10 SLOTS LATER

    let slot = get_slot(&mut ctx).await.expect("get_slot");
    let target_slot = slot + 10;
    info!("wrap to slot {}", target_slot);
    ctx.warp_to_slot(target_slot).expect("warp_to_slot");

    // DEPOSIT 1 USDC TO TRIGGER REWARDS UPDATE

    let amount = LAMPORTS_PER_USDC;
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
    .expect("deposit 2");

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    let total_accrued_rewards_amount = position
        .rewards
        .rewards
        .iter()
        .map(|reward| reward.accrued_amount().unwrap())
        .try_fold(Decimal::ZERO, CheckedAdd::checked_add)
        .unwrap();

    // expect 100 VLP * (10 rewards + 15 rewards) * 10 slots = 25000 rewards
    assert_eq!(total_accrued_rewards_amount, dec!(25000));
}

/// TODO: test description
#[tokio::test]
async fn accrue_rewards_no_space() {
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
            _padding: Default::default(),
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

    // SETUP 8 REWARD RULES TO TAKE ALL 8 SLOTS

    const REWARD_DECIMALS: u8 = 9;

    for idx in 0..8 {
        let mint = Keypair::new();

        set_reward_rule(
            &mut ctx,
            &vault_keypair,
            idx,
            format!("Reward {}", idx),
            mint.pubkey(),
            Decimal::from_str("10.0").unwrap(),
            curator_keypair.pubkey(),
            &vaults_authority_keypair,
        )
        .await
        .expect("set_reward_rule");

        create_empty_mint_account(&mut ctx, &mint, spl_token::id())
            .await
            .expect("create mint");

        initialize_spl_mint(&mut ctx, &mint, REWARD_DECIMALS)
            .await
            .unwrap();

        let (reward_supply, _) = find_reward_supply(&vault_keypair.pubkey(), &mint.pubkey());

        init_reward_supply(
            &mut ctx,
            vault_keypair.pubkey(),
            curator_keypair.pubkey(),
            &vaults_authority_keypair,
            idx,
        )
        .await
        .expect("init_reward_supply");

        mint_tokens(&mut ctx, &mint.pubkey(), &reward_supply, u64::MAX)
            .await
            .unwrap();
    }

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
    .expect("deposit 1");

    // 10 SLOTS LATER

    let slot = get_slot(&mut ctx).await.expect("get_slot");
    let target_slot = slot + 10;
    info!("wrap to slot {}", target_slot);
    ctx.warp_to_slot(target_slot).expect("warp_to_slot");

    // DEPOSIT 1 USDC TO TRIGGER REWARDS UPDATE

    let amount = LAMPORTS_PER_USDC;
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
    .expect("deposit 2");

    // CHECK VAULT POSITION

    let position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position = Position::try_from_bytes(&position_acc.data).expect("cast vault position data");
    let total_accrued_rewards_amount = position
        .rewards
        .rewards
        .iter()
        .map(|reward| reward.accrued_amount().unwrap())
        .try_fold(Decimal::ZERO, CheckedAdd::checked_add)
        .unwrap();

    // expect 100 VLP * (10 rewards * 8 rules) * 10 slots = 80000 rewards
    assert_eq!(total_accrued_rewards_amount, dec!(80000));

    // CLAIM REWARDS FROM FIRST RULE

    let vault_acc = get_account(&mut ctx.banks_client, vault_keypair.pubkey())
        .await
        .expect("get vault");
    let vault = Vault::try_from_bytes(&vault_acc.data).expect("cast vault vault data");
    let mint_0 = vault.reward_rules.rules[0].reward_mint;

    let reward_idx = 0;
    let rule = vault.reward_rules.rules[reward_idx];

    let reward_ata = create_associated_token_account(&mut ctx, &owner_keypair, &rule.reward_mint)
        .await
        .expect("create reward ata");

    claim_reward(
        &mut ctx,
        position_keypair.pubkey(),
        vault_keypair.pubkey(),
        &owner_keypair,
        mint_0,
        reward_ata,
    )
    .await
    .expect("claim reward");

    let reward_token_acc = get_account(&mut ctx.banks_client, reward_ata)
        .await
        .expect("get reward ata");
    let reward_token = Account::unpack(&reward_token_acc.data).unwrap();
    assert_eq!(
        reward_token.amount,
        dec!(10000).to_lamports_round(REWARD_DECIMALS).unwrap()
    );

    // ensure that all 8 reward slots are still taken
    let mut position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position =
        Position::try_from_bytes_mut(&mut position_acc.data).expect("cast vault position data");
    assert!(position.rewards.find_unused_reward().is_none());

    // WITHDRAW FROM POSITION WITH ALL REWARD SLOTS TAKEN

    let amount = 50 * LAMPORTS_PER_USDC;
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

    // ADD NEW REWARD RULE

    let mint = Keypair::new();

    set_reward_rule(
        &mut ctx,
        &vault_keypair,
        0,
        "Reward X".to_string(),
        mint.pubkey(),
        Decimal::from_str("10.0").unwrap(),
        curator_keypair.pubkey(),
        &vaults_authority_keypair,
    )
    .await
    .expect("set_reward_rule");

    create_empty_mint_account(&mut ctx, &mint, spl_token::id())
        .await
        .expect("create mint");

    initialize_spl_mint(&mut ctx, &mint, REWARD_DECIMALS)
        .await
        .unwrap();

    let (reward_supply, _) = find_reward_supply(&vault_keypair.pubkey(), &mint.pubkey());

    init_reward_supply(
        &mut ctx,
        vault_keypair.pubkey(),
        curator_keypair.pubkey(),
        &vaults_authority_keypair,
        0,
    )
    .await
    .expect("init_reward_supply");

    mint_tokens(&mut ctx, &mint.pubkey(), &reward_supply, 1_000_000_000_000)
        .await
        .unwrap();

    // CHECK IF STILL ABLE TO WITHDRAW

    let amount = 25 * LAMPORTS_PER_USDC;
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

    // CLAIM OLD REWARD FROM SLOT 0 AND CHECK IF NEW REWARD PLACED INSTEAD

    let reward_ata = create_associated_token_account(&mut ctx, &owner_keypair, &mint_0)
        .await
        .expect("create reward ata");

    claim_reward(
        &mut ctx,
        position_keypair.pubkey(),
        vault_keypair.pubkey(),
        &owner_keypair,
        mint_0,
        reward_ata,
    )
    .await
    .expect("claim reward");

    let reward_token_acc = get_account(&mut ctx.banks_client, reward_ata)
        .await
        .expect("get reward ata");
    let reward_token = Account::unpack(&reward_token_acc.data).unwrap();
    assert_eq!(
        reward_token.amount,
        dec!(10000).to_lamports_round(REWARD_DECIMALS).unwrap()
    );

    // ensure that new reward is tracked in first slot instead of old one
    let mut position_acc = get_account(&mut ctx.banks_client, position_keypair.pubkey())
        .await
        .expect("get vault position");
    let position =
        Position::try_from_bytes_mut(&mut position_acc.data).expect("cast vault position data");
    let vault_acc = get_account(&mut ctx.banks_client, vault_keypair.pubkey())
        .await
        .expect("get vault");
    let vault = Vault::try_from_bytes(&vault_acc.data).expect("cast vault vault data");
    assert_eq!(
        position.rewards.rewards[0].reward_mint,
        vault.reward_rules.rules[0].reward_mint
    );
}
