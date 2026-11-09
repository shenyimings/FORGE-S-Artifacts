use crate::utils::get_account;
use crate::utils::superlendy_executor::{
    create_curator, create_pool, create_reserve, create_texture_config,
};
use bytemuck::Zeroable;
use price_proxy::state::utils::str_to_array;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::pubkey::Pubkey;
use solana_program_test::{BanksClientError, ProgramTestBanksClientExt, ProgramTestContext};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use std::str::FromStr;
use super_lendy::pda::find_lp_token_mint;
use super_lendy::state::curator::CuratorParams;
use super_lendy::state::pool::PoolParams;
use super_lendy::state::reserve::{Reserve, ReserveConfig, ReserveFeesConfig, RESERVE_TYPE_NORMAL};
use super_lendy::state::texture_cfg::{ReserveTimelock, TextureConfigParams};
use tarpc::context::Context;
use texture_common::account::PodAccount;
use texture_common::math::Decimal;
use vault::instruction::{CreateVault, SetRewardRules};
use vault::pda::{find_vault_liquidity_buffer, find_vault_lp_token_storage};
use vault::state::reward::{RewardRules, REWARD_RULE_NAME_MAX_LEN};
use vault::state::vault::{Vault, VaultParams};

pub async fn setup_env(
    ctx: &mut ProgramTestContext,
    admin_keypair: &Keypair,
    texture_owner_keypair: &Keypair,
    texture_config_keypair: &Keypair,
    pool_authority_keypair: &Keypair,
) -> EnvKeys {
    let vault_keypair = Keypair::new();
    let curator_keypair = Keypair::new();
    let vaults_authority_keypair =
        Keypair::from_base58_string(&pool_authority_keypair.to_base58_string());

    let position_keypair = Keypair::new();
    let position_0_keypair = Keypair::new();

    let main_pool_keypair = Keypair::new();
    let risky1_pool_keypair = Keypair::new();
    let risky2_pool_keypair = Keypair::new();
    let risky3_pool_keypair = Keypair::new();
    let risky4_pool_keypair = Keypair::new();

    let main_pool_reserve_sol_keypair = Keypair::new();
    let main_pool_reserve_usdc_keypair = Keypair::new();
    let risky1_pool_reserve_jup_keypair = Keypair::new();
    let risky1_pool_reserve_usdc_keypair = Keypair::new();
    let risky2_pool_reserve_bonk_keypair = Keypair::new();
    let risky2_pool_reserve_usdc_keypair = Keypair::new();
    let risky3_pool_reserve_usdc_keypair = Keypair::new();
    let risky4_pool_reserve_usdc_keypair = Keypair::new();

    // 1 SOL = 100 USD
    let sol_price_feed = Pubkey::from_str("2Ds9EtKTqMhQsm71oWyuM3YGVWT6fjPDJ8SKXqsb7c6b").unwrap();
    // 1 USDC = 1 USD
    let usdc_price_feed = Pubkey::from_str("Afv1iJ97h6DHwugg7zazC5AvJWy61EASJV7CodMcdXYk").unwrap();
    // 1 JUP = 0.7 USD
    let jup_price_feed = Pubkey::from_str("BY8zZPNuLE7DvTpbVES9Thhy4XmLAorpF6qVSg6dx1Qf").unwrap();
    // 1 BONK = 0.00001 USD
    let bonk_price_feed = Pubkey::from_str("3b6PochAzQ5DJQe2EHGSk4Ach6jDphmjiwMbgGZND7MT").unwrap();

    let irm = Pubkey::from_str("5XY6oUgcK5EoXMfTnMiWzrBnE2Fr9S8gyydWjYLsb5Z4").unwrap();

    let liquidity_sol_mint =
        Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let liquidity_usdc_mint =
        Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let liquidity_jup_mint =
        Pubkey::from_str("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN").unwrap();
    let liquidity_bonk_mint =
        Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap();

    // CREATE TEXTURE CONFIG

    let texture_params = TextureConfigParams {
        borrow_fee_rate_bps: 0,
        performance_fee_rate_bps: 0,
        fees_authority: texture_owner_keypair.pubkey(),
        reserve_timelock: ReserveTimelock {
            market_price_feed_lock_sec: 0,
            irm_lock_sec: 0,
            liquidation_bonus_lock_sec: 0,
            unhealthy_ltv_lock_sec: 0,
            partial_liquidation_factor_lock_sec: 0,
            max_total_liquidity_lock_sec: 0,
            max_borrow_ltv_lock_sec: 0,
            max_borrow_utilization_lock_sec: 0,
            price_stale_threshold_lock_sec: 0,
            max_withdraw_utilization_lock_sec: 0,
            fees_lock_sec: 0,
            _padding: 0,
        },
    };
    create_texture_config(ctx, admin_keypair, texture_config_keypair, texture_params)
        .await
        .expect("create_texture_config");

    // CREATE CURATOR

    let curator_params = CuratorParams {
        owner: texture_owner_keypair.pubkey(),
        fees_authority: pool_authority_keypair.pubkey(),
        pools_authority: pool_authority_keypair.pubkey(),
        vaults_authority: pool_authority_keypair.pubkey(),
        name: [1; 128],
        logo_url: [2; 128],
        website_url: [3; 128],
    };
    create_curator(ctx, &curator_keypair, admin_keypair, curator_params)
        .await
        .expect("create_curator");

    // CREATE MAIN POOL

    let pool_params = PoolParams {
        name: str_to_array("Main Pool"),
        market_price_currency_symbol: str_to_array("USD"),
        visible: 0,
    };

    create_pool(
        ctx,
        &main_pool_keypair,
        pool_authority_keypair,
        curator_keypair.pubkey(),
        pool_params,
    )
    .await
    .expect("create main_pool");

    // CREATE SOL-RESERVE IN MAIN POOL

    let fees_config = ReserveFeesConfig {
        curator_borrow_fee_rate_bps: 0,
        curator_performance_fee_rate_bps: 0,
        _padding: Zeroable::zeroed(),
    };
    let mut reserve_config = ReserveConfig {
        market_price_feed: sol_price_feed,
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

    create_reserve(
        ctx,
        &main_pool_reserve_sol_keypair,
        main_pool_keypair.pubkey(),
        pool_authority_keypair,
        curator_keypair.pubkey(),
        liquidity_sol_mint,
        sol_price_feed,
        reserve_config,
        RESERVE_TYPE_NORMAL,
    )
    .await
    .expect("create main_pool_reserve_sol");

    // CREATE USDC-RESERVE IN MAIN POOL

    reserve_config.market_price_feed = usdc_price_feed;
    create_reserve(
        ctx,
        &main_pool_reserve_usdc_keypair,
        main_pool_keypair.pubkey(),
        pool_authority_keypair,
        curator_keypair.pubkey(),
        liquidity_usdc_mint,
        usdc_price_feed,
        reserve_config,
        RESERVE_TYPE_NORMAL,
    )
    .await
    .expect("create main_pool_reserve_usdc");

    // CREATE RISKY1 POOL

    let pool_params = PoolParams {
        name: str_to_array("Risky1 Pool"),
        market_price_currency_symbol: str_to_array("USD"),
        visible: 0,
    };

    create_pool(
        ctx,
        &risky1_pool_keypair,
        pool_authority_keypair,
        curator_keypair.pubkey(),
        pool_params,
    )
    .await
    .expect("create risky1_pool");

    // CREATE USDC-RESERVE IN RISKY1 POOL

    reserve_config.market_price_feed = usdc_price_feed;
    create_reserve(
        ctx,
        &risky1_pool_reserve_usdc_keypair,
        risky1_pool_keypair.pubkey(),
        pool_authority_keypair,
        curator_keypair.pubkey(),
        liquidity_usdc_mint,
        usdc_price_feed,
        reserve_config,
        RESERVE_TYPE_NORMAL,
    )
    .await
    .expect("create risky1_pool_reserve_usdc");

    // CREATE JUP-RESERVE IN RISKY1 POOL

    reserve_config.market_price_feed = jup_price_feed;
    create_reserve(
        ctx,
        &risky1_pool_reserve_jup_keypair,
        risky1_pool_keypair.pubkey(),
        pool_authority_keypair,
        curator_keypair.pubkey(),
        liquidity_jup_mint,
        jup_price_feed,
        reserve_config,
        RESERVE_TYPE_NORMAL,
    )
    .await
    .expect("create risky1_pool_reserve_jup");

    // CREATE RISKY2 POOL

    let pool_params = PoolParams {
        name: str_to_array("Risky2 Pool"),
        market_price_currency_symbol: str_to_array("USD"),
        visible: 0,
    };

    create_pool(
        ctx,
        &risky2_pool_keypair,
        pool_authority_keypair,
        curator_keypair.pubkey(),
        pool_params,
    )
    .await
    .expect("create risky2_pool");

    // CREATE USDC-RESERVE IN RISKY2 POOL

    reserve_config.market_price_feed = usdc_price_feed;
    create_reserve(
        ctx,
        &risky2_pool_reserve_usdc_keypair,
        risky2_pool_keypair.pubkey(),
        pool_authority_keypair,
        curator_keypair.pubkey(),
        liquidity_usdc_mint,
        usdc_price_feed,
        reserve_config,
        RESERVE_TYPE_NORMAL,
    )
    .await
    .expect("create risky2_pool_reserve_usdc");

    // CREATE BONK-RESERVE IN RISKY2 POOL

    reserve_config.market_price_feed = bonk_price_feed;
    create_reserve(
        ctx,
        &risky2_pool_reserve_bonk_keypair,
        risky2_pool_keypair.pubkey(),
        pool_authority_keypair,
        curator_keypair.pubkey(),
        liquidity_bonk_mint,
        bonk_price_feed,
        reserve_config,
        RESERVE_TYPE_NORMAL,
    )
    .await
    .expect("create risky2_pool_reserve_bonk");

    // CREATE RISKY3 POOL

    let pool_params = PoolParams {
        name: str_to_array("Risky3 Pool"),
        market_price_currency_symbol: str_to_array("USD"),
        visible: 0,
    };

    create_pool(
        ctx,
        &risky3_pool_keypair,
        pool_authority_keypair,
        curator_keypair.pubkey(),
        pool_params,
    )
    .await
    .expect("create risky3_pool");

    // CREATE USDC-RESERVE IN RISKY3 POOL

    reserve_config.market_price_feed = usdc_price_feed;
    create_reserve(
        ctx,
        &risky3_pool_reserve_usdc_keypair,
        risky3_pool_keypair.pubkey(),
        pool_authority_keypair,
        curator_keypair.pubkey(),
        liquidity_usdc_mint,
        usdc_price_feed,
        reserve_config,
        RESERVE_TYPE_NORMAL,
    )
    .await
    .expect("create risky3_pool_reserve_usdc");

    // CREATE RISKY4 POOL

    let pool_params = PoolParams {
        name: str_to_array("Risky4 Pool"),
        market_price_currency_symbol: str_to_array("USD"),
        visible: 0,
    };

    create_pool(
        ctx,
        &risky4_pool_keypair,
        pool_authority_keypair,
        curator_keypair.pubkey(),
        pool_params,
    )
    .await
    .expect("create risky4_pool");

    // CREATE USDC-RESERVE IN RISKY4 POOL

    reserve_config.market_price_feed = usdc_price_feed;
    create_reserve(
        ctx,
        &risky4_pool_reserve_usdc_keypair,
        risky4_pool_keypair.pubkey(),
        pool_authority_keypair,
        curator_keypair.pubkey(),
        liquidity_usdc_mint,
        usdc_price_feed,
        reserve_config,
        RESERVE_TYPE_NORMAL,
    )
    .await
    .expect("create risky4_pool_reserve_usdc");

    EnvKeys {
        main_pool: main_pool_keypair,
        risky1_pool: risky1_pool_keypair,
        risky2_pool: risky2_pool_keypair,
        main_pool_reserves: (
            main_pool_reserve_sol_keypair,
            main_pool_reserve_usdc_keypair,
        ),
        risky1_pool_reserves: (
            risky1_pool_reserve_jup_keypair,
            risky1_pool_reserve_usdc_keypair,
        ),
        risky2_pool_reserves: (
            risky2_pool_reserve_bonk_keypair,
            risky2_pool_reserve_usdc_keypair,
        ),
        risky3_pool_reserve_usdc_keypair,
        risky4_pool_reserve_usdc_keypair,
        vault_keypair,
        curator_keypair,
        vaults_authority_keypair,
        position_keypair,
        position_0_keypair,
    }
}

pub struct EnvKeys {
    pub main_pool: Keypair,
    pub risky1_pool: Keypair,
    pub risky2_pool: Keypair,
    pub main_pool_reserves: (Keypair, Keypair),
    pub risky1_pool_reserves: (Keypair, Keypair),
    pub risky2_pool_reserves: (Keypair, Keypair),
    pub risky3_pool_reserve_usdc_keypair: Keypair,
    pub risky4_pool_reserve_usdc_keypair: Keypair,

    pub vault_keypair: Keypair,
    pub curator_keypair: Keypair,
    pub vaults_authority_keypair: Keypair,

    pub position_keypair: Keypair,
    pub position_0_keypair: Keypair,
}

pub async fn setup_vault(
    context: &mut ProgramTestContext,
    vault: &Keypair,
    curator: Pubkey,
    params: VaultParams,
    curator_vaults_authority: &Keypair,
    reserves: Vec<Pubkey>,
) -> Result<(), BanksClientError> {
    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");

    let mut lp_token_mints = vec![];
    let mut lp_token_storages = vec![];

    for reserve_key in reserves.clone().iter() {
        let (lp_token_mint_key, _) = find_lp_token_mint(reserve_key);
        let (lp_token_storage_key, _) =
            find_vault_lp_token_storage(&vault.pubkey(), &lp_token_mint_key);

        lp_token_mints.push(lp_token_mint_key);
        lp_token_storages.push(lp_token_storage_key);
    }

    let reserve_acc = get_account(&mut context.banks_client, reserves[0])
        .await
        .expect("get Reserve");
    let unpacked_reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("unpacking Reserve");
    let liquidity_mint = unpacked_reserve.liquidity.mint;
    let liquidity_buffer = find_vault_liquidity_buffer(&vault.pubkey()).0;

    let ixs = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(1_000_000),
        CreateVault {
            vault: vault.pubkey(),
            curator_vaults_authority: curator_vaults_authority.pubkey(),
            curator,
            liquidity_mint,
            liquidity_buffer,
            reserves,
            lp_token_mints,
            lp_token_storages,
            params,
            liquidity_token_program: spl_token::id(),
        }
        .into_instruction(),
    ];

    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&context.payer.pubkey()),
        &[&context.payer, vault, curator_vaults_authority],
        blockhash,
    );
    context.banks_client.process_transaction(tx).await
}

