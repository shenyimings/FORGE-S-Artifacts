use solana_program::account_info::AccountInfo;
use solana_program::clock::Clock;
use solana_program::msg;
use solana_program::program::invoke_signed;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use spl_token::state::Account as SplTokenAccount;
use spl_token_2022::extension::{BaseStateWithExtensions, ExtensionType, PodStateWithExtensions};
use spl_token_2022::pod::PodMint;
use spl_token_2022::state::Account;
use super_lendy::instruction::{DepositLiquidity, WithdrawLiquidity};
use super_lendy::pda::find_lp_token_mint;
use super_lendy::state::curator::Curator;
use super_lendy::state::reserve::{Reserve, RESERVE_MODE_RETAIN_LIQUIDITY};
use texture_common::math::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Decimal};
use texture_common::{
    account::PodAccount,
    error,
    remote::{system::SystemProgram, token::SplToken},
    utils::verify_key,
};

use crate::distributor::item::Item;
use crate::distributor::solution::{Deposit, Withdrawal};
use crate::distributor::solver::Solver;
use crate::distributor::Delta;
use crate::instruction::{
    AddDistributionItemAccounts, ClaimFeesAccounts, ProposeDistributionItemAccounts,
    RemoveDistributionItemAccounts,
};
use crate::processor::position::{lp_amounts, lp_exchange_rates};
use crate::processor::{mint_decimals, verify_token_program, vlp_price};
use crate::state::vault::{
    DistributionProposal, MAX_DISTRIBUTION_PROPOSALS, SECONDS_IN_THREE_DAYS,
};
use crate::MIN_FEE_CLAIM_INTERVAL_SEC;
use crate::{
    error::VaultError,
    instruction::{AlterVaultAccounts, CreateVaultAccounts, RebalanceAccounts},
    pda::{
        self, find_program_authority, find_vault_liquidity_buffer, find_vault_lp_token_storage,
        AUTHORITY_SEED,
    },
    processor::{seedvec, SeedVec},
    state::vault::{Vault, VaultParams},
    VaultResult,
};

use super::{verify_account_count, verify_program_authority, Processor};

