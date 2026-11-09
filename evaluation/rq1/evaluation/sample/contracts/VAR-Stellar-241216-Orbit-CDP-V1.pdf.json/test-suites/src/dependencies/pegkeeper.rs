use soroban_sdk::{testutils::Address as _, Address, Env};

mod pegkeeper_contract {
    soroban_sdk::contractimport!(file = "../wasm/orbit/pegkeeper.wasm");
}

pub use pegkeeper::{PegkeeperClient, PegkeeperContract};

pub fn create_pegkeeper<'a>(e: &Env, wasm: bool) -> (Address, PegkeeperClient<'a>) {
    let contract_id = Address::generate(e);
    if wasm {
        e.register_at(&contract_id, pegkeeper_contract::WASM, ());
    } else {
        e.register_at(&contract_id, PegkeeperContract {}, ());
    }
    (contract_id.clone(), PegkeeperClient::new(e, &contract_id))
}
