use std::{cmp::min, collections::HashSet};

use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use solana_program::{clock::Slot, msg, pubkey::Pubkey};
use texture_common::math::{CheckedAdd, CheckedMul, Decimal};

use crate::{error::VaultError, VaultResult};

pub const MAX_REWARD_RULES: usize = 8;
pub const REWARD_RULE_NAME_MAX_LEN: usize = 8;

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, Eq, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct RewardRule {
    /// Reward token mint. This is the key to match Rules and actual Rewards.
    pub reward_mint: Pubkey,
    /// Human-readable rule name. It's here because anyway need to align this struct on 16 :-)
    pub name: [u8; REWARD_RULE_NAME_MAX_LEN],
    /// Solana slot from which this rule takes effect.
    pub start_slot: Slot,
    /// Amount of reward tokens to accrue per slot for deposited liquidity
    pub rate: i128,
}

impl RewardRule {
    /// Functions to read and set Decimal amounts which are i128 inside
    pub fn rate(&self) -> VaultResult<Decimal> {
        Decimal::from_bits(self.rate).map_err(From::from)
    }

    pub fn set_rate(&mut self, value: Decimal) -> VaultResult<()> {
        self.rate = value.into_bits()?;
        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct RewardRules {
    pub rules: [RewardRule; MAX_REWARD_RULES],
}

/// Reward record tracks interaction of user's position with ALL Reward Rules with given
/// reward_mint. Rewards from all such rules accumulated on one Rewards record.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Reward {
    /// Reward token counted in this record
    /// Default value marks record as unused
    pub reward_mint: Pubkey,

    /// Slot when `accrued_amount` was calculated
    pub accrued_slot: Slot,
    _padding: u64,

    /// This is total amount or Rewards token accrued at the time of `accrued_slot`
    /// Let it be Decimal with WAD representation. This will allow to accrue very small (e.g. 10^-9 of
    /// lamport) amounts.
    pub accrued_amount: i128,
}

impl Reward {
    pub fn accrued_amount(&self) -> VaultResult<Decimal> {
        Decimal::from_bits(self.accrued_amount).map_err(From::from)
    }

    pub fn set_accrued_amount(&mut self, value: Decimal) -> VaultResult<()> {
        self.accrued_amount = value.into_bits()?;
        Ok(())
    }

    /// Calculates increase of reward amount.
    /// This function does NOT update accrued_slot leaving open possibility to accrue more Rules to
    /// the same reward record. After all rules are processed reward record should be updated with
    /// proper accrued_slot.
    ///
    /// Calculated amount is written right in to self but also this time accrued amount is returned.
    ///
    /// `rule` - particular reward rule to apply
    /// `current_slot` - current Solana slot
    /// `reward_base` - this is either amount deposited to a particular Reserve or amount borrowed
    pub fn accrue(
        &mut self,
        rule: &RewardRule,
        current_slot: Slot,
        reward_base: Decimal,
    ) -> VaultResult<Decimal> {
        if rule.reward_mint != self.reward_mint {
            msg!(
                "rule.reward_mint {}   self.reward_mint {}",
                rule.reward_mint,
                self.reward_mint
            );
            return Err(VaultError::OperationCanNotBePerformed);
        }

        if current_slot < self.accrued_slot {
            msg!(
                "can't accrue rewards as current_slot {} less then accrued_slot {} ",
                current_slot,
                self.accrued_slot
            );
            return Err(VaultError::OperationCanNotBePerformed);
        }

        let slots_since_last_accrual = current_slot - self.accrued_slot;

        if rule.start_slot >= current_slot {
            msg!(
                "rule {} not applied as its start_slot {} while Solana slot is {}",
                String::from_utf8_lossy(&rule.name),
                rule.start_slot,
                current_slot
            );
            return Ok(Decimal::ZERO);
        }

        let rule_age = current_slot - rule.start_slot;

        let slots_to_accrue = min(slots_since_last_accrual, rule_age);

        let rewards_increase = reward_base
            .checked_mul(rule.rate()?)?
            .checked_mul(Decimal::from_i128_with_scale(slots_to_accrue as i128, 0)?)?;

        if rewards_increase != Decimal::ZERO {
            let new_amount = self.accrued_amount()?.checked_add(rewards_increase)?;
            self.set_accrued_amount(new_amount)?;

            msg!(
                "rewards for rule {}: delta {} for slots {}. balance {}",
                String::from_utf8_lossy(&rule.name),
                rewards_increase,
                slots_to_accrue,
                new_amount
            );
        }

        Ok(rewards_increase)
    }

    pub fn claim(&mut self, decimals: u8) -> VaultResult<u64> {
        let decimal_amount_to_claim = self.accrued_amount()?;

        self.set_accrued_amount(Decimal::ZERO)?;
        self.reward_mint = Pubkey::default(); // Mark record as unused
        self.accrued_slot = 0;

        Ok(decimal_amount_to_claim.to_lamports_floor(decimals)?)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct Rewards {
    pub rewards: [Reward; MAX_REWARD_RULES],
}

impl Rewards {
    /// Accrues rewards for holding VLP at Position.
    /// `amount` - VLP amount to use as base for rewards accrual
    /// `reward_rules` - rules from deserialized Vault account deposit made to
    /// `current_slot` - current Solana slot
    ///
    /// Returns set of reward record indexes, touched by this call
    ///
    /// This function:
    /// 1. Search for new rules which can be applied to the Position and init new `reward` records
    ///    for them
    /// 2. Accrues reward tokens for existing `reward` records
    pub fn accrue_rewards(
        &mut self,
        amount: Decimal,
        reward_rules: &RewardRules,
        current_slot: Slot,
    ) -> VaultResult<HashSet<usize>> {
        let mut touched_reward_records = HashSet::new();

        for rule in reward_rules.rules {
            match self.find_reward(&rule.reward_mint) {
                None => {
                    // Try to allocate new `rewards` record. Because we just found Reward Rule and
                    // have no previous accrual with timestamp - we can't calculate reward this time.
                    match self.find_unused_reward() {
                        None => {
                            msg!(
                                "warning: no free space for rewards for rule {} reward_mint {}",
                                String::from_utf8_lossy(&rule.name),
                                rule.reward_mint
                            );
                        }
                        Some(new_record) => {
                            new_record.reward_mint = rule.reward_mint;
                            new_record.accrued_slot = current_slot;
                            new_record.set_accrued_amount(Decimal::ZERO)?;
                        }
                    }
                }
                Some((index, reward_record)) => {
                    reward_record.accrue(&rule, current_slot, amount)?;
                    touched_reward_records.insert(index);
                }
            }
        }

        Ok(touched_reward_records)
    }

    pub fn find_reward(&mut self, reward_mint: &Pubkey) -> Option<(usize, &mut Reward)> {
        self.rewards
            .iter_mut()
            .enumerate()
            .find(|(_index, reward)| &reward.reward_mint == reward_mint)
    }

    // Unused record is either:
    // 1. Default initialized with default (zero) pubkey.
    pub fn find_unused_reward(&mut self) -> Option<&mut Reward> {
        self.rewards
            .iter_mut()
            .find(|reward| reward.reward_mint == Pubkey::default())
    }

    pub fn set_accrued_slot(&mut self, current_slot: Slot, indexes: HashSet<usize>) {
        for idx in indexes {
            self.rewards[idx].accrued_slot = current_slot;
        }
    }
}