impl<'a, 'b> Processor<'a, 'b> {
    #[inline(never)]
    pub(super) fn create_vault(
        &self,
        params: VaultParams,
        reserves_count: u8,
        lp_token_mints_count: u8,
        lp_token_storages_count: u8,
    ) -> VaultResult<()> {
        msg!("create_vault ix: {:?}", params);

        let CreateVaultAccounts {
            curator_vaults_authority,
            vault,
            curator,
            program_authority,
            liquidity_mint,
            liquidity_buffer,
            lp_token_program,
            liquidity_token_program,
            system_program,
            reserves,
            lp_token_mints,
            lp_token_storages,
        } = CreateVaultAccounts::from_iter(
            &mut self.accounts.iter(),
            reserves_count as usize,
            lp_token_mints_count as usize,
            lp_token_storages_count as usize,
            self.program_id,
        )?;

        if reserves_count as usize > params.distribution_bps.len() {
            msg!("reserves account list has {} items which is greater then space for distribution {}", reserves_count, params.distribution_bps.len());
            return Err(VaultError::InvalidConfig);
        }

        verify_account_count(
            reserves_count,
            lp_token_mints_count,
            lp_token_storages_count,
            reserves_count, // dummy
        )?;
        verify_program_authority(program_authority.key)?;

        // Check that vault authority is the one from Curator account
        let curator_data = curator.data.borrow();
        let unpacked_curator = Curator::try_from_bytes(curator_data.as_ref())?;

        verify_key(
            curator_vaults_authority.key,
            &unpacked_curator.vaults_authority,
            "vaults authority",
        )?;

        let (expected_liquidity_buffer, liquidity_buffer_bump) =
            find_vault_liquidity_buffer(vault.key);

        verify_key(
            liquidity_buffer.key,
            &expected_liquidity_buffer,
            "liqudity buffer",
        )?;

        params.validate()?;

        // All reserves must operate on same liquidity token
        for (idx, reserve) in reserves.iter().enumerate() {
            let reserve_data = reserve.data.borrow();
            let unpacked_reserve = Reserve::try_from_bytes(reserve_data.as_ref())?;

            if *liquidity_mint.key != unpacked_reserve.liquidity.mint {
                msg!("Reserve at index {} has incorrect mint {} while expected one is {}. All reserves must have same liquidity mint!", idx, unpacked_reserve.liquidity.mint, liquidity_mint.key);
                return Err(VaultError::OperationCanNotBePerformed);
            }
        }

        let system_program = SystemProgram::new(system_program);
        let liquidity_spl_token = SplToken::new(liquidity_token_program);
        let rent = Rent::get().expect("No Rent");

        system_program
            .create_account(
                curator_vaults_authority,
                vault,
                Vault::SIZE as u64,
                rent.minimum_balance(Vault::SIZE),
                self.program_id,
            )
            .call()?;

        let reserves_keys = reserves.iter().map(|r| *r.key).collect();
        let mut vault_data = vault.data.borrow_mut();
        Vault::init_bytes(
            vault_data.as_mut(),
            (params, *liquidity_mint.key, *curator.key, reserves_keys),
        )?;

        let vault_bytes = vault.key.to_bytes();
        let mut seeds = seedvec![&vault_bytes, pda::LIQUIDITY_BUFFER_SEED];
        let bump = [liquidity_buffer_bump];
        seeds.push(&bump);

        let liquidity_buffer_data_len = if liquidity_mint.owner == &spl_token_2022::id() {
            let mint_data = liquidity_mint.data.borrow();
            let state = PodStateWithExtensions::<PodMint>::unpack(&mint_data).map_err(|_| {
                msg!("unpack Token2022 mint failed");
                VaultError::OperationCanNotBePerformed
            })?;

            let mint_extensions = state.get_extension_types().map_err(|_| {
                msg!("get_extension_types for Token2022 failed");
                VaultError::OperationCanNotBePerformed
            })?;

            let required_extensions =
                ExtensionType::get_required_init_account_extensions(&mint_extensions);

            ExtensionType::try_calculate_account_len::<Account>(&required_extensions).map_err(
                |_| {
                    msg!("try_calculate_account_len failed for Token2022 account");
                    VaultError::OperationCanNotBePerformed
                },
            )?
        } else if liquidity_mint.owner == &spl_token::id() {
            spl_token::state::Account::LEN
        } else {
            msg!("unrecognized liquidity mint owner {}", liquidity_mint.owner);
            return Err(VaultError::OperationCanNotBePerformed);
        };

        system_program
            .create_account(
                curator_vaults_authority,
                liquidity_buffer,
                liquidity_buffer_data_len as u64,
                rent.minimum_balance(liquidity_buffer_data_len),
                liquidity_token_program.key,
            )
            .signed(&[&seeds])?;

        liquidity_spl_token
            .init_account3(liquidity_buffer, liquidity_mint, program_authority)?
            .signed(&[&seeds])?;

        let lp_spl_token = SplToken::new(lp_token_program);

        // Prepare accounts for LP token storage
        for (lp_token_storage, lp_token_mint) in lp_token_storages.iter().zip(lp_token_mints) {
            let (expected_lp_token_storage, lp_token_storage_bump) =
                find_vault_lp_token_storage(vault.key, lp_token_mint.key);

            verify_key(
                lp_token_storage.key,
                &expected_lp_token_storage,
                "LP token storage",
            )?;

            let lp_token_mint_bytes = lp_token_mint.key.to_bytes();
            let vault_bytes = vault.key.to_bytes();

            let seeds = seedvec![
                &vault_bytes,
                &lp_token_mint_bytes,
                pda::LP_STORAGE_TOKEN_SEED
            ];

            let nonce = [lp_token_storage_bump];
            let mut seeds: SeedVec<'_> = seeds;
            seeds.push(&nonce);

            system_program
                .create_account(
                    curator_vaults_authority,
                    lp_token_storage,
                    spl_token::state::Account::LEN as u64,
                    rent.minimum_balance(spl_token::state::Account::LEN),
                    &spl_token::id(),
                )
                .signed(&[&seeds])?;

            lp_spl_token
                .init_account3(lp_token_storage, lp_token_mint, program_authority)?
                .call()?;
        }

        Ok(())
    }

