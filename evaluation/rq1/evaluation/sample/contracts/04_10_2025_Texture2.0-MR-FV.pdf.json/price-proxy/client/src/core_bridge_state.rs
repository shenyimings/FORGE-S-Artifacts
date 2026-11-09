use anchor_lang::prelude::*;

/// Encoded VAA's processing status.
#[derive(
    Default, Copy, Debug, AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace,
)]
pub enum ProcessingStatus {
    /// `EncodedVaa` account is uninitialized.
    #[default]
    Unset,
    /// VAA is still being written to the `EncodedVaa` account.
    Writing,
    /// VAA is verified (i.e. validating message attestation is complete).
    Verified,
}

/// `EncodedVaa` account header.
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub struct Header {
    /// Processing status. **This encoded VAA is only considered usable when this status is set
    /// to [Verified](ProcessingStatus::Verified).**
    pub status: ProcessingStatus,
    /// The authority that has write privilege to this account.
    pub write_authority: Pubkey,
    /// VAA version. Only when the VAA is verified is this version set to a value.
    pub version: u8,
}
