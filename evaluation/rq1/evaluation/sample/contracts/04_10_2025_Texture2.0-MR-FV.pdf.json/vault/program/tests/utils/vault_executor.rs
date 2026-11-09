use crate::utils::{get_account, update_prices};
use solana_program::pubkey::Pubkey;
use solana_program_test::{BanksClientError, ProgramTestBanksClientExt, ProgramTestContext};
use solana_sdk::program_pack::Pack;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use spl_token::state::Mint;
use super_lendy::instruction::RefreshReserve;
use super_lendy::pda::{find_liquidity_supply, find_lp_token_mint};
use super_lendy::state::reserve::Reserve;
use texture_common::account::PodAccount;
use vault::instruction::{
    AddDistributionItem, AlterVault, ClaimReward, ClosePosition, CreatePosition, CreateVault,
    Deposit, InitRewardSupply, Rebalance, RemoveDistributionItem, Version, Withdraw,
};
use vault::pda::{find_vault_liquidity_buffer, find_vault_lp_token_storage};
use vault::state::{
    position::Position,
    vault::{Vault, VaultParams},
};

#[allow(clippy::too_many_arguments)]
pub async fn create_vault(
    context: &mut ProgramTestContext,
    vault: &Keypair,
    curator: Pubkey,
    params: VaultParams,
    curator_vaults_authority: &Keypair,
    liquidity_mint: Pubkey,
    liquidity_buffer: Pubkey,
    reserves: Vec<Pubkey>,
    lp_token_mints: Vec<Pubkey>,
    lp_token_storages: Vec<Pubkey>,
) -> Result<(), BanksClientError> {
    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");

    let mint_acc = get_account(&mut context.banks_client, liquidity_mint)
        .await
        .expect("getting mint");
    let liquidity_token_program = mint_acc.owner;

    let tx = Transaction::new_signed_with_payer(
        &[CreateVault {
            vault: vault.pubkey(),
            curator_vaults_authority: curator_vaults_authority.pubkey(),
            curator,
            liquidity_mint,
            liquidity_buffer,
            reserves,
            lp_token_mints,
            lp_token_storages,
            params,
            liquidity_token_program,
        }
        .into_instruction()],
        Some(&context.payer.pubkey()),
        &[&context.payer, vault, curator_vaults_authority],
        blockhash,
    );
    context.banks_client.process_transaction(tx).await
}

pub async fn alter_vault(
    context: &mut ProgramTestContext,
    vault: Pubkey,
    curator: Pubkey,
    params: VaultParams,
    curator_vaults_authority: &Keypair,
    distribution_manager: Pubkey,
) -> Result<(), BanksClientError> {
    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");

    let ixs = vec![
        AlterVault {
            vault,
            curator_vaults_authority: curator_vaults_authority.pubkey(),
            distribution_manager,
            curator,
            params,
        }
        .into_instruction(),
        Version {
            print_version: false,
        }
        .into_instruction(),
    ];

    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&curator_vaults_authority.pubkey()),
        &[curator_vaults_authority],
        blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn remove_distribution_item(
    context: &mut ProgramTestContext,
    vault: Pubkey,
    curator: Pubkey,
    curator_vaults_authority: &Keypair,
    lp_token_storage: Pubkey,
    reserve: Pubkey,
) -> Result<(), BanksClientError> {
    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");

    let tx = Transaction::new_signed_with_payer(
        &[RemoveDistributionItem {
            vault,
            curator_vaults_authority: curator_vaults_authority.pubkey(),
            curator,
            lp_token_storage,
            reserve,
        }
        .into_instruction()],
        Some(&context.payer.pubkey()),
        &[&context.payer, curator_vaults_authority],
        blockhash,
    );
    context.banks_client.process_transaction(tx).await
}