    /// AlterVault IX can change name, description and distribution of the Vault.
    /// Distribution can be changed only by means of changing shares. Distribution items can NOT be
    /// removed or added. It is possible to set 0% share though. Which (upon) successful rebalance
    /// will lead to 0 LP tokens from corresponding Reserve to be managed by Vault. Such zeroed
    /// distribution item can be deleted via RemoveDistributionItem IX.
    /// New distribution item can be added via ProposeDistributionItem+AddDistributionItem IX.
    /// It is added with 0% share. Thereafter, share could be increased via AlterVault.
    #[inline(never)]
    pub(super) fn alter_vault(&self, params: VaultParams) -> VaultResult<()> {
        msg!("alter_vault ix: {:?}", params);

        let AlterVaultAccounts {
            curator_vaults_authority,
            vault,
            curator,
            distribution_manager,
        } = AlterVaultAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        let curator_data = curator.data.borrow();
        let unpacked_curator = Curator::try_from_bytes(curator_data.as_ref())?;

        verify_key(
            curator_vaults_authority.key,
            &unpacked_curator.vaults_authority,
            "vaults authority",
        )?;

        let mut vault_data = vault.data.borrow_mut();
        let unpacked_vault = Vault::try_from_bytes_mut(vault_data.as_mut())?;

        verify_key(curator.key, &unpacked_vault.curator, "curator")?;

        verify_key(
            distribution_manager.key,
            &unpacked_vault.distribution_manager,
            "distribution_manager",
        )?;

        let mut requires_curator_authority = false;

        if params.name != unpacked_vault.name {
            unpacked_vault.name = params.name;
            requires_curator_authority = true;
            msg!("name updated");
        }

        if params.description != unpacked_vault.description {
            unpacked_vault.description = params.description;
            requires_curator_authority = true;
            msg!("description updated");
        }

        if params.distribution_manager != unpacked_vault.distribution_manager {
            unpacked_vault.distribution_manager = params.distribution_manager;
            requires_curator_authority = true;
            msg!("distribution_manager updated");
        }

        if params.lut != unpacked_vault.lut {
            unpacked_vault.lut = params.lut;
            requires_curator_authority = true;
            msg!("lut updated");
        }

        if params.periodic_fee_rate_bps != unpacked_vault.periodic_fee_rate_bps() {
            unpacked_vault.set_periodic_fee_rate_bps(params.periodic_fee_rate_bps)?;
            requires_curator_authority = true;
            msg!("periodic_fees_config.fee_rate_bps updated");
        }

        // Name, description and distribution_manager can only be changed by curator_vaults_authority
        if requires_curator_authority && !curator_vaults_authority.is_signer {
            msg!("requested change requires curator_vaults_authority to sign");
            return Err(VaultError::MissingSignature(error::MissingSignature(
                *curator_vaults_authority.key,
            )));
        }

        if params.distribution_bps.len() != unpacked_vault.distribution.0.len() {
            msg!(
                "passed distribution is of len {} while {} expected",
                params.distribution_bps.len(),
                unpacked_vault.distribution.0.len()
            );
            return Err(VaultError::InvalidConfig);
        }

        // Distribution percentage can be changed either by curator_vaults_authority OR by distribution_manager
        if !curator_vaults_authority.is_signer && !distribution_manager.is_signer {
            msg!("either curator_vaults_authority or distribution_manager must sign to change distribution percentage");
            return Err(VaultError::OperationCanNotBePerformed);
        }

        params.validate()?;

        // We can only change distribution items. They must be provided in order. We just rely on
        // proper order.
        // When unused (vacant) item is present in the distribution 0% share for it must be passed
        // in distribution_bps. Thus distribution_bps must be specified for ALL distribution items
        // - even for vacant. This introduces at least partial check of the proper order.
        for (item_from_vault, new_share_bps) in unpacked_vault
            .distribution
            .0
            .iter_mut()
            .zip(params.distribution_bps)
        {
            if item_from_vault.is_vacant() && new_share_bps != 0 {
                msg!(
                    "share for vacant distribution items must be 0 while {} bps provided",
                    new_share_bps
                );
                return Err(VaultError::InvalidConfig);
            }

            if item_from_vault.share_bps() != new_share_bps {
                msg!(
                    "Share for {} changed from {} to {}",
                    item_from_vault.reserve,
                    item_from_vault.share_bps(),
                    new_share_bps
                );
                item_from_vault.set_share_bps(new_share_bps);
            }
        }

        Ok(())
    }

    pub fn verify_curator(
        vault: &AccountInfo<'_>,
        curator: &AccountInfo<'_>,
        curator_vaults_authority: &AccountInfo<'_>,
    ) -> VaultResult<()> {
        let curator_data = curator.data.borrow();
        let unpacked_curator = Curator::try_from_bytes(curator_data.as_ref())?;

        verify_key(
            curator_vaults_authority.key,
            &unpacked_curator.vaults_authority,
            "vaults authority",
        )?;

        let vault_data = vault.data.borrow();
        let unpacked_vault = Vault::try_from_bytes(vault_data.as_ref())?;

        verify_key(curator.key, &unpacked_vault.curator, "curator")?;

        Ok(())
    }

