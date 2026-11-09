use std::{fmt::Display, num::ParseFloatError, str::FromStr};

use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use solana_program::clock::{UnixTimestamp, SECONDS_PER_DAY};
use solana_program::msg;
use solana_program::pubkey::ParsePubkeyError;
use solana_program::pubkey::Pubkey;
use texture_common::math::CheckedAdd;
use texture_common::math::CheckedSub;
use texture_common::{
    account::{PodAccount, PodAccountError},
    math::{CheckedMul, Decimal, MathError},
};
use thiserror::Error;

use crate::error::VaultError;
use crate::VaultResult;

use super::reward::RewardRules;
use super::VAULT_DISCRIMINATOR;

pub const VAULT_NAME_MAX_LEN: usize = 64;

pub const VAULT_DESCRIPTION_MAX_LEN: usize = 256;

/// Max distribution items in Vault account
pub const VAULT_DISTRIBUTION_MAX_ITEMS: usize = 16;

const MAX_FEE_RATE_BPS: u16 = 3000;

#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, Pod, Zeroable, PartialEq)]
#[repr(C)]
pub struct Distribution(pub [DistributionItem; VAULT_DISTRIBUTION_MAX_ITEMS]);

impl Default for Distribution {
    fn default() -> Self {
        Distribution([DistributionItem::default(); VAULT_DISTRIBUTION_MAX_ITEMS])
    }
}

impl Distribution {
    /// Returns number of "used" distribution items
    pub fn count(&self) -> usize {
        self.0.iter().filter(|item| !item.is_vacant()).count()
    }

    pub fn find_vacant(&mut self) -> Option<&mut DistributionItem> {
        self.0.iter_mut().find(|item| item.is_vacant())
    }

    /// Finds distribution item based on Reserve's key
    pub fn find(&mut self, reserve: Pubkey) -> Option<&mut DistributionItem> {
        self.0.iter_mut().find(|item| item.reserve == reserve)
    }

    pub fn effective_items(&self) -> impl Iterator<Item = &DistributionItem> {
        self.0.iter().filter(|item| !item.is_vacant())
    }

    pub fn from_vec(items: Vec<DistributionItem>) -> Self {
        let mut distribution = Distribution::default();
        for (idx, item) in items.into_iter().enumerate() {
            distribution.0[idx] = item;
        }
        distribution
    }

    pub fn to_bps_array(&self) -> [u16; VAULT_DISTRIBUTION_MAX_ITEMS] {
        let mut distribution_bps = [0; VAULT_DISTRIBUTION_MAX_ITEMS];
        for (idx, item) in self.0.iter().enumerate() {
            distribution_bps[idx] = item.share_bps();
        }
        distribution_bps
    }
}

impl Display for Distribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in self.0 {
            let deci = item
                .share_decimal()
                .expect("Decimal from distribution item share bits");
            writeln!(
                f,
                "Reserve {} - {}%",
                item.reserve,
                deci.checked_mul(Decimal::from_i128_with_scale(100, 0).unwrap())
                    .unwrap()
            )?;
        }
        Ok(())
    }
}

#[derive(
    BorshSerialize, BorshDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Pod, Zeroable,
)]
#[repr(C)]
pub struct DistributionItem {
    pub reserve: Pubkey,
    pub share_bps: u16,
    pub _padding: [u8; 6],
}

