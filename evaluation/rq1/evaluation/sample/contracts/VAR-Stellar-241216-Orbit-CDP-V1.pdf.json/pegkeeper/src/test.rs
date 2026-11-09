#[cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};
use crate::{PegkeeperContract, PegkeeperClient};

#[test]
#[should_panic(expected = "Error(Contract, #1501)")] // AlreadyInitializedError
fn test_initialization() {
    let env: Env = Default::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let router = Address::generate(&env);

    let pegkeeper_address = env.register(PegkeeperContract, ());
    let pegkeeper_client = PegkeeperClient::new(&env, &pegkeeper_address);

    pegkeeper_client.initialize(&admin, &router);
    pegkeeper_client.initialize(&admin, &router);
}

#[test]
#[should_panic(expected = "Error(WasmVm, InvalidAction)")] // Fails because no admin is set
fn test_uninitialized() {
    let env: Env = Default::default();
    env.mock_all_auths();

    let pegkeeper_address = env.register(PegkeeperContract, ());
    let pegkeeper_client = PegkeeperClient::new(&env, &pegkeeper_address);

    pegkeeper_client.fl_receive(&Address::generate(&env), &0, &Address::generate(&env), &Address::generate(&env), &Address::generate(&env), &0, &0, &Address::generate(&env), &Address::generate(&env));
}