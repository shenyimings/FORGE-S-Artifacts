use borsh::io::Error as BorshIoError;
use bytemuck::PodCastError;
use solana_program::program_error::ProgramError;
use solana_program::program_error::ProgramError::Custom;
use solana_program::pubkey::{Pubkey, PubkeyError};
use solana_program::system_instruction::SystemError;
use strum::EnumDiscriminants;
use thiserror::Error;

use texture_common::account;
use texture_common::error;
use texture_common::math::MathError;
use texture_common::remote::RemoteError;

use crate::state::price_feed::PriceFeedSource;

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
pub enum PriceProxyError {
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

    #[error("internal logic error: {0}")]
    Internal(String),

    #[error("deserialized account contains unexpected values")]
    InvalidAccountData,

    #[error("requested operation can not be performed due to inappropriate state")]
    OperationCanNotBePerformed,

    #[error("invalid realloc")]
    InvalidRealloc,

    #[error("owner specified doesn't match expected one")]
    OwnerMismatch,

    #[error("mint specified doesn't match expected one")]
    MintMismatch,

    #[error("invalid source '{current}', expected '{expected}'")]
    InvalidSource {
        current: PriceFeedSource,
        expected: PriceFeedSource,
    },

    #[error("timestamp is not recent")]
    TimestampIsNotRecent,

    #[error("not enough balance to perform requested operation")]
    NotEnoughBalance,

    #[error("feed has not been updated in {0} seconds")]
    StaleFeed(u64),

    #[error("invalid price or exponent")]
    InvalidPriceOrExpo,

    // NaN
    #[error("system program error: {0}")]
    SystemProgram(#[from] RemoteError<SystemError>),
}

texture_common::from_account_parse_error!(PriceProxyError);

impl From<PriceProxyError> for ProgramError {
    fn from(error: PriceProxyError) -> Self {
        match error {
            PriceProxyError::SystemProgram(RemoteError::Unrecognized(err)) => err,
            PriceProxyError::SystemProgram(RemoteError::Recognized(err)) => Custom(err as u32),
            err => Custom(PriceProxyErrorDiscriminants::from(err) as u32),
        }
    }
}

texture_common::convert_remote_err!(
    system_err,
    texture_common::remote::system::SystemError,
    PriceProxyError
);
