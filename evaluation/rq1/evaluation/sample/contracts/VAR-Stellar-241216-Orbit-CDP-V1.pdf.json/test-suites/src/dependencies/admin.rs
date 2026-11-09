use soroban_sdk::{testutils::Address as _, Address, Env};
mod admin_contract {
    use::bridge_oracle::Asset;

    soroban_sdk::contractimport!(file = "../wasm/orbit/admin.wasm");
}

pub use admin::{AdminClient, AdminContract};

pub fn create_admin<'a>(e: &Env, wasm: bool) -> (Address, AdminClient<'a>) {
    let contract_id = Address::generate(e);
    if wasm {
        e.register_at(&contract_id, admin_contract::WASM, ());
    } else {
        e.register_at(&contract_id, AdminContract {}, ());
    }
    (contract_id.clone(), AdminClient::new(e, &contract_id))
}