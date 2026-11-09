use soroban_sdk::{contract, contractclient, contractimpl, Address, Env, panic_with_error, BytesN};
use crate::error::AdminError;
use crate::storage;
use crate::dependencies::treasury::{Client as TreasuryClient};
use crate::dependencies::bridge_oracle::{Client as BridgeOracleClient, Asset};
#[contract]
pub struct AdminContract;

#[contractclient(name = "AdminClient")]
pub trait Admin {

    /// Initializes the bridge oracle
    /// # Arguments
    /// * `admin` - The admin address
    /// * `treasury` - The treasury address
    /// * `bridge_oracle` - The bridge oracle address
    fn initialize(e: Env, admin: Address, treasury: Address, bridge_oracle: Address);

    /// Creates a new stablecoin
    /// # Arguments
    /// * `token` - The address of the token to add
    /// * `asset` - The asset of the fiat currency to peg the new token to
    /// * `blend_pool` - The address of the blend pool
    /// * `initial_supply` - The initial supply of the new token lended to the blend pool
    fn new_stablecoin(e: Env, token: Address, asset: Asset, blend_pool: Address, initial_supply: i128);

    /// Updates the supply of a token
    /// # Arguments
    /// * `token` - The address of the token
    /// * `amount` - The amount to update the supply by can be positive or negative
    fn update_supply(e: Env, token: Address, amount: i128);

    /// Set the pegkeeper
    /// # Arguments
    /// * `pegkeeper` - The address of the pegkeeper
    fn set_pegkeeper(e: Env, pegkeeper: Address);

    /// Set the oracle
    /// # Arguments
    /// * `oracle` - The address of the oracle
    fn set_oracle(e: Env, oracle: Address);

    /// Sets the admin address
    /// # Arguments
    /// * `admin` - The address of the new admin
    fn set_admin(e: Env, admin: Address);

    /// Updates this contract to a new version
    /// # Arguments
    /// * `new_wasm_hash` - The new wasm hash
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>);

    /// Updates the treasury contract to a new version
    /// # Arguments
    /// * `new_wasm_hash` - The new wasm hash
    fn upgrade_treasury(e: Env, new_wasm_hash: BytesN<32>);

    /// Updates the bridge oracle contract to a new version
    /// # Arguments
    /// * `new_wasm_hash` - The new wasm hash
    fn upgrade_bridge_oracle(e: Env, new_wasm_hash: BytesN<32>);

}

#[contractimpl]
impl Admin for AdminContract {

    fn initialize(e: Env, admin: Address, treasury: Address, bridge_oracle: Address) {
        storage::extend_instance(&e);
        if storage::is_init(&e) {
            panic_with_error!(&e, AdminError::AlreadyInitializedError);
        }
        storage::set_admin(&e, &admin);
        storage::set_treasury(&e, &treasury);
        storage::set_bridge_oracle(&e, &bridge_oracle);
    }

    fn new_stablecoin(e: Env, token: Address, asset: Asset, blend_pool: Address, initial_supply: i128) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        let treasury = storage::get_treasury(&e);
        let treasury_client = TreasuryClient::new(&e, &treasury);
        let bridge_oracle = BridgeOracleClient::new(&e, &storage::get_bridge_oracle(&e));
        let token_asset = Asset::Stellar(token.clone());

        bridge_oracle.add_asset(&token_asset, &asset);
        treasury_client.add_stablecoin(&token, &blend_pool);
        treasury_client.increase_supply(&token, &initial_supply);
    }

    fn update_supply(e: Env, token: Address, amount: i128) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        let treasury = TreasuryClient::new(&e, &storage::get_treasury(&e));
        if amount > 0 {
            treasury.increase_supply(&token, &amount);
        } else {
            treasury.decrease_supply(&token, &amount.abs());
        }
    }

    fn set_pegkeeper(e: Env, pegkeeper: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        let treasury = TreasuryClient::new(&e, &storage::get_treasury(&e));
        treasury.set_pegkeeper(&pegkeeper);
    }

    fn set_oracle(e: Env, oracle: Address) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        let bridge_oracle = BridgeOracleClient::new(&e, &storage::get_bridge_oracle(&e));
        bridge_oracle.set_oracle(&oracle);
    }

    fn set_admin(e: Env, admin: Address) {
        storage::extend_instance(&e);
        let current_admin = storage::get_admin(&e);
        current_admin.require_auth();
        storage::set_admin(&e, &admin);
    }

    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    fn upgrade_treasury(e: Env, new_wasm_hash: BytesN<32>) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        let treasury = TreasuryClient::new(&e, &storage::get_treasury(&e));
        treasury.upgrade(&new_wasm_hash);
    }

    fn upgrade_bridge_oracle(e: Env, new_wasm_hash: BytesN<32>) {
        storage::extend_instance(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();
        let bridge_oracle = BridgeOracleClient::new(&e, &storage::get_bridge_oracle(&e));
        bridge_oracle.upgrade(&new_wasm_hash);
    }
}