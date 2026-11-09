macro_rules! gen_crate_docs {
    () => {
        concat!(" ", "Wormhole Core Bridge Solana", " v", "0.0.1-alpha.5",
        " program interface generated from Anchor IDL.")
    };
}
pub(crate) use gen_crate_docs;
pub use anchor_interface::prelude::*;
pub mod instruction {
    #[allow(unused_imports)]
    use super::types::*;
    #[derive(Debug)]
    pub enum WormholeCoreBridgeSolanaInstruction {
        #[doc = concat!(
            " ",
            "Processor for initializing a new draft [PostedMessageV1](crate::state::PostedMessageV1)"
        )]
        #[doc = concat!(
            " ",
            "account for writing. The emitter authority is established at this point and the payload size"
        )]
        #[doc = concat!(
            " ",
            "is inferred from the size of the created account. This instruction handler also allows an"
        )]
        #[doc = concat!(
            " ",
            "integrator to publish Wormhole messages using his program's ID as the emitter address"
        )]
        #[doc = concat!(
            " ",
            "(by passing `Some(crate::ID)` to the [cpi_program_id](InitMessageV1Args::cpi_program_id)"
        )]
        #[doc = concat!(
            " ",
            "argument). **Be aware that the emitter authority's seeds must only be \\[b\"emitter\"\\] in this"
        )]
        #[doc = concat!(" ", "case.**")]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ",
            "This instruction should be followed up with `write_message_v1` and `finalize_message_v1` to"
        )]
        #[doc = concat!(
            " ",
            "write and indicate that the message is ready for publishing respectively (to prepare it for"
        )]
        #[doc = concat!(" ", "publishing via the")]
        #[doc = concat!(
            " ",
            "[post message instruction](crate::legacy::instruction::LegacyInstruction::PostMessage))."
        )]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ",
            "NOTE: If you wish to publish a small message (one where the data does not overflow the"
        )]
        #[doc = concat!(
            " ",
            "Solana transaction size), it is recommended that you use an [sdk](crate::sdk::cpi) method to"
        )]
        #[doc = concat!(
            " ", "either prepare your message or post a message as a program ID emitter."
        )]
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer]` emitter authority
        /// 1. `[writable]` draft message
        InitMessageV1 { args: InitMessageV1Args },
        #[doc = concat!(
            " ",
            "Processor used to write to a draft [PostedMessageV1](crate::state::PostedMessageV1) account."
        )]
        #[doc = concat!(
            " ",
            "This instruction requires an authority (the emitter authority) to interact with the message"
        )]
        #[doc = concat!(" ", "account.")]
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer]` emitter authority
        /// 1. `[writable]` draft message
        WriteMessageV1 { args: WriteMessageV1Args },
        #[doc = concat!(
            " ",
            "Processor used to finalize a draft [PostedMessageV1](crate::state::PostedMessageV1) account."
        )]
        #[doc = concat!(
            " ",
            "Once finalized, this message account cannot be written to again. A finalized message is the"
        )]
        #[doc = concat!(
            " ",
            "only state the legacy post message instruction can accept before publishing. This"
        )]
        #[doc = concat!(
            " ",
            "instruction requires an authority (the emitter authority) to interact with the message"
        )]
        #[doc = concat!(" ", "account.")]
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer]` emitter authority
        /// 1. `[writable]` draft message
        FinalizeMessageV1,
        #[doc = concat!(
            " ",
            "Processor used to process a draft [PostedMessageV1](crate::state::PostedMessageV1) account."
        )]
        #[doc = concat!(
            " ",
            "This instruction requires an authority (the emitter authority) to interact with the message"
        )]
        #[doc = concat!(" ", "account.")]
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer]` emitter authority
        /// 1. `[writable]` draft message
        /// 2. `[writable]` close account destination
        CloseMessageV1,
        #[doc = concat!(
            " ",
            "Processor used to intialize a created account as [EncodedVaa](crate::state::EncodedVaa). An"
        )]
        #[doc = concat!(
            " ", "authority (the write authority) is established with this instruction."
        )]
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer]` write authority
        /// 1. `[writable]` encoded vaa
        InitEncodedVaa,
        #[doc = concat!(
            " ",
            "Processor used to close an [EncodedVaa](crate::state::EncodedVaa). This instruction requires"
        )]
        #[doc = concat!(
            " ",
            "an authority (the write authority) to interact witht he encoded VAA account."
        )]
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer, writable]` write authority
        /// 1. `[writable]` encoded vaa
        CloseEncodedVaa,
        #[doc = concat!(
            " ",
            "Processor used to write to an [EncodedVaa](crate::state::EncodedVaa) account. This"
        )]
        #[doc = concat!(
            " ",
            "instruction requires an authority (the write authority) to interact with the encoded VAA"
        )]
        #[doc = concat!(" ", "account.")]
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer]` write authority
        /// 1. `[writable]` draft vaa
        WriteEncodedVaa { args: WriteEncodedVaaArgs },
        #[doc = concat!(
            " ",
            "Processor used to verify an [EncodedVaa](crate::state::EncodedVaa) account as a version 1"
        )]
        #[doc = concat!(
            " ",
            "VAA (guardian signatures attesting to this observation). This instruction requires an"
        )]
        #[doc = concat!(
            " ",
            "authority (the write authority) to interact with the encoded VAA account."
        )]
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer]` write authority
        /// 1. `[writable]` draft vaa
        /// 2. `[]` guardian set
        VerifyEncodedVaaV1,
        #[doc = concat!(
            " ",
            "Processor used to close an [EncodedVaa](crate::state::EncodedVaa) account to create a"
        )]
        #[doc = concat!(
            " ", "[PostedMessageV1](crate::state::PostedMessageV1) account in its place."
        )]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ",
            "NOTE: Because the legacy verify signatures instruction was not required for the Posted VAA"
        )]
        #[doc = concat!(
            " ",
            "account to exist, the encoded [SignatureSet](crate::state::SignatureSet) is the default"
        )]
        #[doc = concat!(" ", "[Pubkey].")]
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer, writable]` payer
        /// 1. `[]` encoded vaa
        /// 2. `[writable]` posted vaa
        /// 3. `[]` system program
        PostVaaV1,
        #[doc = concat!(
            " ",
            "Processor used to close a [SignatureSet](crate::state::SignatureSet), which was used to"
        )]
        #[doc = concat!(
            " ", "verify the VAA using the legacy parse and verify procedure."
        )]
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer, writable]` sol destination
        /// 1. `[]` posted vaa
        /// 2. `[writable]` signature set
        CloseSignatureSet,
    }
    impl WormholeCoreBridgeSolanaInstruction {
        pub fn sighash(&self) -> &'static [u8; 8] {
            match self {
                Self::InitMessageV1 { .. } => {
                    &[247u8, 187u8, 26u8, 16u8, 122u8, 198u8, 106u8, 247u8]
                }
                Self::WriteMessageV1 { .. } => {
                    &[35u8, 67u8, 197u8, 233u8, 94u8, 117u8, 124u8, 143u8]
                }
                Self::FinalizeMessageV1 => {
                    &[245u8, 208u8, 215u8, 228u8, 129u8, 56u8, 51u8, 251u8]
                }
                Self::CloseMessageV1 => {
                    &[36u8, 185u8, 40u8, 107u8, 239u8, 13u8, 51u8, 162u8]
                }
                Self::InitEncodedVaa => {
                    &[209u8, 193u8, 173u8, 25u8, 91u8, 202u8, 181u8, 218u8]
                }
                Self::CloseEncodedVaa => {
                    &[48u8, 221u8, 174u8, 198u8, 231u8, 7u8, 152u8, 38u8]
                }
                Self::WriteEncodedVaa { .. } => {
                    &[199u8, 208u8, 110u8, 177u8, 150u8, 76u8, 118u8, 42u8]
                }
                Self::VerifyEncodedVaaV1 => {
                    &[103u8, 56u8, 177u8, 229u8, 240u8, 103u8, 68u8, 73u8]
                }
                Self::PostVaaV1 => &[0u8, 57u8, 97u8, 3u8, 225u8, 37u8, 254u8, 31u8],
                Self::CloseSignatureSet => {
                    &[64u8, 154u8, 185u8, 168u8, 234u8, 229u8, 218u8, 103u8]
                }
            }
        }
        pub fn pack(self) -> Vec<u8> {
            use ::borsh::BorshSerialize;
            let mut out = Vec::new();
            out.extend(self.sighash());
            let data = self.try_to_vec().unwrap();
            out.extend(data);
            out
        }
        pub fn unpack(data: &[u8]) -> ::std::io::Result<Self> {
            use ::borsh::BorshDeserialize;
            let (sighash, ix_data) = data.split_at(8);
            Ok(
                match sighash {
                    [247u8, 187u8, 26u8, 16u8, 122u8, 198u8, 106u8, 247u8] => {
                        InitMessageV1Deserializer::try_from_slice(ix_data)?.into()
                    }
                    [35u8, 67u8, 197u8, 233u8, 94u8, 117u8, 124u8, 143u8] => {
                        WriteMessageV1Deserializer::try_from_slice(ix_data)?.into()
                    }
                    [245u8, 208u8, 215u8, 228u8, 129u8, 56u8, 51u8, 251u8] => {
                        FinalizeMessageV1Deserializer::try_from_slice(ix_data)?.into()
                    }
                    [36u8, 185u8, 40u8, 107u8, 239u8, 13u8, 51u8, 162u8] => {
                        CloseMessageV1Deserializer::try_from_slice(ix_data)?.into()
                    }
                    [209u8, 193u8, 173u8, 25u8, 91u8, 202u8, 181u8, 218u8] => {
                        InitEncodedVaaDeserializer::try_from_slice(ix_data)?.into()
                    }
                    [48u8, 221u8, 174u8, 198u8, 231u8, 7u8, 152u8, 38u8] => {
                        CloseEncodedVaaDeserializer::try_from_slice(ix_data)?.into()
                    }
                    [199u8, 208u8, 110u8, 177u8, 150u8, 76u8, 118u8, 42u8] => {
                        WriteEncodedVaaDeserializer::try_from_slice(ix_data)?.into()
                    }
                    [103u8, 56u8, 177u8, 229u8, 240u8, 103u8, 68u8, 73u8] => {
                        VerifyEncodedVaaV1Deserializer::try_from_slice(ix_data)?.into()
                    }
                    [0u8, 57u8, 97u8, 3u8, 225u8, 37u8, 254u8, 31u8] => {
                        PostVaaV1Deserializer::try_from_slice(ix_data)?.into()
                    }
                    [64u8, 154u8, 185u8, 168u8, 234u8, 229u8, 218u8, 103u8] => {
                        CloseSignatureSetDeserializer::try_from_slice(ix_data)?.into()
                    }
                    _ => {
                        return Err(
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "invalid sighash",
                            ),
                        );
                    }
                },
            )
        }
    }
    impl ::borsh::BorshSerialize for WormholeCoreBridgeSolanaInstruction {
        fn serialize<W: ::borsh::maybestd::io::Write>(
            &self,
            writer: &mut W,
        ) -> ::core::result::Result<(), ::borsh::maybestd::io::Error> {
            match self {
                Self::InitMessageV1 { args } => {
                    ::borsh::BorshSerialize::serialize(args, writer)?;
                }
                Self::WriteMessageV1 { args } => {
                    ::borsh::BorshSerialize::serialize(args, writer)?;
                }
                Self::FinalizeMessageV1 => {}
                Self::CloseMessageV1 => {}
                Self::InitEncodedVaa => {}
                Self::CloseEncodedVaa => {}
                Self::WriteEncodedVaa { args } => {
                    ::borsh::BorshSerialize::serialize(args, writer)?;
                }
                Self::VerifyEncodedVaaV1 => {}
                Self::PostVaaV1 => {}
                Self::CloseSignatureSet => {}
            }
            Ok(())
        }
    }
    struct InitMessageV1Deserializer(WormholeCoreBridgeSolanaInstruction);
    impl ::borsh::de::BorshDeserialize for InitMessageV1Deserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(WormholeCoreBridgeSolanaInstruction::InitMessageV1 {
                    args: ::borsh::BorshDeserialize::deserialize(_buf)?,
                }),
            )
        }
    }
    impl From<InitMessageV1Deserializer> for WormholeCoreBridgeSolanaInstruction {
        fn from(
            helper: InitMessageV1Deserializer,
        ) -> WormholeCoreBridgeSolanaInstruction {
            helper.0
        }
    }
    struct WriteMessageV1Deserializer(WormholeCoreBridgeSolanaInstruction);
    impl ::borsh::de::BorshDeserialize for WriteMessageV1Deserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(WormholeCoreBridgeSolanaInstruction::WriteMessageV1 {
                    args: ::borsh::BorshDeserialize::deserialize(_buf)?,
                }),
            )
        }
    }
    impl From<WriteMessageV1Deserializer> for WormholeCoreBridgeSolanaInstruction {
        fn from(
            helper: WriteMessageV1Deserializer,
        ) -> WormholeCoreBridgeSolanaInstruction {
            helper.0
        }
    }
    struct FinalizeMessageV1Deserializer(WormholeCoreBridgeSolanaInstruction);
    impl ::borsh::de::BorshDeserialize for FinalizeMessageV1Deserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(WormholeCoreBridgeSolanaInstruction::FinalizeMessageV1 {
                }),
            )
        }
    }
    impl From<FinalizeMessageV1Deserializer> for WormholeCoreBridgeSolanaInstruction {
        fn from(
            helper: FinalizeMessageV1Deserializer,
        ) -> WormholeCoreBridgeSolanaInstruction {
            helper.0
        }
    }
    struct CloseMessageV1Deserializer(WormholeCoreBridgeSolanaInstruction);
    impl ::borsh::de::BorshDeserialize for CloseMessageV1Deserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(WormholeCoreBridgeSolanaInstruction::CloseMessageV1 {
                }),
            )
        }
    }
    impl From<CloseMessageV1Deserializer> for WormholeCoreBridgeSolanaInstruction {
        fn from(
            helper: CloseMessageV1Deserializer,
        ) -> WormholeCoreBridgeSolanaInstruction {
            helper.0
        }
    }
    struct InitEncodedVaaDeserializer(WormholeCoreBridgeSolanaInstruction);
    impl ::borsh::de::BorshDeserialize for InitEncodedVaaDeserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(WormholeCoreBridgeSolanaInstruction::InitEncodedVaa {
                }),
            )
        }
    }
    impl From<InitEncodedVaaDeserializer> for WormholeCoreBridgeSolanaInstruction {
        fn from(
            helper: InitEncodedVaaDeserializer,
        ) -> WormholeCoreBridgeSolanaInstruction {
            helper.0
        }
    }
    struct CloseEncodedVaaDeserializer(WormholeCoreBridgeSolanaInstruction);
    impl ::borsh::de::BorshDeserialize for CloseEncodedVaaDeserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(WormholeCoreBridgeSolanaInstruction::CloseEncodedVaa {
                }),
            )
        }
    }
    impl From<CloseEncodedVaaDeserializer> for WormholeCoreBridgeSolanaInstruction {
        fn from(
            helper: CloseEncodedVaaDeserializer,
        ) -> WormholeCoreBridgeSolanaInstruction {
            helper.0
        }
    }
    struct WriteEncodedVaaDeserializer(WormholeCoreBridgeSolanaInstruction);
    impl ::borsh::de::BorshDeserialize for WriteEncodedVaaDeserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(WormholeCoreBridgeSolanaInstruction::WriteEncodedVaa {
                    args: ::borsh::BorshDeserialize::deserialize(_buf)?,
                }),
            )
        }
    }
    impl From<WriteEncodedVaaDeserializer> for WormholeCoreBridgeSolanaInstruction {
        fn from(
            helper: WriteEncodedVaaDeserializer,
        ) -> WormholeCoreBridgeSolanaInstruction {
            helper.0
        }
    }
    struct VerifyEncodedVaaV1Deserializer(WormholeCoreBridgeSolanaInstruction);
    impl ::borsh::de::BorshDeserialize for VerifyEncodedVaaV1Deserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(WormholeCoreBridgeSolanaInstruction::VerifyEncodedVaaV1 {
                }),
            )
        }
    }
    impl From<VerifyEncodedVaaV1Deserializer> for WormholeCoreBridgeSolanaInstruction {
        fn from(
            helper: VerifyEncodedVaaV1Deserializer,
        ) -> WormholeCoreBridgeSolanaInstruction {
            helper.0
        }
    }
    struct PostVaaV1Deserializer(WormholeCoreBridgeSolanaInstruction);
    impl ::borsh::de::BorshDeserialize for PostVaaV1Deserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(WormholeCoreBridgeSolanaInstruction::PostVaaV1 {
                }),
            )
        }
    }
    impl From<PostVaaV1Deserializer> for WormholeCoreBridgeSolanaInstruction {
        fn from(helper: PostVaaV1Deserializer) -> WormholeCoreBridgeSolanaInstruction {
            helper.0
        }
    }
    struct CloseSignatureSetDeserializer(WormholeCoreBridgeSolanaInstruction);
    impl ::borsh::de::BorshDeserialize for CloseSignatureSetDeserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(WormholeCoreBridgeSolanaInstruction::CloseSignatureSet {
                }),
            )
        }
    }
    impl From<CloseSignatureSetDeserializer> for WormholeCoreBridgeSolanaInstruction {
        fn from(
            helper: CloseSignatureSetDeserializer,
        ) -> WormholeCoreBridgeSolanaInstruction {
            helper.0
        }
    }
    #[derive(Debug)]
    pub struct InitMessageV1 {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub emitter_authority: ::solana_program::pubkey::Pubkey,
        pub draft_message: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
        pub args: InitMessageV1Args,
    }
    impl InitMessageV1 {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self {
                program_id,
                emitter_authority,
                draft_message,
                trailing_accounts,
                args,
            } = self;
            let mut accounts = vec![
                AccountMeta::new_readonly(emitter_authority, true),
                AccountMeta::new(draft_message, false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = WormholeCoreBridgeSolanaInstruction::InitMessageV1 {
                args,
            }
                .pack();
            Instruction {
                program_id,
                data,
                accounts,
            }
        }
    }
    #[derive(Debug)]
    pub struct InitMessageV1AccountIndexes {
        pub emitter_authority: usize,
        pub draft_message: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl InitMessageV1AccountIndexes {
        pub const EMITTER_AUTHORITY: usize = 0usize;
        pub const DRAFT_MESSAGE: usize = 1usize;
    }
    impl<'a> TryFrom<&'a [u8]> for InitMessageV1AccountIndexes {
        type Error = ::anchor_interface::errors::TryAccountIndexesError;
        fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
            let mut iter = indexes.iter().map(|idx| (*idx) as usize);
            Ok(Self {
                emitter_authority: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(emitter_authority),
                            0usize,
                        ),
                    )?,
                draft_message: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(draft_message),
                            1usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct WriteMessageV1 {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub emitter_authority: ::solana_program::pubkey::Pubkey,
        pub draft_message: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
        pub args: WriteMessageV1Args,
    }
    impl WriteMessageV1 {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self {
                program_id,
                emitter_authority,
                draft_message,
                trailing_accounts,
                args,
            } = self;
            let mut accounts = vec![
                AccountMeta::new_readonly(emitter_authority, true),
                AccountMeta::new(draft_message, false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = WormholeCoreBridgeSolanaInstruction::WriteMessageV1 {
                args,
            }
                .pack();
            Instruction {
                program_id,
                data,
                accounts,
            }
        }
    }
    #[derive(Debug)]
    pub struct WriteMessageV1AccountIndexes {
        pub emitter_authority: usize,
        pub draft_message: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl WriteMessageV1AccountIndexes {
        pub const EMITTER_AUTHORITY: usize = 0usize;
        pub const DRAFT_MESSAGE: usize = 1usize;
    }
    impl<'a> TryFrom<&'a [u8]> for WriteMessageV1AccountIndexes {
        type Error = ::anchor_interface::errors::TryAccountIndexesError;
        fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
            let mut iter = indexes.iter().map(|idx| (*idx) as usize);
            Ok(Self {
                emitter_authority: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(emitter_authority),
                            0usize,
                        ),
                    )?,
                draft_message: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(draft_message),
                            1usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct FinalizeMessageV1 {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub emitter_authority: ::solana_program::pubkey::Pubkey,
        pub draft_message: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
    }
    impl FinalizeMessageV1 {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self {
                program_id,
                emitter_authority,
                draft_message,
                trailing_accounts,
            } = self;
            let mut accounts = vec![
                AccountMeta::new_readonly(emitter_authority, true),
                AccountMeta::new(draft_message, false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = WormholeCoreBridgeSolanaInstruction::FinalizeMessageV1 {
            }
                .pack();
            Instruction {
                program_id,
                data,
                accounts,
            }
        }
    }
    #[derive(Debug)]
    pub struct FinalizeMessageV1AccountIndexes {
        pub emitter_authority: usize,
        pub draft_message: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl FinalizeMessageV1AccountIndexes {
        pub const EMITTER_AUTHORITY: usize = 0usize;
        pub const DRAFT_MESSAGE: usize = 1usize;
    }
    impl<'a> TryFrom<&'a [u8]> for FinalizeMessageV1AccountIndexes {
        type Error = ::anchor_interface::errors::TryAccountIndexesError;
        fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
            let mut iter = indexes.iter().map(|idx| (*idx) as usize);
            Ok(Self {
                emitter_authority: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(emitter_authority),
                            0usize,
                        ),
                    )?,
                draft_message: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(draft_message),
                            1usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct CloseMessageV1 {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub emitter_authority: ::solana_program::pubkey::Pubkey,
        pub draft_message: ::solana_program::pubkey::Pubkey,
        pub close_account_destination: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
    }
    impl CloseMessageV1 {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self {
                program_id,
                emitter_authority,
                draft_message,
                close_account_destination,
                trailing_accounts,
            } = self;
            let mut accounts = vec![
                AccountMeta::new_readonly(emitter_authority, true),
                AccountMeta::new(draft_message, false),
                AccountMeta::new(close_account_destination, false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = WormholeCoreBridgeSolanaInstruction::CloseMessageV1 {
            }
                .pack();
            Instruction {
                program_id,
                data,
                accounts,
            }
        }
    }
    #[derive(Debug)]
    pub struct CloseMessageV1AccountIndexes {
        pub emitter_authority: usize,
        pub draft_message: usize,
        pub close_account_destination: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl CloseMessageV1AccountIndexes {
        pub const EMITTER_AUTHORITY: usize = 0usize;
        pub const DRAFT_MESSAGE: usize = 1usize;
        pub const CLOSE_ACCOUNT_DESTINATION: usize = 2usize;
    }
    impl<'a> TryFrom<&'a [u8]> for CloseMessageV1AccountIndexes {
        type Error = ::anchor_interface::errors::TryAccountIndexesError;
        fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
            let mut iter = indexes.iter().map(|idx| (*idx) as usize);
            Ok(Self {
                emitter_authority: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(emitter_authority),
                            0usize,
                        ),
                    )?,
                draft_message: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(draft_message),
                            1usize,
                        ),
                    )?,
                close_account_destination: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(close_account_destination),
                            2usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct InitEncodedVaa {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub write_authority: ::solana_program::pubkey::Pubkey,
        pub encoded_vaa: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
    }
    impl InitEncodedVaa {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self { program_id, write_authority, encoded_vaa, trailing_accounts } = self;
            let mut accounts = vec![
                AccountMeta::new_readonly(write_authority, true),
                AccountMeta::new(encoded_vaa, false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = WormholeCoreBridgeSolanaInstruction::InitEncodedVaa {
            }
                .pack();
            Instruction {
                program_id,
                data,
                accounts,
            }
        }
    }
    #[derive(Debug)]
    pub struct InitEncodedVaaAccountIndexes {
        pub write_authority: usize,
        pub encoded_vaa: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl InitEncodedVaaAccountIndexes {
        pub const WRITE_AUTHORITY: usize = 0usize;
        pub const ENCODED_VAA: usize = 1usize;
    }
    impl<'a> TryFrom<&'a [u8]> for InitEncodedVaaAccountIndexes {
        type Error = ::anchor_interface::errors::TryAccountIndexesError;
        fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
            let mut iter = indexes.iter().map(|idx| (*idx) as usize);
            Ok(Self {
                write_authority: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(write_authority),
                            0usize,
                        ),
                    )?,
                encoded_vaa: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(encoded_vaa),
                            1usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct CloseEncodedVaa {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub write_authority: ::solana_program::pubkey::Pubkey,
        pub encoded_vaa: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
    }
    impl CloseEncodedVaa {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self { program_id, write_authority, encoded_vaa, trailing_accounts } = self;
            let mut accounts = vec![
                AccountMeta::new(write_authority, true), AccountMeta::new(encoded_vaa,
                false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = WormholeCoreBridgeSolanaInstruction::CloseEncodedVaa {
            }
                .pack();
            Instruction {
                program_id,
                data,
                accounts,
            }
        }
    }
    #[derive(Debug)]
    pub struct CloseEncodedVaaAccountIndexes {
        pub write_authority: usize,
        pub encoded_vaa: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl CloseEncodedVaaAccountIndexes {
        pub const WRITE_AUTHORITY: usize = 0usize;
        pub const ENCODED_VAA: usize = 1usize;
    }
    impl<'a> TryFrom<&'a [u8]> for CloseEncodedVaaAccountIndexes {
        type Error = ::anchor_interface::errors::TryAccountIndexesError;
        fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
            let mut iter = indexes.iter().map(|idx| (*idx) as usize);
            Ok(Self {
                write_authority: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(write_authority),
                            0usize,
                        ),
                    )?,
                encoded_vaa: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(encoded_vaa),
                            1usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct WriteEncodedVaa {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub write_authority: ::solana_program::pubkey::Pubkey,
        pub draft_vaa: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
        pub args: WriteEncodedVaaArgs,
    }
    impl WriteEncodedVaa {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self {
                program_id,
                write_authority,
                draft_vaa,
                trailing_accounts,
                args,
            } = self;
            let mut accounts = vec![
                AccountMeta::new_readonly(write_authority, true),
                AccountMeta::new(draft_vaa, false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = WormholeCoreBridgeSolanaInstruction::WriteEncodedVaa {
                args,
            }
                .pack();
            Instruction {
                program_id,
                data,
                accounts,
            }
        }
    }
    #[derive(Debug)]
    pub struct WriteEncodedVaaAccountIndexes {
        pub write_authority: usize,
        pub draft_vaa: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl WriteEncodedVaaAccountIndexes {
        pub const WRITE_AUTHORITY: usize = 0usize;
        pub const DRAFT_VAA: usize = 1usize;
    }
    impl<'a> TryFrom<&'a [u8]> for WriteEncodedVaaAccountIndexes {
        type Error = ::anchor_interface::errors::TryAccountIndexesError;
        fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
            let mut iter = indexes.iter().map(|idx| (*idx) as usize);
            Ok(Self {
                write_authority: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(write_authority),
                            0usize,
                        ),
                    )?,
                draft_vaa: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(draft_vaa),
                            1usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct VerifyEncodedVaaV1 {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub write_authority: ::solana_program::pubkey::Pubkey,
        pub draft_vaa: ::solana_program::pubkey::Pubkey,
        pub guardian_set: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
    }
    impl VerifyEncodedVaaV1 {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self {
                program_id,
                write_authority,
                draft_vaa,
                guardian_set,
                trailing_accounts,
            } = self;
            let mut accounts = vec![
                AccountMeta::new_readonly(write_authority, true),
                AccountMeta::new(draft_vaa, false),
                AccountMeta::new_readonly(guardian_set, false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = WormholeCoreBridgeSolanaInstruction::VerifyEncodedVaaV1 {
            }
                .pack();
            Instruction {
                program_id,
                data,
                accounts,
            }
        }
    }
    #[derive(Debug)]
    pub struct VerifyEncodedVaaV1AccountIndexes {
        pub write_authority: usize,
        pub draft_vaa: usize,
        pub guardian_set: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl VerifyEncodedVaaV1AccountIndexes {
        pub const WRITE_AUTHORITY: usize = 0usize;
        pub const DRAFT_VAA: usize = 1usize;
        pub const GUARDIAN_SET: usize = 2usize;
    }
    impl<'a> TryFrom<&'a [u8]> for VerifyEncodedVaaV1AccountIndexes {
        type Error = ::anchor_interface::errors::TryAccountIndexesError;
        fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
            let mut iter = indexes.iter().map(|idx| (*idx) as usize);
            Ok(Self {
                write_authority: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(write_authority),
                            0usize,
                        ),
                    )?,
                draft_vaa: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(draft_vaa),
                            1usize,
                        ),
                    )?,
                guardian_set: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(guardian_set),
                            2usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct PostVaaV1 {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub payer: ::solana_program::pubkey::Pubkey,
        pub encoded_vaa: ::solana_program::pubkey::Pubkey,
        pub posted_vaa: ::solana_program::pubkey::Pubkey,
        pub system_program: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
    }
    impl PostVaaV1 {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self {
                program_id,
                payer,
                encoded_vaa,
                posted_vaa,
                system_program,
                trailing_accounts,
            } = self;
            let mut accounts = vec![
                AccountMeta::new(payer, true), AccountMeta::new_readonly(encoded_vaa,
                false), AccountMeta::new(posted_vaa, false),
                AccountMeta::new_readonly(system_program, false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = WormholeCoreBridgeSolanaInstruction::PostVaaV1 {
            }
                .pack();
            Instruction {
                program_id,
                data,
                accounts,
            }
        }
    }
    #[derive(Debug)]
    pub struct PostVaaV1AccountIndexes {
        pub payer: usize,
        pub encoded_vaa: usize,
        pub posted_vaa: usize,
        pub system_program: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl PostVaaV1AccountIndexes {
        pub const PAYER: usize = 0usize;
        pub const ENCODED_VAA: usize = 1usize;
        pub const POSTED_VAA: usize = 2usize;
        pub const SYSTEM_PROGRAM: usize = 3usize;
    }
    impl<'a> TryFrom<&'a [u8]> for PostVaaV1AccountIndexes {
        type Error = ::anchor_interface::errors::TryAccountIndexesError;
        fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
            let mut iter = indexes.iter().map(|idx| (*idx) as usize);
            Ok(Self {
                payer: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(payer),
                            0usize,
                        ),
                    )?,
                encoded_vaa: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(encoded_vaa),
                            1usize,
                        ),
                    )?,
                posted_vaa: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(posted_vaa),
                            2usize,
                        ),
                    )?,
                system_program: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(system_program),
                            3usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct CloseSignatureSet {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub sol_destination: ::solana_program::pubkey::Pubkey,
        pub posted_vaa: ::solana_program::pubkey::Pubkey,
        pub signature_set: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
    }
    impl CloseSignatureSet {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self {
                program_id,
                sol_destination,
                posted_vaa,
                signature_set,
                trailing_accounts,
            } = self;
            let mut accounts = vec![
                AccountMeta::new(sol_destination, true),
                AccountMeta::new_readonly(posted_vaa, false),
                AccountMeta::new(signature_set, false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = WormholeCoreBridgeSolanaInstruction::CloseSignatureSet {
            }
                .pack();
            Instruction {
                program_id,
                data,
                accounts,
            }
        }
    }
    #[derive(Debug)]
    pub struct CloseSignatureSetAccountIndexes {
        pub sol_destination: usize,
        pub posted_vaa: usize,
        pub signature_set: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl CloseSignatureSetAccountIndexes {
        pub const SOL_DESTINATION: usize = 0usize;
        pub const POSTED_VAA: usize = 1usize;
        pub const SIGNATURE_SET: usize = 2usize;
    }
    impl<'a> TryFrom<&'a [u8]> for CloseSignatureSetAccountIndexes {
        type Error = ::anchor_interface::errors::TryAccountIndexesError;
        fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
            let mut iter = indexes.iter().map(|idx| (*idx) as usize);
            Ok(Self {
                sol_destination: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(sol_destination),
                            0usize,
                        ),
                    )?,
                posted_vaa: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(posted_vaa),
                            1usize,
                        ),
                    )?,
                signature_set: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(signature_set),
                            2usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
}
pub mod types {
    #[doc = concat!(" ", "Arguments used to initialize the Core Bridge program.")]
    #[derive(Clone, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct InitializeArgs {
        pub guardian_set_ttl_seconds: u32,
        pub fee_lamports: u64,
        pub initial_guardians: Vec<[u8; 20usize]>,
    }
    #[doc = concat!(
        " ", "Arguments used to post a new Wormhole (Core Bridge) message either using"
    )]
    #[doc = concat!(" ", "[post_message](crate::legacy::instruction::post_message) or")]
    #[doc = concat!(
        " ",
        "[post_message_unreliable](crate::legacy::instruction::post_message_unreliable)."
    )]
    #[derive(Clone, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct PostMessageArgs {
        #[doc = concat!(" ", "Unique id for this message.")]
        pub nonce: u32,
        #[doc = concat!(" ", "Encoded message.")]
        pub payload: Vec<u8>,
        #[doc = concat!(" ", "Solana commitment level for Guardian observation.")]
        pub commitment: Commitment,
    }
    #[doc = concat!(" ", "Arguments to post new VAA data after signature verification.")]
    #[doc = concat!(" ", "")]
    #[doc = concat!(
        " ",
        "NOTE: It is preferred to use the new process of verifying a VAA using the new Core Bridge Anchor"
    )]
    #[doc = concat!(
        " ",
        "instructions. See [init_encoded_vaa](crate::wormhole_core_bridge_solana::init_encoded_vaa) and"
    )]
    #[doc = concat!(
        " ",
        "[write_encoded_vaa](crate::wormhole_core_bridge_solana::write_encoded_vaa) for more info."
    )]
    #[derive(Clone, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct PostVaaArgs {
        #[doc = concat!(" ", "Unused data.")]
        pub gap0: [u8; 5usize],
        #[doc = concat!(" ", "Time the message was submitted.")]
        pub timestamp: u32,
        #[doc = concat!(" ", "Unique ID for this message.")]
        pub nonce: u32,
        #[doc = concat!(
            " ", "The Wormhole chain ID denoting the origin of this message."
        )]
        pub emitter_chain: u16,
        #[doc = concat!(" ", "Emitter of the message.")]
        pub emitter_address: [u8; 32usize],
        #[doc = concat!(" ", "Sequence number of this message.")]
        pub sequence: u64,
        #[doc = concat!(" ", "Level of consistency requested by the emitter.")]
        pub consistency_level: u8,
        #[doc = concat!(" ", "Message payload.")]
        pub payload: Vec<u8>,
    }
    #[doc = concat!(" ", "Arguments to verify specific guardian indices.")]
    #[doc = concat!(" ", "")]
    #[doc = concat!(
        " ",
        "NOTE: It is preferred to use the new process of verifying a VAA using the new Core Bridge Anchor"
    )]
    #[doc = concat!(
        " ",
        "instructions. See [init_encoded_vaa](crate::wormhole_core_bridge_solana::init_encoded_vaa) and"
    )]
    #[doc = concat!(
        " ",
        "[write_encoded_vaa](crate::wormhole_core_bridge_solana::write_encoded_vaa) for more info."
    )]
    #[derive(Clone, Copy, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct VerifySignaturesArgs {
        #[doc = concat!(
            " ",
            "Indices of verified guardian signatures, where -1 indicates a missing value. There is a"
        )]
        #[doc = concat!(
            " ",
            "missing value if the guardian at this index is not expected to have its signature verfied by"
        )]
        #[doc = concat!(
            " ", "the Sig Verify native program in the instruction invoked prior)."
        )]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ",
            "NOTE: In the legacy implementation, this argument being a fixed-sized array of 19 only"
        )]
        #[doc = concat!(
            " ",
            "allows the first 19 guardians of any size guardian set to be verified. Because of this, it"
        )]
        #[doc = concat!(
            " ", "is absolutely important to use the new process of verifying a VAA."
        )]
        pub signer_indices: [i8; 19usize],
    }
    #[doc = concat!(" ", "Unit struct used to represent an empty instruction argument.")]
    #[derive(Clone, Copy, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct EmptyArgs {}
    #[doc = concat!(
        " ",
        "Account used to store the current configuration of the bridge, including tracking Wormhole fee"
    )]
    #[doc = concat!(
        " ",
        "payments. For governance decrees, the guardian set index is used to determine whether a decree"
    )]
    #[doc = concat!(" ", "was attested for using the latest guardian set.")]
    #[derive(Clone, Copy, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct Config {
        #[doc = concat!(
            " ",
            "The current guardian set index, used to decide which signature sets to accept."
        )]
        pub guardian_set_index: u32,
        #[doc = concat!(
            " ",
            "Gap. In the old implementation, this was an amount that kept track of message fees that"
        )]
        #[doc = concat!(" ", "were paid to the program's fee collector.")]
        pub gap0: [u8; 8usize],
        #[doc = concat!(
            " ",
            "Period for how long a guardian set is valid after it has been replaced by a new one.  This"
        )]
        #[doc = concat!(
            " ",
            "guarantees that VAAs issued by that set can still be submitted for a certain period.  In"
        )]
        #[doc = concat!(" ", "this period we still trust the old guardian set.")]
        pub guardian_set_ttl: Duration,
        #[doc = concat!(
            " ",
            "Amount of lamports that needs to be paid to the protocol to post a message"
        )]
        pub fee_lamports: u64,
    }
    #[doc = concat!(
        " ", "Account used to store the current sequence number for a given emitter."
    )]
    #[derive(Clone, Copy, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct LegacyEmitterSequence {
        #[doc = concat!(
            " ",
            "Current sequence number, which will be used the next time this emitter publishes a message."
        )]
        pub value: u64,
    }
    #[derive(Clone, Copy, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct EmitterSequence {
        pub legacy: LegacyEmitterSequence,
        pub bump: u8,
        pub emitter_type: EmitterType,
    }
    #[doc = concat!(
        " ", "Account used to store a published (reusable) Wormhole message."
    )]
    #[derive(Clone, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct PostedMessageV1Unreliable {
        pub data: PostedMessageV1Data,
    }
    #[doc = concat!(
        " ", "Message metadata defining information about a published Wormhole message."
    )]
    #[derive(Clone, Copy, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct PostedMessageV1Info {
        #[doc = concat!(" ", "Level of consistency requested by the emitter.")]
        pub consistency_level: u8,
        #[doc = concat!(
            " ",
            "Authority used to write the message. This field is set to default when the message is"
        )]
        #[doc = concat!(" ", "posted.")]
        pub emitter_authority: ::solana_program::pubkey::Pubkey,
        #[doc = concat!(
            " ",
            "If a message is being written to, this status is used to determine which state this"
        )]
        #[doc = concat!(
            " ",
            "account is in (e.g. [MessageStatus::Writing] indicates that the emitter authority is still"
        )]
        #[doc = concat!(
            " ",
            "writing its message to this account). When this message is posted, this value will be"
        )]
        #[doc = concat!(" ", "set to [MessageStatus::Published].")]
        pub status: MessageStatus,
        #[doc = concat!(" ", "No data is stored here.")]
        pub gap0: [u8; 3usize],
        #[doc = concat!(" ", "Time the posted message was created.")]
        pub posted_timestamp: Timestamp,
        #[doc = concat!(" ", "Unique id for this message.")]
        pub nonce: u32,
        #[doc = concat!(" ", "Sequence number of this message.")]
        pub sequence: u64,
        #[doc = concat!(" ", "Always `1`.")]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ",
            "NOTE: Saving this value is silly, but we are keeping it to be consistent with how the posted"
        )]
        #[doc = concat!(" ", "message account is written.")]
        pub solana_chain_id: ChainIdSolanaOnly,
        #[doc = concat!(
            " ",
            "Emitter of the message. This may either be the emitter authority or a program ID."
        )]
        pub emitter: ::solana_program::pubkey::Pubkey,
    }
    #[doc = concat!(
        " ",
        "Underlying data for either [PostedMessageV1](crate::legacy::state::PostedMessageV1) or"
    )]
    #[doc = concat!(
        " ",
        "[PostedMessageV1Unreliable](crate::legacy::state::PostedMessageV1Unreliable)."
    )]
    #[derive(Clone, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct PostedMessageV1Data {
        #[doc = concat!(" ", "Message metadata.")]
        pub info: PostedMessageV1Info,
        #[doc = concat!(" ", "Encoded message.")]
        pub payload: Vec<u8>,
    }
    #[doc = concat!(" ", "Account used to store a published Wormhole message.")]
    #[doc = concat!(" ", "")]
    #[doc = concat!(
        " ", "NOTE: If your integration requires reusable message accounts, please see"
    )]
    #[doc = concat!(
        " ",
        "[PostedMessageV1Unreliable](crate::legacy::state::PostedMessageV1Unreliable)."
    )]
    #[derive(Clone, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct PostedMessageV1 {
        #[doc = concat!(" ", "Message data.")]
        pub data: PostedMessageV1Data,
    }
    #[doc = concat!(
        " ",
        "VAA metadata defining information about a Wormhole message attested for by an active guardian"
    )]
    #[doc = concat!(" ", "set.")]
    #[derive(Clone, Copy, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct PostedVaaV1Info {
        #[doc = concat!(" ", "Level of consistency requested by the emitter.")]
        pub consistency_level: u8,
        #[doc = concat!(" ", "Time the message was submitted.")]
        pub timestamp: Timestamp,
        #[doc = concat!(
            " ",
            "Pubkey of [SignatureSet](crate::state::SignatureSet) account that represents this VAA's"
        )]
        #[doc = concat!(" ", "signature verification.")]
        pub signature_set: ::solana_program::pubkey::Pubkey,
        #[doc = concat!(
            " ",
            "Guardian set index used to verify signatures for [SignatureSet](crate::state::SignatureSet)."
        )]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ",
            "NOTE: In the previous implementation, this member was referred to as the \"posted timestamp\","
        )]
        #[doc = concat!(
            " ",
            "which is zero for VAA data (posted messages and VAAs resemble the same account schema). By"
        )]
        #[doc = concat!(
            " ",
            "changing this to the guardian set index, we patch a bug with verifying governance VAAs for"
        )]
        #[doc = concat!(
            " ",
            "the Core Bridge (other Core Bridge implementations require that the guardian set that"
        )]
        #[doc = concat!(" ", "attested for the governance VAA is the current one).")]
        pub guardian_set_index: u32,
        #[doc = concat!(" ", "Unique ID for this message.")]
        pub nonce: u32,
        #[doc = concat!(" ", "Sequence number of this message.")]
        pub sequence: u64,
        #[doc = concat!(
            " ", "The Wormhole chain ID denoting the origin of this message."
        )]
        pub emitter_chain: u16,
        #[doc = concat!(" ", "Emitter of the message.")]
        pub emitter_address: [u8; 32usize],
    }
    #[doc = concat!(" ", "Account used to store a verified VAA.")]
    #[derive(Clone, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct PostedVaaV1 {
        #[doc = concat!(" ", "VAA metadata.")]
        pub info: PostedVaaV1Info,
        #[doc = concat!(" ", "Message payload.")]
        pub payload: Vec<u8>,
    }
    #[doc = concat!(
        " ",
        "Arguments for the [write_encoded_vaa](crate::wormhole_core_bridge_solana::write_encoded_vaa)"
    )]
    #[doc = concat!(" ", "instruction.")]
    #[derive(Clone, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct WriteEncodedVaaArgs {
        #[doc = concat!(" ", "Index of VAA buffer.")]
        pub index: u32,
        #[doc = concat!(
            " ", "Data representing subset of VAA buffer starting at specified index."
        )]
        pub data: Vec<u8>,
    }
    #[doc = concat!(
        " ",
        "Arguments for the [init_message_v1](crate::wormhole_core_bridge_solana::init_message_v1)"
    )]
    #[doc = concat!(" ", "instruction.")]
    #[derive(Clone, Copy, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct InitMessageV1Args {
        #[doc = concat!(" ", "Unique id for this message.")]
        pub nonce: u32,
        #[doc = concat!(" ", "Solana commitment level for Guardian observation.")]
        pub commitment: Commitment,
        #[doc = concat!(
            " ", "Optional program ID if the emitter address will be your program ID."
        )]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ",
            "NOTE: If `Some(program_id)`, your emitter authority seeds to be \\[b\"emitter\\]."
        )]
        pub cpi_program_id: Option<::solana_program::pubkey::Pubkey>,
    }
    #[doc = concat!(
        " ",
        "Arguments for the [write_message_v1](crate::wormhole_core_bridge_solana::write_message_v1)"
    )]
    #[doc = concat!(" ", "instruction.")]
    #[derive(Clone, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct WriteMessageV1Args {
        #[doc = concat!(" ", "Index of message buffer.")]
        pub index: u32,
        #[doc = concat!(
            " ",
            "Data representing subset of message buffer starting at specified index."
        )]
        pub data: Vec<u8>,
    }
    #[doc = concat!(" ", "`EncodedVaa` account header.")]
    #[derive(Clone, Copy, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct Header {
        #[doc = concat!(
            " ",
            "Processing status. **This encoded VAA is only considered usable when this status is set"
        )]
        #[doc = concat!(" ", "to [Verified](ProcessingStatus::Verified).**")]
        pub status: ProcessingStatus,
        #[doc = concat!(" ", "The authority that has write privilege to this account.")]
        pub write_authority: ::solana_program::pubkey::Pubkey,
        #[doc = concat!(
            " ",
            "VAA version. Only when the VAA is verified is this version set to a value."
        )]
        pub version: u8,
    }
    #[doc = concat!(
        " ",
        "This struct defines unix timestamp as u32 (as opposed to more modern systems that have adopted"
    )]
    #[doc = concat!(
        " ",
        "i64). Methods for this struct are meant to convert Solana's clock type to this type assuming we"
    )]
    #[doc = concat!(" ", "are far from year 2038.")]
    #[derive(Clone, Copy, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct Timestamp {
        pub value: u32,
    }
    #[doc = concat!(
        " ",
        "To be used with the [Timestamp] type, this struct defines a duration in seconds."
    )]
    #[derive(Clone, Copy, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct Duration {
        pub seconds: u32,
    }
    #[doc = concat!(" ", "This type is used to represent a message hash (keccak).")]
    #[derive(Clone, Copy, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct MessageHash {
        pub bytes: [u8; 32usize],
    }
    #[doc = concat!(
        " ",
        "This type is kind of silly. But because [PostedMessageV1](crate::state::PostedMessageV1) has the"
    )]
    #[doc = concat!(
        " ",
        "emitter chain ID as a field, which is unnecessary since it is always Solana's chain ID, we use"
    )]
    #[doc = concat!(
        " ", "this type to guarantee that the encoded chain ID is always `1`."
    )]
    #[derive(Clone, Copy, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct ChainIdSolanaOnly {
        pub chain_id: u16,
    }
    #[derive(Clone, Copy, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct EmitterInfo {
        pub chain: u16,
        pub address: [u8; 32usize],
        pub sequence: u64,
    }
    #[doc = concat!(" ", "Legacy instruction selector.")]
    #[doc = concat!(" ", "")]
    #[doc = concat!(
        " ",
        "NOTE: No more instructions should be added to this enum. Instead, add them as Anchor instruction"
    )]
    #[doc = concat!(" ", "handlers, which will inevitably live in")]
    #[doc = concat!(
        " ", "[wormhole_core_bridge_solana](crate::wormhole_core_bridge_solana)."
    )]
    #[derive(Clone, Copy, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub enum LegacyInstruction {
        #[doc = concat!(" ", "Legacy instruction selector.")]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ",
            "NOTE: No more instructions should be added to this enum. Instead, add them as Anchor instruction"
        )]
        #[doc = concat!(" ", "handlers, which will inevitably live in")]
        #[doc = concat!(
            " ", "[wormhole_core_bridge_solana](crate::wormhole_core_bridge_solana)."
        )]
        Initialize,
        #[doc = concat!(" ", "Legacy instruction selector.")]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ",
            "NOTE: No more instructions should be added to this enum. Instead, add them as Anchor instruction"
        )]
        #[doc = concat!(" ", "handlers, which will inevitably live in")]
        #[doc = concat!(
            " ", "[wormhole_core_bridge_solana](crate::wormhole_core_bridge_solana)."
        )]
        PostMessage,
        #[doc = concat!(" ", "Legacy instruction selector.")]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ",
            "NOTE: No more instructions should be added to this enum. Instead, add them as Anchor instruction"
        )]
        #[doc = concat!(" ", "handlers, which will inevitably live in")]
        #[doc = concat!(
            " ", "[wormhole_core_bridge_solana](crate::wormhole_core_bridge_solana)."
        )]
        PostVaa,
        #[doc = concat!(" ", "Legacy instruction selector.")]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ",
            "NOTE: No more instructions should be added to this enum. Instead, add them as Anchor instruction"
        )]
        #[doc = concat!(" ", "handlers, which will inevitably live in")]
        #[doc = concat!(
            " ", "[wormhole_core_bridge_solana](crate::wormhole_core_bridge_solana)."
        )]
        SetMessageFee,
        #[doc = concat!(" ", "Legacy instruction selector.")]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ",
            "NOTE: No more instructions should be added to this enum. Instead, add them as Anchor instruction"
        )]
        #[doc = concat!(" ", "handlers, which will inevitably live in")]
        #[doc = concat!(
            " ", "[wormhole_core_bridge_solana](crate::wormhole_core_bridge_solana)."
        )]
        TransferFees,
        #[doc = concat!(" ", "Legacy instruction selector.")]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ",
            "NOTE: No more instructions should be added to this enum. Instead, add them as Anchor instruction"
        )]
        #[doc = concat!(" ", "handlers, which will inevitably live in")]
        #[doc = concat!(
            " ", "[wormhole_core_bridge_solana](crate::wormhole_core_bridge_solana)."
        )]
        UpgradeContract,
        #[doc = concat!(" ", "Legacy instruction selector.")]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ",
            "NOTE: No more instructions should be added to this enum. Instead, add them as Anchor instruction"
        )]
        #[doc = concat!(" ", "handlers, which will inevitably live in")]
        #[doc = concat!(
            " ", "[wormhole_core_bridge_solana](crate::wormhole_core_bridge_solana)."
        )]
        GuardianSetUpdate,
        #[doc = concat!(" ", "Legacy instruction selector.")]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ",
            "NOTE: No more instructions should be added to this enum. Instead, add them as Anchor instruction"
        )]
        #[doc = concat!(" ", "handlers, which will inevitably live in")]
        #[doc = concat!(
            " ", "[wormhole_core_bridge_solana](crate::wormhole_core_bridge_solana)."
        )]
        VerifySignatures,
        #[doc = concat!(" ", "Legacy instruction selector.")]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ",
            "NOTE: No more instructions should be added to this enum. Instead, add them as Anchor instruction"
        )]
        #[doc = concat!(" ", "handlers, which will inevitably live in")]
        #[doc = concat!(
            " ", "[wormhole_core_bridge_solana](crate::wormhole_core_bridge_solana)."
        )]
        PostMessageUnreliable,
    }
    #[derive(Clone, Copy, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub enum EmitterType {
        Unset,
        Legacy,
        Executable,
    }
    #[doc = concat!(" ", "Status of a message. When a message is posted, its status is")]
    #[doc = concat!(" ", "[Published](MessageStatus::Published).")]
    #[derive(Clone, Copy, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub enum MessageStatus {
        #[doc = concat!(
            " ", "Status of a message. When a message is posted, its status is"
        )]
        #[doc = concat!(" ", "[Published](MessageStatus::Published).")]
        Published,
        #[doc = concat!(
            " ", "Status of a message. When a message is posted, its status is"
        )]
        #[doc = concat!(" ", "[Published](MessageStatus::Published).")]
        Writing,
        #[doc = concat!(
            " ", "Status of a message. When a message is posted, its status is"
        )]
        #[doc = concat!(" ", "[Published](MessageStatus::Published).")]
        ReadyForPublishing,
    }
    #[doc = concat!(
        " ", "Directive used to determine how to post a Core Bridge message."
    )]
    #[derive(Clone, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub enum PublishMessageDirective {
        #[doc = concat!(
            " ", "Directive used to determine how to post a Core Bridge message."
        )]
        Message { nonce: u32, payload: Vec<u8>, commitment: Commitment },
        #[doc = concat!(
            " ", "Directive used to determine how to post a Core Bridge message."
        )]
        ProgramMessage {
            program_id: ::solana_program::pubkey::Pubkey,
            nonce: u32,
            payload: Vec<u8>,
            commitment: Commitment,
        },
        #[doc = concat!(
            " ", "Directive used to determine how to post a Core Bridge message."
        )]
        PreparedMessage,
    }
    #[doc = concat!(" ", "Encoded VAA's processing status.")]
    #[derive(Clone, Copy, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub enum ProcessingStatus {
        #[doc = concat!(" ", "Encoded VAA's processing status.")]
        Unset,
        #[doc = concat!(" ", "Encoded VAA's processing status.")]
        Writing,
        #[doc = concat!(" ", "Encoded VAA's processing status.")]
        Verified,
    }
    #[doc = concat!(
        " ",
        "Representation of Solana's commitment levels. This enum is not exhaustive because Wormhole only"
    )]
    #[doc = concat!(
        " ", "considers these two commitment levels in its Guardian observation."
    )]
    #[doc = concat!(" ", "")]
    #[doc = concat!(
        " ", "See <https://docs.solana.com/cluster/commitments> for more info."
    )]
    #[derive(Clone, Copy, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub enum Commitment {
        #[doc = concat!(
            " ",
            "Representation of Solana's commitment levels. This enum is not exhaustive because Wormhole only"
        )]
        #[doc = concat!(
            " ", "considers these two commitment levels in its Guardian observation."
        )]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ", "See <https://docs.solana.com/cluster/commitments> for more info."
        )]
        Confirmed,
        #[doc = concat!(
            " ",
            "Representation of Solana's commitment levels. This enum is not exhaustive because Wormhole only"
        )]
        #[doc = concat!(
            " ", "considers these two commitment levels in its Guardian observation."
        )]
        #[doc = concat!(" ", "")]
        #[doc = concat!(
            " ", "See <https://docs.solana.com/cluster/commitments> for more info."
        )]
        Finalized,
    }
}
pub mod state {
    #[allow(unused_imports)]
    use super::types::*;
    #[doc = concat!(
        " ",
        "Account used to store a guardian set. The keys encoded in this account are Ethereum pubkeys."
    )]
    #[doc = concat!(
        " ",
        "Its expiration time is determined at the time a guardian set is updated to a new set, where the"
    )]
    #[doc = concat!(" ", "current network clock time is used with")]
    #[doc = concat!(" ", "[guardian_set_ttl](crate::state::Config::guardian_set_ttl).")]
    #[doc = concat!(" ", "")]
    #[doc = concat!(
        " ",
        "NOTE: The account schema is the same as legacy guardian sets, but this account now has a"
    )]
    #[doc = concat!(
        " ",
        "discriminator generated by Anchor's [account] macro. When the Core Bridge program performs a"
    )]
    #[doc = concat!(
        " ",
        "guardian set update with this implementation, guardian sets will now have this Anchor-generated"
    )]
    #[doc = concat!(" ", "discriminator.")]
    #[derive(Clone, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct GuardianSet {
        #[doc = concat!(
            " ",
            "Index representing an incrementing version number for this guardian set."
        )]
        pub index: u32,
        #[doc = concat!(" ", "Ethereum-style public keys.")]
        pub keys: Vec<[u8; 20usize]>,
        #[doc = concat!(
            " ", "Timestamp representing the time this guardian became active."
        )]
        pub creation_time: Timestamp,
        #[doc = concat!(
            " ", "Expiration time when VAAs issued by this set are no longer valid."
        )]
        pub expiration_time: Timestamp,
    }
    impl ::anchor_interface::Account for GuardianSet {
        fn discriminator() -> &'static [u8; 8] {
            &[120u8, 77u8, 74u8, 98u8, 34u8, 83u8, 96u8, 125u8]
        }
    }
    impl ::anchor_interface::AccountSerialize for GuardianSet {
        fn try_serialize<W: std::io::Write>(
            &self,
            writer: &mut W,
        ) -> std::io::Result<()> {
            use ::anchor_interface::Account;
            writer.write_all(Self::discriminator())?;
            ::borsh::BorshSerialize::serialize(self, writer)?;
            Ok(())
        }
    }
    impl ::anchor_interface::AccountDeserialize for GuardianSet {
        fn try_deserialize(data: &mut &[u8]) -> std::io::Result<Self> {
            use ::anchor_interface::Account;
            if data.len() < 8 || &data[..8] != Self::discriminator() {
                return Err(
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "invalid discriminator",
                    ),
                );
            }
            let t = ::borsh::BorshDeserialize::try_from_slice(&data[8..])?;
            Ok(t)
        }
    }
    #[doc = concat!(
        " ",
        "Account used to store information about a guardian set used to sign a VAA. There is only one"
    )]
    #[doc = concat!(" ", "signature set for each verified VAA (associated with a")]
    #[doc = concat!(
        " ",
        "[PostedVaaV1](crate::legacy::state::PostedVaaV1) account). This account is created using the"
    )]
    #[doc = concat!(" ", "verify signatures legacy instruction.")]
    #[doc = concat!(" ", "")]
    #[doc = concat!(
        " ",
        "NOTE: The account schema is the same as legacy signature sets, but this account now has a"
    )]
    #[doc = concat!(
        " ",
        "discriminator generated by Anchor's [account] macro. When the Core Bridge program upgrades to"
    )]
    #[doc = concat!(
        " ",
        "this implementation from the old one, integrators in the middle of verifying signatures will"
    )]
    #[doc = concat!(" ", "have to use a new keypair for this account and try again.")]
    #[derive(Clone, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct SignatureSet {
        #[doc = concat!(" ", "Signatures of validators")]
        pub sig_verify_successes: Vec<bool>,
        #[doc = concat!(" ", "Hash of the VAA message body.")]
        pub message_hash: MessageHash,
        #[doc = concat!(" ", "Index of the guardian set")]
        pub guardian_set_index: u32,
    }
    impl ::anchor_interface::Account for SignatureSet {
        fn discriminator() -> &'static [u8; 8] {
            &[17u8, 212u8, 246u8, 114u8, 183u8, 159u8, 65u8, 246u8]
        }
    }
    impl ::anchor_interface::AccountSerialize for SignatureSet {
        fn try_serialize<W: std::io::Write>(
            &self,
            writer: &mut W,
        ) -> std::io::Result<()> {
            use ::anchor_interface::Account;
            writer.write_all(Self::discriminator())?;
            ::borsh::BorshSerialize::serialize(self, writer)?;
            Ok(())
        }
    }
    impl ::anchor_interface::AccountDeserialize for SignatureSet {
        fn try_deserialize(data: &mut &[u8]) -> std::io::Result<Self> {
            use ::anchor_interface::Account;
            if data.len() < 8 || &data[..8] != Self::discriminator() {
                return Err(
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "invalid discriminator",
                    ),
                );
            }
            let t = ::borsh::BorshDeserialize::try_from_slice(&data[8..])?;
            Ok(t)
        }
    }
    #[doc = concat!(" ", "Account used to warehouse VAA buffer.")]
    #[doc = concat!(" ", "")]
    #[doc = concat!(
        " ",
        "NOTE: This account should not be used by an external application unless the header's status is"
    )]
    #[doc = concat!(
        " ",
        "`Verified`. It is encouraged to use the `EncodedVaa` zero-copy account struct instead."
    )]
    #[derive(Clone, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct EncodedVaa {
        #[doc = concat!(" ", "Status, write authority and VAA version.")]
        pub header: Header,
        #[doc = concat!(" ", "VAA buffer.")]
        pub buf: Vec<u8>,
    }
    impl ::anchor_interface::Account for EncodedVaa {
        fn discriminator() -> &'static [u8; 8] {
            &[226u8, 101u8, 163u8, 4u8, 133u8, 160u8, 84u8, 245u8]
        }
    }
    impl ::anchor_interface::AccountSerialize for EncodedVaa {
        fn try_serialize<W: std::io::Write>(
            &self,
            writer: &mut W,
        ) -> std::io::Result<()> {
            use ::anchor_interface::Account;
            writer.write_all(Self::discriminator())?;
            ::borsh::BorshSerialize::serialize(self, writer)?;
            Ok(())
        }
    }
    impl ::anchor_interface::AccountDeserialize for EncodedVaa {
        fn try_deserialize(data: &mut &[u8]) -> std::io::Result<Self> {
            use ::anchor_interface::Account;
            if data.len() < 8 || &data[..8] != Self::discriminator() {
                return Err(
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "invalid discriminator",
                    ),
                );
            }
            let t = ::borsh::BorshDeserialize::try_from_slice(&data[8..])?;
            Ok(t)
        }
    }
}
pub mod error {
    #[allow(unused_imports)]
    use super::types::*;
    use ::num_derive::FromPrimitive;
    use ::thiserror::Error;
    use ::solana_program::{decode_error::DecodeError, program_error::ProgramError};
    #[derive(Error, Clone, Copy, Debug, FromPrimitive, PartialEq, Eq)]
    #[repr(u32)]
    pub enum WormholeCoreBridgeSolanaError {
        #[error("InvalidInstructionArgument")]
        InvalidInstructionArgument = 6002u32,
        #[error("AccountNotZeroed")]
        AccountNotZeroed = 6003u32,
        #[error("InvalidDataConversion")]
        InvalidDataConversion = 6004u32,
        #[error("U64Overflow")]
        U64Overflow = 6006u32,
        #[error("InvalidComputeSize")]
        InvalidComputeSize = 6008u32,
        #[error("InvalidChain")]
        InvalidChain = 6016u32,
        #[error("InvalidGovernanceEmitter")]
        InvalidGovernanceEmitter = 6032u32,
        #[error("InvalidGovernanceAction")]
        InvalidGovernanceAction = 6034u32,
        #[error("LatestGuardianSetRequired")]
        LatestGuardianSetRequired = 6036u32,
        #[error("GovernanceForAnotherChain")]
        GovernanceForAnotherChain = 6038u32,
        #[error("InvalidGovernanceVaa")]
        InvalidGovernanceVaa = 6040u32,
        #[error("InsufficientFees")]
        InsufficientFees = 6256u32,
        #[error("EmitterMismatch")]
        EmitterMismatch = 6258u32,
        #[error("NotReadyForPublishing")]
        NotReadyForPublishing = 6260u32,
        #[error("InvalidPreparedMessage")]
        InvalidPreparedMessage = 6262u32,
        #[error("ExecutableEmitter")]
        ExecutableEmitter = 6264u32,
        #[error("LegacyEmitter")]
        LegacyEmitter = 6266u32,
        #[error("InvalidSignatureSet")]
        InvalidSignatureSet = 6512u32,
        #[error("InvalidMessageHash")]
        InvalidMessageHash = 6514u32,
        #[error("NoQuorum")]
        NoQuorum = 6515u32,
        #[error("MessageMismatch")]
        MessageMismatch = 6516u32,
        #[error("NotEnoughLamports")]
        NotEnoughLamports = 7024u32,
        #[error("InvalidFeeRecipient")]
        InvalidFeeRecipient = 7026u32,
        #[error("ImplementationMismatch")]
        ImplementationMismatch = 7280u32,
        #[error("InvalidGuardianSetIndex")]
        InvalidGuardianSetIndex = 7536u32,
        #[error("GuardianSetMismatch")]
        GuardianSetMismatch = 7792u32,
        #[error("InstructionAtWrongIndex")]
        InstructionAtWrongIndex = 7794u32,
        #[error("EmptySigVerifyInstruction")]
        EmptySigVerifyInstruction = 7795u32,
        #[error("InvalidSigVerifyInstruction")]
        InvalidSigVerifyInstruction = 7796u32,
        #[error("GuardianSetExpired")]
        GuardianSetExpired = 7798u32,
        #[error("InvalidGuardianKeyRecovery")]
        InvalidGuardianKeyRecovery = 7800u32,
        #[error("SignerIndicesMismatch")]
        SignerIndicesMismatch = 7802u32,
        #[error("PayloadSizeMismatch")]
        PayloadSizeMismatch = 8048u32,
        #[error("ZeroGuardians")]
        ZeroGuardians = 10112u32,
        #[error("GuardianZeroAddress")]
        GuardianZeroAddress = 10128u32,
        #[error("DuplicateGuardianAddress")]
        DuplicateGuardianAddress = 10144u32,
        #[error("MessageAlreadyPublished")]
        MessageAlreadyPublished = 10160u32,
        #[error("VaaWritingDisallowed")]
        VaaWritingDisallowed = 10176u32,
        #[error("VaaAlreadyVerified")]
        VaaAlreadyVerified = 10192u32,
        #[error("InvalidGuardianIndex")]
        InvalidGuardianIndex = 10208u32,
        #[error("InvalidSignature")]
        InvalidSignature = 10224u32,
        #[error("UnverifiedVaa")]
        UnverifiedVaa = 10256u32,
        #[error("VaaStillProcessing")]
        VaaStillProcessing = 10258u32,
        #[error("InWritingStatus")]
        InWritingStatus = 10260u32,
        #[error("NotInWritingStatus")]
        NotInWritingStatus = 10262u32,
        #[error("InvalidMessageStatus")]
        InvalidMessageStatus = 10264u32,
        #[error("HashNotComputed")]
        HashNotComputed = 10266u32,
        #[error("InvalidVaaVersion")]
        InvalidVaaVersion = 10268u32,
        #[error("InvalidCreatedAccountSize")]
        InvalidCreatedAccountSize = 10270u32,
        #[error("DataOverflow")]
        DataOverflow = 10272u32,
        #[error("ExceedsMaxPayloadSize (30KB)")]
        ExceedsMaxPayloadSize = 10274u32,
        #[error("CannotParseVaa")]
        CannotParseVaa = 10276u32,
        #[error("EmitterAuthorityMismatch")]
        EmitterAuthorityMismatch = 10278u32,
        #[error("InvalidProgramEmitter")]
        InvalidProgramEmitter = 10280u32,
        #[error("WriteAuthorityMismatch")]
        WriteAuthorityMismatch = 10282u32,
        #[error("PostedVaaPayloadTooLarge")]
        PostedVaaPayloadTooLarge = 10284u32,
        #[error("ExecutableDisallowed")]
        ExecutableDisallowed = 10286u32,
    }
    impl DecodeError<WormholeCoreBridgeSolanaError> for WormholeCoreBridgeSolanaError {
        fn type_of() -> &'static str {
            "WormholeCoreBridgeSolanaError"
        }
    }
    impl From<WormholeCoreBridgeSolanaError> for ProgramError {
        fn from(err: WormholeCoreBridgeSolanaError) -> Self {
            Self::Custom(err as u32)
        }
    }
}