    #[inline(never)]
    pub(super) fn propose_distribution_item(&self, index: u8) -> VaultResult<()> {
        msg!("propose_distribution_item ix");

        let ProposeDistributionItemAccounts {
            curator_vaults_authority,
            vault,
            curator,
            reserve,
        } = ProposeDistributionItemAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        if index as usize >= MAX_DISTRIBUTION_PROPOSALS {
            msg!(
                "Proposed distribution item index {} greater than max allowed {}",
                index,
                MAX_DISTRIBUTION_PROPOSALS - 1
            );
            return Err(VaultError::OperationCanNotBePerformed);
        }

        Self::verify_curator(vault, curator, curator_vaults_authority)?;

        let mut vault_data = vault.data.borrow_mut();
        let unpacked_vault = Vault::try_from_bytes_mut(vault_data.as_mut())?;

        if *reserve.key == Pubkey::default() {
            // Reset distribution_proposal
            unpacked_vault.distribution_proposals.0[index as usize] =
                DistributionProposal::default();
            return Ok(());
        }

        // Reserve provided must operate on same liquidity as Vault itself.
        let reserve_data = reserve.data.borrow();
        let unpacked_reserve = Reserve::try_from_bytes(&reserve_data)?;

        verify_key(
            &unpacked_reserve.liquidity.mint,
            &unpacked_vault.mint,
            "reserve mint",
        )?;

        let clock = Clock::get().expect("no clock");

        // This IX will override previous item at given index - this is intended.
        unpacked_vault.distribution_proposals.0[index as usize].can_be_applied_at =
            clock.unix_timestamp + SECONDS_IN_THREE_DAYS;
        unpacked_vault.distribution_proposals.0[index as usize].reserve = *reserve.key;

        Ok(())
    }

    #[inline(never)]
    pub(super) fn add_distribution_item(&self, index: u8) -> VaultResult<()> {
        msg!("add_distribution_item ix");

        let AddDistributionItemAccounts {
            curator_vaults_authority,
            vault,
            curator,
            lp_token_mint,
            lp_token_storage,
            program_authority,
            token_program,
            system_program,
        } = AddDistributionItemAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        if index as usize >= MAX_DISTRIBUTION_PROPOSALS {
            msg!(
                "Proposed distribution item index {} greater than max allowed {}",
                index,
                MAX_DISTRIBUTION_PROPOSALS - 1
            );
            return Err(VaultError::OperationCanNotBePerformed);
        }

        Self::verify_curator(vault, curator, curator_vaults_authority)?;

        verify_program_authority(program_authority.key)?;

        let mut vault_data = vault.data.borrow_mut();
        let unpacked_vault = Vault::try_from_bytes_mut(vault_data.as_mut())?;

        let proposal = unpacked_vault.distribution_proposals.0[index as usize];
        let clock = Clock::get().expect("no clock");

        if proposal.can_be_applied_at > clock.unix_timestamp {
            msg!(
                "proposal at index {} can be applied at {}. Wait for {} sec and try again.",
                index,
                proposal.can_be_applied_at,
                proposal.can_be_applied_at - clock.unix_timestamp
            );
            return Err(VaultError::OperationCanNotBePerformed);
        }

        if proposal.reserve == Pubkey::default() {
            msg!(
                "proposal at index {} can NOT be applied as there is no Reserve address.",
                index,
            );
            return Err(VaultError::OperationCanNotBePerformed);
        }

        if let Some(item) = unpacked_vault.distribution.find(proposal.reserve) {
            msg!(
                "distribution item for reserve {} with share {} already exist",
                item.reserve,
                item.share_decimal()?
            );
            return Err(VaultError::InvalidConfig);
        }

        let new_item = if let Some(item) = unpacked_vault.distribution.find_vacant() {
            item
        } else {
            msg!("No space left in distribution array");
            return Err(VaultError::ResourceExhausted);
        };

        new_item.reserve = proposal.reserve;
        new_item.set_share_bps(0);

        // Check that LP token mint corresponds to Reserve from proposal
        let expected_lp_mint = find_lp_token_mint(&proposal.reserve);
        verify_key(lp_token_mint.key, &expected_lp_mint.0, "lp mint")?;

        let (expected_lp_token_storage, lp_token_storage_bump) =
            find_vault_lp_token_storage(vault.key, lp_token_mint.key);

        verify_key(
            lp_token_storage.key,
            &expected_lp_token_storage,
            "LP token storage",
        )?;

        let system_program = SystemProgram::new(system_program);
        let spl_token = SplToken::new(token_program);
        let rent = Rent::get().expect("No Rent");

        let create_account = |account_info: &AccountInfo<'b>, length, seeds: SeedVec<'_>, bump| {
            let init_lamports = rent.minimum_balance(length);

            let nonce = [bump];
            let mut seeds: SeedVec<'_> = seeds;
            seeds.push(&nonce);

            system_program
                .create_account(
                    curator_vaults_authority,
                    account_info,
                    length as u64,
                    init_lamports,
                    &spl_token::id(),
                )
                .signed(&[&seeds])
                .map_err(VaultError::from)
        };

        let lp_token_mint_bytes = lp_token_mint.key.to_bytes();
        let vault_bytes = vault.key.to_bytes();
        let seeds = seedvec![
            &vault_bytes,
            &lp_token_mint_bytes,
            pda::LP_STORAGE_TOKEN_SEED
        ];
        create_account(
            lp_token_storage,
            spl_token::state::Account::LEN,
            seeds,
            lp_token_storage_bump,
        )?;

        spl_token
            .init_account3(lp_token_storage, lp_token_mint, program_authority)?
            .call()?;

        // Reset applied proposal
        unpacked_vault.distribution_proposals.0[index as usize] = DistributionProposal::default();

        Ok(())
    }

