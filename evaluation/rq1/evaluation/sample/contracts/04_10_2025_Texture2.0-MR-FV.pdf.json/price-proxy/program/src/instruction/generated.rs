#![allow(unexpected_cfgs)]
use super::*;
///[PriceProxyInstruction::CreatePriceFeed] Builder struct
pub struct CreatePriceFeed {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Price-feed account to create.
    pub price_feed: solana_program::pubkey::Pubkey,
    ///Price-feed update authority. Will fund account.
    pub authority: solana_program::pubkey::Pubkey,
    ///When source is Pyth - pass feed address. Get Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta. After use cli::get_feed_id_from_hex to convert to pubkey.
    ///When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.
    ///When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.
    ///When source is Superlendy - pass reserve address.
    ///When source is OffChain - pass `authority`.
    pub source_address: solana_program::pubkey::Pubkey,
    ///When source is Pyth - pass feed address. Get Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta. After use cli::get_feed_id_from_hex to convert to pubkey.
    ///When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.
    ///When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.
    ///When source is Superlendy - pass reserve address.
    ///When source is OffChain - pass `authority`.
    ///For Transform feed type only. If type is Direct - pass source_address here.
    pub transform_source_address: solana_program::pubkey::Pubkey,
    pub params: PriceFeedParams,
}
impl CreatePriceFeed {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            price_feed,
            authority,
            source_address,
            transform_source_address,
            params,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = crate::ID;
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts
            .extend([solana_program::instruction::AccountMeta::new(price_feed, true)]);
        accounts
            .extend([solana_program::instruction::AccountMeta::new(authority, true)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(source_address, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(
                    transform_source_address,
                    false,
                ),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    solana_program::system_program::ID,
                    false,
                ),
            ]);
        let ix = PriceProxyInstruction::CreatePriceFeed {
            params,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[PriceProxyInstruction::WritePrice] Builder struct
pub struct WritePrice {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Price-feed account for update.
    pub price_feed: solana_program::pubkey::Pubkey,
    ///Price-feed update authority.
    pub authority: solana_program::pubkey::Pubkey,
    pub price: Decimal,
    /// UTC unix-timestamp of price
    pub price_timestamp: i64,
}
impl WritePrice {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            price_feed,
            authority,
            price,
            price_timestamp,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = crate::ID;
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts
            .extend([solana_program::instruction::AccountMeta::new(price_feed, false)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(authority, true),
            ]);
        let ix = PriceProxyInstruction::WritePrice {
            price,
            price_timestamp,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[PriceProxyInstruction::UpdatePrice] Builder struct
pub struct UpdatePrice {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Price-feed account to update.
    pub price_feed: solana_program::pubkey::Pubkey,
    ///When source is Pyth - pass PriceUpdate acc address, created by Pyth` PostUpdate ix.
    ///When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.
    ///When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.
    ///When source is Superlendy - pass reserve address.
    pub source_address: solana_program::pubkey::Pubkey,
    ///When source is Pyth - pass PriceUpdate acc address, created by Pyth` PostUpdate ix.
    ///When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.
    ///When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.
    ///When source is Superlendy - pass reserve address.
    ///For Transform feed type only. If type is Direct - pass source_address here.
    pub transform_source_address: solana_program::pubkey::Pubkey,
    /// Maximum age of price in secs
    pub maximum_age_sec: u64,
}
impl UpdatePrice {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            price_feed,
            source_address,
            transform_source_address,
            maximum_age_sec,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = crate::ID;
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts
            .extend([solana_program::instruction::AccountMeta::new(price_feed, false)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(source_address, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(
                    transform_source_address,
                    false,
                ),
            ]);
        let ix = PriceProxyInstruction::UpdatePrice {
            maximum_age_sec,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[PriceProxyInstruction::AlterPriceFeed] Builder struct
pub struct AlterPriceFeed {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Price-feed account to create.
    pub price_feed: solana_program::pubkey::Pubkey,
    ///Price-feed update authority.
    pub authority: solana_program::pubkey::Pubkey,
    ///When source is Pyth - pass feed address. Get Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta. After use cli::get_feed_id_from_hex to convert to pubkey.
    ///When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.
    ///When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.
    ///When source is Superlendy - pass reserve address.
    ///When source is OffChain - pass `authority`.
    pub source_address: solana_program::pubkey::Pubkey,
    ///When source is Pyth - pass feed address. Get Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta. After use cli::get_feed_id_from_hex to convert to pubkey.
    ///When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.
    ///When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.
    ///When source is Superlendy - pass reserve address.
    ///When source is OffChain - pass `authority`.
    ///For Transform feed type only. If type is Direct - pass source_address here.
    pub transform_source_address: solana_program::pubkey::Pubkey,
    pub params: PriceFeedParams,
}
impl AlterPriceFeed {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            price_feed,
            authority,
            source_address,
            transform_source_address,
            params,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = crate::ID;
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts
            .extend([solana_program::instruction::AccountMeta::new(price_feed, false)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(authority, true),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(source_address, false),
            ]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new(
                    transform_source_address,
                    false,
                ),
            ]);
        let ix = PriceProxyInstruction::AlterPriceFeed {
            params,
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[PriceProxyInstruction::DeletePriceFeed] Builder struct
pub struct DeletePriceFeed {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
    ///Price-feed account to delete.
    pub price_feed: solana_program::pubkey::Pubkey,
    ///Price-feed update authority.
    pub authority: solana_program::pubkey::Pubkey,
}
impl DeletePriceFeed {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self {
            #[cfg(feature = "program-id-manually")]
            program_id,
            price_feed,
            authority,
        } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = crate::ID;
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts
            .extend([solana_program::instruction::AccountMeta::new(price_feed, false)]);
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(authority, true),
            ]);
        let ix = PriceProxyInstruction::DeletePriceFeed {
        };
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
///[PriceProxyInstruction::Version] Builder struct
pub struct Version {
    #[cfg(feature = "program-id-manually")]
    /// Current program ID
    pub program_id: solana_program::pubkey::Pubkey,
}
impl Version {
    #[track_caller]
    pub fn into_instruction(self) -> solana_program::instruction::Instruction {
        let Self { #[cfg(feature = "program-id-manually")] program_id } = self;
        #[cfg(not(feature = "program-id-manually"))]
        let program_id = crate::ID;
        #[allow(unused_mut)]
        let mut accounts = vec![];
        accounts
            .extend([
                solana_program::instruction::AccountMeta::new_readonly(
                    solana_program::system_program::ID,
                    false,
                ),
            ]);
        let ix = PriceProxyInstruction::Version {};
        solana_program::instruction::Instruction::new_with_borsh(
            program_id,
            &ix,
            accounts,
        )
    }
}
/// [PriceProxyInstruction::CreatePriceFeed] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct CreatePriceFeedAccountIndexes {
    pub price_feed: usize,
    pub authority: usize,
    pub source_address: usize,
    pub transform_source_address: usize,
    pub system_program: usize,
}
impl CreatePriceFeedAccountIndexes {
    pub const COUNT: usize = 5usize;
    pub const PRICE_FEED: usize = 0usize;
    pub const AUTHORITY: usize = 1usize;
    pub const SOURCE_ADDRESS: usize = 2usize;
    pub const TRANSFORM_SOURCE_ADDRESS: usize = 3usize;
    pub const SYSTEM_PROGRAM: usize = 4usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            price_feed: iter.next().unwrap(),
            authority: iter.next().unwrap(),
            source_address: iter.next().unwrap(),
            transform_source_address: iter.next().unwrap(),
            system_program: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            price_feed: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            source_address: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            transform_source_address: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            system_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
        })
    }
}
impl<'a> TryFrom<&'a [u8]> for CreatePriceFeedAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for CreatePriceFeedAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for CreatePriceFeedAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for CreatePriceFeedAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [PriceProxyInstruction::WritePrice] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct WritePriceAccountIndexes {
    pub price_feed: usize,
    pub authority: usize,
}
impl WritePriceAccountIndexes {
    pub const COUNT: usize = 2usize;
    pub const PRICE_FEED: usize = 0usize;
    pub const AUTHORITY: usize = 1usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            price_feed: iter.next().unwrap(),
            authority: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            price_feed: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
        })
    }
}
impl<'a> TryFrom<&'a [u8]> for WritePriceAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for WritePriceAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for WritePriceAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for WritePriceAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [PriceProxyInstruction::UpdatePrice] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct UpdatePriceAccountIndexes {
    pub price_feed: usize,
    pub source_address: usize,
    pub transform_source_address: usize,
}
impl UpdatePriceAccountIndexes {
    pub const COUNT: usize = 3usize;
    pub const PRICE_FEED: usize = 0usize;
    pub const SOURCE_ADDRESS: usize = 1usize;
    pub const TRANSFORM_SOURCE_ADDRESS: usize = 2usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            price_feed: iter.next().unwrap(),
            source_address: iter.next().unwrap(),
            transform_source_address: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            price_feed: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            source_address: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            transform_source_address: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
        })
    }
}
impl<'a> TryFrom<&'a [u8]> for UpdatePriceAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for UpdatePriceAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for UpdatePriceAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for UpdatePriceAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [PriceProxyInstruction::AlterPriceFeed] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct AlterPriceFeedAccountIndexes {
    pub price_feed: usize,
    pub authority: usize,
    pub source_address: usize,
    pub transform_source_address: usize,
}
impl AlterPriceFeedAccountIndexes {
    pub const COUNT: usize = 4usize;
    pub const PRICE_FEED: usize = 0usize;
    pub const AUTHORITY: usize = 1usize;
    pub const SOURCE_ADDRESS: usize = 2usize;
    pub const TRANSFORM_SOURCE_ADDRESS: usize = 3usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            price_feed: iter.next().unwrap(),
            authority: iter.next().unwrap(),
            source_address: iter.next().unwrap(),
            transform_source_address: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            price_feed: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            source_address: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            transform_source_address: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
        })
    }
}
impl<'a> TryFrom<&'a [u8]> for AlterPriceFeedAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for AlterPriceFeedAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for AlterPriceFeedAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for AlterPriceFeedAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [PriceProxyInstruction::DeletePriceFeed] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct DeletePriceFeedAccountIndexes {
    pub price_feed: usize,
    pub authority: usize,
}
impl DeletePriceFeedAccountIndexes {
    pub const COUNT: usize = 2usize;
    pub const PRICE_FEED: usize = 0usize;
    pub const AUTHORITY: usize = 1usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            price_feed: iter.next().unwrap(),
            authority: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            price_feed: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
            authority: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
        })
    }
}
impl<'a> TryFrom<&'a [u8]> for DeletePriceFeedAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for DeletePriceFeedAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for DeletePriceFeedAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for DeletePriceFeedAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
/// [PriceProxyInstruction::Version] instruction account indexes helper
#[derive(Debug, PartialEq)]
pub struct VersionAccountIndexes {
    pub system_program: usize,
}
impl VersionAccountIndexes {
    pub const COUNT: usize = 1usize;
    pub const SYSTEM_PROGRAM: usize = 0usize;
    pub fn new_direct_order() -> Self {
        let mut iter = std::iter::repeat(()).enumerate().map(|(idx, ())| idx);
        Self {
            system_program: iter.next().unwrap(),
        }
    }
    pub fn try_from_indexes<'a>(
        indexes: impl IntoIterator<Item = &'a u8>,
    ) -> Result<Self, usize> {
        let mut iter = indexes.into_iter().map(|idx| (*idx) as usize);
        let mut idx = 0_usize;
        Ok(Self {
            system_program: {
                idx += 1;
                iter.next().ok_or(idx - 1)?
            },
        })
    }
}
impl<'a> TryFrom<&'a [u8]> for VersionAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<'a, const N: usize> TryFrom<&'a [u8; N]> for VersionAccountIndexes {
    type Error = usize;
    fn try_from(indexes: &'a [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(indexes)
    }
}
impl<const N: usize> TryFrom<[u8; N]> for VersionAccountIndexes {
    type Error = usize;
    fn try_from(indexes: [u8; N]) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
impl TryFrom<Vec<u8>> for VersionAccountIndexes {
    type Error = usize;
    fn try_from(indexes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from_indexes(&indexes)
    }
}
///[PriceProxyInstruction::CreatePriceFeed] instruction account infos helper
#[derive(Debug)]
pub struct CreatePriceFeedAccounts<'a, 'i> {
    ///Price-feed account to create.
    pub price_feed: &'a solana_program::account_info::AccountInfo<'i>,
    ///Price-feed update authority. Will fund account.
    pub authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///When source is Pyth - pass feed address. Get Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta. After use cli::get_feed_id_from_hex to convert to pubkey.
    ///When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.
    ///When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.
    ///When source is Superlendy - pass reserve address.
    ///When source is OffChain - pass `authority`.
    pub source_address: &'a solana_program::account_info::AccountInfo<'i>,
    ///When source is Pyth - pass feed address. Get Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta. After use cli::get_feed_id_from_hex to convert to pubkey.
    ///When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.
    ///When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.
    ///When source is Superlendy - pass reserve address.
    ///When source is OffChain - pass `authority`.
    ///For Transform feed type only. If type is Direct - pass source_address here.
    pub transform_source_address: &'a solana_program::account_info::AccountInfo<'i>,
    ///System program.
    pub system_program: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> CreatePriceFeedAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let price_feed = texture_common::utils::next_account_info(iter)?;
        let authority = texture_common::utils::next_account_info(iter)?;
        let source_address = texture_common::utils::next_account_info(iter)?;
        let transform_source_address = texture_common::utils::next_account_info(iter)?;
        let system_program = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &crate::ID,
            "self_program_id",
        )?;
        if !price_feed.is_writable {
            solana_program::msg!(concat!(stringify!(price_feed), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*price_feed.key).into());
        }
        if !price_feed.is_signer {
            return Err(texture_common::error::MissingSignature(*price_feed.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            price_feed.owner,
            &solana_program::system_program::ID,
            concat!(stringify!(price_feed), " owner"),
        )?;
        if !authority.is_writable {
            solana_program::msg!(concat!(stringify!(authority), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*authority.key).into());
        }
        if !authority.is_signer {
            return Err(texture_common::error::MissingSignature(*authority.key).into());
        }
        if !source_address.is_writable {
            solana_program::msg!(
                concat!(stringify!(source_address), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*source_address.key).into(),
            );
        }
        if !transform_source_address.is_writable {
            solana_program::msg!(
                concat!(stringify!(transform_source_address), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*transform_source_address.key)
                    .into(),
            );
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            system_program.key,
            &solana_program::system_program::ID,
            stringify!(system_program),
        )?;
        Ok(Self {
            price_feed,
            authority,
            source_address,
            transform_source_address,
            system_program,
        })
    }
}
///[PriceProxyInstruction::WritePrice] instruction account infos helper
#[derive(Debug)]
pub struct WritePriceAccounts<'a, 'i> {
    ///Price-feed account for update.
    pub price_feed: &'a solana_program::account_info::AccountInfo<'i>,
    ///Price-feed update authority.
    pub authority: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> WritePriceAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let price_feed = texture_common::utils::next_account_info(iter)?;
        let authority = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &crate::ID,
            "self_program_id",
        )?;
        if !price_feed.is_writable {
            solana_program::msg!(concat!(stringify!(price_feed), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*price_feed.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            price_feed.owner,
            &__self_program_id__,
            concat!(stringify!(price_feed), " owner"),
        )?;
        if !authority.is_signer {
            return Err(texture_common::error::MissingSignature(*authority.key).into());
        }
        Ok(Self { price_feed, authority })
    }
}
///[PriceProxyInstruction::UpdatePrice] instruction account infos helper
#[derive(Debug)]
pub struct UpdatePriceAccounts<'a, 'i> {
    ///Price-feed account to update.
    pub price_feed: &'a solana_program::account_info::AccountInfo<'i>,
    ///When source is Pyth - pass PriceUpdate acc address, created by Pyth` PostUpdate ix.
    ///When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.
    ///When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.
    ///When source is Superlendy - pass reserve address.
    pub source_address: &'a solana_program::account_info::AccountInfo<'i>,
    ///When source is Pyth - pass PriceUpdate acc address, created by Pyth` PostUpdate ix.
    ///When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.
    ///When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.
    ///When source is Superlendy - pass reserve address.
    ///For Transform feed type only. If type is Direct - pass source_address here.
    pub transform_source_address: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> UpdatePriceAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let price_feed = texture_common::utils::next_account_info(iter)?;
        let source_address = texture_common::utils::next_account_info(iter)?;
        let transform_source_address = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &crate::ID,
            "self_program_id",
        )?;
        if !price_feed.is_writable {
            solana_program::msg!(concat!(stringify!(price_feed), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*price_feed.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            price_feed.owner,
            &__self_program_id__,
            concat!(stringify!(price_feed), " owner"),
        )?;
        if !source_address.is_writable {
            solana_program::msg!(
                concat!(stringify!(source_address), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*source_address.key).into(),
            );
        }
        if !transform_source_address.is_writable {
            solana_program::msg!(
                concat!(stringify!(transform_source_address), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*transform_source_address.key)
                    .into(),
            );
        }
        Ok(Self {
            price_feed,
            source_address,
            transform_source_address,
        })
    }
}
///[PriceProxyInstruction::AlterPriceFeed] instruction account infos helper
#[derive(Debug)]
pub struct AlterPriceFeedAccounts<'a, 'i> {
    ///Price-feed account to create.
    pub price_feed: &'a solana_program::account_info::AccountInfo<'i>,
    ///Price-feed update authority.
    pub authority: &'a solana_program::account_info::AccountInfo<'i>,
    ///When source is Pyth - pass feed address. Get Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta. After use cli::get_feed_id_from_hex to convert to pubkey.
    ///When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.
    ///When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.
    ///When source is Superlendy - pass reserve address.
    ///When source is OffChain - pass `authority`.
    pub source_address: &'a solana_program::account_info::AccountInfo<'i>,
    ///When source is Pyth - pass feed address. Get Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta. After use cli::get_feed_id_from_hex to convert to pubkey.
    ///When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.
    ///When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.
    ///When source is Superlendy - pass reserve address.
    ///When source is OffChain - pass `authority`.
    ///For Transform feed type only. If type is Direct - pass source_address here.
    pub transform_source_address: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> AlterPriceFeedAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let price_feed = texture_common::utils::next_account_info(iter)?;
        let authority = texture_common::utils::next_account_info(iter)?;
        let source_address = texture_common::utils::next_account_info(iter)?;
        let transform_source_address = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &crate::ID,
            "self_program_id",
        )?;
        if !price_feed.is_writable {
            solana_program::msg!(concat!(stringify!(price_feed), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*price_feed.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            price_feed.owner,
            &__self_program_id__,
            concat!(stringify!(price_feed), " owner"),
        )?;
        if !authority.is_signer {
            return Err(texture_common::error::MissingSignature(*authority.key).into());
        }
        if !source_address.is_writable {
            solana_program::msg!(
                concat!(stringify!(source_address), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*source_address.key).into(),
            );
        }
        if !transform_source_address.is_writable {
            solana_program::msg!(
                concat!(stringify!(transform_source_address), " is not writable")
            );
            return Err(
                texture_common::error::InvalidAccount(*transform_source_address.key)
                    .into(),
            );
        }
        Ok(Self {
            price_feed,
            authority,
            source_address,
            transform_source_address,
        })
    }
}
///[PriceProxyInstruction::DeletePriceFeed] instruction account infos helper
#[derive(Debug)]
pub struct DeletePriceFeedAccounts<'a, 'i> {
    ///Price-feed account to delete.
    pub price_feed: &'a solana_program::account_info::AccountInfo<'i>,
    ///Price-feed update authority.
    pub authority: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> DeletePriceFeedAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let price_feed = texture_common::utils::next_account_info(iter)?;
        let authority = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &crate::ID,
            "self_program_id",
        )?;
        if !price_feed.is_writable {
            solana_program::msg!(concat!(stringify!(price_feed), " is not writable"));
            return Err(texture_common::error::InvalidAccount(*price_feed.key).into());
        }
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            price_feed.owner,
            &__self_program_id__,
            concat!(stringify!(price_feed), " owner"),
        )?;
        if !authority.is_signer {
            return Err(texture_common::error::MissingSignature(*authority.key).into());
        }
        Ok(Self { price_feed, authority })
    }
}
///[PriceProxyInstruction::Version] instruction account infos helper
#[derive(Debug)]
pub struct VersionAccounts<'a, 'i> {
    ///System Program.
    pub system_program: &'a solana_program::account_info::AccountInfo<'i>,
}
impl<'a, 'i> VersionAccounts<'a, 'i> {
    pub fn from_iter<I>(
        iter: &mut I,
        program_id: &solana_program::pubkey::Pubkey,
    ) -> std::result::Result<Self, texture_common::macros::accounts::AccountParseError>
    where
        I: Iterator<Item = &'a solana_program::account_info::AccountInfo<'i>>,
    {
        let __self_program_id__ = program_id;
        let system_program = texture_common::utils::next_account_info(iter)?;
        #[cfg(not(feature = "program-id-manually"))] #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            __self_program_id__,
            &crate::ID,
            "self_program_id",
        )?;
        #[allow(clippy::needless_borrow)]
        texture_common::utils::verify_key(
            system_program.key,
            &solana_program::system_program::ID,
            stringify!(system_program),
        )?;
        Ok(Self { system_program })
    }
}
pub(crate) mod ix_docs {
    macro_rules! create_price_feed {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable, signer\\]", "</b> ", "Price-feed account to create.", "\n",
            " ", "\n", "<b><i>", "1", "</i></b>. <b>", "\\[writable, signer\\]", "</b> ",
            "Price-feed update authority. Will fund account.", "\n", " ", "\n", "<b><i>",
            "2", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "When source is Pyth - pass feed address. Get Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta. After use cli::get_feed_id_from_hex to convert to pubkey.",
            "\n",
            "When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.",
            "\n",
            "When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.",
            "\n", "When source is Superlendy - pass reserve address.", "\n",
            "When source is OffChain - pass `authority`.", "\n", " ", "\n", "<b><i>",
            "3", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "When source is Pyth - pass feed address. Get Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta. After use cli::get_feed_id_from_hex to convert to pubkey.",
            "\n",
            "When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.",
            "\n",
            "When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.",
            "\n", "When source is Superlendy - pass reserve address.", "\n",
            "When source is OffChain - pass `authority`.", "\n",
            "For Transform feed type only. If type is Direct - pass source_address here.",
            "\n", " ", "\n", "<b><i>", "4", "</i></b>. <b>", "\\[\\]", "</b> ",
            "System program.", "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [CreatePriceFeed]", " ",
            "(method [into_instruction][CreatePriceFeed::into_instruction]).", " ",
            "\n\n", " ",
            "For parse accounts infos from processor use struct [CreatePriceFeedAccounts]",
            " ", "(method [from_iter][CreatePriceFeedAccounts::from_iter]).", " ",
            "\n\n", " ",
            "For work with account indexes use struct [CreatePriceFeedAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use create_price_feed;
    macro_rules! write_price {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "Price-feed account for update.", "\n", " ", "\n",
            "<b><i>", "1", "</i></b>. <b>", "\\[signer\\]", "</b> ",
            "Price-feed update authority.", "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [WritePrice]", " ",
            "(method [into_instruction][WritePrice::into_instruction]).", " ", "\n\n",
            " ",
            "For parse accounts infos from processor use struct [WritePriceAccounts]",
            " ", "(method [from_iter][WritePriceAccounts::from_iter]).", " ", "\n\n",
            " ", "For work with account indexes use struct [WritePriceAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use write_price;
    macro_rules! update_price {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "Price-feed account to update.", "\n", " ", "\n",
            "<b><i>", "1", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "When source is Pyth - pass PriceUpdate acc address, created by Pyth` PostUpdate ix.",
            "\n",
            "When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.",
            "\n",
            "When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.",
            "\n", "When source is Superlendy - pass reserve address.", "\n", " ", "\n",
            "<b><i>", "2", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "When source is Pyth - pass PriceUpdate acc address, created by Pyth` PostUpdate ix.",
            "\n",
            "When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.",
            "\n",
            "When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.",
            "\n", "When source is Superlendy - pass reserve address.", "\n",
            "For Transform feed type only. If type is Direct - pass source_address here.",
            "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [UpdatePrice]", " ",
            "(method [into_instruction][UpdatePrice::into_instruction]).", " ", "\n\n",
            " ",
            "For parse accounts infos from processor use struct [UpdatePriceAccounts]",
            " ", "(method [from_iter][UpdatePriceAccounts::from_iter]).", " ", "\n\n",
            " ", "For work with account indexes use struct [UpdatePriceAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use update_price;
    macro_rules! alter_price_feed {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "Price-feed account to create.", "\n", " ", "\n",
            "<b><i>", "1", "</i></b>. <b>", "\\[signer\\]", "</b> ",
            "Price-feed update authority.", "\n", " ", "\n", "<b><i>", "2",
            "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "When source is Pyth - pass feed address. Get Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta. After use cli::get_feed_id_from_hex to convert to pubkey.",
            "\n",
            "When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.",
            "\n",
            "When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.",
            "\n", "When source is Superlendy - pass reserve address.", "\n",
            "When source is OffChain - pass `authority`.", "\n", " ", "\n", "<b><i>",
            "3", "</i></b>. <b>", "\\[writable\\]", "</b> ",
            "When source is Pyth - pass feed address. Get Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta. After use cli::get_feed_id_from_hex to convert to pubkey.",
            "\n",
            "When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.",
            "\n",
            "When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.",
            "\n", "When source is Superlendy - pass reserve address.", "\n",
            "When source is OffChain - pass `authority`.", "\n",
            "For Transform feed type only. If type is Direct - pass source_address here.",
            "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [AlterPriceFeed]", " ",
            "(method [into_instruction][AlterPriceFeed::into_instruction]).", " ",
            "\n\n", " ",
            "For parse accounts infos from processor use struct [AlterPriceFeedAccounts]",
            " ", "(method [from_iter][AlterPriceFeedAccounts::from_iter]).", " ", "\n\n",
            " ",
            "For work with account indexes use struct [AlterPriceFeedAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use alter_price_feed;
    macro_rules! delete_price_feed {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[writable\\]", "</b> ", "Price-feed account to delete.", "\n", " ", "\n",
            "<b><i>", "1", "</i></b>. <b>", "\\[signer\\]", "</b> ",
            "Price-feed update authority.", "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [DeletePriceFeed]", " ",
            "(method [into_instruction][DeletePriceFeed::into_instruction]).", " ",
            "\n\n", " ",
            "For parse accounts infos from processor use struct [DeletePriceFeedAccounts]",
            " ", "(method [from_iter][DeletePriceFeedAccounts::from_iter]).", " ",
            "\n\n", " ",
            "For work with account indexes use struct [DeletePriceFeedAccountIndexes].",
            "\n", }
        };
    }
    pub(crate) use delete_price_feed;
    macro_rules! version {
        () => {
            concat! { " ## Accounts", "\n", " ", "\n", "<b><i>", "0", "</i></b>. <b>",
            "\\[\\]", "</b> ", "System Program.", "\n", "\n", " ## Usage", "\n", " ",
            "For create instruction use builder struct [Version]", " ",
            "(method [into_instruction][Version::into_instruction]).", " ", "\n\n", " ",
            "For parse accounts infos from processor use struct [VersionAccounts]", " ",
            "(method [from_iter][VersionAccounts::from_iter]).", " ", "\n\n", " ",
            "For work with account indexes use struct [VersionAccountIndexes].", "\n", }
        };
    }
    pub(crate) use version;
}
