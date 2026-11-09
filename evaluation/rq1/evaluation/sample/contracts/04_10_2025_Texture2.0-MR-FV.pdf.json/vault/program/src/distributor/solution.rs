use texture_common::math::{CheckedAdd, Decimal};

use super::solver::SolverError;

use super::item::{Action, Decision, Items};

#[derive(Debug)]
pub struct Withdrawal {
    pub reserve_idx: usize,
    pub lp_amount_to_return: u64,
    pub expected_liquidity: u64,
}

#[derive(Debug)]
pub struct Deposit {
    pub reserve_idx: usize,
    pub liquidity_amount: u64,
}

#[derive(Clone, Debug)]
pub struct Solution {
    pub items: Items,
    pub balance_deviation_before: Decimal,
    pub balance_deviation_after: Decimal,
    pub requested_delta: Decimal,
    pub expected_buffer_change: i128,
}

impl Solution {
    pub fn new(
        items: Items,
        balance_deviation_before: Decimal,
        balance_deviation_after: Decimal,
        requested_delta: Decimal,
        decimals: u8,
    ) -> Result<Self, SolverError> {
        let mut solution = Self {
            items,
            balance_deviation_before,
            balance_deviation_after,
            requested_delta,
            expected_buffer_change: 0,
        };

        let total_withdrawn = solution
            .withdrawals(decimals)
            .map(|withdrawal| Ok::<_, SolverError>(withdrawal?.expected_liquidity))
            .try_fold(0, |acc, item| {
                Ok::<_, SolverError>(CheckedAdd::checked_add(acc, item?)?)
            })?;

        let total_deposited = solution
            .deposits(decimals)
            .map(|deposit| Ok::<_, SolverError>(deposit?.liquidity_amount))
            .try_fold(0, |acc, item| {
                Ok::<_, SolverError>(CheckedAdd::checked_add(acc, item?)?)
            })?;

        let available_liquidity = CheckedAdd::checked_add(
            total_withdrawn,
            requested_delta
                .max(Decimal::ZERO)
                .to_lamports_round(decimals)?,
        )?;

        // Expand deposits if delta >= 0 to use all released liquidity in case of usual rebalancing or when user adds liquidity
        if solution.requested_delta >= Decimal::ZERO && total_deposited < available_liquidity {
            let first_deposit = solution.items.items.iter_mut().find_map(|item| {
                let Some(Decision::Action(Action::Deposit(deposit))) = item.decision.as_mut()
                else {
                    return None;
                };
                Some(deposit)
            });
            if let Some(first_deposit) = first_deposit {
                let excess_liquidity = available_liquidity - total_deposited;
                *first_deposit = first_deposit
                    .checked_add(Decimal::from_lamports(excess_liquidity, decimals)?)?;
            }
        } else if total_deposited > available_liquidity {
            // Fit deposits into available liquidity amount
            let mut liquidity_deficit = total_deposited - available_liquidity;

            for deposit in solution.items.items.iter_mut().filter_map(move |item| {
                let Some(Decision::Action(Action::Deposit(amount))) = &mut item.decision else {
                    return None;
                };
                Some(amount)
            }) {
                let mut liquidity_amount = deposit.to_lamports_round(decimals)?;

                if liquidity_amount >= liquidity_deficit {
                    liquidity_amount -= liquidity_deficit;
                    liquidity_deficit = 0;
                    *deposit = Decimal::from_lamports(liquidity_amount, decimals)?;
                    break;
                } else {
                    liquidity_deficit -= liquidity_amount;
                    liquidity_amount = 0;
                    *deposit = Decimal::from_lamports(liquidity_amount, decimals)?;
                }
            }

            if liquidity_deficit > 0 {
                return Err(SolverError::CantFitDeposits);
            }
        }

        let mut expected_buffer_change: i128 = 0;
        for withdrawal in solution.withdrawals(decimals) {
            expected_buffer_change = expected_buffer_change
                .checked_add(withdrawal?.expected_liquidity as i128)
                .expect("expected_buffer_change overflow");
        }
        for deposit in solution.deposits(decimals) {
            expected_buffer_change = expected_buffer_change
                .checked_sub(deposit?.liquidity_amount as i128)
                .expect("expected_buffer_change overflow");
        }
        solution.expected_buffer_change = expected_buffer_change;

        Ok(solution)
    }
}

impl<'a> Solution {
    pub fn withdrawals(
        &'a self,
        decimals: u8,
    ) -> impl Iterator<Item = Result<Withdrawal, SolverError>> + 'a {
        self.items.items.iter().filter_map(move |item| {
            let Some(Decision::Action(Action::Withdraw(amount))) = item.decision else {
                return None;
            };

            let withdrawal = (|| {
                let lp_amount = item.exchange_rate.decimal_liquidity_to_lp(amount)?;
                let lp_amount_to_return = if self.requested_delta < Decimal::ZERO {
                    // Act greedy on negative requested deltas (user withdraw liquidity)
                    lp_amount.to_lamports_floor(decimals)?.min(item.lp_amount)
                } else {
                    // Act fair on positive requested deltas (target better balance)
                    lp_amount.to_lamports_round(decimals)?.min(item.lp_amount)
                };
                let expected_liquidity = item.exchange_rate.lp_to_liquidity(lp_amount_to_return)?;

                Ok(Withdrawal {
                    reserve_idx: item.id,
                    lp_amount_to_return,
                    expected_liquidity,
                })
            })();

            Some(withdrawal)
        })
    }

    pub fn deposits(
        &'a self,
        decimals: u8,
    ) -> impl Iterator<Item = Result<Deposit, SolverError>> + 'a {
        self.items.items.iter().filter_map(move |item| {
            let Some(Decision::Action(Action::Deposit(amount))) = item.decision else {
                return None;
            };

            let liquidity_amount = amount.to_lamports_round(decimals);

            Some(
                liquidity_amount
                    .map(|liquidity_amount| Deposit {
                        reserve_idx: item.id,
                        liquidity_amount,
                    })
                    .map_err(Into::into),
            )
        })
    }
}
