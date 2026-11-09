use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{Deref, DerefMut, RangeInclusive},
};

use super_lendy::state::reserve::LpExchangeRate;
use texture_common::math::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Decimal};

use super::solver::SolverError;

#[derive(Clone, Debug)]
pub struct Item {
    /// Some identificator
    pub id: usize,
    /// Possible liquidity range
    pub bounds: RangeInclusive<Decimal>,
    /// Current liquidity amount
    pub liquidity_amount: Decimal,
    // /// Current LP amount
    pub lp_amount: u64,
    /// Target share
    pub share: Decimal,
    /// Exchange rate
    pub exchange_rate: LpExchangeRate,
    /// Decision
    pub decision: Option<Decision>,
}

impl Item {
    pub fn new(
        id: usize,
        bounds: RangeInclusive<Decimal>,
        liquidity_amount: Decimal,
        lp_amount: u64,
        share: Decimal,
        exchange_rate: LpExchangeRate,
    ) -> Self {
        Self {
            id,
            bounds,
            liquidity_amount,
            lp_amount,
            share,
            exchange_rate,
            decision: None,
        }
    }

    fn unsolved(&self) -> bool {
        self.decision.is_none()
    }

    fn final_liquidity(&self) -> Result<Decimal, SolverError> {
        let final_liquidity = if let Some(Decision::Action(action)) = &self.decision {
            match action {
                Action::Deposit(decimal) => self.liquidity_amount.checked_add(*decimal),
                Action::Withdraw(decimal) => self.liquidity_amount.checked_sub(*decimal),
            }?
        } else {
            self.liquidity_amount
        };
        Ok(final_liquidity)
    }
}

/// A view on an item in some narrowed-down context, where
/// its `ideal_amount`` is calculated based on its share among all
/// context participants and total amount targeted for distribution
#[derive(Debug)]
pub struct ItemView<T> {
    pub ideal_amount: Decimal,
    pub item: T,
}

impl<T> ItemView<T> {
    fn new(item: T, ctx: Context) -> Result<Self, SolverError>
    where
        T: Deref<Target = Item>,
    {
        let normalized_target_share = item.share.checked_div(ctx.total_share)?;

        let ideal_amount = ctx.total_amount.checked_mul(normalized_target_share)?;

        Ok(Self { ideal_amount, item })
    }
}

impl<T: Deref<Target = Item>> ItemView<T> {
    /// How much item would violate its bounds if it use ideal_amount as amount
    /// Zero for item which has decision.
    pub fn bounds_violation(&self) -> Result<Decimal, SolverError> {
        if self.item.decision.is_some() {
            return Ok(Decimal::ZERO);
        }
        Ok(if &self.ideal_amount > self.item.bounds.end() {
            self.ideal_amount.checked_sub(*self.item.bounds.end())?
        } else if &self.ideal_amount < self.item.bounds.start() {
            self.item.bounds.start().checked_sub(self.ideal_amount)?
        } else {
            Decimal::ZERO
        })
    }

    pub fn fits(&self) -> bool {
        self.item.bounds.contains(&self.ideal_amount)
    }
}

impl<T: std::fmt::Debug + DerefMut<Target = Item>> ItemView<T> {
    pub fn fit(&mut self) -> Result<Decision, SolverError> {
        assert!(
            self.item.decision.is_none(),
            "trying to fit item which has decision"
        );

        let target_amount = self
            .ideal_amount
            .clamp(*self.item.bounds.start(), *self.item.bounds.end());

        let diff = target_amount.checked_sub(self.item.liquidity_amount)?;

        if diff < Decimal::from_i128_with_scale(1, 9)?
            && diff > Decimal::from_i128_with_scale(-1, 9)?
        {
            // Diff too small, don't do anything
            self.item.decision = Some(Decision::Keep);
        } else {
            self.item.decision = Some(match target_amount.cmp(&self.item.liquidity_amount) {
                Ordering::Less => Decision::Action(Action::Withdraw(
                    self.item.liquidity_amount.checked_sub(target_amount)?,
                )),
                Ordering::Equal => Decision::Keep,
                Ordering::Greater => Decision::Action(Action::Deposit(
                    target_amount.checked_sub(self.item.liquidity_amount)?,
                )),
            });
        }

        Ok(self
            .item
            .decision
            .as_ref()
            .cloned()
            .expect("prepared decision"))
    }
}

