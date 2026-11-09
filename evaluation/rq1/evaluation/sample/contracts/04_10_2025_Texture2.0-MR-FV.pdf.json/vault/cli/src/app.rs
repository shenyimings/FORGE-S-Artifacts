use std::fmt;
use std::fmt::Display;
use std::io::IsTerminal;
use std::time::Duration;

use anyhow::Result;
use async_recursion::async_recursion;
use bytemuck::Zeroable;
use futures::future::join_all;
use hex::ToHex;
use itertools::MultiUnzip;
use price_proxy::state::price_feed::{
    FeedType, PriceFeed, PriceFeedSource, WormholeVerificationLevel,
};
use price_proxy_client::PriceProxyClient;
use serde_json::json;
use solana_account_decoder::UiAccountData;
use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_request::{RpcError, RpcResponseErrorData, TokenAccountsFilter};
use solana_client::rpc_response::RpcSimulateTransactionResult;
use solana_program::address_lookup_table::instruction::{create_lookup_table, extend_lookup_table};
use solana_program::address_lookup_table::{state::AddressLookupTable, AddressLookupTableAccount};
use solana_program::message::v0;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::VersionedMessage;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::signer::Signer;
use solana_sdk::signers::Signers;
use solana_sdk::transaction::{Transaction, VersionedTransaction};

use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_associated_token_account::instruction::create_associated_token_account;
use super_lendy::instruction::RefreshReserve;
use super_lendy::pda::{find_liquidity_supply, find_lp_token_mint};
use super_lendy::state::curator::Curator;
use super_lendy::state::reserve::Reserve;
use super_lendy::state::{SCALE, WAD};
use texture_common::math::Decimal;
use utils::loaders::{load_position, load_positions, load_vault, load_vaults};
use vault::instruction::{
    AddDistributionItem, AlterVault, ClaimFees, ClaimReward, ClosePosition, CreatePosition,
    CreateVault, Deposit, InitRewardSupply, ProposeDistributionItem, Rebalance,
    RemoveDistributionItem, SetRewardRules, Version, Withdraw, WithdrawReward,
};
use vault::pda::{
    find_reward_supply, find_rewards_program_authority, find_vault_liquidity_buffer,
    find_vault_lp_token_storage,
};
use vault::state::reward::{MAX_REWARD_RULES, REWARD_RULE_NAME_MAX_LEN};
use vault::state::vault::{
    DistributionItem, Vault, VaultParams, MAX_DISTRIBUTION_PROPOSALS, VAULT_DISTRIBUTION_MAX_ITEMS,
};

use texture_common::account::PodAccount;

use crate::{created_vault, to_zero_padded};

macro_rules! println_cmd_out {
    ($out:expr) => {{
        let out = serde_json::to_string_pretty($out).expect("json");
        println!("{out}");
    }};
}

pub struct App {
    pub rpc: RpcClient,
    pub url: String,
    pub authority: Keypair,
    pub priority_fee: Option<u64>,
}

impl App {
    pub async fn send_transaction_by(
        &self,
        mut ixs: Vec<Instruction>,
        signers: &impl Signers,
    ) -> Result<Signature> {
        if let Some(priority_fee) = self.priority_fee {
            let priority_fee_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);
            ixs.push(priority_fee_ix);
        }

        let mut tx = Transaction::new_with_payer(ixs.as_ref(), Some(&self.authority.pubkey()));
        let blockhash = self.rpc.get_latest_blockhash().await?;

        tx.sign(signers, blockhash);

        let signature = self
            .rpc
            .send_and_confirm_transaction_with_spinner(&tx)
            .await
            .map_err(with_logs)?;