pub async fn add_distribution_item(
    context: &mut ProgramTestContext,
    vault: Pubkey,
    curator: Pubkey,
    curator_vaults_authority: &Keypair,
    lp_token_storage: Pubkey,
    lp_token_mint: Pubkey,
    index: u8,
) -> Result<(), BanksClientError> {
    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");

    let tx = Transaction::new_signed_with_payer(
        &[AddDistributionItem {
            vault,
            curator_vaults_authority: curator_vaults_authority.pubkey(),
            curator,
            lp_token_storage,
            lp_token_mint,
            index,
        }
        .into_instruction()],
        Some(&context.payer.pubkey()),
        &[&context.payer, curator_vaults_authority],
        blockhash,
    );
    context.banks_client.process_transaction(tx).await
}

pub async fn create_position(
    context: &mut ProgramTestContext,
    position: &Keypair,
    vault: Pubkey,
    owner: &Keypair,
) -> Result<(), BanksClientError> {
    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");

    let tx = Transaction::new_signed_with_payer(
        &[CreatePosition {
            position: position.pubkey(),
            vault,
            owner: owner.pubkey(),
        }
        .into_instruction()],
        Some(&context.payer.pubkey()),
        &[&context.payer, position, owner],
        blockhash,
    );
    context.banks_client.process_transaction(tx).await
}

#[allow(clippy::too_many_arguments)]
pub async fn deposit(
    context: &mut ProgramTestContext,
    position: Pubkey,
    vault: Pubkey,
    owner: &Keypair,
    liquidity_mint: Pubkey,
    liquidity_source: Pubkey,
    superlendy_program_authority: Pubkey,
    amount: u64,
) -> Result<(), BanksClientError> {
    let vault_acc = get_account(&mut context.banks_client, vault)
        .await
        .expect("getting Vault account");
    let unpacked_vault = Vault::try_from_bytes(&vault_acc.data).expect("unpacking Vault");

    let liquidity_token_program = get_account(&mut context.banks_client, liquidity_mint)
        .await
        .expect("getting mint")
        .owner;

    let mut reserves = vec![];
    let mut reserve_liquidity_supplies = vec![];
    let mut lp_token_mints = vec![];
    let mut lp_token_storages = vec![];
    let mut ixs_1 = vec![];
    let mut ixs_2 = vec![];

    for item in unpacked_vault.distribution.0 {
        if item.is_vacant() {
            break;
        }
        let reserve_key = item.reserve;
        let (lp_supply, _) = find_liquidity_supply(&reserve_key);
        let (lp_token_mint_key, _) = find_lp_token_mint(&reserve_key);
        let (lp_token_storage_key, _) = find_vault_lp_token_storage(&vault, &lp_token_mint_key);

        let reserve_acc = get_account(&mut context.banks_client, reserve_key)
            .await
            .expect("getting Reserve account");
        let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("unpacking Reserve");

        let refresh = RefreshReserve {
            reserve: reserve_key,
            market_price_feed: reserve.config.market_price_feed,
            irm: reserve.config.irm,
        }
        .into_instruction();

        ixs_1.push(refresh);

        reserves.push(reserve_key);
        reserve_liquidity_supplies.push(lp_supply);
        lp_token_mints.push(lp_token_mint_key);
        lp_token_storages.push(lp_token_storage_key);
    }

    ixs_2.push(
        Deposit {
            position,
            vault,
            owner: owner.pubkey(),
            liquidity_mint,
            liquidity_source,
            liquidity_buffer: find_vault_liquidity_buffer(&vault).0,
            superlendy_program_authority,
            reserves: reserves.clone(),
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
            liquidity_token_program,
            amount,
        }
        .into_instruction(),
    );
    for _ in 0..5 {
        ixs_2.push(
            Version {
                print_version: false,
            }
            .into_instruction(),
        );
    }

    update_prices(context, &[*reserves.first().unwrap()]).await;

    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");
    let tx = Transaction::new_signed_with_payer(
        &ixs_1,
        Some(&context.payer.pubkey()),
        &[&context.payer],
        blockhash,
    );
    context.banks_client.process_transaction(tx).await?;

    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");
    let tx = Transaction::new_signed_with_payer(
        &ixs_2,
        Some(&context.payer.pubkey()),
        &[&context.payer, owner],
        blockhash,
    );
    context.banks_client.process_transaction(tx).await
}

