use soroban_sdk::{contract, contractclient, contractimpl, panic_with_error, token, Address, Env, Symbol};
use crate::{errors::PegkeeperError, storage, helper};

#[contract]
pub struct PegkeeperContract;

#[contractclient(name="PegkeeperClient")]
pub trait Pegkeeper {

    /// Initialize the treasury
    ///
    /// ### Arguments
    /// * `admin` - The Address for the admin
    /// * `maximum_duration` - The maximum_duration for swap transaction
    fn initialize(e: Env, admin: Address, router: Address);

    /// Execute operation
    ///
    /// ### Arguments
    /// * `token` - The Address for the token
    /// * `amount` - The amount received of the token
    /// * `blend_pool` - The Address for the blend pool
    /// * `auction` - The Address for the auction
    /// * `collateral_token` - The Address for the collateral token
    /// * `lot_amount` - The amount of the collateral token to withdraw after liquidation
    /// * `liq_amount` - The amount to liquidate in percentage 0-100
    /// * `amm` - The Address for the AMM
    /// * `fee_taker` - The Address for the fee taker
    fn fl_receive(e: Env, token: Address, amount: i128, blend_pool: Address, auction: Address, collateral_token: Address, lot_amount: i128, liq_amount: i128, amm: Address, fee_taker: Address);
}

#[contractimpl]
impl Pegkeeper for PegkeeperContract {

    fn initialize(e: Env, admin: Address, router: Address) {
        storage::extend_instance(&e);

        if storage::is_init(&e) {
            panic_with_error!(&e, PegkeeperError::AlreadyInitializedError);
        }

        storage::set_router(&e, &router);
        storage::set_admin(&e, &admin);
        e.events().publish(("Pegkeeper", Symbol::new(&e, "init")), (admin.clone(), router.clone()));
    }

    fn fl_receive(e: Env, token: Address, amount: i128, blend_pool: Address, auction: Address, collateral_token: Address, lot_amount: i128, liq_amount: i128, amm: Address, fee_taker: Address) {
        storage::extend_instance(&e);

        let admin = storage::get_admin(&e);
        admin.require_auth();

        let token_client = token::Client::new(&e, &token);
        let collateral_client = token::Client::new(&e, &collateral_token);
        let balance_before = token_client.balance(&e.current_contract_address());
        let collateral_balance = collateral_client.balance(&e.current_contract_address());

        helper::liquidate(&e, auction, token.clone(), amount.clone(), collateral_token.clone(), lot_amount.clone(), blend_pool.clone(), liq_amount.clone());

        let collateral_balance_after = collateral_client.balance(&e.current_contract_address());
        let lot_amount = collateral_balance_after - collateral_balance;

        helper::swap(&e, amm, collateral_token.clone(), token.clone(), lot_amount.clone(), 0);

        let balance_after = token_client.balance(&e.current_contract_address());

        if balance_before > balance_after {
            panic_with_error!(&e, PegkeeperError::NotProfitable);
        }

        let profit = balance_after - balance_before;
        token_client.transfer(&e.current_contract_address(), &fee_taker, &profit);

        token_client.approve(
            &e.current_contract_address(),
            &admin,
            &amount,
            &(e.ledger().sequence() + 1),
        );

        e.events().publish(("Pegkeeper", Symbol::new(&e, "fl_receive")), (token.clone(), amount.clone()));
    }
}
