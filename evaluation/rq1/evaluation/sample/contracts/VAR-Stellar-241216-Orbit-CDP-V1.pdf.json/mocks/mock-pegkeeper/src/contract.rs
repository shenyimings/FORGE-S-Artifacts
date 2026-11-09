use soroban_sdk::{contract, contractclient, contractimpl, panic_with_error, Address, Env, Symbol};
use crate::errors::MockPegkeeperError;
use crate::{helper, storage};

#[contract]
pub struct MockPegkeeperContract;

#[contractclient(name="MockPegkeeperClient")]
pub trait MockPegkeepeer {

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
    fn fl_receive(e: Env, token: Address, amount: i128);

    /// Liquidate
    ///
    /// ### Arguments
    /// * `auction` - The Address for the blend auction
    /// * `token` - The Address for the treasury token
    /// * `bid_amount` - The amount of the bid
    /// * `collateral_token` - The Address for the collateral token
    /// * `lot_amount` - The amount of the lot
    /// * `blend_pool` - The Address for the blend pool
    /// * `liq_amount` - The amount of the liquidation
    fn liquidate(e: Env, auction: Address, token: Address, bid_amount: i128, collateral_token: Address, lot_amount: i128, blend_pool: Address, liq_amount: i128);

    /// Swap
    ///
    /// ### Arguments
    /// * `amm` - The Address for the AMM
    /// * `token` - The Address for the treasury token
    /// * `collateral_token` - The Address for the collateral token
    /// * `lot_amount` - The amount of the lot
    /// * `fee` - The amount of the fee
    fn swap(e: Env, amm: Address, token: Address, collateral_token: Address, lot_amount: i128, fee: i128);
}

#[contractimpl]
impl MockPegkeepeer for MockPegkeeperContract {
    fn initialize(e: Env, admin: Address, router: Address) {
        storage::extend_instance(&e);

        if storage::is_init(&e) {
            panic_with_error!(&e, MockPegkeeperError::AlreadyInitializedError);
        }

        storage::set_router(&e, &router);
        storage::set_admin(&e, &admin);

        e.events().publish(("Pegkeeper", Symbol::new(&e, "init")), (admin.clone(), router.clone()));
    }
    fn fl_receive(e: Env, token: Address, amount: i128) {
        storage::extend_instance(&e);

        let admin = storage::get_admin(&e);
        admin.require_auth();

        e.events().publish(("Pegkeeper", Symbol::new(&e, "init")), (token.clone(), amount.clone()));
    }

    fn liquidate(e: Env, auction: Address, token: Address, bid_amount: i128, collateral_token: Address, lot_amount: i128, blend_pool: Address, liq_amount: i128) {
        storage::extend_instance(&e);

        helper::liquidate(&e, auction, token.clone(), bid_amount.clone(), collateral_token.clone(), lot_amount.clone(), blend_pool.clone(), liq_amount.clone());

        e.events().publish(("Pegkeeper", Symbol::new(&e, "liquidate")), ());
    }

    fn swap(e: Env, amm: Address, token: Address, collateral_token: Address, lot_amount: i128, fee: i128) {
        storage::extend_instance(&e);

        helper::swap(&e, amm, collateral_token.clone(), token.clone(), lot_amount.clone(), fee.clone());
    }
}
