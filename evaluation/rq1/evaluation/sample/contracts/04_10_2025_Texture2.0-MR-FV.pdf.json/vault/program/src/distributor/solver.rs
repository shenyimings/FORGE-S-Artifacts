use texture_common::math::{CheckedAdd, CheckedSub, Decimal, MathError};
use thiserror::Error;

use super::{
    item::{Action, Decision, Item, Items},
    solution::Solution,
};

#[derive(Debug, Error)]
pub enum SolverError {
    #[error("math error: {0}")]
    MathError(#[from] MathError),
    #[error("all available liquidity exhausted")]
    LiquidityExhausted,
    #[error("reserve limits reached, requested delta - {requested_delta}, best possible - {actual_delta}")]
    AllLimitsReached {
        #[allow(dead_code)]
        actual_delta: Decimal,
        #[allow(dead_code)]
        requested_delta: Decimal,
    },
    #[error("internal solver error: can't fit deposits value into withdrawed amount")]
    CantFitDeposits,
}

#[derive(Clone, Debug)]
pub struct Solver {
    pub items: Items,
    pub requested_delta: Decimal,
}

#[derive(Debug)]
pub enum History {
    Round(usize),
    Step {
        id: usize,
        decision: Decision,
        reason: StepReason,
    },
}

#[derive(Debug)]
pub enum StepReason {
    Limited,
    Normal,
}

impl Solver {
    pub fn new(items: Vec<Item>, delta: Decimal) -> Self {
        Self {
            items: Items::new(items, delta),
            requested_delta: delta,
        }
    }

    pub fn set_delta(&mut self, delta: Decimal) {
        self.requested_delta = delta;
        self.items.set_delta(delta);
    }

    pub fn solve(
        mut self,
        mut step_storage: impl FnMut(History),
        decimals: u8,
    ) -> Result<Solution, SolverError> {
        let balance_deviation_before = self.items.balance_deviation(Decimal::ZERO)?;

        let mut round_idx = 0;
        loop {
            round_idx += 1;
            step_storage(History::Round(round_idx));

            let all_fits = self.items.iter_mut_unsolved()?.all(|item| match item {
                Ok(item) => item.fits(),
                Err(err) => panic!("iter_mut_unsolved: {}", err),
            });

            if all_fits {
                let items = self.items.iter_mut_unsolved()?;
                for item in items {
                    let mut item = item?;
                    let decision = item.fit()?;
                    step_storage(History::Step {
                        id: item.item.id,
                        decision,
                        reason: StepReason::Normal,
                    });
                }
                break;
            }

            // Some ideal amounts do not fit into item bounds, let's work on them.
            // Move most bounds-violating items to the end of array
            self.items.sort()?;

            let items = self.items.iter_mut_unsolved()?;
            let mut last_violation = None;
            let baddest_violators = items
                .rev()
                .map(|item| item.expect("iter_mut_unsolved"))
                .filter(|item| !item.fits())
                .take_while(|item| match last_violation {
                    None => {
                        let item_bounds_violation =
                            item.bounds_violation().expect("bounds violation calc");
                        last_violation = Some(item_bounds_violation);
                        true
                    }
                    Some(prev) => {
                        let item_bounds_violation =
                            item.bounds_violation().expect("bounds violation calc");
                        last_violation = Some(item_bounds_violation);
                        item_bounds_violation == prev
                    }
                });
            let mut delta = Decimal::ZERO;
            for mut item in baddest_violators {
                let decision = item.fit()?;
                step_storage(History::Step {
                    id: item.item.id,
                    decision: decision.clone(),
                    reason: StepReason::Limited,
                });
                match decision {
                    Decision::Action(action) => match action {
                        Action::Deposit(decimal) => {
                            delta = delta.checked_sub(decimal)?;
                        }
                        Action::Withdraw(decimal) => {
                            delta = delta.checked_add(decimal)?;
                        }
                    },
                    Decision::Keep => {}
                }
            }
            self.items.adjust_delta(delta)?;
        }

        if self.requested_delta != Decimal::ZERO {
            let actual_delta = self
                .items
                .items
                .iter()
                .map(|item| {
                    let change = match item.decision.as_ref().expect("decision") {
                        Decision::Action(action) => match action {
                            Action::Deposit(amount) => *amount,
                            Action::Withdraw(amount) => Decimal::ZERO.checked_sub(*amount)?,
                        },
                        Decision::Keep => Decimal::ZERO,
                    };
                    Ok(change)
                })
                .try_fold(Decimal::ZERO, |acc, item| acc.checked_add(item?))?;
            if self.requested_delta > Decimal::ZERO && actual_delta < self.requested_delta {
                let deficit = self.requested_delta.checked_sub(actual_delta)?;
                if deficit > Decimal::from_i128_with_scale(1, 9)? {
                    return Err(SolverError::AllLimitsReached {
                        actual_delta,
                        requested_delta: self.requested_delta,
                    });
                }
            }
        }

        let balance_deviation_after = self.items.balance_deviation(self.requested_delta)?;

        Solution::new(
            self.items,
            balance_deviation_before,
            balance_deviation_after,
            self.requested_delta,
            decimals,
        )
    }
}
