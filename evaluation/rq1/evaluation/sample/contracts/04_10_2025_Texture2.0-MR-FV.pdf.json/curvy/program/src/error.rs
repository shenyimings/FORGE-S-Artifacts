use borsh::io::Error as BorshIoError;
use bytemuck::PodCastError;
use solana_program::program_error::ProgramError;
use solana_program::program_error::ProgramError::Custom;
use solana_program::pubkey::{Pubkey, PubkeyError};
use solana_program::system_instruction::SystemError;
use thiserror::Error;

use texture_common::account;
use texture_common::error;
use texture_common::math::MathError;
use texture_common::remote::RemoteError;

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

#[derive(Debug, Error)]
pub enum CurvyError {
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

    #[error("curve parameters provided are not valid")]
    InvalidParams,

    // NaN
    #[error("system program error: {0}")]
    SystemProgram(#[from] RemoteError<SystemError>),
}

texture_common::from_account_parse_error!(CurvyError);

impl From<CurvyError> for ProgramError {
    fn from(error: CurvyError) -> Self {
        match error {
            CurvyError::MathError(..) => Custom(3),
            CurvyError::Borsh(..) => Custom(4),
            CurvyError::Serialize(..) => Custom(5),
            CurvyError::PodAccount(..) | CurvyError::PodAccountExt(..) => Custom(6),
            CurvyError::InvalidKey { .. } => Custom(8),
            CurvyError::InvalidAccount(..) => Custom(9),
            CurvyError::NotEnoughAccountKeys(..) => Custom(10),
            CurvyError::MissingSignature(..) => Custom(11),
            CurvyError::Unimplemented => Custom(12),
            CurvyError::UninitializedAccount(..) => Custom(13),
            CurvyError::AddressCreation(..) => Custom(14),
            CurvyError::AccountUnpackError(..) => Custom(15),
            CurvyError::Internal(..) => Custom(23),
            CurvyError::InvalidAccountData => Custom(24),
            CurvyError::OperationCanNotBePerformed => Custom(25),
            CurvyError::InvalidRealloc => Custom(27),
            CurvyError::OwnerMismatch => Custom(28),
            CurvyError::InvalidParams => Custom(29),

            CurvyError::SystemProgram(RemoteError::Unrecognized(err)) => err,
            CurvyError::SystemProgram(RemoteError::Recognized(err)) => Custom(err as u32),
        }
    }
}

texture_common::convert_remote_err!(
    system_err,
    texture_common::remote::system::SystemError,
    CurvyError
);
