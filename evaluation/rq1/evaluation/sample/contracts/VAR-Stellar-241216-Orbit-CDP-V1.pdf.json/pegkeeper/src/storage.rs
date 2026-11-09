use soroban_sdk::{Address, Env, unwrap::UnwrapOptimized, contracttype};

const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5s a ledger
const LEDGER_THRESHOLD_INSTANCE: u32 = ONE_DAY_LEDGERS * 30; // ~ 30 days
const LEDGER_BUMP_INSTANCE: u32 = LEDGER_THRESHOLD_INSTANCE + ONE_DAY_LEDGERS; // ~ 31 days

#[derive(Clone)]
#[contracttype]
pub enum PegkeeperDataKey {
    ADMIN,
    ROUTER,
}

pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_INSTANCE, LEDGER_BUMP_INSTANCE);
}

pub fn is_init(e: &Env) -> bool { e.storage().instance().has(&PegkeeperDataKey::ADMIN) }

pub fn get_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&PegkeeperDataKey::ADMIN)
        .unwrap_optimized()
}

pub fn set_admin(e: &Env, new_admin: &Address) {
    e.storage()
        .instance()
        .set(&PegkeeperDataKey::ADMIN, new_admin);
}

pub fn get_router(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&PegkeeperDataKey::ROUTER)
        .unwrap_optimized()
}

pub fn set_router(e: &Env, new_router: &Address) {
    e.storage()
        .instance()
        .set(&PegkeeperDataKey::ROUTER, new_router);
}
