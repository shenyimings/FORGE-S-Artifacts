use solana_program::account_info::AccountInfo;
use solana_program::msg;
use solana_program::program_pack::Pack;
use solana_program::rent::Rent;
use solana_program::sysvar::clock::Clock;
use solana_program::sysvar::Sysvar;
use spl_token::state::Account as SplTokenAccount;
use super_lendy::state::reserve::{LpExchangeRate, Reserve};
use texture_common::math::CheckedAdd;
use texture_common::remote::token::SplToken;
use texture_common::{
    account::PodAccount,
    math::{CheckedDiv, CheckedMul, Decimal},
    remote::system::SystemProgram,
    utils::verify_key,
};

use crate::distributor::Delta;
use crate::pda;
use crate::processor::vault::rebalance;
use crate::processor::{mint_decimals, verify_token_program, vlp_price};
use crate::{
    error::VaultError,
    instruction::{
        ClosePositionAccounts, CreatePositionAccounts, DepositAccounts, WithdrawAccounts,
    },
    state::{
        position::{Position, PositionParams},
        vault::Vault,
    },
    VaultResult,
};

use super::{verify_account_count, verify_program_authority, Processor};

impl<'a, 'b> Processor<'a, 'b> {
    #[inline(never)]
    pub fn create_position(&self) -> VaultResult<()> {
        msg!("create_position ix");

        let CreatePositionAccounts {
            vault,
            position,
            owner,
            program_authority,
            system_program,
        } = CreatePositionAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        verify_program_authority(program_authority.key)?;

        // Try to deserialize Vault to make sure its really a vault
        let vault_data = vault.data.borrow();
        let _unpacked_vault = Vault::try_from_bytes(&vault_data)?;

        let clock = Clock::get().map_err(VaultError::Clock)?;
        let current_timestamp = clock.unix_timestamp;

        let system_program = SystemProgram::new(system_program);
        let rent = Rent::get().expect("No Rent");

        system_program
            .create_account(
                owner,
                position,
                Position::SIZE as u64,
                rent.minimum_balance(Position::SIZE),
                self.program_id,
            )
            .call()?;

        let params = PositionParams {
            vault: *vault.key,
            created_at: current_timestamp,
        };

        let mut position_data = position.data.borrow_mut();
        Position::init_bytes(position_data.as_mut(), (params, *owner.key))?;

        Ok(())
    }

    #[inline(never)]
    pub fn deposit(
        &self,
        amount: u64,
        reserves_count: u8,
        reserve_liquidity_supplies_count: u8,
        lp_token_mints_count: u8,
        lp_token_storages_count: u8,
    ) -> VaultResult<()> {
        msg!("deposit ix: {}", amount);

        let DepositAccounts {
            vault,
            position,
            owner,
            liquidity_mint,
            liquidity_source,
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
        } = DepositAccounts::from_iter(
            &mut self.accounts.iter(),
            reserves_count as usize,
            reserve_liquidity_supplies_count as usize,
            lp_token_mints_count as usize,
            lp_token_storages_count as usize,
            self.program_id,
        )?;

        verify_token_program(liquidity_token_program)?;

        let mut position_data = position.data.borrow_mut();
        let unpacked_position = Position::try_from_bytes_mut(position_data.as_mut())?;

        let mut vault_data = vault.data.borrow_mut();
        let unpacked_vault = Vault::try_from_bytes_mut(vault_data.as_mut())?;
        let distribution_count = unpacked_vault.distribution.count() as u8;
        if distribution_count != reserves_count {
            return Err(VaultError::InvalidAccountsCount);
        }

        // Owner provided must be the owner of the position. Only the owner can deposit/withdraw.
        verify_key(owner.key, &unpacked_position.owner, "position owner")?;

        // Position must belong to the Vault provided
        verify_key(
            vault.key,
            &unpacked_position.vault,
            "vault vs. position.vault",
        )?;

        verify_account_count(
            reserves_count,
            reserve_liquidity_supplies_count,
            lp_token_mints_count,
            lp_token_storages_count,
        )?;
        verify_program_authority(program_authority.key)?;

        let liquidity_decimals = {
            let reserve_acc = reserves.first().unwrap();
            let reserve_data = reserve_acc.data.borrow();
            Reserve::try_from_bytes(&reserve_data)?
                .liquidity
                .mint_decimals
        };

        for (distribution_item, reserve_acc) in unpacked_vault.distribution.0.iter().zip(&reserves)
        {
            if reserve_acc.key != &distribution_item.reserve {
                msg!("reserves are expected to be in order of vault distribution items. Reserve {} distribution_item.reserve {}",
                    reserve_acc.key, distribution_item.reserve);
                return Err(VaultError::InvalidAccountsOrder);
            }
        }

        let amount_dec = Decimal::from_i128_with_scale(amount as i128, liquidity_decimals as u32)?;

        let current_vlp_supply = unpacked_vault.vlp_supply()?;

        let lp_exchange_rates = lp_exchange_rates(&reserves)?;
        let lp_amounts = lp_amounts(&lp_token_storages, liquidity_decimals)?;

        let vlp_price = vlp_price(&lp_exchange_rates, &lp_amounts, current_vlp_supply)?;

        let vlp_amount_to_mint = amount_dec.checked_div(vlp_price)?;

        let spl_token = SplToken::new(liquidity_token_program);
        spl_token
            .transfer(
                liquidity_source,
                Some(liquidity_mint),
                liquidity_buffer,
                owner,
                amount,
                Some(liquidity_decimals),
            )?
            .call()?;

        drop(vault_data);

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
            Delta::Increase(amount),
        )?;