        Ok(signature)
    }

    pub async fn send_versioned_transaction_by(
        &self,
        mut ixs: Vec<Instruction>,
        signers: &impl Signers,
        lut: Pubkey,
    ) -> Result<Signature> {
        if let Some(priority_fee) = self.priority_fee {
            let priority_fee_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);
            ixs.push(priority_fee_ix);
        }

        let raw_account = self.rpc.get_account(&lut).await?;
        let address_lookup_table = AddressLookupTable::deserialize(&raw_account.data)?;
        let address_lookup_table_account = AddressLookupTableAccount {
            key: lut,
            addresses: address_lookup_table.addresses.to_vec(),
        };

        let recent_blockhash = self.rpc.get_latest_blockhash().await?;

        let v0_msg = VersionedMessage::V0(v0::Message::try_compile(
            &self.authority.pubkey(),
            &ixs,
            &[address_lookup_table_account],
            recent_blockhash,
        )?);

        let tx = VersionedTransaction::try_new(v0_msg, signers)?;

        let signature = self
            .rpc
            .send_and_confirm_transaction_with_spinner(&tx)
            .await
            .map_err(with_logs)?;

        Ok(signature)
    }

    pub async fn account_exists(&self, key: &Pubkey) -> Result<bool> {
        match self.rpc.get_account(key).await {
            Ok(_) => Ok(true),
            Err(ClientError {
                kind: ClientErrorKind::RpcError(RpcError::ForUser(msg)),
                ..
            }) if msg.starts_with("AccountNotFound") => Ok(false),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn contract_version(&self) {
        let ix = Version {
            print_version: true,
        }
        .into_instruction();

        self.send_transaction_by(vec![ix], &[&self.authority])
            .await
            .expect("Read logs above to see the contract version");
    }

    /// Creates lookup table and extends it with accounts based on passed reserves.
    /// For each Reserve it puts:
    /// 1. Reserve address
    /// 2. Reserve's LP mint
    /// 3. Reserve's liquidity supply
    /// 4. Vault LP token storage (associated with that Reserve's LP tokens)
    ///
    /// Returns address of created table
    pub async fn create_lut(&self, vault: Pubkey, reserves: &[Pubkey]) -> Pubkey {
        let finalized_client =
            RpcClient::new_with_commitment(self.url.clone(), CommitmentConfig::finalized());

        let (create_table_ix, table_address) = create_lookup_table(
            self.authority.pubkey(),
            self.authority.pubkey(),
            finalized_client.get_slot().await.unwrap(),
        );

        self.send_transaction_by(vec![create_table_ix], &[&self.authority])
            .await
            .expect("Sending TX");

        for reserve in reserves {
            let mut addresses = Vec::new();

            let (lp_token_mint, _) = find_lp_token_mint(reserve);
            let (lp_token_storage, _) = find_vault_lp_token_storage(&vault, &lp_token_mint);
            let (supply, _) = find_liquidity_supply(reserve);

            addresses.push(*reserve);
            addresses.push(lp_token_mint);
            addresses.push(supply);
            addresses.push(lp_token_storage);

            let extend_lut = extend_lookup_table(
                table_address,
                self.authority.pubkey(),
                Some(self.authority.pubkey()),
                addresses,
            );

            self.send_transaction_by(vec![extend_lut], &[&self.authority])
                .await
                .expect("Sending TX");
        }

        println!("Created LUT: {}", table_address);

        table_address
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_vault(
        &self,
        name: String,
        description: String,
        mint: Pubkey,
        distribution_manager: Pubkey,
        curator: Pubkey,
        distribution: Vec<DistributionItem>,
        periodic_fee_rate_bps: u16,
    ) {
        if distribution.len() > 24 {
            println!("Can't create Vault with more than 24 distribution items at once. Create 24 items first. And then add items one bu one up to {}", VAULT_DISTRIBUTION_MAX_ITEMS);
            return;
        }

        let vault_keypair = Keypair::new();
        let vault = vault_keypair.pubkey();
        let (liquidity_buffer, _) = find_vault_liquidity_buffer(&vault);

        let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
        for (idx, item) in distribution.iter().enumerate() {
            distribution_bps[idx] = item.share_bps();
        }

        let reserves: Vec<Pubkey> = distribution.iter().map(|i| i.reserve).collect();
        let lut = self.create_lut(vault, &reserves).await;

        let params = VaultParams {
            name: to_zero_padded(&name).expect("name zero padded"),
            description: to_zero_padded(&description).expect("description zero padded"),
            lut,
            distribution_manager,
            distribution_bps,
            periodic_fee_rate_bps,
        };

        let (reserves, lp_token_mints, lp_token_storages) = distribution
            .iter()
            .map(|item| {
                let (lp_token_mint, _) = find_lp_token_mint(&item.reserve);
                let (lp_token_storage, _) = find_vault_lp_token_storage(&vault, &lp_token_mint);

                (item.reserve, lp_token_mint, lp_token_storage)
            })
            .multiunzip();

        let mint_account = self
            .rpc
            .get_account(&mint)
            .await
            .expect("getting mint account");

        let ix = CreateVault {
            curator_vaults_authority: self.authority.pubkey(),
            vault,
            curator,
            liquidity_mint: mint,
            liquidity_buffer,
            params,
            liquidity_token_program: mint_account.owner,
            reserves,
            lp_token_mints,
            lp_token_storages,
        }
        .into_instruction();

        tokio::time::sleep(Duration::from_secs(5)).await;

        // Each 3 reserves eats 200k of compute budget. Thus to process 32 Reserves we need to add
        // 32 / 3 = 10 NOP IXes.
        let nop = Version {
            print_version: false,
        }
        .into_instruction();

        let signature = self
            .send_versioned_transaction_by(
                vec![
                    ix,
                    nop.clone(),
                    nop.clone(),
                    nop.clone(),
                    nop.clone(),
                    nop.clone(),
                    nop.clone(),
                    nop.clone(),
                    nop.clone(),
                    nop.clone(),
                    nop,
                ],
                &[&self.authority, &vault_keypair],
                lut,
            )
            .await
            .expect("Sending TX");

        println_cmd_out!(&created_vault(signature, vault));
        println!("Created vault: {}", vault);
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn alter_vault(
        &self,
        vault: Pubkey,
        name: Option<String>,
        description: Option<String>,
        distribution: Option<Vec<DistributionItem>>,
        distribution_manager: Option<Pubkey>,
        periodic_fee_rate_bps: Option<u16>,
    ) {
        let vault_obj = load_vault(&self.rpc, vault).await.expect("loading vault");
        let curator_data = self
            .rpc
            .get_account_data(&vault_obj.curator)
            .await
            .expect("getting Curator account");
        let curator_obj = Curator::try_from_bytes(&curator_data).expect("unpacking Curator");

        let mut params = VaultParams {
            name: vault_obj.name,
            description: vault_obj.description,
            lut: vault_obj.lut,
            distribution_manager: vault_obj.distribution_manager,
            distribution_bps: vault_obj.distribution.to_bps_array(),
            periodic_fee_rate_bps: vault_obj.periodic_fee_rate_bps(),
        };

        if let Some(name) = name {
            params.name = super::to_zero_padded(&name).expect("name len");
        }

        if let Some(description) = description {
            params.description = super::to_zero_padded(&description).expect("name len");
        }

        if let Some(distribution_manager) = distribution_manager {
            params.distribution_manager = distribution_manager;
        }

        if let Some(distribution) = distribution {
            let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
            for (idx, item) in distribution.iter().enumerate() {
                distribution_bps[idx] = item.share_bps();
            }

            params.distribution_bps = distribution_bps;
        }

        if let Some(periodic_fee_rate_bps) = periodic_fee_rate_bps {
            params.periodic_fee_rate_bps = periodic_fee_rate_bps;
        }

        // Authority could be either curator_vaults_authority or distribution_manager. User chooses
        // which authority to use.

        let ix = AlterVault {
            vault,
            curator_vaults_authority: curator_obj.vaults_authority,
            distribution_manager: vault_obj.distribution_manager, // there should be current distribution manager
            curator: vault_obj.curator,
            params,
        }
        .into_instruction();

        self.send_transaction_by(vec![ix], &[&self.authority])
            .await
            .expect("Sending TX");

        println!("Altered vault: {}", vault);
    }

    pub async fn propose_distribution_item(&self, vault: Pubkey, reserve: Pubkey, index: u8) {
        let vault_obj = load_vault(&self.rpc, vault).await.expect("loading vault");

        let ix = ProposeDistributionItem {
            curator_vaults_authority: self.authority.pubkey(),
            vault,
            curator: vault_obj.curator,
            reserve,
            index,
        }
        .into_instruction();

        self.send_transaction_by(vec![ix], &[&self.authority])
            .await
            .expect("Sending TX");

        println!("Proposed");
    }

    pub async fn add_distribution_item(&self, vault: Pubkey, index: u8) {
        let vault_obj = load_vault(&self.rpc, vault).await.expect("loading vault");

        if index as usize >= MAX_DISTRIBUTION_PROPOSALS {
            println!(
                "Index must be in range [0 - {}]",
                MAX_DISTRIBUTION_PROPOSALS - 1
            );
            return;
        }

        let (lp_token_mint, _) =
            find_lp_token_mint(&vault_obj.distribution_proposals.0[index as usize].reserve);
        let (lp_token_storage, _) = find_vault_lp_token_storage(&vault, &lp_token_mint);

        let ix = AddDistributionItem {
            curator_vaults_authority: self.authority.pubkey(),
            vault,
            curator: vault_obj.curator,
            lp_token_mint,
            lp_token_storage,
            index,
        }
        .into_instruction();

        self.send_transaction_by(vec![ix], &[&self.authority])
            .await
            .expect("Sending TX");

        println!("Added");
    }

    pub async fn remove_distribution_item(&self, vault: Pubkey, reserve: Pubkey) {
        let vault_obj = load_vault(&self.rpc, vault).await.expect("loading vault");

        let (lp_token_mint, _) = find_lp_token_mint(&reserve);
        let (lp_token_storage, _) = find_vault_lp_token_storage(&vault, &lp_token_mint);

        let ix = RemoveDistributionItem {
            curator_vaults_authority: self.authority.pubkey(),
            vault,
            curator: vault_obj.curator,
            reserve,
            lp_token_storage,
        }
        .into_instruction();

        self.send_transaction_by(vec![ix], &[&self.authority])
            .await
            .expect("Sending TX");

        println!("Removed");
    }

    pub async fn rebalance(&self, vault: Pubkey) {
        let (liquidity_buffer, _) = find_vault_liquidity_buffer(&vault);

        let (superlendy_program_authority, _) = super_lendy::pda::find_program_authority();

        let vault_obj = load_vault(&self.rpc, vault).await.expect("loading vault");

        let mint_account = self
            .rpc
            .get_account(&vault_obj.mint)
            .await
            .expect("getting mint account");

        let VaultDeps {
            reserves,
            lp_token_mints,
            lp_token_storages,
            reserve_liquidity_supplies,
            refresh_ixs: mut ixs,
        } = self.find_vault_deps(&vault, &vault_obj).await;

        let ix = Rebalance {
            vault,
            liquidity_mint: vault_obj.mint,
            liquidity_buffer,
            superlendy_program_authority,
            liquidity_token_program: mint_account.owner,
            reserves: reserves.clone(),
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
        }
        .into_instruction();

        ixs.push(ix);

        self.update_prices(&reserves).await;

        self.send_transaction_by(ixs, &[&self.authority])
            .await
            .expect("Sending TX");

        println!("Rebalanced Vault {}", vault);
    }

    pub async fn list_vaults(&self) {
        let vaults = load_vaults(&self.rpc).await.expect("loading vaults");

        for (key, vault) in vaults.iter() {
            println!("Vault at address {}", key);
            println!("Name         : {}", String::from_utf8_lossy(&vault.name));
            println!(
                "Description  : {}",
                String::from_utf8_lossy(&vault.description)
            );
            println!("Mint         : {}", &vault.mint);
            println!("Curator      : {}", vault.curator);
            println!("Distrib. mgr.: {}", vault.distribution_manager);
            println!("LUT          : {}", vault.lut);
            println!("Distribution : \n{}", vault.distribution);
            println!("---------------- Proposed distribution items -----------------------");
            for (index, proposal) in vault.distribution_proposals.0.iter().enumerate() {
                if proposal.can_be_applied_at == 0 {
                    println!("{}: vacant", index);
                } else {
                    let seconds_to_apply =
                        proposal.can_be_applied_at - chrono::Utc::now().timestamp();
                    println!(
                        "{}: time_to_apply: {}  reserve {}",
                        index,
                        if seconds_to_apply < 0 {
                            "NOW".to_string()
                        } else {
                            format!("{} sec", seconds_to_apply)
                        },
                        proposal.reserve
                    );
                }
            }
            println!("------------------------- Reward rules ----------------------------");
            for (index, rule) in vault.reward_rules.rules.iter().enumerate() {
                if rule.reward_mint == Pubkey::default() {
                    println!("{}: vacant", index);
                } else {
                    println!(
                        "{}: name: {}   rate: {}  reward mint: {} ",
                        index,
                        String::from_utf8_lossy(&rule.name),
                        rule.rate().expect("rate from bits"),
                        rule.reward_mint
                    );
                }
            }

            println!("==========================================================================");
        }

        println!("Total vaults number {}", vaults.len());
    }

    pub async fn list_positions(&self) {
        let positions = load_positions(&self.rpc).await.expect("loading positions");

        for (key, position) in positions.iter() {
            println!("Position at address {}: {:#?}", key, position);
            println!("-------------------------------------");
        }

        println!("Total positions number {}", positions.len());
    }

    pub async fn create_position(&self, vault: Pubkey) {
        let position_keypair = Keypair::new();

        let mut ixs = vec![];

        let ix = CreatePosition {
            vault,
            position: position_keypair.pubkey(),
            owner: self.authority.pubkey(),
        }
        .into_instruction();

        ixs.push(ix);

        self.send_transaction_by(ixs, &[&self.authority, &position_keypair])
            .await
            .expect("Sending TX");

        if is_interactive_shell() {
            println!(
                "Created Position {} in Vault {}",
                position_keypair.pubkey(),
                vault
            );
        } else {
            let msg = serde_json::to_string(&json!({
                "event": "created position",
                "position": position_keypair.pubkey().to_string()
            }))
            .unwrap();
            println!("{}", msg);
        }
    }

    pub async fn deposit(&self, position: Pubkey, amount: u64) {
        let position_obj = load_position(&self.rpc, position)
            .await
            .expect("loading position");

        let vault = position_obj.vault;
        let vault_obj = load_vault(&self.rpc, vault).await.expect("loading vault");

        let mint_account = self
            .rpc
            .get_account(&vault_obj.mint)
            .await
            .expect("getting mint account");

        let liquidity_source = get_associated_token_address_with_program_id(
            &self.authority.pubkey(),
            &vault_obj.mint,
            &mint_account.owner,
        );

        let (superlendy_program_authority, _) = super_lendy::pda::find_program_authority();

        let ReserveDeps {
            reserves,
            lp_token_mints,
            lp_token_storages,
            reserve_liquidity_supplies,
            refresh_ixs: mut ixs,
        } = self.prepare_reserve_deps(&vault, &vault_obj).await;

        self.update_prices(&reserves).await;

        let (liquidity_buffer, _) = find_vault_liquidity_buffer(&vault);

        let ix = Deposit {
            vault,
            position,
            owner: self.authority.pubkey(),
            liquidity_mint: vault_obj.mint,
            liquidity_source,
            liquidity_buffer,
            liquidity_token_program: mint_account.owner,
            reserves,
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
            superlendy_program_authority,
            amount,
        }
        .into_instruction();

        ixs.push(ix);

        for _ in 0..4 {
            ixs.push(
                Version {
                    print_version: false,
                }
                .into_instruction(),
            );
        }

        self.send_transaction_by(ixs, &[&self.authority])
            .await
            .expect("Sending TX");

        println!("Deposited {} liquidity to Vault {}", amount, vault);
    }

    pub async fn withdraw(&self, position: Pubkey, vlp_amount: u64) {
        let position_obj = load_position(&self.rpc, position)
            .await
            .expect("loading position");

        let vault = position_obj.vault;

        let vault_obj = load_vault(&self.rpc, vault).await.expect("loading vault");

        let mint_account = self
            .rpc
            .get_account(&vault_obj.mint)
            .await
            .expect("getting mint account");

        let liquidity_destination = get_associated_token_address_with_program_id(
            &self.authority.pubkey(),
            &vault_obj.mint,
            &mint_account.owner,
        );

        let ReserveDeps {
            reserves,
            lp_token_mints,
            lp_token_storages,
            reserve_liquidity_supplies,
            refresh_ixs: mut ixs,
        } = self.prepare_reserve_deps(&vault, &vault_obj).await;

        self.update_prices(&reserves).await;

        let (superlendy_program_authority, _) = super_lendy::pda::find_program_authority();

        let (liquidity_buffer, _) = find_vault_liquidity_buffer(&vault);

        let ix = Withdraw {
            vault,
            position,
            owner: self.authority.pubkey(),
            liquidity_mint: vault_obj.mint,
            liquidity_destination,
            liquidity_buffer,
            liquidity_token_program: mint_account.owner,
            reserves,
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
            vlp_amount,
            superlendy_program_authority,
        }
        .into_instruction();

        ixs.push(ix);

        self.send_transaction_by(ixs, &[&self.authority])
            .await
            .expect("Sending TX");

        println!("Returned {} VLP to vault {}", vlp_amount, vault);
    }

    pub async fn claim_fees(&self, vault: Pubkey, min_fee_amount: Option<u64>) {
        let vault_obj = load_vault(&self.rpc, vault).await.expect("loading vault");
        let mint_account = self
            .rpc
            .get_account(&vault_obj.mint)
            .await
            .expect("getting mint account");

        let liquidity_destination = get_associated_token_address_with_program_id(
            &self.authority.pubkey(),
            &vault_obj.mint,
            &mint_account.owner,
        );

        let ReserveDeps {
            reserves,
            lp_token_mints,
            lp_token_storages,
            reserve_liquidity_supplies,
            refresh_ixs: mut ixs,
        } = self.prepare_reserve_deps(&vault, &vault_obj).await;

        self.update_prices(&reserves).await;

        let (superlendy_program_authority, _) = super_lendy::pda::find_program_authority();

        let (liquidity_buffer, _) = find_vault_liquidity_buffer(&vault);

        let ix = ClaimFees {
            vault,
            curator_vaults_authority: self.authority.pubkey(),
            curator: vault_obj.curator,
            liquidity_mint: vault_obj.mint,
            fee_receiver: liquidity_destination,
            liquidity_buffer,
            superlendy_program_authority,
            liquidity_token_program: mint_account.owner,
            reserves,
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
            min_fee_amount: min_fee_amount.unwrap_or_default(),
        }
        .into_instruction();

        ixs.push(ix);

        self.send_transaction_by(ixs, &[&self.authority])
            .await
            .expect("Sending TX");

        println!("Claimed fees for Vault {}", vault);
    }

    pub async fn close_position(&self, position: Pubkey) {
        let position_obj = load_position(&self.rpc, position)
            .await
            .expect("loading position");

        let mut ixs = vec![];

        for reward in position_obj.rewards.rewards {
            if reward.reward_mint == Pubkey::zeroed() {
                continue;
            }
            let mint_account = self
                .rpc
                .get_account(&reward.reward_mint)
                .await
                .expect("getting mint account");

            let destination_wallet = get_associated_token_address_with_program_id(
                &self.authority.pubkey(),
                &reward.reward_mint,
                &mint_account.owner,
            );

            if !self
                .account_exists(&destination_wallet)
                .await
                .expect("check destination_wallet existance")
            {
                println!("Creating wallet {} for reward tokens.", destination_wallet);

                let ix = create_associated_token_account(
                    &self.authority.pubkey(),
                    &self.authority.pubkey(),
                    &reward.reward_mint,
                    &mint_account.owner,
                );

                dbg!(&ix);

                self.send_transaction_by(vec![ix], &vec![&self.authority])
                    .await
                    .expect("Sending create ATA TX");
            }

            let claim_reward_ix = ClaimReward {
                position,
                destination_wallet,
                reward_mint: reward.reward_mint,
                vault: position_obj.vault,
                position_owner: self.authority.pubkey(),
                token_program: mint_account.owner,
            }
            .into_instruction();
            ixs.push(claim_reward_ix);
        }

        let ix = ClosePosition {
            position,
            vault: position_obj.vault,
            owner: self.authority.pubkey(),
        }
        .into_instruction();
        ixs.push(ix);

        self.send_transaction_by(ixs, &[&self.authority])
            .await
            .expect("Sending TX");

        println!("Closed Position {}", position);
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn set_reward_rule(
        &self,
        vault_key: Pubkey,
        index: usize,
        name: String,
        reward_mint: Pubkey,
        rate: f64,
    ) {
        // E.g. MAX_REWARD_RULES = 10. Then index can be 0 .. 9
        if index >= MAX_REWARD_RULES {
            println!("Rule index must be in range [0;{})", MAX_REWARD_RULES);
            return;
        }

        let decimal_rate = Decimal::from_i128_with_scale((rate * WAD as f64) as i128, SCALE)
            .expect("rate conversion");

        let vault_data = self
            .rpc
            .get_account_data(&vault_key)
            .await
            .expect("getting vault account");
        let vault = Vault::try_from_bytes(&vault_data).expect("unpacking vault");

        let mut new_rules = vault.reward_rules;

        let mut name_bytes_zero_ended = [0; REWARD_RULE_NAME_MAX_LEN];
        let name_bytes = name.as_bytes();
        name_bytes_zero_ended[..name_bytes.len()].copy_from_slice(name_bytes);

        new_rules.rules[index].name = name_bytes_zero_ended;
        new_rules.rules[index].reward_mint = reward_mint;

        new_rules.rules[index]
            .set_rate(decimal_rate)
            .expect("set_rate");

        // Theoretically we can specify slot in the past or in future - that's all valid.
        // When 0 is specified - the contract will put current Solana slot there.
        new_rules.rules[index].start_slot = 0;

        let ix = SetRewardRules {
            vault: vault_key,
            curator_vaults_authority: self.authority.pubkey(),
            rules: new_rules,
            curator: vault.curator,
        }
        .into_instruction();

        self.send_transaction_by(vec![ix], &[&self.authority])
            .await
            .expect("Sending TX");

        println!("Rule set");
    }

    pub async fn init_reward_supply(&self, vault_key: Pubkey, reward_mint: Pubkey) {
        let vault_data = self
            .rpc
            .get_account_data(&vault_key)
            .await
            .expect("getting vault account");
        let vault = Vault::try_from_bytes(&vault_data).expect("unpacking vault");

        let mint_account = self
            .rpc
            .get_account(&reward_mint)
            .await
            .expect("getting mint account");

        let ix = InitRewardSupply {
            reward_mint,
            vault: vault_key,
            curator_vaults_authority: self.authority.pubkey(),
            curator: vault.curator,
            token_program: mint_account.owner,
        }
        .into_instruction();

        self.send_transaction_by(vec![ix], &[&self.authority])
            .await
            .expect("Sending TX");

        println!(
            "Created reward supply: {}",
            find_reward_supply(&vault_key, &reward_mint).0
        );
    }

    pub async fn claim_reward(&self, position: Pubkey, reward_mint: Pubkey) {
        let position_obj = load_position(&self.rpc, position)
            .await
            .expect("load position");

        let mint_account = self
            .rpc
            .get_account(&reward_mint)
            .await
            .expect("getting mint account");

        let destination_wallet = get_associated_token_address_with_program_id(
            &self.authority.pubkey(),
            &reward_mint,
            &mint_account.owner,
        );

        if !self
            .account_exists(&destination_wallet)
            .await
            .expect("check destination_wallet existance")
        {
            println!("Creating wallet {} for reward tokens.", destination_wallet);

            let ix = create_associated_token_account(
                &self.authority.pubkey(),
                &self.authority.pubkey(),
                &reward_mint,
                &mint_account.owner,
            );

            self.send_transaction_by(vec![ix], &vec![&self.authority])
                .await
                .expect("Sending create ATA TX");
        }

        let claim_reward_ix = ClaimReward {
            position,
            destination_wallet,
            reward_mint,
            vault: position_obj.vault,
            position_owner: self.authority.pubkey(),
            token_program: mint_account.owner,
        }
        .into_instruction();

        let mut ixs = vec![];

        ixs.push(claim_reward_ix);

        let version = Version {
            print_version: false,
        }
        .into_instruction();
        ixs.push(version);

        self.send_transaction_by(ixs, &[&self.authority])
            .await
            .expect("Sending TX");

        println!("Claimed");
    }

    pub fn reward_supply_addr(&self, vault_key: Pubkey, reward_mint: Pubkey) {
        println!("{}", find_reward_supply(&vault_key, &reward_mint).0);
    }

    pub async fn deposit_reward(&self, vault_key: Pubkey, reward_mint: Pubkey, amount: u64) {
        let reward_supply = find_reward_supply(&vault_key, &reward_mint).0;

        let mint_account = self
            .rpc
            .get_account(&reward_mint)
            .await
            .expect("getting mint account");

        let source_wallet = get_associated_token_address_with_program_id(
            &self.authority.pubkey(),
            &reward_mint,
            &mint_account.owner,
        );

        if !self
            .account_exists(&source_wallet)
            .await
            .expect("check source_wallet existance")
        {
            println!(
                "Error: you don't have SPL account with {} tokens to fund rewards",
                reward_mint
            );
            return;
        }

        let ix = spl_token::instruction::transfer(
            &spl_token::id(),
            &source_wallet,
            &reward_supply,
            &self.authority.pubkey(),
            &[&self.authority.pubkey()],
            amount,
        )
        .unwrap();

        self.send_transaction_by(vec![ix], &[&self.authority])
            .await
            .expect("Sending TX");

        println!("Deposited");
    }

    pub async fn withdraw_reward(&self, vault_key: Pubkey, reward_mint: Pubkey, amount: u64) {
        let vault_data = self
            .rpc
            .get_account_data(&vault_key)
            .await
            .expect("getting vault account");
        let vault = Vault::try_from_bytes(&vault_data).expect("unpacking vault");

        let mint_account = self
            .rpc
            .get_account(&reward_mint)
            .await
            .expect("getting mint account");

        let destination_wallet = get_associated_token_address_with_program_id(
            &self.authority.pubkey(),
            &reward_mint,
            &mint_account.owner,
        );

        if !self
            .account_exists(&destination_wallet)
            .await
            .expect("check destination_wallet existance")
        {
            println!("Creating wallet {} for reward tokens.", destination_wallet);

            let ix = create_associated_token_account(
                &self.authority.pubkey(),
                &self.authority.pubkey(),
                &reward_mint,
                &mint_account.owner,
            );

            self.send_transaction_by(vec![ix], &vec![&self.authority])
                .await
                .expect("Sending create ATA TX");
        }

        let ix = WithdrawReward {
            destination_wallet,
            reward_mint,
            vault: vault_key,
            curator_vaults_authority: self.authority.pubkey(),
            curator: vault.curator,
            token_program: mint_account.owner,
            amount,
        }
        .into_instruction();

        self.send_transaction_by(vec![ix], &[&self.authority])
            .await
            .expect("Sending TX");

        println!("Withdrawn");
    }

    pub async fn list_rewards_balances(&self, vault: Pubkey) {
        let rewards_authority = find_rewards_program_authority(&vault);

        let accounts = self
            .rpc
            .get_token_accounts_by_owner(
                &rewards_authority.0,
                TokenAccountsFilter::ProgramId(spl_token::id()),
            )
            .await
            .expect("getting token accounts");

        println!(
            "Rewards balances for vault {} with rewards authority {}",
            vault, rewards_authority.0
        );

        for account in accounts {
            // println!("account {}    data {:?}", account.pubkey, account.account.data);
            match account.account.data {
                UiAccountData::LegacyBinary(_) => {}
                UiAccountData::Json(parsed) => {
                    println!(
                        "mint {}   amount {}",
                        parsed.parsed.get("info").unwrap().get("mint").unwrap(),
                        parsed
                            .parsed
                            .get("info")
                            .unwrap()
                            .get("tokenAmount")
                            .unwrap()
                            .get("amount")
                            .unwrap()
                    );
                }
                UiAccountData::Binary(_, _) => {}
            }
        }
    }

    #[async_recursion]
    pub async fn update_prices(&self, reserves: &[Pubkey]) {
        let client = PriceProxyClient {
            rpc: RpcClient::new(self.url.clone()),
            authority: Keypair::from_base58_string(&self.authority.to_base58_string()),
            priority_fee: None,
        };
        for reserve in reserves {
            let reserve_data = self
                .rpc
                .get_account_data(reserve)
                .await
                .expect("getting Reserve account");
            let unpacked_reserve =
                Reserve::try_from_bytes(&reserve_data).expect("unpacking Reserve");
            let price_feed_key = unpacked_reserve.config.market_price_feed;
            let price_feed_data = self
                .rpc
                .get_account_data(&price_feed_key)
                .await
                .expect("getting PriceFeed account");
            let price_feed =
                PriceFeed::try_from_bytes(&price_feed_data).expect("unpacking PriceFeed");

            let transform_price_update = if price_feed.feed_type() == FeedType::Transform
                && price_feed.transform_source() == PriceFeedSource::Pyth
            {
                let price_update_keypair = Keypair::new();
                let price_update_key = price_update_keypair.pubkey();

                // Convert feed pubkey to hex.
                let hex: String = price_feed.transform_source_address.encode_hex();

                // Get a Hermes update from Hermes stable.
                let message = client
                    .get_message_by_hex(&hex, None)
                    .await
                    .expect("get message");
                let message = message.first().unwrap();

                // 1. Post a Pyth price update onto Solana.
                let price_update = match price_feed.verification_level() {
                    WormholeVerificationLevel::Full => {
                        let encoded_vaa_keypair = Keypair::new();
                        let encoded_vaa = encoded_vaa_keypair.pubkey();

                        let write_encoded_vaa =
                            client.write_encoded_vaa_ix(message, encoded_vaa).await;
                        self.send_transaction_by(
                            write_encoded_vaa,
                            &[&self.authority, &encoded_vaa_keypair],
                        )
                        .await
                        .expect("Sending TX");
                        client
                            .post_update_ix(price_update_key, message, encoded_vaa)
                            .await
                    }
                    WormholeVerificationLevel::Partial => {
                        client
                            .post_update_atomic_ix(price_update_key, message)
                            .await
                    }
                };
                self.send_transaction_by(price_update, &[&self.authority, &price_update_keypair])
                    .await
                    .expect("Sending TX");

                Some(price_update_key)
            } else {
                None
            };

            let transform_source_address =
                if let Some(transform_price_update) = transform_price_update {
                    transform_price_update
                } else {
                    price_feed.transform_source_address
                };

            match price_feed.source() {
                PriceFeedSource::Pyth => {
                    let price_update_keypair = Keypair::new();
                    let price_update_key = price_update_keypair.pubkey();

                    // Convert feed pubkey to hex.
                    let hex: String = price_feed.source_address.encode_hex();

                    // Get a Hermes update from Hermes stable.
                    let message = client
                        .get_message_by_hex(&hex, None)
                        .await
                        .expect("get message");
                    let message = message.first().unwrap();

                    // 1. Post a Pyth price update onto Solana.
                    let mut price_update = match price_feed.verification_level() {
                        WormholeVerificationLevel::Full => {
                            let encoded_vaa_keypair = Keypair::new();
                            let encoded_vaa = encoded_vaa_keypair.pubkey();

                            let write_encoded_vaa =
                                client.write_encoded_vaa_ix(message, encoded_vaa).await;
                            self.send_transaction_by(
                                write_encoded_vaa,
                                &[&self.authority, &encoded_vaa_keypair],
                            )
                            .await
                            .expect("Sending TX");
                            client
                                .post_update_ix(price_update_key, message, encoded_vaa)
                                .await
                        }
                        WormholeVerificationLevel::Partial => {
                            client
                                .post_update_atomic_ix(price_update_key, message)
                                .await
                        }
                    };

                    // 2. Update Price-feed.
                    let update_price = client
                        .update_price_ix(
                            price_feed_key,
                            price_update_key,
                            transform_source_address,
                            u32::MAX as u64,
                        )
                        .await;

                    price_update.extend(update_price);
                    self.send_transaction_by(
                        price_update,
                        &[&self.authority, &price_update_keypair],
                    )
                    .await
                    .expect("Sending TX");
                    println!("Updated price for feed: {}", price_feed_key);

                    // 3. Close a price update account, recovering the rent.
                    let close_price_update = client.close_price_update_ix(price_update_key).await;
                    self.send_transaction_by(close_price_update, &[&self.authority])
                        .await
                        .expect("Sending TX");
                    println!(
                        "Close a price update account: {}, recovering the rent",
                        price_update_key
                    );
                }
                PriceFeedSource::OffChain => {
                    client
                        .write_price_ix(
                            price_feed_key,
                            price_feed.try_price().unwrap(),
                            chrono::Utc::now().timestamp(),
                        )
                        .await;
                    println!("Updated price for feed: {}", price_feed_key);
                }
                PriceFeedSource::Switchboard | PriceFeedSource::StakePool => {
                    // For switchboard or stakepool we just call PriceProxy::update_price for relevant price feed.
                    let update_price_ixs = client
                        .update_price_ix(
                            price_feed_key,
                            price_feed.source_address,
                            transform_source_address,
                            u32::MAX as u64,
                        )
                        .await;

                    self.send_transaction_by(update_price_ixs, &[&self.authority])
                        .await
                        .expect("Sending TX");
                    println!("Updated price for feed: {}", price_feed_key);
                }
                PriceFeedSource::SuperLendy => {
                    // The algorithm:
                    // 1. Refresh Reserve0 which is the source of LP tokens (used as liquidity in Reserve1 -
                    //    reserve of our final interest)
                    // 2. Call PriceProxy::UpdatePrice for the price feed used in Reserve1. This action will use
                    //    LP token price, calculated by SuperLendy on step 1.
                    self.update_prices(&[price_feed.source_address]).await;

                    let mut ixs = self.refresh_reserves_ix(&[price_feed.source_address]).await;

                    let update_price_ixs = client
                        .update_price_ix(
                            price_feed_key,
                            price_feed.source_address,
                            transform_source_address,
                            u32::MAX as u64,
                        )
                        .await;

                    ixs.extend(update_price_ixs);

                    self.send_transaction_by(ixs, &[&self.authority])
                        .await
                        .expect("Sending TX");
                    println!("Updated price for feed: {}", price_feed_key);
                }
                _ => {
                    println!("ERROR Bad source {}", price_feed.source());
                }
            }

            if let Some(transform_price_update) = transform_price_update {
                // Close a price update account, recovering the rent.
                let close_price_update = client.close_price_update_ix(transform_price_update).await;
                self.send_transaction_by(close_price_update, &[&self.authority])
                    .await
                    .expect("Sending TX");
                println!(
                    "Close a price update account: {}, recovering the rent",
                    transform_price_update
                );
            }
        }
    }

    async fn _update_price(&self, unpacked_reserve: &Reserve) {
        let client = PriceProxyClient {
            rpc: RpcClient::new(self.url.clone()),
            authority: Keypair::from_base58_string(&self.authority.to_base58_string()),
            priority_fee: None,
        };

        let price_feed_key = unpacked_reserve.config.market_price_feed;
        let price_feed_data = self
            .rpc
            .get_account_data(&price_feed_key)
            .await
            .expect("getting PriceFeed account");
        let price_feed = PriceFeed::try_from_bytes(&price_feed_data).expect("unpacking PriceFeed");

        match price_feed.source() {
            PriceFeedSource::Pyth => {
                let price_update_keypair = Keypair::new();
                let price_update_key = price_update_keypair.pubkey();

                // Convert feed pubkey to hex.
                let hex: String = price_feed.source_address.encode_hex();

                // Get a Hermes update from Hermes stable.
                let message = client
                    .get_message_by_hex(&hex, None)
                    .await
                    .expect("get message");
                let message = message.first().unwrap();

                // 1. Post a Pyth price update onto Solana.
                let mut price_update = match price_feed.verification_level() {
                    WormholeVerificationLevel::Full => {
                        let encoded_vaa_keypair = Keypair::new();
                        let encoded_vaa = encoded_vaa_keypair.pubkey();

                        let write_encoded_vaa =
                            client.write_encoded_vaa_ix(message, encoded_vaa).await;
                        self.send_transaction_by(
                            write_encoded_vaa,
                            &[&self.authority, &encoded_vaa_keypair],
                        )
                        .await
                        .expect("Sending TX");
                        client
                            .post_update_ix(price_update_key, message, encoded_vaa)
                            .await
                    }
                    WormholeVerificationLevel::Partial => {
                        client
                            .post_update_atomic_ix(price_update_key, message)
                            .await
                    }
                };

                // 2. Update Price-feed.
                let update_price = client
                    .update_price_ix(
                        price_feed_key,
                        price_update_key,
                        price_update_key,
                        u32::MAX as u64,
                    )
                    .await;

                price_update.extend(update_price);
                self.send_transaction_by(price_update, &[&self.authority, &price_update_keypair])
                    .await
                    .expect("Sending TX");

                // 3. Close a price update account, recovering the rent.
                let close_price_update = client.close_price_update_ix(price_update_key).await;
                self.send_transaction_by(close_price_update, &[&self.authority])
                    .await
                    .expect("Sending TX");
            }
            PriceFeedSource::OffChain => {
                println!(
                    "Skipping price update for OffChain price feed {}",
                    price_feed_key
                );
            }
            _ => {
                println!("ERROR Bad source {}", price_feed.source());
            }
        }
    }

    pub async fn refresh_reserves_ix(&self, reserves: &[Pubkey]) -> Vec<Instruction> {
        let mut ixs = Vec::new();

        for reserve in reserves {
            let reserve_data = self
                .rpc
                .get_account_data(reserve)
                .await
                .expect("getting Reserve account");
            let unpacked_reserve =
                Reserve::try_from_bytes(&reserve_data).expect("unpacking Reserve");

            let refresh_reserve = RefreshReserve {
                reserve: *reserve,
                market_price_feed: unpacked_reserve.config.market_price_feed,
                irm: unpacked_reserve.config.irm,
            }
            .into_instruction();

            ixs.push(refresh_reserve);
        }

        ixs
    }

    async fn prepare_reserve_deps(&self, vault: &Pubkey, vault_obj: &Vault) -> ReserveDeps {
        let (reserves, lp_token_mints, lp_token_storages, reserve_liquidity_supplies, refresh_ixs) =
            join_all(vault_obj.distribution.effective_items().map(|item| async {
                let (lp_token_mint_key, _) = find_lp_token_mint(&item.reserve);
                let (lp_token_storage_key, _) =
                    find_vault_lp_token_storage(vault, &lp_token_mint_key);
                let (lp_supply, _) = find_liquidity_supply(&item.reserve);

                let reserve_data = self
                    .rpc
                    .get_account_data(&item.reserve)
                    .await
                    .expect("getting Reserve account");
                let reserve = Reserve::try_from_bytes(&reserve_data).expect("unpacking Reserve");

                let refresh_ix = RefreshReserve {
                    reserve: item.reserve,
                    market_price_feed: reserve.config.market_price_feed,
                    irm: reserve.config.irm,
                }
                .into_instruction();

                (
                    item.reserve,
                    lp_token_mint_key,
                    lp_token_storage_key,
                    lp_supply,
                    refresh_ix,
                )
            }))
            .await
            .into_iter()
            .multiunzip();

        ReserveDeps {
            reserves,
            lp_token_mints,
            lp_token_storages,
            reserve_liquidity_supplies,
            refresh_ixs,
        }
    }

    async fn find_vault_deps(&self, vault: &Pubkey, vault_obj: &Vault) -> VaultDeps {
        let (reserves, lp_token_mints, lp_token_storages, reserve_liquidity_supplies, refresh_ixs) =
            join_all(vault_obj.distribution.effective_items().map(|item| async {
                let (lp_token_mint_key, _) = find_lp_token_mint(&item.reserve);
                let (lp_token_storage_key, _) =
                    find_vault_lp_token_storage(vault, &lp_token_mint_key);
                let (lp_supply, _) = find_liquidity_supply(&item.reserve);

                let reserve_data = self
                    .rpc
                    .get_account_data(&item.reserve)
                    .await
                    .expect("getting Reserve account");
                let reserve = Reserve::try_from_bytes(&reserve_data).expect("unpacking Reserve");

                let refresh = RefreshReserve {
                    reserve: item.reserve,
                    market_price_feed: reserve.config.market_price_feed,
                    irm: reserve.config.irm,
                }
                .into_instruction();

                (
                    item.reserve,
                    lp_token_mint_key,
                    lp_token_storage_key,
                    lp_supply,
                    refresh,
                )
            }))
            .await
            .into_iter()
            .multiunzip();
        VaultDeps {
            reserves,
            lp_token_mints,
            lp_token_storages,
            reserve_liquidity_supplies,
            refresh_ixs,
        }
    }
}

struct ReserveDeps {
    reserves: Vec<Pubkey>,
    lp_token_mints: Vec<Pubkey>,
    lp_token_storages: Vec<Pubkey>,
    reserve_liquidity_supplies: Vec<Pubkey>,
    refresh_ixs: Vec<Instruction>,
}

struct VaultDeps {
    reserves: Vec<Pubkey>,
    lp_token_mints: Vec<Pubkey>,
    lp_token_storages: Vec<Pubkey>,
    reserve_liquidity_supplies: Vec<Pubkey>,
    refresh_ixs: Vec<Instruction>,
}

struct Logs(Vec<String>);

impl Display for Logs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\nLogs:")?;

        for (i, log) in self.0.iter().enumerate() {
            writeln!(f, "    {:>3}: {}", i + 1, log)?;
        }
        Ok(())
    }
}

pub fn with_logs(mut error: ClientError) -> anyhow::Error {
    let logs = match error.kind {
        ClientErrorKind::RpcError(RpcError::RpcResponseError {
            data:
                RpcResponseErrorData::SendTransactionPreflightFailure(RpcSimulateTransactionResult {
                    ref mut logs,
                    ..
                }),
            ..
        }) => logs.take().map(Logs),
        _ => None,
    };

    if let Some(logs) = logs {
        anyhow::Error::from(error).context(logs)
    } else {
        error.into()
    }
}

fn is_interactive_shell() -> bool {
    let stdout = std::io::stdout();
    stdout.is_terminal()
}
