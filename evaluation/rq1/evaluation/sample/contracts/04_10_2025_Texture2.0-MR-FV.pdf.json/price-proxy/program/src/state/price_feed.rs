use std::{convert::Infallible, str::FromStr};

use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use derive_more::Display;
use num_enum::{FromPrimitive, IntoPrimitive};
use solana_program::pubkey::Pubkey;

use texture_common::{
    account::{PodAccount, PodAccountError},
    math::{Decimal, MathResult},
};

use crate::state::PRICE_FEED_DISCRIMINATOR;

#[cfg(feature = "with-serde")]
use super::utils::with_serde::{
    array_as_str_serde, decimal_bits_serde, display_from_str_serde, timestamp_as_datetime_serde,
    As, Deserialize, DisplayAsJsonPretty, DisplayFromStr, FromInto, Serialize,
};

pub const SYMBOL_MAX_SIZE: usize = 16;
pub const LOGO_URL_MAX_LEN: usize = 128;

static_assertions::const_assert_eq!(PriceFeed::SIZE, std::mem::size_of::<PriceFeed>());
static_assertions::const_assert_eq!(0, std::mem::size_of::<PriceFeed>() % 8);

super::utils::source_enum_from_str_derive_infallibale!(
    #[derive(
        Debug,
        Display,
        Clone,
        Copy,
        PartialEq,
        BorshSerialize,
        BorshDeserialize,
        FromPrimitive,
        IntoPrimitive,
    )]
    #[borsh(use_discriminant = false)]
    #[repr(u8)]
    pub enum PriceFeedSource {
        #[num_enum(default)]
        Unknown = 0,
        #[from_str("o", "off-chain", "offchain")]
        OffChain,
        #[from_str("p", "pyth")]
        Pyth,
        #[from_str("s", "switchboard")]
        Switchboard,
        #[from_str("l", "super-lendy", "superlendy")]
        SuperLendy,
        #[from_str("st", "stake-pool", "stakepool")]
        StakePool,
    }
);

impl<'a> From<&'a str> for PriceFeedSource {
    fn from(value: &'a str) -> Self {
        PriceFeedSource::from_str(value).unwrap()
    }
}

#[cfg(feature = "with-serde")]
impl serde::Serialize for PriceFeedSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        display_from_str_serde::serialize(self, serializer)
    }
}
#[cfg(feature = "with-serde")]
impl<'de> serde::Deserialize<'de> for PriceFeedSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        display_from_str_serde::deserialize(deserializer)
    }
}

#[derive(
    Debug,
    Display,
    Clone,
    Copy,
    PartialEq,
    BorshSerialize,
    BorshDeserialize,
    FromPrimitive,
    IntoPrimitive,
)]
#[borsh(use_discriminant = false)]
#[repr(u8)]
pub enum QuoteSymbol {
    #[num_enum(default)]
    USD = 0,
    SOL,
}

impl FromStr for QuoteSymbol {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "usd" | "u" => Ok(Self::USD),
            "sol" | "s" => Ok(Self::SOL),
            _ => Err(format!("`{}` is not a valid quote", s)),
        }
    }
}

impl<'a> From<&'a str> for QuoteSymbol {
    fn from(value: &'a str) -> Self {
        QuoteSymbol::from_str(value).unwrap()
    }
}

#[cfg(feature = "with-serde")]
impl serde::Serialize for QuoteSymbol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        display_from_str_serde::serialize(self, serializer)
    }
}
#[cfg(feature = "with-serde")]
impl<'de> serde::Deserialize<'de> for QuoteSymbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        display_from_str_serde::deserialize(deserializer)
    }
}

#[derive(
    Debug,
    Display,
    Clone,
    Copy,
    PartialEq,
    BorshSerialize,
    BorshDeserialize,
    FromPrimitive,
    IntoPrimitive,
)]
#[borsh(use_discriminant = false)]
#[repr(u8)]
pub enum WormholeVerificationLevel {
    #[num_enum(default)]
    Full = 0,
    Partial,
}

impl FromStr for WormholeVerificationLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "full" | "f" => Ok(Self::Full),
            "partial" | "p" => Ok(Self::Partial),
            _ => Err(format!("`{}` is not a valid level", s)),
        }
    }
}

impl<'a> From<&'a str> for WormholeVerificationLevel {
    fn from(value: &'a str) -> Self {
        WormholeVerificationLevel::from_str(value).unwrap()
    }
}

#[cfg(feature = "with-serde")]
impl serde::Serialize for WormholeVerificationLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        display_from_str_serde::serialize(self, serializer)
    }
}
#[cfg(feature = "with-serde")]
impl<'de> serde::Deserialize<'de> for WormholeVerificationLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        display_from_str_serde::deserialize(deserializer)
    }
}

#[derive(
    Debug,
    Display,
    Clone,
    Copy,
    PartialEq,
    BorshSerialize,
    BorshDeserialize,
    FromPrimitive,
    IntoPrimitive,
)]
#[borsh(use_discriminant = false)]
#[repr(u8)]
pub enum FeedType {
    #[num_enum(default)]
    Direct = 0,
    Transform,
}