#[allow(clippy::too_many_arguments)]
pub async fn set_reward_rule(
    context: &mut ProgramTestContext,
    vault: &Keypair,
    index: usize,
    name: String,
    reward_mint: Pubkey,
    rate: Decimal,
    curator: Pubkey,
    curator_vaults_authority: &Keypair,
) -> Result<(), BanksClientError> {
    let vault_acc = get_account(&mut context.banks_client, vault.pubkey())
        .await
        .expect("get vault");
    let unpacked_vault = Vault::try_from_bytes(&vault_acc.data).expect("unpacking Vault");

    let mut rules = unpacked_vault.reward_rules.rules;
    let rule = &mut rules[index];

    let mut name_bytes_zero_ended = [0; REWARD_RULE_NAME_MAX_LEN];
    let name_bytes = name.as_bytes();
    name_bytes_zero_ended[..name_bytes.len()].copy_from_slice(name_bytes);

    rule.name = name_bytes_zero_ended;
    rule.reward_mint = reward_mint;
    rule.set_rate(rate).unwrap();

    let ixs = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(1_000_000),
        SetRewardRules {
            vault: vault.pubkey(),
            curator_vaults_authority: curator_vaults_authority.pubkey(),
            curator,
            rules: RewardRules { rules },
        }
        .into_instruction(),
    ];

    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");

    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&context.payer.pubkey()),
        &[&context.payer, curator_vaults_authority],
        blockhash,
    );
    context.banks_client.process_transaction(tx).await
}

pub async fn get_slot(context: &mut ProgramTestContext) -> Result<u64, BanksClientError> {
    context
        .banks_client
        .get_slot_with_context(
            Context::current(),
            solana_sdk::commitment_config::CommitmentLevel::Confirmed,
        )
        .await
}
