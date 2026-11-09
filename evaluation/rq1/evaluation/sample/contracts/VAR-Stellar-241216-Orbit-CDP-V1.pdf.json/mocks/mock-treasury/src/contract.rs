use crate::storage;
use crate::dependencies::pool::{Client as PoolClient, Request};
use soroban_sdk::{contract, contractclient, contractimpl, panic_with_error, token, vec, Address, Env, IntoVal, Symbol, TryFromVal, Val, Vec};
use soroban_sdk::auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation};
use crate::errors::{MockTreasuryError};

#[contract]
pub struct MockTreasuryContract;

#[contractclient(name="MockTreasuryClient")]
pub trait MockTreasury {

    /// Initialize the treasury
    ///
    /// ### Arguments
    /// * `admin` - The Address for the admin
    /// * `token` - The Address for the token
    /// * `pegkeeper` - The Address for the pegkeeper
    ///
    /// ### Panics
    /// If the contract is already initialized
    fn initialize(e: Env, admin: Address, pegkeeper: Address);

    /// (Admin only) add a stablecoin
    ///
    /// ### Arguments
    /// * `token` - The Address for the token
    /// * `blend_pool` - The Address for the blend pool
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn add_stablecoin(e: Env, token: Address, blend_pool: Address);

    /// (Admin only) Increase the supply of the pool
    ///
    /// ### Arguments
    /// * `amount` - The amount to increase the supply by
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn increase_supply(e: Env, token: Address, amount: i128);

    /// (Admin only) Decrease the supply of the pool
    ///
    /// ### Arguments
    /// * `amount` - The amount to decrease the supply by
    ///
    /// ### Panics
    /// If the caller is not the admin
    /// If the supply is less than the amount
    fn decrease_supply(e: Env, token: Address, amount: i128);

    /// Flashloan function for keeping the peg of stablecoins
    ///
    /// ### Arguments
    /// * `name` - The name of the function to call
    /// * `args` - The arguments for the function first argument has to be token and second amount
    ///
    /// ### Panics
    /// If the flashloan is not profitable
    fn keep_peg(e: Env, name: Symbol, args: Vec<Val>);

    /// (Admin only) Set a new address as the pegkeeper
    ///
    /// ### Arguments
    /// * `pegkeeper` - The new pegkeeper address
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn set_pegkeeper(e: Env, pegkeeper: Address);
}

#[contractimpl]
impl MockTreasury for MockTreasuryContract {

    fn initialize(e: Env, admin: Address, pegkeeper: Address) {
        storage::extend_instance(&e);
        if storage::is_init(&e) {
            panic_with_error!(&e, MockTreasuryError::AlreadyInitializedError);
        }

        storage::set_pegkeeper(&e, &pegkeeper);
        storage::set_admin(&e, &admin);

        e.events().publish(("Treasury", Symbol::new(&e, "initialize")), (admin.clone(), pegkeeper.clone()));
    }

    fn add_stablecoin(e: Env, token: Address, blend_pool: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        storage::set_blend_pool(&e, &token, &blend_pool);

        e.events().publish(("Treasury", Symbol::new(&e, "add_stablecoin")), (token.clone(), blend_pool.clone()));
    }

    fn increase_supply(e: Env, token: Address, amount: i128) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        if amount <= 0 {
            panic_with_error!(e, MockTreasuryError::InvalidAmount);
        }

        token::StellarAssetClient::new(&e, &token).mint(&e.current_contract_address(), &amount);

        let blend = storage::get_blend_pool(&e, &token);
        let args: Vec<Val> = vec![
            &e,
            e.current_contract_address().into_val(&e),
            blend.into_val(&e),
            amount.into_val(&e),
        ];
        e.authorize_as_current_contract(vec![
            &e,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: token.clone(),
                    fn_name: Symbol::new(&e, "transfer"),
                    args: args.clone(),
                },
                sub_invocations: vec![&e],
            })
        ]);
        PoolClient::new(&e, &blend).submit(&e.current_contract_address(), &e.current_contract_address(), &e.current_contract_address(), &vec![
            &e,
            Request {
                request_type: 0_u32, // SUPPLY RequestType
                address: token.clone(),
                amount,
            },
        ]);

        e.events().publish(("Treasury", Symbol::new(&e, "increase_supply")), (token.clone(), amount.clone()));
    }

    fn decrease_supply(e: Env, token: Address, amount: i128) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        if amount <= 0 {
            panic_with_error!(e, MockTreasuryError::InvalidAmount);
        }

        let blend = storage::get_blend_pool(&e, &token);
        PoolClient::new(&e, &blend).submit(&e.current_contract_address(), &e.current_contract_address(), &e.current_contract_address(), &vec![
            &e,
            Request {
                request_type: 1_u32, // WITHDRAW RequestType
                address: token.clone(),
                amount,
            },
        ]);

        token::TokenClient::new(&e, &token).burn(&e.current_contract_address(), &amount);
        e.events().publish(("Treasury", Symbol::new(&e, "decrease_supply")), (token.clone(), amount.clone()));
    }

    fn keep_peg(e: Env, name: Symbol, args: Vec<Val>) {
        storage::extend_instance(&e);

        let token = Address::try_from_val(&e, &args.get(0).unwrap()).unwrap();
        let amount = i128::try_from_val(&e, &args.get(1).unwrap()).unwrap();
        let pegkeeper: Address = storage::get_pegkeeper(&e);

        if amount <= 0 {
            panic_with_error!(e, MockTreasuryError::InvalidAmount);
        }

        token::StellarAssetClient::new(&e, &token).mint(&pegkeeper, &amount);

        e.invoke_contract::<Val>(&pegkeeper, &name, args.clone());

        e.events().publish(("Treasury", Symbol::new(&e, "keep_peg")), (token.clone(), amount.clone()));
    }

    fn set_pegkeeper(e: Env, new_pegkeeper: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        storage::set_pegkeeper(&e, &new_pegkeeper);

        e.events().publish(("Treasury", Symbol::new(&e, "set_pegkeeper")), new_pegkeeper.clone());
    }
}
