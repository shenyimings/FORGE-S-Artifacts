use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MockTreasuryError {
    AlreadyInitializedError = 501,
    InvalidAmount = 502,
    FlashloanFailedError = 503,
}