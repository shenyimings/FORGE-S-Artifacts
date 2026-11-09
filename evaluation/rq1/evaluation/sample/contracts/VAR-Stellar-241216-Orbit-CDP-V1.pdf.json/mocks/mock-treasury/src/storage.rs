use soroban_sdk::{Address, contracttype, Env};
use soroban_sdk::unwrap::UnwrapOptimized;

pub(crate) const LEDGER_THRESHOLD_SHARED: u32 = 172800; // ~ 10 days
pub(crate) const LEDGER_BUMP_SHARED: u32 = 241920; // ~ 14 days

#[derive(Clone)]
#[contracttype]
pub enum TreasuryDataKey {
    ADMIN,
    BLENDPOOL(Address), // mapping token address to the blend pool addres
    PEGKEEPER,
}

pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
}

pub fn is_init(e: &Env) -> bool { e.storage().instance().has(&TreasuryDataKey::ADMIN) }

pub fn get_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&TreasuryDataKey::ADMIN)
        .unwrap_optimized()
}

pub fn set_admin(e: &Env, new_admin: &Address) {
    e.storage()
        .instance()
        .set(&TreasuryDataKey::ADMIN, new_admin);
}

pub fn get_pegkeeper(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&TreasuryDataKey::PEGKEEPER)
        .unwrap_optimized()
}

pub fn set_pegkeeper(e: &Env, new_pegkeeper: &Address) {
    e.storage()
        .instance()
        .set(&TreasuryDataKey::PEGKEEPER, new_pegkeeper);
}

pub fn get_blend_pool(e: &Env, token_address: &Address) -> Address {
    e.storage()
        .instance()
        .get(&TreasuryDataKey::BLENDPOOL(token_address.clone()))
        .unwrap_optimized()
}

pub fn set_blend_pool(e: &Env, token_address: &Address, blend_pool: &Address) {
    e.storage()
        .instance()
        .set(&TreasuryDataKey::BLENDPOOL(token_address.clone()), blend_pool);
}