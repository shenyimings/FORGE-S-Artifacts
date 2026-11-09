mod pool_contract {
    soroban_sdk::contractimport!(file = "../wasm/blend/pool.wasm");
}

pub use pool_contract::{Client as PoolClient, ReserveConfig, Positions, PoolDataKey, ReserveData, ReserveEmissionsData, ReserveEmissionsConfig, PoolConfig,  Request, ReserveEmissionMetadata, Reserve, WASM as POOL_WASM};

#[derive(Clone, PartialEq)]
#[repr(u32)]
pub enum RequestType {
    Supply = 0,
    Withdraw = 1,
    SupplyCollateral = 2,
    WithdrawCollateral = 3,
    Borrow = 4,
    Repay = 5,
    FillUserLiquidationAuction = 6,
    FillBadDebtAuction = 7,
    FillInterestAuction = 8,
    DeleteLiquidationAuction = 9,
}