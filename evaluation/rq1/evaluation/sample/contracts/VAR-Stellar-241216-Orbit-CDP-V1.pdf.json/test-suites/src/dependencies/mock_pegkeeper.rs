use soroban_sdk::{testutils::Address as _, Address, Env};

mod mock_pegkeeper_contract {
    soroban_sdk::contractimport!(file = "../wasm/orbit/mock_pegkeeper.wasm");
}

pub use mock_pegkeeper::{MockPegkeeperClient, MockPegkeeperContract};

pub fn create_mock_pegkeeper<'a>(e: &Env, wasm: bool) -> (Address, MockPegkeeperClient<'a>) {
    let contract_id = Address::generate(e);
    if wasm {
        e.register_at(&contract_id, mock_pegkeeper_contract::WASM, ());
    } else {
        e.register_at(&contract_id, MockPegkeeperContract {}, ());
    }
    (contract_id.clone(), MockPegkeeperClient::new(e, &contract_id))
}
