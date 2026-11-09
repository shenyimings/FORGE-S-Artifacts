#![cfg(feature = "test-bpf")]

use bytemuck::Zeroable;
use std::str::FromStr;

use price_proxy::state::utils::str_to_array;
use solana_program::pubkey::Pubkey;
use solana_program_test::ProgramTestBanksClientExt;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use texture_common::account::PodAccount;
use tracing::info;
use vault::instruction::AlterVault;

use vault::state::vault::{DistributionItem, Vault, VaultParams, VAULT_DISTRIBUTION_MAX_ITEMS};

use crate::utils::setup::{setup_env, setup_vault, EnvKeys};
use crate::utils::vault_executor::alter_vault;
use crate::utils::{
    add_curve_acc, add_price_feed_acc, admin_keypair, get_account, init_program_test,
    init_token_accounts, texture_config_keypair, Runner, LAMPORTS,
};

pub mod utils;

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#10bb5734a220805ca49deb6330f7bc4d
#[tokio::test]
async fn alter_success() {
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

    // ALTER VAULT FIELDS

    let old_manager = params.distribution_manager;

    params.name = str_to_array("New Vault");
    params.description = str_to_array("New Description");
    params.lut = Pubkey::new_unique();
    params.distribution_manager = Pubkey::new_unique();
    // TODO: also try to alter fee config

    macro_rules! validate {
        ($this:ident, $params:ident) => {
            assert_eq!($this.name, $params.name);
            assert_eq!($this.description, $params.description);
            assert_eq!($this.lut, $params.lut);
            assert_eq!($this.distribution_manager, $params.distribution_manager);
        };
    }

    info!("alter vault fields");
    alter_vault(
        &mut ctx,
        vault_keypair.pubkey(),
        curator_keypair.pubkey(),
        params.clone(),
        &vaults_authority_keypair,
        old_manager,
    )
    .await
    .expect("alter_vault");
    let vault_acc = get_account(&mut ctx.banks_client, vault_keypair.pubkey())
        .await
        .expect("get pool");
    let vault = Vault::try_from_bytes(&vault_acc.data).expect("cast vault data");
    validate!(vault, params);
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#c42a9bdbfe334e309f38f50e8143bfb1
#[tokio::test]
async fn alter_from_incorrect_owner() {
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

    // TRY TO ALTER VAULT BY INCORRECT OWNER

    info!("try to alter vault by incorrect owner");
    let result = alter_vault(
        &mut ctx,
        vault_keypair.pubkey(),
        curator_keypair.pubkey(),
        params.clone(),
        &admin_keypair,
        params.distribution_manager,
    )
    .await;
    assert!(result.is_err());
}

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#6f9e2139b0324921a28f58c0354f7c52
#[tokio::test]
async fn alter_distribution() {
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
        risky3_pool_reserve_usdc_keypair,
        risky4_pool_reserve_usdc_keypair,
        vault_keypair,
        curator_keypair,
        vaults_authority_keypair,
        ..
    } = env_keys;

    let reserves = vec![
        main_pool_reserve_usdc_keypair.pubkey(),
        risky1_pool_reserve_usdc_keypair.pubkey(),
        risky2_pool_reserve_usdc_keypair.pubkey(),
        risky3_pool_reserve_usdc_keypair.pubkey(),
        risky4_pool_reserve_usdc_keypair.pubkey(),
    ];

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 2000;
    distribution_bps[1] = 0;
    distribution_bps[2] = 2000;
    distribution_bps[3] = 2000;
    distribution_bps[4] = 4000;
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

    // TRY TO ALTER SHARE TO 101%

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 2000;
    distribution_bps[1] = 0;
    distribution_bps[2] = 2000;
    distribution_bps[3] = 2000;
    distribution_bps[4] = 4100;
    params.distribution_bps = distribution_bps;

    info!("try to alter share to 101%");
    let result = alter_vault(
        &mut ctx,
        vault_keypair.pubkey(),
        curator_keypair.pubkey(),
        params.clone(),
        &vaults_authority_keypair,
        params.distribution_manager,
    )
    .await;
    assert!(result.is_err());

    // ALTER SHARE

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 2000;
    distribution_bps[1] = 0;
    distribution_bps[2] = 2000;
    distribution_bps[3] = 6000;
    distribution_bps[4] = 0;
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

    let vault_acc = get_account(&mut ctx.banks_client, vault_keypair.pubkey())
        .await
        .expect("get pool");
    let vault = Vault::try_from_bytes(&vault_acc.data).expect("cast vault data");
    assert_eq!(vault.distribution.0[0].share_bps, 2000);
    assert_eq!(vault.distribution.0[1].share_bps, 0);
    assert_eq!(vault.distribution.0[2].share_bps, 2000);
    assert_eq!(vault.distribution.0[3].share_bps, 6000);
    assert_eq!(vault.distribution.0[4].share_bps, 0);

    // TRY TO ADD SHARE_BPS

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 2000;
    distribution_bps[1] = 0;
    distribution_bps[2] = 2000;
    distribution_bps[3] = 5000;
    distribution_bps[4] = 0;
    distribution_bps[5] = 1000; // not associated with item
    params.distribution_bps = distribution_bps;

    info!("try to add share_bps");
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

/// See test description in
/// https://www.notion.so/Vault-3ef8527d0b0f49ce8b73ea0a5fac7a76?pvs=4#10bb5734a22080488c35d301810f2968
#[tokio::test]
async fn alter_by_distribution_manager() {
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

    // TRY TO ALTER VAULT.name BY DISTRIBUTION MANAGER

    let mut new_params = params.clone();
    new_params.name = str_to_array("new name");
    let blockhash = ctx
        .banks_client
        .get_new_latest_blockhash(&ctx.last_blockhash)
        .await
        .expect("get latest blockhash");

    let tx = Transaction::new_signed_with_payer(
        &[AlterVault {
            vault: vault_keypair.pubkey(),
            curator_vaults_authority: vaults_authority_keypair.pubkey(),
            distribution_manager: texture_owner_keypair.pubkey(),
            curator: curator_keypair.pubkey(),
            params: new_params.clone(),
        }
        .into_instruction()],
        Some(&texture_owner_keypair.pubkey()),
        &[&texture_owner_keypair],
        blockhash,
    );
    info!("alter vault.name by distribution manager");
    let result = ctx.banks_client.process_transaction(tx).await;
    assert!(result.is_err());

    // ALTER VAULT.distribution BY DISTRIBUTION MANAGER

    let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
    distribution_bps[0] = 300;
    distribution_bps[1] = 6300;
    distribution_bps[2] = 3400;
    params.distribution_bps = distribution_bps;

    let blockhash = ctx
        .banks_client
        .get_new_latest_blockhash(&ctx.last_blockhash)
        .await
        .expect("get latest blockhash");

    let tx = Transaction::new_signed_with_payer(
        &[AlterVault {
            vault: vault_keypair.pubkey(),
            curator_vaults_authority: vaults_authority_keypair.pubkey(),
            distribution_manager: texture_owner_keypair.pubkey(),
            curator: curator_keypair.pubkey(),
            params: params.clone(),
        }
        .into_instruction()],
        Some(&texture_owner_keypair.pubkey()),
        &[&texture_owner_keypair],
        blockhash,
    );
    info!("try to alter vault.distribution by distribution manager");
    ctx.banks_client
        .process_transaction(tx)
        .await
        .expect("alter_vault");

    let vault_acc = get_account(&mut ctx.banks_client, vault_keypair.pubkey())
        .await
        .expect("get pool");
    let vault = Vault::try_from_bytes(&vault_acc.data).expect("cast vault data");
    assert_eq!(vault.distribution.to_bps_array(), distribution_bps)
}