        let mut vault_data = vault.data.borrow_mut();
        let unpacked_vault = Vault::try_from_bytes_mut(vault_data.as_mut())?;
        unpacked_vault.deposit(vlp_amount_to_mint)?;

        let clock = Clock::get().expect("no clock");

        let touched_rewards_records = unpacked_position.rewards.accrue_rewards(
            unpacked_position.issued_vlp_amount()?,
            &unpacked_vault.reward_rules,
            clock.slot,
        )?;
        unpacked_position
            .rewards
            .set_accrued_slot(clock.slot, touched_rewards_records);

        unpacked_position.deposit(vlp_amount_to_mint, vlp_price)?;

        Ok(())
    }

    #[inline(never)]
    pub fn withdraw(
        &self,
        vlp_amount: u64,
        reserves_count: u8,
        reserve_liquidity_supplies_count: u8,
        lp_token_mints_count: u8,
        lp_token_storages_count: u8,
    ) -> VaultResult<()> {
        msg!("withdraw ix");

        let WithdrawAccounts {
            vault,
            owner,
            liquidity_mint,
            liquidity_destination,
            liquidity_buffer,
            program_authority,
            lp_token_program,
            liquidity_token_program,
            reserves,
            reserve_liquidity_supplies,
            lp_token_mints,
            lp_token_storages,
            superlendy_program_authority,
            superlendy_program,
            position,
        } = WithdrawAccounts::from_iter(
            &mut self.accounts.iter(),
            reserves_count as usize,
            reserve_liquidity_supplies_count as usize,
            lp_token_mints_count as usize,
            lp_token_storages_count as usize,
            self.program_id,
        )?;

        verify_token_program(liquidity_token_program)?;

        let mut position_data = position.data.borrow_mut();
        let unpacked_position = Position::try_from_bytes_mut(position_data.as_mut())?;

        // Owner provided must be the owner of the position. Only the owner can deposit/withdraw.
        verify_key(owner.key, &unpacked_position.owner, "position owner")?;

        // Position must belong to the Vault provided
        verify_key(
            vault.key,
            &unpacked_position.vault,
            "vault vs. position.vault",
        )?;

        verify_account_count(
            reserves_count,
            reserve_liquidity_supplies_count,
            lp_token_mints_count,
            lp_token_storages_count,
        )?;

        let mut vault_data = vault.data.borrow_mut();
        let unpacked_vault = Vault::try_from_bytes_mut(vault_data.as_mut())?;
        let distribution_count = unpacked_vault.distribution.count() as u8;
        if distribution_count != reserves_count {
            return Err(VaultError::InvalidAccountsCount);
        }

        let authority_bump = verify_program_authority(program_authority.key)?;

        let liquidity_decimals = mint_decimals(liquidity_mint)?;

        for (distribution_item, reserve_acc) in unpacked_vault.distribution.0.iter().zip(&reserves)
        {
            if reserve_acc.key != &distribution_item.reserve {
                msg!("reserves are expected to be in order of vault distribution items. Reserve {} distribution_item.reserve {}",
                    reserve_acc.key, distribution_item.reserve);
                return Err(VaultError::InvalidAccountsOrder);
            }
        }

        let lp_exchange_rates = lp_exchange_rates(&reserves)?;
        let lp_amounts = lp_amounts(&lp_token_storages, liquidity_decimals)?;
        let vlp_price = vlp_price(
            &lp_exchange_rates,
            &lp_amounts,
            unpacked_vault.vlp_supply()?,
        )?;

        let vlp_amount_dec = if vlp_amount == u64::MAX {
            let mut max_liquidity_withdraw_amount = Decimal::ZERO;
            for reserve in &reserves {
                let reserve_data = reserve.data.borrow();
                let unpacked_reserve = Reserve::try_from_bytes(&reserve_data)?;
                max_liquidity_withdraw_amount = max_liquidity_withdraw_amount
                    .checked_add(unpacked_reserve.max_withdraw_liquidity_amount()?)?;
            }
            let max_vlp_withdraw_amount = max_liquidity_withdraw_amount.checked_div(vlp_price)?;
            unpacked_position
                .issued_vlp_amount()?
                .min(max_vlp_withdraw_amount)
        } else {
            Decimal::from_i128_with_scale(vlp_amount as i128, liquidity_decimals as u32)?
        };

        let desired_liquidity_amount = vlp_amount_dec
            .checked_mul(vlp_price)?
            .to_lamports_floor(liquidity_decimals)?;

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

        let mut vault_data = vault.data.borrow_mut();
        let unpacked_vault = Vault::try_from_bytes_mut(vault_data.as_mut())?;

        let spl_token = SplToken::new(liquidity_token_program);
        spl_token
            .transfer(
                liquidity_buffer,
                Some(liquidity_mint),
                liquidity_destination,
                program_authority,
                rebalance_outcome.expected_buffer_change as u64,
                Some(liquidity_decimals),
            )?
            .signed(&[&[pda::AUTHORITY_SEED, &[authority_bump]]])?;

        unpacked_vault.withdraw(vlp_amount_dec)?;

        let clock = Clock::get().expect("no clock");

        let touched_rewards_records = unpacked_position.rewards.accrue_rewards(
            unpacked_position.issued_vlp_amount()?,
            &unpacked_vault.reward_rules,
            clock.slot,
        )?;
        unpacked_position
            .rewards
            .set_accrued_slot(clock.slot, touched_rewards_records);

        unpacked_position.withdraw(vlp_amount_dec, vlp_price)?;

        Ok(())
    }

    #[inline(never)]
    pub fn close_position(&self) -> VaultResult<()> {
        msg!("close_position ix");

        let ClosePositionAccounts {
            position,
            vault,
            owner,
        } = ClosePositionAccounts::from_iter(&mut self.accounts.iter(), self.program_id)?;

        let mut position_data = position.data.borrow_mut();
        let unpacked_position = Position::try_from_bytes_mut(position_data.as_mut())?;

        let vault_data = vault.data.borrow();
        let unpacked_vault = Vault::try_from_bytes(vault_data.as_ref())?;

        verify_key(owner.key, &unpacked_position.owner, "position owner")?;
        verify_key(vault.key, &unpacked_position.vault, "position vault")?;

        let clock = Clock::get().expect("no clock");

        let touched_rewards_records = unpacked_position.rewards.accrue_rewards(
            unpacked_position.issued_vlp_amount()?,
            &unpacked_vault.reward_rules,
            clock.slot,
        )?;
        unpacked_position
            .rewards
            .set_accrued_slot(clock.slot, touched_rewards_records);

        unpacked_position.closable()?;

        let balance = {
            let lamports_data = position.lamports.borrow();
            **lamports_data
        };

        crate::processor::transfer_lamports(position, owner, balance)?;

        Ok(())
    }
}

pub(crate) fn lp_exchange_rates(
    reserves: &Vec<&AccountInfo<'_>>,
) -> Result<Vec<LpExchangeRate>, VaultError> {
    let mut lp_exchange_rates = vec![];
    for reserve_acc in reserves {
        let reserve_data = reserve_acc.data.borrow();
        let unpacked_reserve = Reserve::try_from_bytes(&reserve_data)?;
        let lp_exchange_rate = unpacked_reserve.lp_exchange_rate()?;
        lp_exchange_rates.push(lp_exchange_rate);
    }

    Ok(lp_exchange_rates)
}

pub(crate) fn lp_amounts(
    lp_token_storages: &[&AccountInfo<'_>],
    liquidity_decimals: u8,
) -> Result<Vec<Decimal>, VaultError> {
    let mut lp_amounts = vec![];
    for lp_token_storage in lp_token_storages {
        let token_account: SplTokenAccount =
            SplTokenAccount::unpack(&lp_token_storage.data.borrow())
                .map_err(VaultError::SplTokenUnpack)?;
        let lp_amount =
            Decimal::from_i128_with_scale(token_account.amount as i128, liquidity_decimals as u32)?;
        lp_amounts.push(lp_amount);
    }

    Ok(lp_amounts)
}