    #[inline(never)]
    pub(super) fn remove_distribution_item(&self, reserve: Pubkey) -> VaultResult<()> {
        msg!("remove_distribution_item ix");

        let RemoveDistributionItemAccounts {
            curator_vaults_authority,
            vault,
            curator,
            lp_token_storage,
            program_authority,
            token_program,
        } = RemoveDistributionItemAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        let authority_bump = verify_program_authority(program_authority.key)?;

        let curator_data = curator.data.borrow();
        let unpacked_curator = Curator::try_from_bytes(curator_data.as_ref())?;

        verify_key(
            curator_vaults_authority.key,
            &unpacked_curator.vaults_authority,
            "vaults authority",
        )?;

        let lp_mint = find_lp_token_mint(&reserve);

        let (expected_lp_token_storage, _) = find_vault_lp_token_storage(vault.key, &lp_mint.0);

        verify_key(
            lp_token_storage.key,
            &expected_lp_token_storage,
            "LP token storage",
        )?;

        let mut vault_data = vault.data.borrow_mut();
        let unpacked_vault = Vault::try_from_bytes_mut(vault_data.as_mut())?;

        verify_key(curator.key, &unpacked_vault.curator, "curator")?;

        let unpacked_lp_storage = SplTokenAccount::unpack(&lp_token_storage.data.borrow())
            .map_err(VaultError::SplTokenUnpack)?;

        if unpacked_lp_storage.amount != 0 {
            msg!("distribution item with non zero associated LP balance can not be deleted. lp balance {}", unpacked_lp_storage.amount);
            return Err(VaultError::OperationCanNotBePerformed);
        }

        if let Some(item) = unpacked_vault.distribution.find(reserve) {
            if item.share_bps() != 0 {
                msg!("distribution item has {} share. Set it to 0 first and then try remove it again.", item.share_bps());
                return Err(VaultError::OperationCanNotBePerformed);
            }
            item.mark_vacant();
        } else {
            msg!("distribution item for reserve {} not found", reserve);
            return Err(VaultError::OperationCanNotBePerformed);
        }

        let spl_token = SplToken::new(token_program);
        spl_token
            .close(
                lp_token_storage,
                curator_vaults_authority,
                program_authority,
            )?
            .signed(&[&[pda::AUTHORITY_SEED, &[authority_bump]]])?;

        Ok(())
    }

    #[inline(never)]
    pub fn rebalance(
        &self,
        reserves_count: u8,
        reserve_liquidity_supplies_count: u8,
        lp_token_mints_count: u8,
        lp_token_storages_count: u8,
    ) -> VaultResult<()> {
        msg!("rebalance ix");

        let RebalanceAccounts {
            vault,
            program_authority,
            liquidity_mint,
            liquidity_buffer,
            superlendy_program_authority,
            lp_token_program,
            liquidity_token_program,
            superlendy_program,
            reserves,
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
        } = RebalanceAccounts::from_iter(
            &mut self.accounts.iter(),
            reserves_count as usize,
            reserve_liquidity_supplies_count as usize,
            lp_token_mints_count as usize,
            lp_token_storages_count as usize,
            self.program_id,
        )?;

        verify_token_program(liquidity_token_program)?;

        verify_account_count(
            reserves_count,
            reserve_liquidity_supplies_count,
            lp_token_mints_count,
            lp_token_storages_count,
        )?;

        let vault_data = vault.data.borrow();
        let unpacked_vault = Vault::try_from_bytes(&vault_data)?;
        let distribution_count = unpacked_vault.distribution.count() as u8;
        if distribution_count != reserves_count {
            return Err(VaultError::InvalidAccountsCount);
        }

        verify_program_authority(program_authority.key)?;

        rebalance(
            vault,
            lp_token_storages,
            reserves,
            reserve_liquidity_supplies,
            lp_token_mints,
            program_authority,
            liquidity_buffer,
            superlendy_program_authority,
            lp_token_program,
            liquidity_token_program,
            superlendy_program,
            liquidity_mint,
            Delta::Keep,
        )?;

        Ok(())
    }

