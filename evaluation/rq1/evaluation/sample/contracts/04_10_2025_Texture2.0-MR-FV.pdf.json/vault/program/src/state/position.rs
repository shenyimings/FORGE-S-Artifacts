use bytemuck::{Pod, Zeroable};
use solana_program::msg;
use solana_program::pubkey::Pubkey;
use texture_common::math::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use texture_common::{
    account::{PodAccount, PodAccountError},
    math::Decimal,
};

use crate::error::VaultError;
use crate::VaultResult;

use super::{reward::Rewards, POSITION_DISCRIMINATOR};

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Position {
    pub discriminator: [u8; 8],
    pub version: u8,
    pub _padding: [u8; 7],

    pub owner: Pubkey,
    pub vault: Pubkey,

    pub issued_vlp_amount: i128,
    pub weighted_average_vlp_price: i128,
    pub realized_profit: i128,

    /// Tracks rewards for the position
    pub rewards: Rewards,

    pub created_at: i64,
    pub _padding_2: [u8; 128 + 8],
}

impl Position {
    pub fn issued_vlp_amount(&self) -> VaultResult<Decimal> {
        Decimal::from_bits(self.issued_vlp_amount).map_err(From::from)
    }

    pub fn set_issued_vlp_amount(&mut self, value: Decimal) -> VaultResult<()> {
        self.issued_vlp_amount = value.into_bits()?;
        Ok(())
    }

    pub fn weighted_average_vlp_price(&self) -> VaultResult<Decimal> {
        Decimal::from_bits(self.weighted_average_vlp_price).map_err(From::from)
    }

    pub fn set_weighted_average_vlp_price(&mut self, value: Decimal) -> VaultResult<()> {
        self.weighted_average_vlp_price = value.into_bits()?;
        Ok(())
    }

    pub fn realized_profit(&self) -> VaultResult<Decimal> {
        Decimal::from_bits(self.realized_profit).map_err(From::from)
    }

    fn set_realized_profit(&mut self, value: Decimal) -> VaultResult<()> {
        self.realized_profit = value.into_bits()?;
        Ok(())
    }

    /// Checks is position can be safely closed.
    /// Returns Ok(()) - when position can be safely closed.
    /// Returns Err(reason why it can NOT be closed) - when  position can NOT be safely closed.
    pub fn closable(&self) -> VaultResult<()> {
        let issued_vlp_amount = self.issued_vlp_amount()?;

        if issued_vlp_amount.gt(&Decimal::ZERO) {
            return Err(VaultError::NonEmptyPosition(issued_vlp_amount));
        }

        for (idx, reward) in self.rewards.rewards.iter().enumerate() {
            let accrued_amount = reward.accrued_amount()?;

            if accrued_amount > Decimal::ZERO {
                return Err(VaultError::UnclaimedRewards {
                    idx,
                    accrued_amount,
                });
            }
        }

        Ok(())
    }

    // Estimated profit for remaining amount of VLP
    pub fn estimated_profit(&self, vlp_price: Decimal) -> VaultResult<Decimal> {
        let weighted_average_vlp_price = self.weighted_average_vlp_price()?;
        let issued_vlp_amount = self.issued_vlp_amount()?;

        estimated_profit(weighted_average_vlp_price, issued_vlp_amount, vlp_price)
    }

    // Total estimated profit (already realized + estimated profit for remaining VLP)
    pub fn total_estimated_profit(&self, vlp_price: Decimal) -> VaultResult<Decimal> {
        let realized_profit = self.realized_profit()?;
        let added_profit = self.estimated_profit(vlp_price)?;

        Ok(realized_profit.checked_add(added_profit)?)
    }

    pub fn deposit(&mut self, vlp_amount: Decimal, vlp_price: Decimal) -> VaultResult<()> {
        let issued_vlp_amount = self.issued_vlp_amount()?;
        let weighted_average_vlp_price = self.weighted_average_vlp_price()?;

        let issued_vlp_value = issued_vlp_amount.checked_mul(weighted_average_vlp_price)?;
        let added_vlp_value = vlp_amount.checked_mul(vlp_price)?;
        let total_vlp_value = issued_vlp_value.checked_add(added_vlp_value)?;

        let total_vlp_amount = issued_vlp_amount.checked_add(vlp_amount)?;
        let new_weighted_average_vlp_price = total_vlp_value.checked_div(total_vlp_amount)?;

        self.set_issued_vlp_amount(total_vlp_amount)?;
        self.set_weighted_average_vlp_price(new_weighted_average_vlp_price)?;

        Ok(())
    }

    pub fn withdraw(&mut self, vlp_amount: Decimal, vlp_price: Decimal) -> VaultResult<()> {
        let issued_vlp_amount = self.issued_vlp_amount()?;
        if vlp_amount > issued_vlp_amount {
            msg!(
                "trying to withdraw more VLP than available: {} > {}",
                vlp_amount,
                issued_vlp_amount
            );
            return Err(VaultError::InvalidAmount);
        }

        let average_value = vlp_amount.checked_mul(self.weighted_average_vlp_price()?)?;
        let realization_value = vlp_amount.checked_mul(vlp_price)?;
        let added_profit = realization_value.checked_sub(average_value)?;

        self.set_realized_profit(self.realized_profit()?.checked_add(added_profit)?)?;
        self.set_issued_vlp_amount(issued_vlp_amount.checked_sub(vlp_amount)?)?;

        Ok(())
    }
}