impl FromStr for FeedType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "direct" | "f" => Ok(Self::Direct),
            "transform" | "p" => Ok(Self::Transform),
            _ => Err(format!("`{}` is not a valid type", s)),
        }
    }
}

impl<'a> From<&'a str> for FeedType {
    fn from(value: &'a str) -> Self {
        FeedType::from_str(value).unwrap()
    }
}

#[cfg(feature = "with-serde")]
impl serde::Serialize for FeedType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        display_from_str_serde::serialize(self, serializer)
    }
}
#[cfg(feature = "with-serde")]
impl<'de> serde::Deserialize<'de> for FeedType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        display_from_str_serde::deserialize(deserializer)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub struct PriceFeedParams {
    pub feed_type: FeedType,
    pub symbol: [u8; SYMBOL_MAX_SIZE],
    pub quote_symbol: QuoteSymbol,
    pub verification_level: WormholeVerificationLevel,
    pub logo_url: [u8; LOGO_URL_MAX_LEN],
    pub source: PriceFeedSource,
    pub transform_source: PriceFeedSource,
}

impl PriceFeedParams {
    pub fn new(
        feed_type: impl Into<FeedType>,
        symbol: &str,
        quote_symbol: impl Into<QuoteSymbol>,
        verification_level: impl Into<WormholeVerificationLevel>,
        logo_url: &str,
        source: impl Into<PriceFeedSource>,
        transform_source: impl Into<PriceFeedSource>,
    ) -> Self {
        Self {
            feed_type: feed_type.into(),
            symbol: super::utils::str_to_array(symbol),
            quote_symbol: quote_symbol.into(),
            verification_level: verification_level.into(),
            logo_url: super::utils::str_to_array(logo_url),
            source: source.into(),
            transform_source: transform_source.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Pod, Zeroable, PartialEq)]
#[cfg_attr(
    feature = "with-serde",
    derive(Serialize, Deserialize, DisplayAsJsonPretty),
    serde_with::serde_as
)]
#[repr(C)]
pub struct PriceFeed {
    #[cfg_attr(feature = "with-serde", serde(with = "array_as_str_serde"))]
    pub discriminator: [u8; 8],
    pub version: u8,

    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub _padding0: [u8; 2],

    /// Direct or Transform, which get price from two sources. See `FeedType`.
    #[cfg_attr(
        feature = "with-serde",
        serde(rename = "feed_type", with = "As::<FromInto<FeedType>>")
    )]
    pub feed_type: u8,

    /// Quote currency symbol (see `symbol` for more info). See `QuoteSymbol`.
    #[cfg_attr(
        feature = "with-serde",
        serde(rename = "quote_symbol", with = "As::<FromInto<QuoteSymbol>>")
    )]
    pub quote_symbol: u8,

    /// Verification level represents how much a Pyth price update has been verified. See `WormholeVerificationLevel`.
    #[cfg_attr(
        feature = "with-serde",
        serde(
            rename = "verification_level",
            with = "As::<FromInto<WormholeVerificationLevel>>"
        )
    )]
    pub verification_level: u8,

    /// source of asset price. See `PriceFeedSource` Is set when creating a price feed account.
    #[cfg_attr(
        feature = "with-serde",
        serde(rename = "source", with = "As::<FromInto<PriceFeedSource>>")
    )]
    pub source_raw: u8,

    /// source of second asset price. See `PriceFeedSource` Is set when creating a price feed account.
    /// For Transform feed type only.
    #[cfg_attr(
        feature = "with-serde",
        serde(rename = "transform_source", with = "As::<FromInto<PriceFeedSource>>")
    )]
    pub transform_source_raw: u8,

    /// address of the Pyth or Switchboard or StakePool account or SuperLendy reserve, where to get the data from.
    /// For Off-chain sources, there is an authority who has the right to update the data.
    #[cfg_attr(feature = "with-serde", serde(with = "As::<DisplayFromStr>"))]
    pub source_address: Pubkey,

    /// second address of the Pyth or Switchboard or StakePool account or SuperLendy reserve, where to get the data from.
    /// For Transform feed type only.
    #[cfg_attr(feature = "with-serde", serde(with = "As::<DisplayFromStr>"))]
    pub transform_source_address: Pubkey,

    /// a human-readable symbol of a currency or asset whose price is broadcast through this account.
    #[cfg_attr(feature = "with-serde", serde(with = "array_as_str_serde"))]
    pub symbol: [u8; SYMBOL_MAX_SIZE],

    /// Logo url
    #[cfg_attr(feature = "with-serde", serde(with = "array_as_str_serde"))]
    pub logo_url: [u8; LOGO_URL_MAX_LEN],

    /// someone who created this price feed and who can change and delete it.
    #[cfg_attr(feature = "with-serde", serde(with = "As::<DisplayFromStr>"))]
    pub update_authority: Pubkey,

    /// Price born time taken from the system/protocol used as the price source.
    #[cfg_attr(feature = "with-serde", serde(with = "timestamp_as_datetime_serde"))]
    pub update_timestamp: i64,

    pub update_slot: u64,

    /// asset price at `update_timestamp`, decimal with scale 18.
    #[cfg_attr(
        feature = "with-serde",
        serde(rename = "price", with = "decimal_bits_serde")
    )]
    pub price_raw: i128,

    #[cfg_attr(
        feature = "with-serde",
        serde(skip, default = "Zeroable::zeroed"),
        serde_as(as = "Bytes")
    )]
    pub _padding: [u8; 128],
}

