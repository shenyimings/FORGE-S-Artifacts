use solana_program::clock::Clock;
use solana_program::msg;
use solana_program::program_pack::Pack;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use spl_token_2022::extension::{BaseStateWithExtensions, ExtensionType, PodStateWithExtensions};
use spl_token_2022::pod::PodMint;
use spl_token_2022::state::Account;
use texture_common::account::PodAccount;
use texture_common::remote::system::SystemProgram;
use texture_common::remote::token::SplToken;
use texture_common::utils::verify_key;

use crate::error::VaultError;
use crate::instruction::{
    ClaimRewardAccounts, InitRewardSupplyAccounts, SetRewardRulesAccounts, WithdrawRewardAccounts,
};
use crate::pda;
use crate::pda::{find_reward_supply, find_rewards_program_authority};
use crate::processor::{mint_decimals, seedvec, verify_token_program};
use crate::state::position::Position;
use crate::state::reward::RewardRules;
use crate::state::vault::Vault;
use crate::VaultResult;

impl<'a, 'b> crate::processor::Processor<'a, 'b> {
    #[inline(never)]
    pub fn init_reward_supply(&self) -> VaultResult<()> {
        msg!("init_reward_supply ix");

        let InitRewardSupplyAccounts {
            reward_supply,
            reward_mint,
            vault,
            curator_vaults_authority,
            curator,
            reward_authority,
            token_program,
            system_program,
        } = InitRewardSupplyAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        crate::processor::verify_curator(vault, curator, curator_vaults_authority)?;

        let (expected_reward_supply, reward_supply_bump) =
            find_reward_supply(vault.key, reward_mint.key);

        verify_key(reward_supply.key, &expected_reward_supply, "reward_supply")?;

        let (expected_authority, _authority_bump) = find_rewards_program_authority(vault.key);
        verify_key(
            reward_authority.key,
            &expected_authority,
            "reward authority",
        )?;

        let rent = Rent::get().expect("No Rent");
        let system_program = SystemProgram::new(system_program);

        let reward_supply_data_len = if reward_mint.owner == &spl_token_2022::id() {
            let mint_data = reward_mint.data.borrow();
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
        } else if reward_mint.owner == &spl_token::id() {
            spl_token::state::Account::LEN
        } else {
            msg!("unrecognized reward mint owner {}", reward_mint.owner);
            return Err(VaultError::OperationCanNotBePerformed);
        };

        let pool_key_bytes = vault.key.to_bytes();
        let mint_key_bytes = reward_mint.key.to_bytes();
        let seeds = seedvec![&pool_key_bytes, &mint_key_bytes, pda::REWARD_SUPPLY_SEED];

        let nonce = [reward_supply_bump];
        let mut seeds: crate::processor::SeedVec<'_> = seeds;
        seeds.push(&nonce);

        let init_lamports = rent.minimum_balance(reward_supply_data_len);
        system_program
            .create_account(
                curator_vaults_authority,
                reward_supply,
                reward_supply_data_len as u64,
                init_lamports,
                token_program.key,
            )
            .signed(&[&seeds])?;

        let spl_token = SplToken::new(token_program);
        spl_token
            .init_account3(reward_supply, reward_mint, reward_authority)?
            .call()?;

        Ok(())
    }

    #[inline(never)]
    pub fn set_reward_rules(&self, mut rules: RewardRules) -> VaultResult<()> {
        msg!("set_reward_rules ix");

        let SetRewardRulesAccounts {
            vault,
            curator_vaults_authority,
            curator,
        } = SetRewardRulesAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        crate::processor::verify_curator(vault, curator, curator_vaults_authority)?;

        let mut vault_data = vault.data.borrow_mut();
        let unpacked_vault = Vault::try_from_bytes_mut(vault_data.as_mut())?;

        // When caller doesn't provide non zero start slot Contract puts current Solana slot there.
        let clock = Clock::get().expect("no clock");
        for rule in rules.rules.iter_mut() {
            if rule.start_slot == 0 {
                rule.start_slot = clock.slot;
            }
        }

        unpacked_vault.reward_rules = rules;

        Ok(())
    }