    #[inline(never)]
    pub fn claim_fees(
        &self,
        reserves_count: u8,
        reserve_liquidity_supplies_count: u8,
        lp_token_mints_count: u8,
        lp_token_storages_count: u8,
        min_fee_amount: u64,
    ) -> VaultResult<()> {
        msg!("claim_fees ix");

        let ClaimFeesAccounts {
            vault,
            curator_vaults_authority,
            curator,
            liquidity_mint,
            fee_receiver,
            liquidity_buffer,
            program_authority,
            superlendy_program_authority,
            lp_token_program,
            liquidity_token_program,
            superlendy_program,
            reserves,
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
        } = ClaimFeesAccounts::from_iter(
            &mut self.accounts.iter(),
            reserves_count as usize,
            reserve_liquidity_supplies_count as usize,
            lp_token_mints_count as usize,
            lp_token_storages_count as usize,
            self.program_id,
        )?;

        verify_token_program(liquidity_token_program)?;

        verify_account_count(
            reserves_count,
            reserve_liquidity_supplies_count,
            lp_token_mints_count,
            lp_token_storages_count,
        )?;

        let curator_data = curator.data.borrow();
        let unpacked_curator = Curator::try_from_bytes(curator_data.as_ref())?;

        verify_key(
            curator_vaults_authority.key,
            &unpacked_curator.vaults_authority,
            "vaults authority",
        )?;

        let vault_data = vault.data.borrow();
        let unpacked_vault = Vault::try_from_bytes(&vault_data)?;

        verify_key(curator.key, &unpacked_vault.curator, "curator")?;

        let distribution_count = unpacked_vault.distribution.count() as u8;
        if distribution_count != reserves_count {
            return Err(VaultError::InvalidAccountsCount);
        }

        let authority_bump = verify_program_authority(program_authority.key)?;

        let clock = Clock::get().expect("no clock");
        let time_since_last_fee_claim = (clock.unix_timestamp as u64)
            .checked_sub(unpacked_vault.periodic_fee_claimed_at)
            .ok_or_else(|| VaultError::Internal("fee was claimed in the future".into()))?;
        if time_since_last_fee_claim < MIN_FEE_CLAIM_INTERVAL_SEC.as_secs() {
            return Err(VaultError::EearlyFeesClaim(
                MIN_FEE_CLAIM_INTERVAL_SEC.as_secs() - time_since_last_fee_claim,
            ));
        }

        let last_vlp_price = unpacked_vault.last_vlp_price()?;

        let liquidity_decimals = mint_decimals(liquidity_mint)?;
        let lp_exchange_rates = lp_exchange_rates(&reserves)?;
        let lp_amounts = lp_amounts(&lp_token_storages, liquidity_decimals)?;
        let current_vlp_supply = unpacked_vault.vlp_supply()?;
        let current_vlp_price = vlp_price(&lp_exchange_rates, &lp_amounts, current_vlp_supply)?;

        if current_vlp_price <= last_vlp_price {
            msg!("impossible to claim fees: no profit");
            return Err(VaultError::OperationCanNotBePerformed);
        }

        let periodic_fee_rate_bps = unpacked_vault.decimal_periodic_fee_rate_bps()?;

        let desired_liquidity_amount = periodic_fee(
            current_vlp_price,
            last_vlp_price,
            current_vlp_supply,
            liquidity_decimals,
            periodic_fee_rate_bps,
        )?;

        drop(vault_data);

        let rebalance_outcome = rebalance(
            vault,
            lp_token_storages,
            reserves,
            reserve_liquidity_supplies,
            lp_token_mints,
            program_authority,
            liquidity_buffer,
            superlendy_program_authority,
            lp_token_program,
            liquidity_token_program,
            superlendy_program,
            liquidity_mint,
            Delta::Decrease(desired_liquidity_amount),
        )?;

        let liquidity_amount = rebalance_outcome.expected_buffer_change;

        if liquidity_amount < 0 || liquidity_amount < min_fee_amount as i128 {
            msg!(
                "requested to receive at least {} liquidity as fee, bun can only withdraw {}",
                min_fee_amount,
                liquidity_amount,
            );
            return Err(VaultError::NotEnoguhLiquidity);
        }

        let spl_token = SplToken::new(liquidity_token_program);
        spl_token
            .transfer(
                liquidity_buffer,
                Some(liquidity_mint),
                fee_receiver,
                program_authority,
                liquidity_amount as u64,
                Some(liquidity_decimals),
            )?
            .signed(&[&[pda::AUTHORITY_SEED, &[authority_bump]]])?;

        let mut vault_data = vault.data.borrow_mut();
        let unpacked_vault = Vault::try_from_bytes_mut(&mut vault_data)?;
        unpacked_vault.set_last_vlp_price(current_vlp_price)?;
        unpacked_vault.periodic_fee_claimed_at = clock.unix_timestamp as u64;

        Ok(())
    }
}

