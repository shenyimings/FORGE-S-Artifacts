use soroban_sdk::{Address, Env, contracttype};
use soroban_sdk::unwrap::UnwrapOptimized;

const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5s a ledger
const LEDGER_THRESHOLD_INSTANCE: u32 = ONE_DAY_LEDGERS * 30; // ~ 30 days
const LEDGER_BUMP_INSTANCE: u32 = LEDGER_THRESHOLD_INSTANCE + ONE_DAY_LEDGERS; // ~ 31 days

#[derive(Clone)]
#[contracttype]
pub enum AdminDataKey {
    ADMIN,
    BRIDGEORACLE,
    TREASURY,
}

pub fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_INSTANCE, LEDGER_BUMP_INSTANCE);
}

pub fn is_init(e: &Env) -> bool { e.storage().instance().has(&AdminDataKey::ADMIN) }

pub fn get_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&AdminDataKey::ADMIN)
        .unwrap_optimized()
}

pub fn set_admin(e: &Env, new_admin: &Address) {
    e.storage()
        .instance()
        .set(&AdminDataKey::ADMIN, new_admin);
}

pub fn get_bridge_oracle(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&AdminDataKey::BRIDGEORACLE)
        .unwrap_optimized()
}

pub fn set_bridge_oracle(env: &Env, address: &Address) {
    env.storage()
        .instance()
        .set(&AdminDataKey::BRIDGEORACLE, address);
}

pub fn get_treasury(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&AdminDataKey::TREASURY)
        .unwrap_optimized()
}

pub fn set_treasury(env: &Env, address: &Address) {
    env.storage()
        .instance()
        .set(&AdminDataKey::TREASURY, address);
}