    #[inline(never)]
    pub fn claim_reward(&self) -> VaultResult<()> {
        msg!("claim_reward ix");

        let ClaimRewardAccounts {
            position,
            rewards_supply,
            destination_wallet,
            position_owner,
            vault,
            reward_mint,
            reward_authority,
            token_program,
        } = ClaimRewardAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        verify_token_program(token_program)?;

        let (expected_authority, authority_bump) = find_rewards_program_authority(vault.key);
        verify_key(
            reward_authority.key,
            &expected_authority,
            "reward authority",
        )?;

        let mut position_data = position.data.borrow_mut();
        let position = Position::try_from_bytes_mut(position_data.as_mut())?;

        verify_key(&position.vault, vault.key, "position.vault vs. vault")?;

        verify_key(
            position_owner.key,
            &position.owner,
            "position_owner vs. position.owner",
        )?;

        let expected_rewards_supply = find_reward_supply(vault.key, reward_mint.key);
        verify_key(
            rewards_supply.key,
            &expected_rewards_supply.0,
            "rewards_supply",
        )?;

        let vault_data = vault.data.borrow();
        let unpacked_vault = Vault::try_from_bytes(vault_data.as_ref())?;

        let clock = Clock::get().expect("no clock");

        let touched_rewards_records = position.rewards.accrue_rewards(
            position.issued_vlp_amount()?,
            &unpacked_vault.reward_rules,
            clock.slot,
        )?;
        position
            .rewards
            .set_accrued_slot(clock.slot, touched_rewards_records);

        let reward_record = position.rewards.find_reward(reward_mint.key);

        if reward_record.is_none() {
            msg!("No rewards found for mint {}", reward_mint.key);
            return Err(VaultError::OperationCanNotBePerformed);
        }

        let reward_record = reward_record.unwrap();

        let mint_decimals = mint_decimals(reward_mint)?;

        let claimed_amount = reward_record.1.claim(mint_decimals)?;

        let spl_token = SplToken::new(token_program);
        spl_token
            .transfer(
                rewards_supply,
                Some(reward_mint),
                destination_wallet,
                reward_authority,
                claimed_amount,
                Some(mint_decimals),
            )?
            .signed(&[&[
                &vault.key.to_bytes(),
                pda::AUTHORITY_SEED,
                &[authority_bump],
            ]])?;

        msg!("claimed {}", reward_record.1.accrued_amount);

        let touched_rewards_records = position.rewards.accrue_rewards(
            position.issued_vlp_amount()?,
            &unpacked_vault.reward_rules,
            clock.slot,
        )?;
        position
            .rewards
            .set_accrued_slot(clock.slot, touched_rewards_records);

        Ok(())
    }

    #[inline(never)]
    pub fn withdraw_reward(&self, amount: u64) -> VaultResult<()> {
        msg!("withdraw_reward ix");

        let WithdrawRewardAccounts {
            rewards_supply,
            destination_wallet,
            vault,
            curator_vaults_authority,
            curator,
            reward_mint,
            reward_authority,
            token_program,
        } = WithdrawRewardAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        verify_token_program(token_program)?;

        let (expected_authority, authority_bump) = find_rewards_program_authority(vault.key);
        verify_key(
            reward_authority.key,
            &expected_authority,
            "program authority",
        )?;

        crate::processor::verify_curator(vault, curator, curator_vaults_authority)?;

        // We know at that point that curator authority is OK i.e. it corresponds to vault provided.
        // Also rewards_supply corresponds to the vault provided (checked by accounts macro).
        // Thus Curator can withdraw any amount of rewards from that rewards_supply

        let mint_decimals = mint_decimals(reward_mint)?;

        let spl_token = SplToken::new(token_program);
        spl_token
            .transfer(
                rewards_supply,
                Some(reward_mint),
                destination_wallet,
                reward_authority,
                amount,
                Some(mint_decimals),
            )?
            .signed(&[&[
                &vault.key.to_bytes(),
                pda::AUTHORITY_SEED,
                &[authority_bump],
            ]])?;

        Ok(())
    }
}
