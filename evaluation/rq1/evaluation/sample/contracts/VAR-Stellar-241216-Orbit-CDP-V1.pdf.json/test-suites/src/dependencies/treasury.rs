use soroban_sdk::{testutils::Address as _, Address, Env};

mod treasury_contract {
    soroban_sdk::contractimport!(
        file = "../wasm/orbit/treasury.wasm"
    );
}

pub use treasury_contract::WASM as POOL_WASM;
pub use treasury::{TreasuryClient, TreasuryContract};

pub fn create_treasury<'a>(e: &Env, wasm: bool) -> (Address, TreasuryClient<'a>) {
    let contract_id = Address::generate(e);
    if wasm {
        e.register_at(&contract_id, treasury_contract::WASM, ());
    } else {
        e.register_at(&contract_id, TreasuryContract {}, ());
    }
    (contract_id.clone(), TreasuryClient::new(e, &contract_id))
}