use borsh::io::Error as BorshIoError;
use bytemuck::PodCastError;
use solana_program::program_error::ProgramError;
use solana_program::program_error::ProgramError::Custom;
use solana_program::pubkey::{Pubkey, PubkeyError};
use solana_program::system_instruction::SystemError;
use spl_token::error::TokenError;
use strum::EnumDiscriminants;
use super_lendy::error::SuperLendyError;
use texture_common::account;
use texture_common::error;
use texture_common::math::{Decimal, MathError};
use texture_common::remote::RemoteError;
use thiserror::Error;

use crate::distributor::solver::SolverError;

#[derive(Debug, Error)]
pub enum SerializeError {
    #[error("borsh: {0}")]
    Borsh(#[from] BorshIoError),
    #[error("not enough data")]
    NotEnoughData,
    #[error("not enough space")]
    NotEnoughSpace,
    #[error("too much space")]
    TooMuchSpace,
    #[error("version mismatch: {actual} != {expected}")]
    VersionMismatch { expected: u8, actual: u8 },
    #[error("bytemuck: {0}")]
    Bytemuck(#[from] PodCastError),
    #[error("math: {0}")]
    Math(#[from] MathError),
    #[error("reinitialization attempt")]
    Reinit,
    #[error("uninitialized data")]
    Uninit,
    #[error("invalid data")]
    Invalid,
}

#[derive(Debug, Error, EnumDiscriminants)]
pub enum VaultError {
    #[error("math error: {0}")]
    MathError(#[from] MathError),

    #[error("borsh error: {0}")]
    Borsh(#[from] BorshIoError),

    #[error("serialize error: {0}")]
    Serialize(#[from] SerializeError),

    #[error("pod account: {0}")]
    PodAccount(#[from] account::PodAccountError),

    #[error("pod account: {0}")]
    PodAccountExt(#[from] account::PodAccountErrorWithHeader),

    #[error(transparent)]
    InvalidKey(#[from] error::InvalidKey),

    #[error(transparent)]
    InvalidAccount(#[from] error::InvalidAccount),

    #[error(transparent)]
    NotEnoughAccountKeys(#[from] error::NotEnoughAccountKeys),

    #[error(transparent)]
    MissingSignature(#[from] error::MissingSignature),

    #[error("unimplemented")]
    Unimplemented,

    #[error("uninintialized account: {0}")]
    UninitializedAccount(Pubkey),

    #[error("address creation error: {0}")]
    AddressCreation(#[from] PubkeyError),

    #[error("error unpaking account {0} with error {1}")]
    AccountUnpackError(Pubkey, ProgramError),

    #[error("invalid config")]
    InvalidConfig,

    #[error("invalid amount")]
    InvalidAmount,

    #[error("internal logic error: {0}")]
    Internal(String),

    #[error("deserialized account contains unexpected values")]
    InvalidAccountData,

    #[error("owner specified doesn't match expected one")]
    OwnerMismatch,

    #[error("limited internal resource (e.g. space in account) was exhausted")]
    ResourceExhausted,

    #[error("requested operation can not be performed due to inappropriate state")]
    OperationCanNotBePerformed,

    // NaN
    #[error("spl-token error: {0}")]
    SplToken(#[from] RemoteError<TokenError>),

    #[error("system program error: {0}")]
    SystemProgram(#[from] RemoteError<SystemError>),

    #[error("accounts count doesn't match expected value")]
    InvalidAccountsCount,

    #[error("accounts list is in the wrong order")]
    InvalidAccountsOrder,

    #[error("superlendy error: {0}")]
    SuperLendy(#[from] SuperLendyError),

    #[error("superlendy cpi error: {0}")]
    SuperLendyCpi(ProgramError),

    #[error("spl-token unpack error: {0}")]
    SplTokenUnpack(ProgramError),

    #[error("rent unpack error: {0}")]
    RentUnpack(ProgramError),

    #[error("spl-token error: {0}")]
    SplTokenErr(ProgramError),

    #[error("invalid seeds for token account")]
    InvalidSeeds,

    #[error("clock error: {0}")]
    Clock(ProgramError),

    #[error("position was not refreshed")]
    StalePosition,

    #[error("position is not empty, VLP amount: {0}")]
    NonEmptyPosition(Decimal),

    #[error("unclaimed reward exists. idx {idx} amount {accrued_amount}")]
    UnclaimedRewards { idx: usize, accrued_amount: Decimal },

    #[error("impossible to manage vault balance: {0}")]
    ImpossibleToBalance(#[from] SolverError),

    #[error("balance will get worse: deviation before – {deviation_before}, deviation after - {deviation_after}")]
    WorsenBalance {
        deviation_before: Decimal,
        deviation_after: Decimal,
    },

    #[error("impossible to improve vault balance")]
    ImpossibleToImproveBalance,

    #[error("impossible to withdraw requested amount of liquidity because of underlying reserves utilization")]
    NotEnoguhLiquidity,

    #[error("trying to claim fees too early, please wait another {0} seconds")]
    EearlyFeesClaim(u64),
}

texture_common::from_account_parse_error!(VaultError);

impl From<VaultError> for ProgramError {
    fn from(error: VaultError) -> Self {
        match error {
            VaultError::SplToken(err) => err.into(),
            VaultError::SystemProgram(RemoteError::Unrecognized(err)) => err,
            VaultError::SystemProgram(RemoteError::Recognized(err)) => Custom(err as u32),
            other => Custom(VaultErrorDiscriminants::from(other) as u32),
        }
    }
}

texture_common::convert_remote_err!(
    system_err,
    texture_common::remote::system::SystemError,
    VaultError
);

texture_common::convert_remote_err!(
    token_err,
    texture_common::remote::token::TokenError,
    VaultError
);