#[allow(clippy::too_many_arguments)]
pub async fn withdraw(
    context: &mut ProgramTestContext,
    position: Pubkey,
    vault: Pubkey,
    owner: &Keypair,
    liquidity_mint: Pubkey,
    liquidity_destination: Pubkey,
    superlendy_program_authority: Pubkey,
    vlp_amount: u64,
) -> Result<(), BanksClientError> {
    let vault_acc = get_account(&mut context.banks_client, vault)
        .await
        .expect("get vault acc");
    let unpacked_vault = Vault::try_from_bytes(&vault_acc.data).expect("unpacking Vault");

    let liquidity_token_program = get_account(&mut context.banks_client, liquidity_mint)
        .await
        .expect("getting mint")
        .owner;

    let mut reserves = vec![];
    let mut reserve_liquidity_supplies = vec![];
    let mut lp_token_mints = vec![];
    let mut lp_token_storages = vec![];
    let mut ixs_1 = vec![];
    let mut ixs_2 = vec![];

    for item in unpacked_vault.distribution.0 {
        if item.is_vacant() {
            break;
        }
        let reserve_key = item.reserve;
        let (lp_supply, _) = find_liquidity_supply(&reserve_key);
        let (lp_token_mint_key, _) = find_lp_token_mint(&reserve_key);
        let (lp_token_storage_key, _) = find_vault_lp_token_storage(&vault, &lp_token_mint_key);

        let reserve_acc = get_account(&mut context.banks_client, reserve_key)
            .await
            .expect("getting Reserve account");
        let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("unpacking Reserve");

        let refresh = RefreshReserve {
            reserve: reserve_key,
            market_price_feed: reserve.config.market_price_feed,
            irm: reserve.config.irm,
        }
        .into_instruction();

        ixs_1.push(refresh);

        reserves.push(reserve_key);
        reserve_liquidity_supplies.push(lp_supply);
        lp_token_mints.push(lp_token_mint_key);
        lp_token_storages.push(lp_token_storage_key);
    }

    ixs_2.push(
        Withdraw {
            position,
            vault,
            owner: owner.pubkey(),
            liquidity_mint,
            liquidity_buffer: find_vault_liquidity_buffer(&vault).0,
            superlendy_program_authority,
            reserves: reserves.clone(),
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
            liquidity_destination,
            vlp_amount,
            liquidity_token_program,
        }
        .into_instruction(),
    );
    for _ in 0..5 {
        ixs_2.push(
            Version {
                print_version: false,
            }
            .into_instruction(),
        );
    }

    update_prices(context, &[*reserves.first().unwrap()]).await;

    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");
    let tx = Transaction::new_signed_with_payer(
        &ixs_1,
        Some(&context.payer.pubkey()),
        &[&context.payer],
        blockhash,
    );
    context.banks_client.process_transaction(tx).await?;

    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");
    let tx = Transaction::new_signed_with_payer(
        &ixs_2,
        Some(&context.payer.pubkey()),
        &[&context.payer, owner],
        blockhash,
    );
    context.banks_client.process_transaction(tx).await
}

pub async fn close_position(
    context: &mut ProgramTestContext,
    position: Pubkey,
    owner: &Keypair,
) -> Result<(), BanksClientError> {
    let position_acc = get_account(&mut context.banks_client, position)
        .await
        .expect("getting position account");
    let unpacked_position =
        Position::try_from_bytes(&position_acc.data).expect("unpacking position");

    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");

    let tx = Transaction::new_signed_with_payer(
        &[ClosePosition {
            position,
            vault: unpacked_position.vault,
            owner: owner.pubkey(),
        }
        .into_instruction()],
        Some(&context.payer.pubkey()),
        &[&context.payer, owner],
        blockhash,
    );
    context.banks_client.process_transaction(tx).await
}

