use soroban_sdk::{testutils::Address as _, Address, Env};
mod bridge_oracle_contract {
    soroban_sdk::contractimport!(file = "../wasm/orbit/bridge_oracle.wasm");
}

pub use bridge_oracle::{BridgeOracleClient, BridgeOracleContract};

pub fn create_bridge_oracle<'a>(e: &Env, wasm: bool) -> (Address, BridgeOracleClient<'a>) {
    let contract_id = Address::generate(e);
    if wasm {
        e.register_at(&contract_id, bridge_oracle_contract::WASM, ());
    } else {
        e.register_at(&contract_id, BridgeOracleContract {}, ());
    }
    (contract_id.clone(), BridgeOracleClient::new(e, &contract_id))
}