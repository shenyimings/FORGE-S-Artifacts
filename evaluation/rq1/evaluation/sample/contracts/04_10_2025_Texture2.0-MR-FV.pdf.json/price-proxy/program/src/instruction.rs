use borsh::{BorshDeserialize, BorshSerialize};

use texture_common::macros::Instruction;
use texture_common::math::Decimal;

use crate::state::price_feed::PriceFeedParams;

#[derive(Instruction, BorshSerialize, BorshDeserialize, Debug)]
#[instruction(
    out_dir = "src/instruction",
    out_mod = "generated",
    program_id = crate::ID,
    docs_module = ix_docs
)]
pub enum PriceProxyInstruction {
    /// Create Price-feed account
    ///
    #[doc = ix_docs::create_price_feed!()]
    #[accounts(
        account(
            name = "price_feed",
            flags(writable, signer),
            checks(owner = "system"),
            docs = ["Price-feed account to create."]
        ),
        account(
            name = "authority",
            flags(writable, signer),
            docs = ["Price-feed update authority. Will fund account."],
        ),
        account(
            name = "source_address",
            flags(writable),
            docs = [
                "When source is Pyth - pass feed address. Get Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta. After use cli::get_feed_id_from_hex to convert to pubkey.",
                "When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.",
                "When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.",
                "When source is Superlendy - pass reserve address.",
                "When source is OffChain - pass `authority`.",
            ],
        ),
        account(
            name = "transform_source_address",
            flags(writable),
            docs = [
                "When source is Pyth - pass feed address. Get Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta. After use cli::get_feed_id_from_hex to convert to pubkey.",
                "When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.",
                "When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.",
                "When source is Superlendy - pass reserve address.",
                "When source is OffChain - pass `authority`.",
                "For Transform feed type only. If type is Direct - pass source_address here.",
            ],
        ),
        program(id = "system", docs = ["System program."])
    )]
    CreatePriceFeed { params: PriceFeedParams },
    /// Write price for off-chain Price-feed
    ///
    #[doc = ix_docs::write_price!()]
    #[accounts(
        account(
            name = "price_feed",
            flags(writable),
            checks(owner = "self"),
            docs = ["Price-feed account for update."]
        ),
        account(
            name = "authority",
            flags(signer),
            docs = ["Price-feed update authority."],
        )
    )]
    WritePrice {
        price: Decimal,
        /// UTC unix-timestamp of price
        price_timestamp: i64,
    },
    /// Update price for Pyth, Switchboard Price-feeds
    ///
    #[doc = ix_docs::update_price!()]
    #[accounts(
        account(
            name = "price_feed",
            flags(writable),
            checks(owner = "self"),
            docs = ["Price-feed account to update."]
        ),
        account(
            name = "source_address",
            flags(writable),
            docs = [
                "When source is Pyth - pass PriceUpdate acc address, created by Pyth` PostUpdate ix.",
                "When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.",
                "When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.",
                "When source is Superlendy - pass reserve address.",
            ],
        ),
        account(
            name = "transform_source_address",
            flags(writable),
            docs = [
                "When source is Pyth - pass PriceUpdate acc address, created by Pyth` PostUpdate ix.",
                "When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.",
                "When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.",
                "When source is Superlendy - pass reserve address.",
                "For Transform feed type only. If type is Direct - pass source_address here.",
            ],
        ),
    )]
    UpdatePrice {
        /// Maximum age of price in secs
        maximum_age_sec: u64,
    },
    /// Alter Price-feed account
    ///
    #[doc = ix_docs::alter_price_feed!()]
    #[accounts(
        account(
            name = "price_feed",
            flags(writable),
            checks(owner = "self"),
            docs = ["Price-feed account to create."]
        ),
        account(
            name = "authority",
            flags(signer),
            docs = ["Price-feed update authority."],
        ),
        account(
            name = "source_address",
            flags(writable),
            docs = [
                "When source is Pyth - pass feed address. Get Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta. After use cli::get_feed_id_from_hex to convert to pubkey.",
                "When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.",
                "When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.",
                "When source is Superlendy - pass reserve address.",
                "When source is OffChain - pass `authority`.",
            ],
        ),
        account(
            name = "transform_source_address",
            flags(writable),
            docs = [
                "When source is Pyth - pass feed address. Get Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta. After use cli::get_feed_id_from_hex to convert to pubkey.",
                "When source is Switchboard - pass feed address. Get from https://app.switchboard.xyz/solana/mainnet.",
                "When source is StakePool - pass feed address. Get from sanctum_lst_list.pool https://github.com/igneous-labs/sanctum-lst-list/blob/master/sanctum-lst-list.toml.",
                "When source is Superlendy - pass reserve address.",
                "When source is OffChain - pass `authority`.",
            "For Transform feed type only. If type is Direct - pass source_address here.",
            ],
        ),
    )]
    AlterPriceFeed { params: PriceFeedParams },
    /// Delete Price-feed account
    ///
    #[doc = ix_docs::delete_price_feed!()]
    #[accounts(
    account(
        name = "price_feed",
        flags(writable),
        checks(owner = "self"),
        docs = ["Price-feed account to delete."]
    ),
    account(
        name = "authority",
        flags(signer),
        docs = ["Price-feed update authority."],
    ),
    )]
    DeletePriceFeed,
    /// Always fails but prints contact version in to returned logs
    ///
    #[doc = ix_docs::version!()]
    #[accounts(
        program(
            docs = ["System Program."],
            id = "system",
        ),
    )]
    Version,
}