impl PriceFeed {
    pub fn new(
        params: PriceFeedParams,
        update_authority: Pubkey,
        price_source: Pubkey,
        transform_price_source: Pubkey,
    ) -> Self {
        Self::from_init_params((
            params,
            update_authority,
            price_source,
            transform_price_source,
        ))
    }

    pub fn with_price(mut self, price: impl Into<Decimal>, timestamp: i64, slot: u64) -> Self {
        self.try_set_price(price, timestamp, slot).unwrap();
        self
    }

    pub fn try_price(&self) -> MathResult<Decimal> {
        Decimal::from_bits(self.price_raw)
    }

    pub fn try_set_price(
        &mut self,
        price: impl Into<Decimal>,
        timestamp: i64,
        slot: u64,
    ) -> MathResult<()> {
        self.price_raw = price.into().into_bits()?;
        self.update_timestamp = timestamp;
        self.update_slot = slot;
        Ok(())
    }

    pub fn feed_type(&self) -> FeedType {
        self.feed_type.into()
    }

    #[cfg(not(target_os = "solana"))]
    pub fn symbol(&self) -> std::borrow::Cow<'_, str> {
        super::utils::bytes_to_cow(&self.symbol)
    }

    pub fn quote_symbol(&self) -> QuoteSymbol {
        self.quote_symbol.into()
    }

    pub fn verification_level(&self) -> WormholeVerificationLevel {
        self.verification_level.into()
    }

    #[cfg(not(target_os = "solana"))]
    pub fn logo_url(&self) -> std::borrow::Cow<'_, str> {
        super::utils::bytes_to_cow(&self.logo_url)
    }

    pub fn source(&self) -> PriceFeedSource {
        self.source_raw.into()
    }

    pub fn transform_source(&self) -> PriceFeedSource {
        self.transform_source_raw.into()
    }
}

impl PodAccount for PriceFeed {
    const DISCRIMINATOR: &'static [u8] = PRICE_FEED_DISCRIMINATOR;

    type Version = u8;

    const VERSION: Self::Version = 1;

    type InitParams = (
        /*params:*/ PriceFeedParams,
        /*update_authority:*/ Pubkey,
        /*price_source:*/ Pubkey,
        /*transform_price_source:*/ Pubkey,
    );

    type InitError = PodAccountError;

    fn discriminator(&self) -> &[u8] {
        &self.discriminator
    }

    fn version(&self) -> Self::Version {
        self.version
    }

    fn init_unckecked(
        &mut self,
        (params, authority, price_source, transform_price_source): Self::InitParams,
    ) -> Result<(), Self::InitError> {
        let Self {
            discriminator,
            version,
            _padding0,
            feed_type,
            source_raw: source,
            transform_source_raw: transform_source,
            source_address,
            transform_source_address,
            symbol,
            quote_symbol,
            verification_level,
            logo_url,
            update_authority,
            update_timestamp,
            update_slot,
            price_raw,
            _padding,
        } = self;

        *discriminator = *PRICE_FEED_DISCRIMINATOR;
        *version = Self::VERSION;
        *_padding0 = Zeroable::zeroed();
        *feed_type = params.feed_type as u8;
        *source = params.source as u8;
        *transform_source = params.transform_source as u8;
        *source_address = price_source;
        *transform_source_address = transform_price_source;
        *symbol = params.symbol;
        *quote_symbol = params.quote_symbol as u8;
        *verification_level = params.verification_level as u8;
        *logo_url = params.logo_url;
        *update_authority = authority;
        *update_timestamp = 0;
        *update_slot = 0;
        *price_raw = 0;
        *_padding = Zeroable::zeroed();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "with-serde")]
    #[test]
    fn serde() {
        use dev_utils::pretty_assertions::assert_eq;

        use texture_common::dec;

        use super::*;

        let price_feed = PriceFeed::new(
            PriceFeedParams::new(
                "Direct",
                "SOL",
                "USD",
                "full",
                "/sol",
                "off-chain",
                "off-chain",
            ),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        )
        .with_price(dec!(1.001), chrono::Utc::now().timestamp(), 1);

        let json = serde_json::to_string_pretty(&price_feed).unwrap();
        println!("{json}");

        let new_price_feed: PriceFeed = serde_json::from_str(&json).unwrap();
        assert_eq!(new_price_feed, price_feed);
    }
}