fn periodic_fee(
    current_vlp_price: Decimal,
    last_vlp_price: Decimal,
    current_vlp_supply: Decimal,
    liquidity_decimals: u8,
    periodic_fee_rate_bps: Decimal,
) -> Result<u64, VaultError> {
    let vlp_price_gain_pct = current_vlp_price
        .checked_sub(last_vlp_price)?
        .checked_div(last_vlp_price)?;
    let gain_fee_pct = vlp_price_gain_pct.checked_mul(periodic_fee_rate_bps)?;
    let vlp_amount = current_vlp_supply.checked_mul(gain_fee_pct)?;
    let desired_liquidity_amount = vlp_amount
        .checked_mul(current_vlp_price)?
        .to_lamports_floor(liquidity_decimals)?;
    Ok(desired_liquidity_amount)
}

pub struct RebalanceOutcome {
    pub expected_buffer_change: i128,
}

#[allow(clippy::too_many_arguments)]
pub fn rebalance<'a>(
    vault: &AccountInfo<'a>,
    lp_token_storages: Vec<&AccountInfo<'a>>,
    reserves: Vec<&AccountInfo<'a>>,
    reserve_liquidity_supplies: Vec<&AccountInfo<'a>>,
    lp_token_mints: Vec<&AccountInfo<'a>>,
    program_authority: &AccountInfo<'a>,
    liquidity_buffer: &AccountInfo<'a>,
    superlendy_program_authority: &AccountInfo<'a>,
    lp_token_program: &AccountInfo<'a>,
    liquidity_token_program: &AccountInfo<'a>,
    superlendy_program: &AccountInfo<'a>,
    liquidity_mint: &AccountInfo<'a>,
    liquidity_delta: Delta,
) -> Result<RebalanceOutcome, VaultError> {
    let liquidity_decimals = mint_decimals(liquidity_mint)?;

    let vault_data = vault.data.borrow();
    let unpacked_vault = Vault::try_from_bytes(&vault_data)?;

    let mut items = Vec::with_capacity(reserves.len());

    for (idx, ((lp_token_storage, reserve), distr_item)) in lp_token_storages
        .iter()
        .zip(&reserves)
        .zip(unpacked_vault.distribution.effective_items())
        .enumerate()
    {
        let token_account: SplTokenAccount =
            SplTokenAccount::unpack(&lp_token_storage.data.borrow())
                .map_err(VaultError::SplTokenUnpack)?;
        let lp_amount = token_account.amount;

        let reserve_data = reserve.data.borrow();
        let unpacked_reserve = Reserve::try_from_bytes(&reserve_data)?;
        let lp_exchange_rate = unpacked_reserve.lp_exchange_rate()?;
        let max_withdraw_amount = if unpacked_reserve.mode == RESERVE_MODE_RETAIN_LIQUIDITY {
            Decimal::ZERO
        } else {
            unpacked_reserve.max_withdraw_liquidity_amount()?
        };

        let share = Decimal::from_basis_points(distr_item.share_bps() as u32)?;

        let liquidity_amount = lp_exchange_rate
            .decimal_lp_to_liquidity(Decimal::from_lamports(lp_amount, liquidity_decimals)?)?;

        let lower_bound = liquidity_amount
            .checked_sub(max_withdraw_amount)
            .unwrap()
            .max(Decimal::ZERO);
        let upper_bound = Decimal::from_lamports(
            unpacked_reserve.config.max_total_liquidity,
            liquidity_decimals,
        )?
        .checked_sub(unpacked_reserve.liquidity.total_liquidity()?)
        .unwrap()
        .checked_add(liquidity_amount)?;

        let bounds = lower_bound..=upper_bound;

        let item = Item::new(
            idx,
            bounds,
            liquidity_amount,
            lp_amount,
            share,
            lp_exchange_rate,
        );
        items.push(item);
    }

    let solver = Solver::new(
        items,
        match liquidity_delta {
            Delta::Decrease(amount) => {
                Decimal::ZERO.checked_sub(Decimal::from_lamports(amount, liquidity_decimals)?)?
            }
            Delta::Increase(amount) => Decimal::from_lamports(amount, liquidity_decimals)?,
            Delta::Keep => Decimal::ZERO,
        },
    );

    let solution = solver.solve(|_| {}, liquidity_decimals)?;

    match liquidity_delta {
        Delta::Decrease(_) => {}
        Delta::Increase(_) => {
            // Do not allow to worsen balance when adding funds. Than means that upper limit
            // of some reserve was reached.
            if solution.balance_deviation_after > solution.balance_deviation_before {
                return Err(VaultError::WorsenBalance {
                    deviation_before: solution.balance_deviation_before,
                    deviation_after: solution.balance_deviation_after,
                });
            }
        }
        Delta::Keep => {
            // Asked to keep liquidity in reserves and just rebalance...
            if solution.withdrawals(liquidity_decimals).count() == 0
                && solution.deposits(liquidity_decimals).count() == 0
            {
                // ...but Solver can't find any actions to improve the balance...
                return Err(VaultError::ImpossibleToImproveBalance);
            } else if solution.balance_deviation_after > solution.balance_deviation_before {
                // ...or it only can worsen the balance
                return Err(VaultError::WorsenBalance {
                    deviation_before: solution.balance_deviation_before,
                    deviation_after: solution.balance_deviation_after,
                });
            }
        }
    }

    // Withdraw
    for withdrawal in solution.withdrawals(liquidity_decimals) {
        let Withdrawal {
            reserve_idx,
            lp_amount_to_return,
            ..
        } = withdrawal.expect("withdrawal item");

        if lp_amount_to_return == 0 {
            continue;
        }

        #[cfg(all(not(test), feature = "custom-heap", target_os = "solana"))]
        let heap_start = unsafe {
            use crate::entrypoint::A;
            A.pos()
        };

        let &lp_token_storage = lp_token_storages.get(reserve_idx).unwrap();
        let &reserve = reserves.get(reserve_idx).unwrap();
        let &reserve_liquidity_supply = reserve_liquidity_supplies.get(reserve_idx).unwrap();
        let &lp_token_mint = lp_token_mints.get(reserve_idx).unwrap();

        let withdraw_ix = WithdrawLiquidity {
            authority: *program_authority.key,
            source_lp_wallet: *lp_token_storage.key,
            destination_liquidity_wallet: *liquidity_buffer.key,
            reserve: *reserve.key,
            liquidity_mint: *liquidity_mint.key,
            liquidity_token_program: *liquidity_token_program.key,
            lp_amount: lp_amount_to_return,
        };
        let ix = withdraw_ix.into_instruction();
        let account_infos = [
            program_authority.clone(),
            lp_token_storage.clone(),
            liquidity_buffer.clone(),
            reserve.clone(),
            reserve_liquidity_supply.clone(),
            liquidity_mint.clone(),
            lp_token_mint.clone(),
            superlendy_program_authority.clone(),
            lp_token_program.clone(),
            liquidity_token_program.clone(),
        ];
        let (_, bump_seed) = find_program_authority();
        let seeds_0 = &[AUTHORITY_SEED, &[bump_seed]];
        let seeds = [seeds_0.as_slice()];

        invoke_signed(&ix, &account_infos, seeds.as_slice()).map_err(VaultError::SuperLendyCpi)?;

        #[cfg(all(not(test), feature = "custom-heap", target_os = "solana"))]
        unsafe {
            use crate::entrypoint::A;
            A.move_cursor(heap_start);
        }
    }

    // Deposit
    for deposit in solution.deposits(liquidity_decimals) {
        let Deposit {
            reserve_idx,
            liquidity_amount,
        } = deposit.expect("deposit item");

        if liquidity_amount == 0 {
            continue;
        }

        #[cfg(all(not(test), feature = "custom-heap", target_os = "solana"))]
        let heap_start = unsafe {
            use crate::entrypoint::A;
            A.pos()
        };

        let &lp_token_storage = lp_token_storages.get(reserve_idx).unwrap();
        let &reserve = reserves.get(reserve_idx).unwrap();
        let &reserve_liquidity_supply = reserve_liquidity_supplies.get(reserve_idx).unwrap();
        let &lp_token_mint = lp_token_mints.get(reserve_idx).unwrap();

        let deposit_ix = DepositLiquidity {
            authority: *program_authority.key,
            source_liquidity_wallet: *liquidity_buffer.key,
            destination_lp_wallet: *lp_token_storage.key,
            reserve: *reserve.key,
            liquidity_mint: *liquidity_mint.key,
            liquidity_token_program: *liquidity_token_program.key,
            amount: liquidity_amount,
        };
        let ix = deposit_ix.into_instruction();
        let account_infos = [
            superlendy_program.clone(),
            program_authority.clone(),
            liquidity_buffer.clone(),
            lp_token_storage.as_ref().clone(),
            reserve.as_ref().clone(),
            reserve_liquidity_supply.clone(),
            liquidity_mint.clone(),
            lp_token_mint.clone(),
            superlendy_program_authority.clone(),
            lp_token_program.clone(),
            liquidity_token_program.clone(),
        ];

        let (_, bump_seed) = find_program_authority();
        let seeds_0 = &[AUTHORITY_SEED, &[bump_seed]];
        let seeds = [seeds_0.as_slice()];
        invoke_signed(&ix, &account_infos, &seeds).map_err(VaultError::SuperLendyCpi)?;

        #[cfg(all(not(test), feature = "custom-heap", target_os = "solana"))]
        unsafe {
            use crate::entrypoint::A;
            A.move_cursor(heap_start);
        }
    }

    Ok(RebalanceOutcome {
        expected_buffer_change: solution.expected_buffer_change,
    })
}