#[derive(Clone, Debug)]
pub struct Items {
    /// Items to balance
    pub items: Vec<Item>,
    /// Liquidity delta which starts equal to requested delta and accumulates
    /// later when items hit their bounds
    accumulated_delta: Decimal,
}

impl Items {
    pub fn new(items: Vec<Item>, delta: Decimal) -> Self {
        Self {
            items,
            accumulated_delta: delta,
        }
    }

    pub fn adjust_delta(&mut self, amount: Decimal) -> Result<(), SolverError> {
        self.accumulated_delta = self.accumulated_delta.checked_add(amount)?;
        Ok(())
    }

    pub fn set_delta(&mut self, delta: Decimal) {
        self.accumulated_delta = delta
    }

    fn calculate_total<F, S>(&self, filter: F, selector: S) -> Result<Decimal, SolverError>
    where
        F: Fn(&&Item) -> bool,
        S: Fn(&Item) -> Decimal,
    {
        Ok(self
            .items
            .iter()
            .filter(filter)
            .map(selector)
            .try_fold(Decimal::ZERO, CheckedAdd::checked_add)?)
    }

    /// Sort items by amount of bounds violation
    pub fn sort(&mut self) -> Result<(), SolverError> {
        let ctx = self.context()?;

        self.items.sort_by_key(|item| {
            ItemView::new(item, ctx)
                .expect("item view")
                .bounds_violation()
                .expect("calc bounds violation")
        });

        Ok(())
    }

    pub fn iter_mut_unsolved(
        &mut self,
    ) -> Result<
        impl DoubleEndedIterator<Item = Result<ItemView<&mut Item>, SolverError>>,
        SolverError,
    > {
        let ctx = self.context()?;

        Ok(self
            .items
            .iter_mut()
            .filter(|item| item.unsolved())
            .map(move |item| ItemView::new(item, ctx)))
    }

    fn context(&self) -> Result<Context, SolverError> {
        let total_share = self.calculate_total(|item| item.unsolved(), |item| item.share)?;
        let total_amount =
            self.calculate_total(|item| item.unsolved(), |item| item.liquidity_amount)?;

        let total_amount = total_amount.checked_add(self.accumulated_delta)?;

        if total_amount < Decimal::ZERO {
            return Err(SolverError::LiquidityExhausted);
        }

        Ok(Context {
            total_share,
            total_amount,
        })
    }

    /// How much current balance deviates from target shares (Mean Absolute Deviation)
    pub fn balance_deviation(&self, delta: Decimal) -> Result<Decimal, SolverError> {
        let total_share = self.calculate_total(|_| true, |item| item.share)?;
        let total_amount = self.calculate_total(|_| true, |item| item.liquidity_amount)?;

        let total_amount = total_amount.checked_add(delta)?;

        if self.items.is_empty() || total_amount == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }

        let mean_absolute_deviation = self
            .items
            .iter()
            .map(|item| {
                let final_liquidity = item.final_liquidity()?;
                let normalized_liquidity = final_liquidity.checked_div(total_amount)?;

                let normalized_share = item.share.checked_div(total_share)?;

                let deviation = normalized_liquidity.checked_sub(normalized_share)?.abs()?;
                Ok::<_, SolverError>(deviation)
            })
            .try_fold(Decimal::ZERO, |acc, item| {
                let result = acc.checked_add(item?)?;
                Ok::<_, SolverError>(result)
            })?
            .checked_div(Decimal::from_i128_with_scale(self.items.len() as i128, 0)?)?;

        Ok(mean_absolute_deviation)
    }
}

trait Abs<T> {
    fn abs(&self) -> Result<T, SolverError>;
}

impl Abs<Decimal> for Decimal {
    fn abs(&self) -> Result<Decimal, SolverError> {
        Ok(if self < &Decimal::ZERO {
            Decimal::ZERO.checked_sub(*self)?
        } else {
            *self
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct Context {
    total_share: Decimal,
    total_amount: Decimal,
}

#[derive(Clone, Debug)]
pub enum Decision {
    Action(Action),
    Keep,
}

impl Display for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Decision::Action(action) => match action {
                Action::Deposit(decimal) => {
                    f.write_str(&format!("+{}", decimal.round_to_decimals(2)))
                }
                Action::Withdraw(decimal) => {
                    f.write_str(&format!("-{}", decimal.round_to_decimals(2)))
                }
            },
            Decision::Keep => f.write_str("-"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Action {
    Deposit(Decimal),
    Withdraw(Decimal),
}
