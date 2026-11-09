use std::{fmt::Display, path::PathBuf, str::FromStr, time::Duration};

use derive_more::FromStr;
use solana_sdk::{commitment_config::CommitmentLevel, pubkey::Pubkey};
use structopt::StructOpt;

use price_proxy::state::price_feed::{
    FeedType, PriceFeedSource, QuoteSymbol, WormholeVerificationLevel,
};
use texture_common::math::Decimal;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Opts {
    /// URL of RPC Solana interface.
    #[structopt(
        long,
        short,
        default_value = "http://localhost:8899",
        env = "SOLANA_RPC"
    )]
    pub url: String,

    #[structopt(long, default_value = "confirmed")]
    pub commitment: CommitmentLevel,

    /// Keypair to use for signing instructions.
    #[structopt(long, short = "k", default_value)]
    pub authority: KeypairPath,

    /// Priority fee in microlamports. For priority_rate=1 you pay 0.2 (1) priority lamports for one ix, for 10_000 - 2_000.
    #[structopt(long)]
    pub priority_fee: Option<u64>,

    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum Command {
    /// Creates Price-feed account. Requires ADMIN privileges.
    CreatePriceFeed {
        /// Feed type
        #[structopt(long)]
        feed_type: FeedType,
        /// Symbol name
        #[structopt(long)]
        symbol: String,
        /// Quote Symbol name
        #[structopt(long, default_value = "USD")]
        quote_symbol: QuoteSymbol,
        /// Logo url
        #[structopt(long)]
        logo_url: String,
        /// Asset price source
        #[structopt(long)]
        source: PriceFeedSource,
        /// Asset price source. For Transform feed type only.
        #[structopt(long)]
        transform_source: Option<PriceFeedSource>,
        /// Wormhole Verification Level (for Pyth source only)
        #[structopt(long, default_value = "Full")]
        verification_level: WormholeVerificationLevel,
        /// 1. When source is Pyth - pass feed address. Use GetFeedIdFromHex
        /// 2. When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet
        /// 3. When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml
        /// 4. When source is Superlendy - pass reserve address
        /// 5. When source is OffChain - pass `authority`.
        #[structopt(long)]
        source_address: Pubkey,
        /// 1. When source is Pyth - pass feed address. Use GetFeedIdFromHex
        /// 2. When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet
        /// 3. When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml
        /// 4. When source is Superlendy - pass reserve address
        /// 5. When source is OffChain - pass `authority`.
        /// For Transform feed type only.
        #[structopt(long)]
        transform_source_address: Option<Pubkey>,
    },
    /// Print Price-feed data
    PriceFeed {
        /// Price-feed account pubkey
        key: Pubkey,
    },
    /// Print all Price-feed datas
    PriceFeeds {
        /// Price-feed account pubkey. Can be specified multiple times
        #[structopt(long)]
        key: Vec<Pubkey>,
    },
    /// Write current price for off-chain Price-feed
    WritePrice {
        /// Price-feed account pubkey
        key: Pubkey,
        /// Current price
        price: Decimal,
    },
    /// Update Price-feed from Pyth or Switchboard feed
    UpdatePrice {
        /// Price-feed account pubkey
        key: Pubkey,
        /// Maximum age of price in secs
        #[structopt(long)]
        maximum_age_sec: u64,
        /// URL of Pyth API to receive the message
        #[structopt(long)]
        pyth_api_url: Option<String>,
    },
    /// Force refresh update-timestamp of Price-feed(s). Off-chain source only
    ForcePriceTimestamp {
        /// Auto-fluush period
        #[structopt(long, parse(try_from_str = parse_humantime_duration))]
        period: Option<Duration>,
        /// Price-feed account pubkey. Can be specified multiple times
        #[structopt(long)]
        key: Vec<Pubkey>,
    },
    /// Get FeedID from hex
    GetFeedIdFromHex {
        /// Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta
        #[structopt(long)]
        hex: String,
    },
    /// Update Price-feed account.
    AlterPriceFeed {
        /// Price-feed account pubkey
        key: Pubkey,
        /// Feed type
        #[structopt(long)]
        feed_type: Option<FeedType>,
        /// Symbol name
        #[structopt(long)]
        symbol: Option<String>,
        /// Quote Symbol name
        #[structopt(long)]
        quote_symbol: Option<QuoteSymbol>,
        /// Wormhole Verification Level (for Pyth source only)
        #[structopt(long)]
        verification_level: Option<WormholeVerificationLevel>,
        /// Logo url
        #[structopt(long)]
        logo_url: Option<String>,
        /// Asset price source
        #[structopt(long)]
        source: Option<PriceFeedSource>,
        /// Asset price source. For Transform feed type only.
        #[structopt(long)]
        transform_source: Option<PriceFeedSource>,
        /// Asset price source address
        /// 1. When source is Pyth - pass feed address. Use GetFeedIdFromHex
        /// 2. When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet
        /// 3. When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml
        /// 4. When source is Superlendy - pass reserve address
        /// 5. When source is OffChain - pass `authority`.
        #[structopt(long)]
        source_address: Option<Pubkey>,
        /// Asset price transform source address
        /// 1. When source is Pyth - pass feed address. Use GetFeedIdFromHex
        /// 2. When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet
        /// 3. When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml
        /// 4. When source is Superlendy - pass reserve address
        /// 5. When source is OffChain - pass `authority`.
        /// For Transform feed type only.
        #[structopt(long)]
        transform_source_address: Option<Pubkey>,
    },
    /// Delete Price-feed account.
    DeletePriceFeed {
        /// Price-feed account pubkey
        key: Pubkey,
    },
    ShowStakePoolPrice {
        /// StakePool account pubkey
        key: Pubkey,
        /// Token symbol
        symbol: String,
    },
    /// Get contract version
    ContractVersion {},
}

#[derive(FromStr)]
pub struct KeypairPath(pub PathBuf);

impl Default for KeypairPath {
    fn default() -> Self {
        let mut path = dirs_next::home_dir().expect("home dir");
        path.extend([".config", "solana", "id.json"]);
        Self(path)
    }
}

impl Display for KeypairPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_str().expect("non unicode"))
    }
}

fn parse_humantime_duration(src: &str) -> Result<Duration, humantime::DurationError> {
    humantime::Duration::from_str(src).map(Into::into)
}