pub async fn rebalance(
    context: &mut ProgramTestContext,
    vault: Pubkey,
    liquidity_mint: Pubkey,
    liquidity_buffer: Pubkey,
    superlendy_program_authority: Pubkey,
) -> Result<(), BanksClientError> {
    let vault_acc = get_account(&mut context.banks_client, vault)
        .await
        .expect("getting Vault account");
    let unpacked_vault = Vault::try_from_bytes(&vault_acc.data).expect("unpacking Vault");

    let liquidity_token_program = get_account(&mut context.banks_client, liquidity_mint)
        .await
        .expect("getting mint")
        .owner;

    let mut reserves = vec![];
    let mut reserve_liquidity_supplies = vec![];
    let mut lp_token_mints = vec![];
    let mut lp_token_storages = vec![];
    let mut ixs_1 = vec![];
    let mut ixs_2 = vec![];

    for item in unpacked_vault.distribution.effective_items() {
        let reserve_key = item.reserve;
        let (lp_supply, _) = find_liquidity_supply(&reserve_key);
        let (lp_token_mint_key, _) = find_lp_token_mint(&reserve_key);
        let (lp_token_storage_key, _) = find_vault_lp_token_storage(&vault, &lp_token_mint_key);

        let reserve_acc = get_account(&mut context.banks_client, reserve_key)
            .await
            .expect("getting Reserve account");
        let reserve = Reserve::try_from_bytes(&reserve_acc.data).expect("unpacking Reserve");

        let refresh = RefreshReserve {
            reserve: reserve_key,
            market_price_feed: reserve.config.market_price_feed,
            irm: reserve.config.irm,
        }
        .into_instruction();

        ixs_1.push(refresh);

        reserves.push(reserve_key);
        reserve_liquidity_supplies.push(lp_supply);
        lp_token_mints.push(lp_token_mint_key);
        lp_token_storages.push(lp_token_storage_key);
    }

    ixs_2.push(
        Rebalance {
            vault,
            liquidity_mint,
            liquidity_buffer,
            superlendy_program_authority,
            reserves: reserves.clone(),
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
            liquidity_token_program,
        }
        .into_instruction(),
    );
    for _ in 0..5 {
        ixs_2.push(
            Version {
                print_version: false,
            }
            .into_instruction(),
        );
    }

    update_prices(context, &[*reserves.first().unwrap()]).await;

    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");
    let tx = Transaction::new_signed_with_payer(
        &ixs_1,
        Some(&context.payer.pubkey()),
        &[&context.payer],
        blockhash,
    );
    context.banks_client.process_transaction(tx).await?;

    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");
    let tx = Transaction::new_signed_with_payer(
        &ixs_2,
        Some(&context.payer.pubkey()),
        &[&context.payer],
        blockhash,
    );
    context.banks_client.process_transaction(tx).await
}

