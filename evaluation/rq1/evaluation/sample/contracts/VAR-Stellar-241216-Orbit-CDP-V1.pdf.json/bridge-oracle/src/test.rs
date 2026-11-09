#[cfg(test)]

use sep_40_oracle::Asset;
use sep_40_oracle::testutils::{MockPriceOracleClient, MockPriceOracleWASM};
use sep_40_oracle::testutils::Asset as TestAsset;
use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal, Symbol, vec as svec, symbol_short};
use soroban_sdk::testutils::{AuthorizedFunction, AuthorizedInvocation};
use crate::{BridgeOracleContract, BridgeOracleClient};

pub(crate) fn create_mock_oracle(e: &Env) -> (Address, MockPriceOracleClient) {
    let contract_address = e.register(MockPriceOracleWASM, ());
    (
        contract_address.clone(),
        MockPriceOracleClient::new(e, &contract_address),
    )
}

#[test]
#[should_panic(expected = "Error(Contract, #1501)")] // AlreadyInitializedError
fn test_initialization() {
    let env: Env = Default::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);

    let bridge_oracle_address = env.register(BridgeOracleContract, ());
    let bridge_oracle_client = BridgeOracleClient::new(&env, &bridge_oracle_address);

    bridge_oracle_client.initialize(&admin, &oracle);

    bridge_oracle_client.initialize(&admin, &oracle);
}

#[test]
fn test_add_assets() {
    let env: Env = Default::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    // We don't need actual token contracts for this test
    let token1 = Address::generate(&env);
    let token2 = Address::generate(&env);
    let bridge_oracle_address = env.register(BridgeOracleContract, ());
    let bridge_oracle_client = BridgeOracleClient::new(&env, &bridge_oracle_address);
    let (oracle_address, mock_oracle_client) = create_mock_oracle(&env);

    mock_oracle_client.set_data(
        &admin.clone(),
        &TestAsset::Other(Symbol::new(&env, "USD")),
        &svec![
                &env,
                TestAsset::Other(Symbol::new(&env, "USD")),
                TestAsset::Other(Symbol::new(&env, "EURO")),
            ],
        &7,
        &300,
    );
    mock_oracle_client.set_price_stable(&svec![
            &env,
            1_0000000,
            1_1000000,
        ]);

    bridge_oracle_client.initialize(&admin, &oracle_address);

    let stellar_asset = Asset::Stellar(token1.clone());
    let stellar_asset2 = Asset::Stellar(token2.clone());
    let usd_asset = Asset::Other(Symbol::new(&env, "USD"));
    let euro_asset = Asset::Other(Symbol::new(&env, "EURO"));
    bridge_oracle_client.add_asset(
        &stellar_asset,
        &usd_asset,
    );
    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    bridge_oracle_address.clone(),
                    symbol_short!("add_asset"),
                    (stellar_asset.clone(), usd_asset.clone()).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    bridge_oracle_client.add_asset(
        &stellar_asset2,
        &euro_asset,
    );

    let token1_price = bridge_oracle_client.lastprice(&Asset::Stellar(token1.clone())).unwrap().price;
    let token2_price = bridge_oracle_client.lastprice(&Asset::Stellar(token2.clone())).unwrap().price;
    let usd_price = bridge_oracle_client.lastprice(&Asset::Other(Symbol::new(&env, "USD"))).unwrap().price;
    let euro_price = bridge_oracle_client.lastprice(&Asset::Other(Symbol::new(&env, "EURO"))).unwrap().price;

    assert_eq!(usd_price, 1_0000000);
    assert_eq!(euro_price, 1_1000000);
    assert_eq!(token1_price, usd_price);
    assert_eq!(token2_price, euro_price);
}

#[test]
#[should_panic(expected = "Error(WasmVm, InvalidAction)")] // Fails because no admin is set
fn test_uninitialized() {
    let env: Env = Default::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let bridge_oracle_address = env.register(BridgeOracleContract, ());
    let bridge_oracle_client = BridgeOracleClient::new(&env, &bridge_oracle_address);

    bridge_oracle_client.set_oracle(&Address::generate(&env));
}

#[test]
fn test_update_oracle() {
    let env: Env = Default::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);

    let bridge_oracle_address = env.register(BridgeOracleContract, ());
    let bridge_oracle_client = BridgeOracleClient::new(&env, &bridge_oracle_address);

    bridge_oracle_client.initialize(&admin, &oracle);

    let new_oracle = Address::generate(&env);

    bridge_oracle_client.set_oracle(&new_oracle);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    bridge_oracle_address.clone(),
                    Symbol::new(&env, "set_oracle"),
                    (new_oracle.clone(),).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}