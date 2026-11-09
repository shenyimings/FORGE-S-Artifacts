macro_rules! gen_crate_docs {
    () => {
        concat!(" ", "Pyth Solana Receiver", " v", "0.1.0",
        " program interface generated from Anchor IDL.")
    };
}
pub(crate) use gen_crate_docs;
pub use anchor_interface::prelude::*;
pub mod instruction {
    #[allow(unused_imports)]
    use super::types::*;
    #[derive(Debug)]
    pub enum PythSolanaReceiverInstruction {
        /// Initialize
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer, writable]` payer
        /// 1. `[writable]` config
        /// 2. `[]` system program
        Initialize { initial_config: Config },
        /// Request Governance Authority Transfer
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer]` payer
        /// 1. `[writable]` config
        RequestGovernanceAuthorityTransfer {
            target_governance_authority: ::solana_program::pubkey::Pubkey,
        },
        /// Accept Governance Authority Transfer
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer]` payer
        /// 1. `[writable]` config
        AcceptGovernanceAuthorityTransfer,
        /// Set Data Sources
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer]` payer
        /// 1. `[writable]` config
        SetDataSources { valid_data_sources: Vec<DataSource> },
        /// Set Fee
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer]` payer
        /// 1. `[writable]` config
        SetFee { single_update_fee_in_lamports: u64 },
        /// Set Wormhole Address
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer]` payer
        /// 1. `[writable]` config
        SetWormholeAddress { wormhole: ::solana_program::pubkey::Pubkey },
        /// Set Minimum Signatures
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer]` payer
        /// 1. `[writable]` config
        SetMinimumSignatures { minimum_signatures: u8 },
        #[doc = concat!(" ", "Post a price update using a VAA and a MerklePriceUpdate.")]
        #[doc = concat!(
            " ",
            "This function allows you to post a price update in a single transaction."
        )]
        #[doc = concat!(
            " ",
            "Compared to post_update, it is less secure since you won't be able to verify all guardian signatures if you use this function because of transaction size limitations."
        )]
        #[doc = concat!(
            " ",
            "Typically, you can fit 5 guardian signatures in a transaction that uses this."
        )]
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer, writable]` payer
        /// 1. `[]` guardian set
        /// 2. `[]` config
        /// 3. `[writable]` treasury
        /// 4. `[signer, writable]` price update account
        /// 5. `[]` system program
        /// 6. `[signer]` write authority
        PostUpdateAtomic { params: PostUpdateAtomicParams },
        #[doc = concat!(
            " ",
            "Post a price update using an encoded_vaa account and a MerklePriceUpdate calldata."
        )]
        #[doc = concat!(
            " ",
            "This should be called after the client has already verified the Vaa via the Wormhole contract."
        )]
        #[doc = concat!(
            " ",
            "Check out target_chains/solana/cli/src/main.rs for an example of how to do this."
        )]
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer, writable]` payer
        /// 1. `[]` encoded vaa
        /// 2. `[]` config
        /// 3. `[writable]` treasury
        /// 4. `[signer, writable]` price update account
        /// 5. `[]` system program
        /// 6. `[signer]` write authority
        PostUpdate { params: PostUpdateParams },
        /// Reclaim Rent
        ///
        /// Accounts expected by this instruction:
        /// 0. `[signer, writable]` payer
        /// 1. `[writable]` price update account
        ReclaimRent,
    }
    impl PythSolanaReceiverInstruction {
        pub fn sighash(&self) -> &'static [u8; 8] {
            match self {
                Self::Initialize { .. } => {
                    &[175u8, 175u8, 109u8, 31u8, 13u8, 152u8, 155u8, 237u8]
                }
                Self::RequestGovernanceAuthorityTransfer { .. } => {
                    &[92u8, 18u8, 67u8, 156u8, 27u8, 151u8, 183u8, 224u8]
                }
                Self::AcceptGovernanceAuthorityTransfer => {
                    &[254u8, 39u8, 222u8, 79u8, 64u8, 217u8, 205u8, 127u8]
                }
                Self::SetDataSources { .. } => {
                    &[107u8, 73u8, 15u8, 119u8, 195u8, 116u8, 91u8, 210u8]
                }
                Self::SetFee { .. } => {
                    &[18u8, 154u8, 24u8, 18u8, 237u8, 214u8, 19u8, 80u8]
                }
                Self::SetWormholeAddress { .. } => {
                    &[154u8, 174u8, 252u8, 157u8, 91u8, 215u8, 179u8, 156u8]
                }
                Self::SetMinimumSignatures { .. } => {
                    &[5u8, 210u8, 206u8, 124u8, 43u8, 68u8, 104u8, 149u8]
                }
                Self::PostUpdateAtomic { .. } => {
                    &[49u8, 172u8, 84u8, 192u8, 175u8, 180u8, 52u8, 234u8]
                }
                Self::PostUpdate { .. } => {
                    &[133u8, 95u8, 207u8, 175u8, 11u8, 79u8, 118u8, 44u8]
                }
                Self::ReclaimRent => {
                    &[218u8, 200u8, 19u8, 197u8, 227u8, 89u8, 192u8, 22u8]
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
                    [175u8, 175u8, 109u8, 31u8, 13u8, 152u8, 155u8, 237u8] => {
                        InitializeDeserializer::try_from_slice(ix_data)?.into()
                    }
                    [92u8, 18u8, 67u8, 156u8, 27u8, 151u8, 183u8, 224u8] => {
                        RequestGovernanceAuthorityTransferDeserializer::try_from_slice(
                                ix_data,
                            )?
                            .into()
                    }
                    [254u8, 39u8, 222u8, 79u8, 64u8, 217u8, 205u8, 127u8] => {
                        AcceptGovernanceAuthorityTransferDeserializer::try_from_slice(
                                ix_data,
                            )?
                            .into()
                    }
                    [107u8, 73u8, 15u8, 119u8, 195u8, 116u8, 91u8, 210u8] => {
                        SetDataSourcesDeserializer::try_from_slice(ix_data)?.into()
                    }
                    [18u8, 154u8, 24u8, 18u8, 237u8, 214u8, 19u8, 80u8] => {
                        SetFeeDeserializer::try_from_slice(ix_data)?.into()
                    }
                    [154u8, 174u8, 252u8, 157u8, 91u8, 215u8, 179u8, 156u8] => {
                        SetWormholeAddressDeserializer::try_from_slice(ix_data)?.into()
                    }
                    [5u8, 210u8, 206u8, 124u8, 43u8, 68u8, 104u8, 149u8] => {
                        SetMinimumSignaturesDeserializer::try_from_slice(ix_data)?.into()
                    }
                    [49u8, 172u8, 84u8, 192u8, 175u8, 180u8, 52u8, 234u8] => {
                        PostUpdateAtomicDeserializer::try_from_slice(ix_data)?.into()
                    }
                    [133u8, 95u8, 207u8, 175u8, 11u8, 79u8, 118u8, 44u8] => {
                        PostUpdateDeserializer::try_from_slice(ix_data)?.into()
                    }
                    [218u8, 200u8, 19u8, 197u8, 227u8, 89u8, 192u8, 22u8] => {
                        ReclaimRentDeserializer::try_from_slice(ix_data)?.into()
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
    impl ::borsh::BorshSerialize for PythSolanaReceiverInstruction {
        fn serialize<W: ::borsh::maybestd::io::Write>(
            &self,
            writer: &mut W,
        ) -> ::core::result::Result<(), ::borsh::maybestd::io::Error> {
            match self {
                Self::Initialize { initial_config } => {
                    ::borsh::BorshSerialize::serialize(initial_config, writer)?;
                }
                Self::RequestGovernanceAuthorityTransfer {
                    target_governance_authority,
                } => {
                    ::borsh::BorshSerialize::serialize(
                        target_governance_authority,
                        writer,
                    )?;
                }
                Self::AcceptGovernanceAuthorityTransfer => {}
                Self::SetDataSources { valid_data_sources } => {
                    ::borsh::BorshSerialize::serialize(valid_data_sources, writer)?;
                }
                Self::SetFee { single_update_fee_in_lamports } => {
                    ::borsh::BorshSerialize::serialize(
                        single_update_fee_in_lamports,
                        writer,
                    )?;
                }
                Self::SetWormholeAddress { wormhole } => {
                    ::borsh::BorshSerialize::serialize(wormhole, writer)?;
                }
                Self::SetMinimumSignatures { minimum_signatures } => {
                    ::borsh::BorshSerialize::serialize(minimum_signatures, writer)?;
                }
                Self::PostUpdateAtomic { params } => {
                    ::borsh::BorshSerialize::serialize(params, writer)?;
                }
                Self::PostUpdate { params } => {
                    ::borsh::BorshSerialize::serialize(params, writer)?;
                }
                Self::ReclaimRent => {}
            }
            Ok(())
        }
    }
    struct InitializeDeserializer(PythSolanaReceiverInstruction);
    impl ::borsh::de::BorshDeserialize for InitializeDeserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(PythSolanaReceiverInstruction::Initialize {
                    initial_config: ::borsh::BorshDeserialize::deserialize(_buf)?,
                }),
            )
        }
    }
    impl From<InitializeDeserializer> for PythSolanaReceiverInstruction {
        fn from(helper: InitializeDeserializer) -> PythSolanaReceiverInstruction {
            helper.0
        }
    }
    struct RequestGovernanceAuthorityTransferDeserializer(PythSolanaReceiverInstruction);
    impl ::borsh::de::BorshDeserialize
    for RequestGovernanceAuthorityTransferDeserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(PythSolanaReceiverInstruction::RequestGovernanceAuthorityTransfer {
                    target_governance_authority: ::borsh::BorshDeserialize::deserialize(
                        _buf,
                    )?,
                }),
            )
        }
    }
    impl From<RequestGovernanceAuthorityTransferDeserializer>
    for PythSolanaReceiverInstruction {
        fn from(
            helper: RequestGovernanceAuthorityTransferDeserializer,
        ) -> PythSolanaReceiverInstruction {
            helper.0
        }
    }
    struct AcceptGovernanceAuthorityTransferDeserializer(PythSolanaReceiverInstruction);
    impl ::borsh::de::BorshDeserialize
    for AcceptGovernanceAuthorityTransferDeserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(PythSolanaReceiverInstruction::AcceptGovernanceAuthorityTransfer {
                }),
            )
        }
    }
    impl From<AcceptGovernanceAuthorityTransferDeserializer>
    for PythSolanaReceiverInstruction {
        fn from(
            helper: AcceptGovernanceAuthorityTransferDeserializer,
        ) -> PythSolanaReceiverInstruction {
            helper.0
        }
    }
    struct SetDataSourcesDeserializer(PythSolanaReceiverInstruction);
    impl ::borsh::de::BorshDeserialize for SetDataSourcesDeserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(PythSolanaReceiverInstruction::SetDataSources {
                    valid_data_sources: ::borsh::BorshDeserialize::deserialize(_buf)?,
                }),
            )
        }
    }
    impl From<SetDataSourcesDeserializer> for PythSolanaReceiverInstruction {
        fn from(helper: SetDataSourcesDeserializer) -> PythSolanaReceiverInstruction {
            helper.0
        }
    }
    struct SetFeeDeserializer(PythSolanaReceiverInstruction);
    impl ::borsh::de::BorshDeserialize for SetFeeDeserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(PythSolanaReceiverInstruction::SetFee {
                    single_update_fee_in_lamports: ::borsh::BorshDeserialize::deserialize(
                        _buf,
                    )?,
                }),
            )
        }
    }
    impl From<SetFeeDeserializer> for PythSolanaReceiverInstruction {
        fn from(helper: SetFeeDeserializer) -> PythSolanaReceiverInstruction {
            helper.0
        }
    }
    struct SetWormholeAddressDeserializer(PythSolanaReceiverInstruction);
    impl ::borsh::de::BorshDeserialize for SetWormholeAddressDeserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(PythSolanaReceiverInstruction::SetWormholeAddress {
                    wormhole: ::borsh::BorshDeserialize::deserialize(_buf)?,
                }),
            )
        }
    }
    impl From<SetWormholeAddressDeserializer> for PythSolanaReceiverInstruction {
        fn from(
            helper: SetWormholeAddressDeserializer,
        ) -> PythSolanaReceiverInstruction {
            helper.0
        }
    }
    struct SetMinimumSignaturesDeserializer(PythSolanaReceiverInstruction);
    impl ::borsh::de::BorshDeserialize for SetMinimumSignaturesDeserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(PythSolanaReceiverInstruction::SetMinimumSignatures {
                    minimum_signatures: ::borsh::BorshDeserialize::deserialize(_buf)?,
                }),
            )
        }
    }
    impl From<SetMinimumSignaturesDeserializer> for PythSolanaReceiverInstruction {
        fn from(
            helper: SetMinimumSignaturesDeserializer,
        ) -> PythSolanaReceiverInstruction {
            helper.0
        }
    }
    struct PostUpdateAtomicDeserializer(PythSolanaReceiverInstruction);
    impl ::borsh::de::BorshDeserialize for PostUpdateAtomicDeserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(PythSolanaReceiverInstruction::PostUpdateAtomic {
                    params: ::borsh::BorshDeserialize::deserialize(_buf)?,
                }),
            )
        }
    }
    impl From<PostUpdateAtomicDeserializer> for PythSolanaReceiverInstruction {
        fn from(helper: PostUpdateAtomicDeserializer) -> PythSolanaReceiverInstruction {
            helper.0
        }
    }
    struct PostUpdateDeserializer(PythSolanaReceiverInstruction);
    impl ::borsh::de::BorshDeserialize for PostUpdateDeserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(PythSolanaReceiverInstruction::PostUpdate {
                    params: ::borsh::BorshDeserialize::deserialize(_buf)?,
                }),
            )
        }
    }
    impl From<PostUpdateDeserializer> for PythSolanaReceiverInstruction {
        fn from(helper: PostUpdateDeserializer) -> PythSolanaReceiverInstruction {
            helper.0
        }
    }
    struct ReclaimRentDeserializer(PythSolanaReceiverInstruction);
    impl ::borsh::de::BorshDeserialize for ReclaimRentDeserializer {
        fn deserialize(
            _buf: &mut &[u8],
        ) -> ::core::result::Result<Self, ::borsh::maybestd::io::Error> {
            Ok(
                Self(PythSolanaReceiverInstruction::ReclaimRent {
                }),
            )
        }
    }
    impl From<ReclaimRentDeserializer> for PythSolanaReceiverInstruction {
        fn from(helper: ReclaimRentDeserializer) -> PythSolanaReceiverInstruction {
            helper.0
        }
    }
    #[derive(Debug)]
    pub struct Initialize {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub payer: ::solana_program::pubkey::Pubkey,
        pub config: ::solana_program::pubkey::Pubkey,
        pub system_program: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
        pub initial_config: Config,
    }
    impl Initialize {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self {
                program_id,
                payer,
                config,
                system_program,
                trailing_accounts,
                initial_config,
            } = self;
            let mut accounts = vec![
                AccountMeta::new(payer, true), AccountMeta::new(config, false),
                AccountMeta::new_readonly(system_program, false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = PythSolanaReceiverInstruction::Initialize {
                initial_config,
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
    pub struct InitializeAccountIndexes {
        pub payer: usize,
        pub config: usize,
        pub system_program: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl InitializeAccountIndexes {
        pub const PAYER: usize = 0usize;
        pub const CONFIG: usize = 1usize;
        pub const SYSTEM_PROGRAM: usize = 2usize;
    }
    impl<'a> TryFrom<&'a [u8]> for InitializeAccountIndexes {
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
                config: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(config),
                            1usize,
                        ),
                    )?,
                system_program: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(system_program),
                            2usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct RequestGovernanceAuthorityTransfer {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub payer: ::solana_program::pubkey::Pubkey,
        pub config: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
        pub target_governance_authority: ::solana_program::pubkey::Pubkey,
    }
    impl RequestGovernanceAuthorityTransfer {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self {
                program_id,
                payer,
                config,
                trailing_accounts,
                target_governance_authority,
            } = self;
            let mut accounts = vec![
                AccountMeta::new_readonly(payer, true), AccountMeta::new(config, false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = PythSolanaReceiverInstruction::RequestGovernanceAuthorityTransfer {
                target_governance_authority,
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
    pub struct RequestGovernanceAuthorityTransferAccountIndexes {
        pub payer: usize,
        pub config: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl RequestGovernanceAuthorityTransferAccountIndexes {
        pub const PAYER: usize = 0usize;
        pub const CONFIG: usize = 1usize;
    }
    impl<'a> TryFrom<&'a [u8]> for RequestGovernanceAuthorityTransferAccountIndexes {
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
                config: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(config),
                            1usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct AcceptGovernanceAuthorityTransfer {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub payer: ::solana_program::pubkey::Pubkey,
        pub config: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
    }
    impl AcceptGovernanceAuthorityTransfer {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self { program_id, payer, config, trailing_accounts } = self;
            let mut accounts = vec![
                AccountMeta::new_readonly(payer, true), AccountMeta::new(config, false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = PythSolanaReceiverInstruction::AcceptGovernanceAuthorityTransfer {
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
    pub struct AcceptGovernanceAuthorityTransferAccountIndexes {
        pub payer: usize,
        pub config: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl AcceptGovernanceAuthorityTransferAccountIndexes {
        pub const PAYER: usize = 0usize;
        pub const CONFIG: usize = 1usize;
    }
    impl<'a> TryFrom<&'a [u8]> for AcceptGovernanceAuthorityTransferAccountIndexes {
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
                config: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(config),
                            1usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct SetDataSources {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub payer: ::solana_program::pubkey::Pubkey,
        pub config: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
        pub valid_data_sources: Vec<DataSource>,
    }
    impl SetDataSources {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self {
                program_id,
                payer,
                config,
                trailing_accounts,
                valid_data_sources,
            } = self;
            let mut accounts = vec![
                AccountMeta::new_readonly(payer, true), AccountMeta::new(config, false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = PythSolanaReceiverInstruction::SetDataSources {
                valid_data_sources,
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
    pub struct SetDataSourcesAccountIndexes {
        pub payer: usize,
        pub config: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl SetDataSourcesAccountIndexes {
        pub const PAYER: usize = 0usize;
        pub const CONFIG: usize = 1usize;
    }
    impl<'a> TryFrom<&'a [u8]> for SetDataSourcesAccountIndexes {
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
                config: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(config),
                            1usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct SetFee {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub payer: ::solana_program::pubkey::Pubkey,
        pub config: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
        pub single_update_fee_in_lamports: u64,
    }
    impl SetFee {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self {
                program_id,
                payer,
                config,
                trailing_accounts,
                single_update_fee_in_lamports,
            } = self;
            let mut accounts = vec![
                AccountMeta::new_readonly(payer, true), AccountMeta::new(config, false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = PythSolanaReceiverInstruction::SetFee {
                single_update_fee_in_lamports,
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
    pub struct SetFeeAccountIndexes {
        pub payer: usize,
        pub config: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl SetFeeAccountIndexes {
        pub const PAYER: usize = 0usize;
        pub const CONFIG: usize = 1usize;
    }
    impl<'a> TryFrom<&'a [u8]> for SetFeeAccountIndexes {
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
                config: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(config),
                            1usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct SetWormholeAddress {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub payer: ::solana_program::pubkey::Pubkey,
        pub config: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
        pub wormhole: ::solana_program::pubkey::Pubkey,
    }
    impl SetWormholeAddress {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self { program_id, payer, config, trailing_accounts, wormhole } = self;
            let mut accounts = vec![
                AccountMeta::new_readonly(payer, true), AccountMeta::new(config, false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = PythSolanaReceiverInstruction::SetWormholeAddress {
                wormhole,
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
    pub struct SetWormholeAddressAccountIndexes {
        pub payer: usize,
        pub config: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl SetWormholeAddressAccountIndexes {
        pub const PAYER: usize = 0usize;
        pub const CONFIG: usize = 1usize;
    }
    impl<'a> TryFrom<&'a [u8]> for SetWormholeAddressAccountIndexes {
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
                config: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(config),
                            1usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct SetMinimumSignatures {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub payer: ::solana_program::pubkey::Pubkey,
        pub config: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
        pub minimum_signatures: u8,
    }
    impl SetMinimumSignatures {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self {
                program_id,
                payer,
                config,
                trailing_accounts,
                minimum_signatures,
            } = self;
            let mut accounts = vec![
                AccountMeta::new_readonly(payer, true), AccountMeta::new(config, false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = PythSolanaReceiverInstruction::SetMinimumSignatures {
                minimum_signatures,
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
    pub struct SetMinimumSignaturesAccountIndexes {
        pub payer: usize,
        pub config: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl SetMinimumSignaturesAccountIndexes {
        pub const PAYER: usize = 0usize;
        pub const CONFIG: usize = 1usize;
    }
    impl<'a> TryFrom<&'a [u8]> for SetMinimumSignaturesAccountIndexes {
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
                config: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(config),
                            1usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct PostUpdateAtomic {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub payer: ::solana_program::pubkey::Pubkey,
        pub guardian_set: ::solana_program::pubkey::Pubkey,
        pub config: ::solana_program::pubkey::Pubkey,
        pub treasury: ::solana_program::pubkey::Pubkey,
        pub price_update_account: ::solana_program::pubkey::Pubkey,
        pub system_program: ::solana_program::pubkey::Pubkey,
        pub write_authority: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
        pub params: PostUpdateAtomicParams,
    }
    impl PostUpdateAtomic {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self {
                program_id,
                payer,
                guardian_set,
                config,
                treasury,
                price_update_account,
                system_program,
                write_authority,
                trailing_accounts,
                params,
            } = self;
            let mut accounts = vec![
                AccountMeta::new(payer, true), AccountMeta::new_readonly(guardian_set,
                false), AccountMeta::new_readonly(config, false),
                AccountMeta::new(treasury, false), AccountMeta::new(price_update_account,
                true), AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(write_authority, true),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = PythSolanaReceiverInstruction::PostUpdateAtomic {
                params,
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
    pub struct PostUpdateAtomicAccountIndexes {
        pub payer: usize,
        pub guardian_set: usize,
        pub config: usize,
        pub treasury: usize,
        pub price_update_account: usize,
        pub system_program: usize,
        pub write_authority: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl PostUpdateAtomicAccountIndexes {
        pub const PAYER: usize = 0usize;
        pub const GUARDIAN_SET: usize = 1usize;
        pub const CONFIG: usize = 2usize;
        pub const TREASURY: usize = 3usize;
        pub const PRICE_UPDATE_ACCOUNT: usize = 4usize;
        pub const SYSTEM_PROGRAM: usize = 5usize;
        pub const WRITE_AUTHORITY: usize = 6usize;
    }
    impl<'a> TryFrom<&'a [u8]> for PostUpdateAtomicAccountIndexes {
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
                guardian_set: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(guardian_set),
                            1usize,
                        ),
                    )?,
                config: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(config),
                            2usize,
                        ),
                    )?,
                treasury: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(treasury),
                            3usize,
                        ),
                    )?,
                price_update_account: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(price_update_account),
                            4usize,
                        ),
                    )?,
                system_program: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(system_program),
                            5usize,
                        ),
                    )?,
                write_authority: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(write_authority),
                            6usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct PostUpdate {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub payer: ::solana_program::pubkey::Pubkey,
        pub encoded_vaa: ::solana_program::pubkey::Pubkey,
        pub config: ::solana_program::pubkey::Pubkey,
        pub treasury: ::solana_program::pubkey::Pubkey,
        pub price_update_account: ::solana_program::pubkey::Pubkey,
        pub system_program: ::solana_program::pubkey::Pubkey,
        pub write_authority: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
        pub params: PostUpdateParams,
    }
    impl PostUpdate {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self {
                program_id,
                payer,
                encoded_vaa,
                config,
                treasury,
                price_update_account,
                system_program,
                write_authority,
                trailing_accounts,
                params,
            } = self;
            let mut accounts = vec![
                AccountMeta::new(payer, true), AccountMeta::new_readonly(encoded_vaa,
                false), AccountMeta::new_readonly(config, false),
                AccountMeta::new(treasury, false), AccountMeta::new(price_update_account,
                true), AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(write_authority, true),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = PythSolanaReceiverInstruction::PostUpdate {
                params,
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
    pub struct PostUpdateAccountIndexes {
        pub payer: usize,
        pub encoded_vaa: usize,
        pub config: usize,
        pub treasury: usize,
        pub price_update_account: usize,
        pub system_program: usize,
        pub write_authority: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl PostUpdateAccountIndexes {
        pub const PAYER: usize = 0usize;
        pub const ENCODED_VAA: usize = 1usize;
        pub const CONFIG: usize = 2usize;
        pub const TREASURY: usize = 3usize;
        pub const PRICE_UPDATE_ACCOUNT: usize = 4usize;
        pub const SYSTEM_PROGRAM: usize = 5usize;
        pub const WRITE_AUTHORITY: usize = 6usize;
    }
    impl<'a> TryFrom<&'a [u8]> for PostUpdateAccountIndexes {
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
                config: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(config),
                            2usize,
                        ),
                    )?,
                treasury: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(treasury),
                            3usize,
                        ),
                    )?,
                price_update_account: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(price_update_account),
                            4usize,
                        ),
                    )?,
                system_program: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(system_program),
                            5usize,
                        ),
                    )?,
                write_authority: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(write_authority),
                            6usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
    #[derive(Debug)]
    pub struct ReclaimRent {
        pub program_id: ::solana_program::pubkey::Pubkey,
        pub payer: ::solana_program::pubkey::Pubkey,
        pub price_update_account: ::solana_program::pubkey::Pubkey,
        pub trailing_accounts: Vec<::solana_program::instruction::AccountMeta>,
    }
    impl ReclaimRent {
        pub fn into_instruction(self) -> ::solana_program::instruction::Instruction {
            use ::solana_program::instruction::{AccountMeta, Instruction};
            let Self { program_id, payer, price_update_account, trailing_accounts } = self;
            let mut accounts = vec![
                AccountMeta::new(payer, true), AccountMeta::new(price_update_account,
                false),
            ];
            if !trailing_accounts.is_empty() {
                accounts.extend(trailing_accounts);
            }
            let data = PythSolanaReceiverInstruction::ReclaimRent {
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
    pub struct ReclaimRentAccountIndexes {
        pub payer: usize,
        pub price_update_account: usize,
        pub trailing_accounts: Vec<usize>,
    }
    impl ReclaimRentAccountIndexes {
        pub const PAYER: usize = 0usize;
        pub const PRICE_UPDATE_ACCOUNT: usize = 1usize;
    }
    impl<'a> TryFrom<&'a [u8]> for ReclaimRentAccountIndexes {
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
                price_update_account: iter
                    .next()
                    .ok_or(
                        ::anchor_interface::errors::TryAccountIndexesError::GetIndex(
                            stringify!(price_update_account),
                            1usize,
                        ),
                    )?,
                trailing_accounts: iter.collect(),
            })
        }
    }
}
pub mod types {
    #[derive(Clone, Copy, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct PriceFeedMessage {
        pub feed_id: [u8; 32usize],
        pub price: i64,
        pub conf: u64,
        pub exponent: i32,
        pub publish_time: i64,
        pub prev_publish_time: i64,
        pub ema_price: i64,
        pub ema_conf: u64,
    }
    #[derive(Clone, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct MerklePriceUpdate {
        pub message: Vec<u8>,
        pub proof: Vec<[u8; 20usize]>,
    }
    #[derive(Clone, Copy, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct DataSource {
        pub chain: u16,
        pub emitter: ::solana_program::pubkey::Pubkey,
    }
    #[derive(Clone, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct Config {
        pub governance_authority: ::solana_program::pubkey::Pubkey,
        pub target_governance_authority: Option<::solana_program::pubkey::Pubkey>,
        pub wormhole: ::solana_program::pubkey::Pubkey,
        pub valid_data_sources: Vec<DataSource>,
        pub single_update_fee_in_lamports: u64,
        pub minimum_signatures: u8,
    }
    #[derive(Clone, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct PostUpdateAtomicParams {
        pub vaa: Vec<u8>,
        pub merkle_price_update: MerklePriceUpdate,
        pub treasury_id: u8,
    }
    #[derive(Clone, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct PostUpdateParams {
        pub merkle_price_update: MerklePriceUpdate,
        pub treasury_id: u8,
    }
    #[doc = concat!(
        " ",
        "* This enum represents how many guardian signatures were checked for a Pythnet price update\n * If full, guardian quorum has been attained\n * If partial, at least config.minimum signatures have been verified, but in the case config.minimum_signatures changes in the future we also include the number of signatures that were checked"
    )]
    #[derive(Clone, Copy, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub enum VerificationLevel {
        #[doc = concat!(
            " ",
            "* This enum represents how many guardian signatures were checked for a Pythnet price update\n * If full, guardian quorum has been attained\n * If partial, at least config.minimum signatures have been verified, but in the case config.minimum_signatures changes in the future we also include the number of signatures that were checked"
        )]
        Partial { num_signatures: u8 },
        #[doc = concat!(
            " ",
            "* This enum represents how many guardian signatures were checked for a Pythnet price update\n * If full, guardian quorum has been attained\n * If partial, at least config.minimum signatures have been verified, but in the case config.minimum_signatures changes in the future we also include the number of signatures that were checked"
        )]
        Full,
    }
}
pub mod state {
    #[allow(unused_imports)]
    use super::types::*;
    #[derive(Clone, Default, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct Config {
        pub governance_authority: ::solana_program::pubkey::Pubkey,
        pub target_governance_authority: Option<::solana_program::pubkey::Pubkey>,
        pub wormhole: ::solana_program::pubkey::Pubkey,
        pub valid_data_sources: Vec<DataSource>,
        pub single_update_fee_in_lamports: u64,
        pub minimum_signatures: u8,
    }
    impl ::anchor_interface::Account for Config {
        fn discriminator() -> &'static [u8; 8] {
            &[155u8, 12u8, 170u8, 224u8, 30u8, 250u8, 204u8, 130u8]
        }
    }
    impl ::anchor_interface::AccountSerialize for Config {
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
    impl ::anchor_interface::AccountDeserialize for Config {
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
    #[derive(Clone, Copy, ::borsh::BorshDeserialize, ::borsh::BorshSerialize)]
    #[derive(Debug)]
    pub struct PriceUpdateV2 {
        pub write_authority: ::solana_program::pubkey::Pubkey,
        pub verification_level: VerificationLevel,
        pub price_message: PriceFeedMessage,
        pub posted_slot: u64,
    }
    impl ::anchor_interface::Account for PriceUpdateV2 {
        fn discriminator() -> &'static [u8; 8] {
            &[34u8, 241u8, 35u8, 99u8, 157u8, 126u8, 244u8, 205u8]
        }
    }
    impl ::anchor_interface::AccountSerialize for PriceUpdateV2 {
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
    impl ::anchor_interface::AccountDeserialize for PriceUpdateV2 {
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
    pub enum PythSolanaReceiverError {
        #[error("Received an invalid wormhole message")]
        InvalidWormholeMessage = 6000u32,
        #[error("An error occurred when deserializing the message")]
        DeserializeMessageFailed = 6001u32,
        #[error("Received an invalid price update")]
        InvalidPriceUpdate = 6002u32,
        #[error("This type of message is not supported currently")]
        UnsupportedMessageType = 6003u32,
        #[error(
            "The tuple emitter chain, emitter doesn't match one of the valid data sources."
        )]
        InvalidDataSource = 6004u32,
        #[error("Funds are insufficient to pay the receiving fee")]
        InsufficientFunds = 6005u32,
        #[error("This signer can't write to price update account")]
        WrongWriteAuthority = 6006u32,
        #[error("The posted VAA account has the wrong owner.")]
        WrongVaaOwner = 6007u32,
        #[error("An error occurred when deserializing the VAA.")]
        DeserializeVaaFailed = 6008u32,
        #[error("The number of guardian signatures is below the minimum")]
        InsufficientGuardianSignatures = 6009u32,
        #[error("Invalid VAA version")]
        InvalidVaaVersion = 6010u32,
        #[error("Guardian set version in the VAA doesn't match the guardian set passed")]
        GuardianSetMismatch = 6011u32,
        #[error("Guardian signature indices must be increasing")]
        InvalidGuardianOrder = 6012u32,
        #[error("Guardian index exceeds the number of guardians in the set")]
        InvalidGuardianIndex = 6013u32,
        #[error("A VAA signature is invalid")]
        InvalidSignature = 6014u32,
        #[error("The recovered guardian public key doesn't match the guardian set")]
        InvalidGuardianKeyRecovery = 6015u32,
        #[error("The guardian set account is owned by the wrong program")]
        WrongGuardianSetOwner = 6016u32,
        #[error("The Guardian Set account doesn't match the PDA derivation")]
        InvalidGuardianSetPda = 6017u32,
        #[error("The Guardian Set is expired")]
        GuardianSetExpired = 6018u32,
        #[error("The signer is not authorized to perform this governance action")]
        GovernanceAuthorityMismatch = 6019u32,
        #[error("The signer is not authorized to accept the governance authority")]
        TargetGovernanceAuthorityMismatch = 6020u32,
        #[error("The governance authority needs to request a transfer first")]
        NonexistentGovernanceAuthorityTransferRequest = 6021u32,
    }
    impl DecodeError<PythSolanaReceiverError> for PythSolanaReceiverError {
        fn type_of() -> &'static str {
            "PythSolanaReceiverError"
        }
    }
    impl From<PythSolanaReceiverError> for ProgramError {
        fn from(err: PythSolanaReceiverError) -> Self {
            Self::Custom(err as u32)
        }
    }
}