pub async fn init_reward_supply(
    context: &mut ProgramTestContext,
    vault: Pubkey,
    curator: Pubkey,
    curator_vaults_authority: &Keypair,
    rule_idx: usize,
) -> Result<(), BanksClientError> {
    let vault_acc = get_account(&mut context.banks_client, vault)
        .await
        .expect("get vault acc");
    let unpacked_vault = Vault::try_from_bytes(&vault_acc.data).expect("unpacking Vault");

    let reward_mint = unpacked_vault.reward_rules.rules[rule_idx].reward_mint;
    let token_program = get_account(&mut context.banks_client, reward_mint)
        .await
        .expect("getting mint")
        .owner;

    let ixs = vec![InitRewardSupply {
        reward_mint,
        vault,
        curator_vaults_authority: curator_vaults_authority.pubkey(),
        curator,
        token_program,
    }
    .into_instruction()];

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

pub async fn claim_reward(
    context: &mut ProgramTestContext,
    position: Pubkey,
    vault: Pubkey,
    owner: &Keypair,
    reward_mint: Pubkey,
    reward_destination: Pubkey,
) -> Result<(), BanksClientError> {
    let vault_acc = get_account(&mut context.banks_client, vault)
        .await
        .expect("get vault acc");
    let unpacked_vault = Vault::try_from_bytes(&vault_acc.data).expect("unpacking Vault");

    let token_program = get_account(&mut context.banks_client, reward_mint)
        .await
        .expect("getting mint")
        .owner;

    let mut reserves = vec![];
    let mut reserve_liquidity_supplies = vec![];
    let mut lp_token_mints = vec![];
    let mut lp_token_storages = vec![];
    let mut ixs_2 = vec![];

    for item in unpacked_vault.distribution.0 {
        if item.is_vacant() {
            break;
        }
        let reserve_key = item.reserve;
        let (lp_supply, _) = find_liquidity_supply(&reserve_key);
        let (lp_token_mint_key, _) = find_lp_token_mint(&reserve_key);
        let (lp_token_storage_key, _) = find_vault_lp_token_storage(&vault, &lp_token_mint_key);

        reserves.push(reserve_key);
        reserve_liquidity_supplies.push(lp_supply);
        lp_token_mints.push(lp_token_mint_key);
        lp_token_storages.push(lp_token_storage_key);
    }

    ixs_2.push(
        ClaimReward {
            position,
            vault,
            destination_wallet: reward_destination,
            position_owner: owner.pubkey(),
            reward_mint,
            token_program,
        }
        .into_instruction(),
    );
    for _ in 0..4 {
        ixs_2.push(
            Version {
                print_version: false,
            }
            .into_instruction(),
        );
    }

    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");
    let tx = Transaction::new_signed_with_payer(
        &ixs_2,
        Some(&context.payer.pubkey()),
        &[&context.payer, owner],
        blockhash,
    );
    context.banks_client.process_transaction(tx).await
}

pub async fn create_empty_mint_account(
    context: &mut ProgramTestContext,
    new_account: &Keypair, // New account to be created
    new_owner: Pubkey,
) -> Result<(), BanksClientError> {
    // Define SPL Token program ID

    let rent = context.banks_client.get_rent().await.expect("get rent");
    let lamports = rent.minimum_balance(Mint::LEN);

    // Create instruction to create a new account owned by the SPL Token program
    let create_account_instruction = system_instruction::create_account(
        &context.payer.pubkey(), // Funding account (payer)
        &new_account.pubkey(),   // New account to be created
        lamports,                // Amount of lamports to fund rent-exemption
        Mint::LEN as u64,        // Space required for the SPL Mint
        &new_owner,              // Owner of the new account (SPL Token program)
    );

    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");

    // Create transaction
    let tx = Transaction::new_signed_with_payer(
        &[create_account_instruction], // Add the instruction
        Some(&context.payer.pubkey()), // Fee payer
        &[&context.payer, new_account],
        blockhash,
    );

    // Send transaction
    context.banks_client.process_transaction(tx).await
}

/// Initialize a new SPL Token Mint
pub async fn initialize_spl_mint(
    context: &mut ProgramTestContext,
    mint_account: &Keypair, // Account to be initialized as a mint
    decimals: u8,           // Decimal places for the token
) -> Result<(), BanksClientError> {
    // Create the instruction to initialize the mint
    let init_mint_instruction = spl_token::instruction::initialize_mint(
        &spl_token::id(),        // SPL Token program ID
        &mint_account.pubkey(),  // Mint account to be initialized
        &context.payer.pubkey(), // Authority for minting tokens
        None,                    // Freeze authority (None in this case)
        decimals,                // Number of decimals
    )
    .unwrap();

    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");

    // Create the transaction
    let tx = Transaction::new_signed_with_payer(
        &[init_mint_instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

/// Mint tokens to a specific account
pub async fn mint_tokens(
    context: &mut ProgramTestContext,
    mint_account: &Pubkey,        // Mint account
    destination_account: &Pubkey, // Owner of the destination token account
    amount: u64,                  // Amount of tokens to mint
) -> Result<(), BanksClientError> {
    // Create the instruction to mint tokens
    let mint_to_instruction = spl_token::instruction::mint_to(
        &spl_token::id(),
        mint_account,
        destination_account,
        &context.payer.pubkey(),
        &[], // No multisignature authority required
        amount,
    )
    .unwrap();

    let blockhash = context
        .banks_client
        .get_new_latest_blockhash(&context.last_blockhash)
        .await
        .expect("get latest blockhash");

    // Create the transaction
    let tx = Transaction::new_signed_with_payer(
        &[mint_to_instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        blockhash,
    );

    context.banks_client.process_transaction(tx).await
}
