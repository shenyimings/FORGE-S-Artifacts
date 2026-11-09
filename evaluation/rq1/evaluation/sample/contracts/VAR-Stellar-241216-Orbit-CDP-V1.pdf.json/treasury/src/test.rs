#[cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal, Symbol};
use soroban_sdk::testutils::{AuthorizedFunction, AuthorizedInvocation};
use crate::{TreasuryContract, TreasuryClient};

#[test]
#[should_panic(expected = "Error(Contract, #1501)")] // AlreadyInitializedError
fn test_initialization() {
    let env: Env = Default::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);

    let treasury_address = env.register(TreasuryContract, ());
    let treasury_client = TreasuryClient::new(&env, &treasury_address);

    treasury_client.initialize(&admin, &oracle);
    treasury_client.initialize(&admin, &oracle);
}

#[test]
#[should_panic(expected = "Error(WasmVm, InvalidAction)")] // Fails because no admin is set
fn test_uninitialized() {
    let env: Env = Default::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let treasury_address = env.register(TreasuryContract, ());
    let treasury_client = TreasuryClient::new(&env, &treasury_address);

    treasury_client.set_pegkeeper(&Address::generate(&env));
}

#[test]
fn test_update_pegkeeper() {
    let env: Env = Default::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let pegkeeper = Address::generate(&env);

    let treasury_address = env.register(TreasuryContract, ());
    let treasury_client = TreasuryClient::new(&env, &treasury_address);

    treasury_client.initialize(&admin, &pegkeeper);

    let new_pegkeeper = Address::generate(&env);

    treasury_client.set_pegkeeper(&new_pegkeeper);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    treasury_address.clone(),
                    Symbol::new(&env, "set_pegkeeper"),
                    (new_pegkeeper.clone(),).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}