pub fn estimated_profit(
    weighted_average_vlp_price: Decimal,
    issued_vlp_amount: Decimal,
    vlp_price: Decimal,
) -> VaultResult<Decimal> {
    let average_value = issued_vlp_amount.checked_mul(weighted_average_vlp_price)?;
    let realization_value = issued_vlp_amount.checked_mul(vlp_price)?;

    let added_profit = realization_value.checked_sub(average_value)?;

    Ok(added_profit)
}

#[derive(Debug, Clone)]
pub struct PositionParams {
    pub vault: Pubkey,
    pub created_at: i64,
}

impl PodAccount for Position {
    const DISCRIMINATOR: &'static [u8] = POSITION_DISCRIMINATOR;

    type Version = u8;

    const VERSION: Self::Version = 1;

    type InitParams = (/*params:*/ PositionParams, /*owner:*/ Pubkey);

    type InitError = PodAccountError;

    fn discriminator(&self) -> &[u8] {
        &self.discriminator
    }

    fn version(&self) -> Self::Version {
        self.version
    }

    fn init_unckecked(
        &mut self,
        (params, owner_key): Self::InitParams,
    ) -> Result<(), Self::InitError> {
        let Self {
            discriminator,
            version,
            _padding,
            owner,
            vault,
            created_at,
            issued_vlp_amount,
            weighted_average_vlp_price,
            realized_profit,
            _padding_2,
            rewards,
        } = self;

        *discriminator = *POSITION_DISCRIMINATOR;
        *version = Self::VERSION;

        *owner = owner_key;
        *vault = params.vault;

        *created_at = params.created_at;

        *issued_vlp_amount = Decimal::ZERO.into_bits().unwrap();
        *weighted_average_vlp_price = Decimal::ZERO.into_bits().unwrap();
        *realized_profit = Decimal::ZERO.into_bits().unwrap();

        *rewards = Zeroable::zeroed();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use texture_common::dec;

    use super::*;

    #[test]
    fn test_deposit_withdraw() -> VaultResult<()> {
        let mut position = Position::zeroed();

        position
            .deposit(
                Decimal::from_i128_with_scale(1000, 0)?,
                Decimal::from_i128_with_scale(1, 0)?,
            )
            .unwrap();
        assert_eq!(
            position.issued_vlp_amount().unwrap(),
            Decimal::from_i128_with_scale(1000, 0)?
        );
        assert_eq!(
            position.weighted_average_vlp_price().unwrap(),
            Decimal::from_i128_with_scale(1, 0)?
        );

        position
            .deposit(
                Decimal::from_i128_with_scale(1000, 0)?,
                Decimal::from_i128_with_scale(3, 0)?,
            )
            .unwrap();
        assert_eq!(
            position.issued_vlp_amount().unwrap(),
            Decimal::from_i128_with_scale(2000, 0)?
        );
        assert_eq!(
            position.weighted_average_vlp_price().unwrap(),
            Decimal::from_i128_with_scale(2, 0)?
        );

        // Deposit huge amount. Now weighted_average_vlp_price must be very close to 4.
        position
            .deposit(
                Decimal::from_i128_with_scale(1000000, 0)?,
                Decimal::from_i128_with_scale(4, 0)?,
            )
            .unwrap();
        assert_eq!(
            position.issued_vlp_amount().unwrap(),
            Decimal::from_i128_with_scale(1000000 + 2000, 0)?
        );
        assert_eq!(
            position.weighted_average_vlp_price().unwrap(),
            Decimal::from_i128_with_scale(3996007984031936128, 18).unwrap()
        );

        // Deposit of 0 amount do not change position
        position
            .deposit(
                Decimal::ZERO,
                Decimal::from_i128_with_scale(100000000000, 0)?,
            )
            .unwrap();
        assert_eq!(
            position.issued_vlp_amount().unwrap(),
            Decimal::from_i128_with_scale(1000000 + 2000, 0)?
        );
        assert_eq!(
            position.weighted_average_vlp_price().unwrap(),
            Decimal::from_i128_with_scale(3996007984031936128, 18).unwrap()
        );

        // Withdraw huge amount
        position
            .withdraw(
                Decimal::from_i128_with_scale(1000000, 0)?,
                Decimal::from_i128_with_scale(5, 0)?,
            )
            .unwrap();
        assert_eq!(
            position.issued_vlp_amount().unwrap(),
            Decimal::from_i128_with_scale(2000, 0)?
        );
        // avg price doesn't change
        assert_eq!(
            position.weighted_average_vlp_price().unwrap(),
            Decimal::from_i128_with_scale(3996007984031936128, 18).unwrap()
        );

        Ok(())
    }

    #[test]
    fn test_profit() {
        let mut position = Position::zeroed();

        // Deposit liquidity which were converted to 100 VLP at price of 1 liquidity for 1 VLP
        position.deposit(dec!(100.0), dec!(1.0)).unwrap();
        assert_eq!(position.weighted_average_vlp_price().unwrap(), dec!(1.0));

        position.deposit(dec!(50.0), dec!(4.0)).unwrap();
        assert_eq!(position.weighted_average_vlp_price().unwrap(), dec!(2.0));

        position.deposit(dec!(50.0), dec!(6.0)).unwrap();
        assert_eq!(position.weighted_average_vlp_price().unwrap(), dec!(3.0));

        assert_eq!(position.realized_profit().unwrap(), dec!(0.0));
        assert_eq!(
            position.total_estimated_profit(dec!(4.0)).unwrap(),
            dec!(200.0)
        );

        position.withdraw(dec!(100.0), dec!(4.0)).unwrap();
        assert_eq!(position.weighted_average_vlp_price().unwrap(), dec!(3.0));
        assert_eq!(position.issued_vlp_amount().unwrap(), dec!(100.0));
        assert_eq!(position.realized_profit().unwrap(), dec!(100.0));

        // We have 100 VLP acquired for weighted average price of 3 liquidity (100 VLP * 3 LIQ = 300 LIQ),
        // if we sell them for 4 liquidity we will earn another 100 liquidty because of price difference
        // 100 VLP * 4 LIQ = 400 LIQ
        // already had realized profit of 100 liquidity, and by adding another 100 we getting 200 total estimated profit
        assert_eq!(
            position.total_estimated_profit(dec!(4.0)).unwrap(),
            dec!(200.0)
        );
        // if we sell them for 2 liquidity per VLP we will get only 200 liquidity, it's loss of 100 liquidity
        // so 100 profit earned before + (-100) negative profit will lead us to zero profit
        assert_eq!(
            position.total_estimated_profit(dec!(2.0)).unwrap(),
            dec!(0.0)
        );
        // and if we sell for 1 liquidity per VLP, then we only getting 100 liquidity (e.g. losing 200)
        // and our big loss leads us to negative total estimated profit (100 - 200 = -100)
        assert_eq!(
            position.total_estimated_profit(dec!(1.0)).unwrap(),
            dec!(-100.0)
        );
        // event worse if sold for 0.5
        assert_eq!(
            position.total_estimated_profit(dec!(0.5)).unwrap(),
            dec!(-150.0)
        );

        // let's sell for 0.5 and experience that negative profit
        position.withdraw(dec!(100.0), dec!(0.5)).unwrap();
        assert_eq!(position.issued_vlp_amount().unwrap(), dec!(0.0));
        assert_eq!(position.realized_profit().unwrap(), dec!(-150.0));
        // we have 0 VLP, there is no perspective on getting any extra profit in the future
        // regardless of VLP price
        assert_eq!(position.estimated_profit(dec!(12345.0)).unwrap(), dec!(0));
        assert_eq!(
            position.total_estimated_profit(dec!(12345.0)).unwrap(),
            dec!(-150.0)
        );
        // and our weighted average VLP price should not be changed by withdrawals
        assert_eq!(position.weighted_average_vlp_price().unwrap(), dec!(3.0));

        // But don't worry, there are better days coming and we can "buy the dip",
        // e.g. invest in Vault when VLP price is low and get a lot of VLP for little liquidity:
        position.deposit(dec!(10000), dec!(0.1)).unwrap();
        // Wow, our weighted average VLP price instantly drops to that 0.1 and it's not affected
        // by previous Weighted AVG price of 3.0, because previous amount of VLP was zeroed
        assert_eq!(position.weighted_average_vlp_price().unwrap(), dec!(0.1));
        // 1 year later...
        // Vault reserves getting huge luck and Vault's VLP price jumps to 1.0,
        // let's earn:
        position.withdraw(dec!(5000), dec!(1.0)).unwrap();
        // Needless to say, we are not only recovered our loss of 150 liquidity but already earned
        // some extra: -150 + (5000 * 1.0 - 5000 * 0.1) = 4350
        assert_eq!(position.realized_profit().unwrap(), dec!(4350));
        // and we can get rich in perspective because VLP price is getting even higher:
        // 4350 + (5000 * 2.0 - 5000 * 0.1) = 12950
        assert_eq!(
            position.total_estimated_profit(dec!(2.0)).unwrap(),
            dec!(13850)
        );
        // take it
        position.withdraw(dec!(5000), dec!(2.0)).unwrap();
        // Was profit counted as realized?
        assert_eq!(position.realized_profit().unwrap(), dec!(13850));
        // Any room for earnings? Of course, not, we just burned all our VLP:
        assert_eq!(
            position.total_estimated_profit(dec!(9999999.0)).unwrap(),
            dec!(13850)
        );
        // Not even a single lamport?
        assert_eq!(
            position
                .estimated_profit(dec!(99999999999999999999.0))
                .unwrap(),
            dec!(0)
        );
    }
}