#[derive(Debug, Error)]
pub enum DistributionParseError {
    #[error("Format error")]
    Format,
    #[error(transparent)]
    Pubkey(#[from] ParsePubkeyError),
    #[error(transparent)]
    Share(#[from] ParseFloatError),
    #[error(transparent)]
    Math(#[from] MathError),
}

impl FromStr for DistributionItem {
    type Err = DistributionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((pubkey, share)) = s.split_once(':') else {
            return Err(DistributionParseError::Format);
        };
        let pubkey = Pubkey::from_str(pubkey)?;
        let share = share.parse::<f32>()?;

        Ok(DistributionItem {
            reserve: pubkey,
            share_bps: (share * 100.0).floor() as u16,
            _padding: Zeroable::zeroed(),
        })
    }
}

impl DistributionItem {
    pub fn share_decimal(&self) -> Result<Decimal, MathError> {
        Decimal::from_basis_points(self.share_bps as u32)
    }

    pub fn share_bps(&self) -> u16 {
        self.share_bps
    }

    pub fn set_share_bps(&mut self, val_bps: u16) {
        self.share_bps = val_bps;
    }

    pub fn is_vacant(&self) -> bool {
        self.reserve == Pubkey::default()
    }

    pub fn mark_vacant(&mut self) {
        self.reserve = Pubkey::default();
    }
}

static_assertions::const_assert_eq!(0, std::mem::size_of::<Vault>() % 16);
static_assertions::const_assert_eq!(0, std::mem::size_of::<Distribution>() % 16);
static_assertions::const_assert_eq!(0, std::mem::size_of::<RewardRules>() % 16);
// must be combined in to arrays with total len dividable by 16 e.g. 16 elements
static_assertions::const_assert_eq!(0, std::mem::size_of::<DistributionItem>() % 8);
// must be combined in to arrays with total len dividable by 16 e.g. 4 elements
static_assertions::const_assert_eq!(0, std::mem::size_of::<DistributionProposal>() % 8);

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Vault {
    pub discriminator: [u8; 8],
    pub version: u8,
    pub _padding: [u8; 7],

    /// Curator account which effectively (through curator.vaults_authority) owns this Vault.
    pub curator: Pubkey,

    /// Authority who can change percentage in distribution (usually bot).
    pub distribution_manager: Pubkey,

    pub name: [u8; VAULT_NAME_MAX_LEN],
    pub description: [u8; VAULT_DESCRIPTION_MAX_LEN],

    /// Input token which should be distributed across pools
    pub mint: Pubkey,
    pub distribution: Distribution,
    pub vlp_supply: i128,

    /// Address of lookup table which can be used to call IXes with lists of accounts related to Reserves.
    /// Table contains reserves, lp_mints, reserve_liquidity_supply and lp_storages.
    pub lut: Pubkey,

    /// Reward rules
    pub reward_rules: RewardRules,

    /// Proposed (pending) new distribution items
    pub distribution_proposals: DistributionProposals,

    pub last_vlp_price: i128,
    pub periodic_fee_claimed_at: u64,

    /// Periodic (weekly) fees rate for this Vault
    pub periodic_fee_rate_bps: u16,

    pub _trailing_padding: [u8; 262],
}

impl Vault {
    pub fn vlp_supply(&self) -> VaultResult<Decimal> {
        Decimal::from_bits(self.vlp_supply).map_err(From::from)
    }

    pub fn set_vlp_supply(&mut self, value: Decimal) -> VaultResult<()> {
        self.vlp_supply = value.into_bits()?;
        Ok(())
    }

    pub fn last_vlp_price(&self) -> VaultResult<Decimal> {
        Decimal::from_bits(self.last_vlp_price).map_err(From::from)
    }

    pub fn set_last_vlp_price(&mut self, value: Decimal) -> VaultResult<()> {
        self.last_vlp_price = value.into_bits()?;
        Ok(())
    }

    pub fn deposit(&mut self, issued_vlp_amount: Decimal) -> Result<(), VaultError> {
        self.set_vlp_supply(self.vlp_supply()?.checked_add(issued_vlp_amount)?)?;

        Ok(())
    }

    pub fn withdraw(&mut self, burned_vlp_amount: Decimal) -> Result<(), VaultError> {
        self.set_vlp_supply(self.vlp_supply()?.checked_sub(burned_vlp_amount)?)?;

        Ok(())
    }

    pub fn periodic_fee_rate_bps(&self) -> u16 {
        self.periodic_fee_rate_bps
    }

    pub fn decimal_periodic_fee_rate_bps(&self) -> VaultResult<Decimal> {
        Decimal::from_basis_points(self.periodic_fee_rate_bps as u32).map_err(From::from)
    }

    pub fn set_periodic_fee_rate_bps(&mut self, value: u16) -> VaultResult<()> {
        if value > MAX_FEE_RATE_BPS {
            msg!("fee rate can't exceed {}%", MAX_FEE_RATE_BPS / 100);
            return Err(VaultError::InvalidConfig);
        }
        self.periodic_fee_rate_bps = value;
        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct VaultParams {
    pub name: [u8; VAULT_NAME_MAX_LEN],
    pub description: [u8; VAULT_DESCRIPTION_MAX_LEN],
    pub lut: Pubkey,
    /// Authority who can change percentage in distribution (usually bot).
    pub distribution_manager: Pubkey,

    /// These are distribution percents (as basis points) in the same order as Reserves in the
    /// CreateVault reserves array.
    pub distribution_bps: [u16; VAULT_DISTRIBUTION_MAX_ITEMS],

    /// Periodic fee rate for this Vault
    pub periodic_fee_rate_bps: u16,
}

impl VaultParams {
    pub fn validate(&self) -> VaultResult<()> {
        // Sum of shares must be 100%
        let sum: u64 = self
            .distribution_bps
            .iter()
            .fold(0, |acc, item| acc + *item as u64);

        if sum != 10000 {
            msg!("shares (BPS) sum {} but must be 10000", sum);
            return Err(VaultError::InvalidConfig);
        }

        if self.periodic_fee_rate_bps > 3000 {
            msg!("periodic_fee_rate must be in range [0% - 30%]");
            return Err(VaultError::InvalidConfig);
        }

        Ok(())
    }
}

impl PodAccount for Vault {
    const DISCRIMINATOR: &'static [u8] = VAULT_DISCRIMINATOR;

    type Version = u8;

    const VERSION: Self::Version = 1;

    type InitParams = (
        /*params:*/ VaultParams,
        /*liquidity mint:*/ Pubkey,
        /*owner:*/ Pubkey,
        /*reserves list*/ Vec<Pubkey>,
    );

    type InitError = VaultError;

    fn discriminator(&self) -> &[u8] {
        &self.discriminator
    }

    fn version(&self) -> Self::Version {
        self.version
    }

    fn init_unckecked(
        &mut self,
        (params, liquidity_mint, curator_key, reserves): Self::InitParams,
    ) -> Result<(), Self::InitError> {
        let Self {
            discriminator,
            version,
            curator,
            distribution_manager,
            name,
            description,
            _padding,
            mint,
            distribution,
            vlp_supply,
            lut,
            reward_rules,
            distribution_proposals,
            periodic_fee_rate_bps: _,
            last_vlp_price,
            periodic_fee_claimed_at,
            _trailing_padding,
        } = self;

        *discriminator = *VAULT_DISCRIMINATOR;
        *version = Self::VERSION;
        *curator = curator_key;
        *distribution_manager = params.distribution_manager;
        *name = params.name;
        *mint = liquidity_mint;
        *description = params.description;
        *lut = params.lut;
        *distribution = Distribution::default();
        *distribution_proposals = Zeroable::zeroed();
        *vlp_supply = Decimal::ZERO
            .into_bits()
            .map_err(|_| PodAccountError::Invalid)?;
        *reward_rules = Zeroable::zeroed();
        *last_vlp_price = Decimal::ONE
            .into_bits()
            .map_err(|_| PodAccountError::Invalid)?;
        *periodic_fee_claimed_at = 0;
        *_trailing_padding = Zeroable::zeroed();
        *_padding = Zeroable::zeroed();

        for (idx, (reserve, share_bps)) in reserves
            .into_iter()
            .zip(params.distribution_bps)
            .enumerate()
        {
            if idx >= VAULT_DISTRIBUTION_MAX_ITEMS {
                msg!(
                    "Number of reserves passed {} is greater than Vault capacity {}",
                    idx + 1,
                    VAULT_DISTRIBUTION_MAX_ITEMS
                );
                return Err(VaultError::InvalidConfig);
            }

            distribution.0[idx] = DistributionItem {
                reserve,
                share_bps,
                _padding: Zeroable::zeroed(),
            };
        }

        self.set_periodic_fee_rate_bps(params.periodic_fee_rate_bps)?;

        Ok(())
    }
}

pub const MAX_DISTRIBUTION_PROPOSALS: usize = 4;
pub const SECONDS_IN_THREE_DAYS: UnixTimestamp = SECONDS_PER_DAY as UnixTimestamp * 3;

/// Timelocks for Vaults include just one action - addition of new distribution item in to
/// existing vault. Adding new Reserve (which potentially results in different risk for Investors)
/// is proposed (by Curator) first and then after 3 days it can be applied (by Curator) to the Vault.
/// All proposals are stored in Vault state in a form of unordered `DistributionProposal` items.
/// Previously made proposal can be deactivated (zeroed) by Curator at any time.
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct DistributionProposals(pub [DistributionProposal; MAX_DISTRIBUTION_PROPOSALS]);

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, Pod, Zeroable, PartialEq)]
#[repr(C)]
pub struct DistributionProposal {
    /// Solana time this proposed change can be applied. 0 indicates unused entry.
    pub can_be_applied_at: UnixTimestamp,
    /// Reserve to be added in to distribution (once proposal will be applied)
    pub reserve: Pubkey,
}

impl Default for DistributionProposal {
    fn default() -> Self {
        Self::zeroed()
    }
}
