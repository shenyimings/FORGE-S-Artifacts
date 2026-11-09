use anchor_lang::AccountDeserialize;
use price_proxy_client::{PriceFeedSignatureView, PriceFeedView, SignatureView};
use std::str::FromStr;

use pretty_assertions::assert_eq;
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};
use super_lendy::state::reserve::Reserve;
use switchboard_solana::AggregatorAccountData;
use tracing::info;

use price_proxy::state::price_feed::{PriceFeed, PriceFeedParams, WormholeVerificationLevel};

mod utils;
use utils::*;

#[tokio::test]
async fn update_from_pyth_success() {
    let TestContext {
        price_proxy,
        payer: _,
    } = init_test().await;
    let authority_key = price_proxy.authority.pubkey();

    // Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta
    let hex = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
    let feed_id = get_feed_id_from_hex(hex).expect("feed_id_from_hex");
    let source_address = Pubkey::from(feed_id);

    let expected_params =
        PriceFeedParams::new("Direct", "SOL", "USD", "full", "/sol", "pyth", "pyth");
    let mut expected = PriceFeed::new(
        expected_params,
        authority_key,
        source_address,
        source_address,
    );

    // CREATE PRICE-FEED

    let PriceFeedSignatureView {
        price_feed: feed_key,
        ..
    } = price_proxy
        .create_price_feed(expected_params, source_address, source_address)
        .await
        .expect("create price-feed");

    let PriceFeedView { price_feed, .. } =
        price_proxy.price_feed(&feed_key).await.expect("price-feed");
    assert_eq!(price_feed, expected);

    // POST UPDATE AND UPDATE PRICE

    info!("post update and update price");
    let message = price_proxy
        .get_message_by_hex(hex, None)
        .await
        .expect("get message");
    let price_update = price_proxy
        .post_update(message.first().unwrap(), WormholeVerificationLevel::Full)
        .await
        .expect("update price from Pyth");
    let max_age_sec = 60;
    let SignatureView { signature: _ } = price_proxy
        .update_price(feed_key, price_update.0, price_update.0, max_age_sec)
        .await
        .expect("update price");

    let PriceFeedView { price_feed, .. } =
        price_proxy.price_feed(&feed_key).await.expect("price-feed");

    let price_update_acc = price_proxy
        .rpc
        .get_account(&price_update.0)
        .await
        .expect("get price_update acc");
    let price_update = PriceUpdateV2::try_deserialize(&mut price_update_acc.data.as_slice())
        .expect("deserialize source address");
    let pyth_price = price_update
        .get_price_unchecked(&price_update.price_message.feed_id)
        .expect("get Pyth price");
    expected
        .try_set_price(
            Decimal::from_i128_with_scale(
                pyth_price.price as i128,
                pyth_price.exponent.unsigned_abs(),
            )
            .unwrap(),
            pyth_price.publish_time,
            price_feed.update_slot,
        )
        .unwrap();

    assert_eq!(price_feed, expected);
}

#[tokio::test]
async fn update_from_pyth_incorrect_source() {
    let TestContext {
        price_proxy,
        payer: _,
    } = init_test().await;
    let authority_key = price_proxy.authority.pubkey();

    // Price Feed ID in hex from https://pyth.network/developers/price-feed-ids#solana-mainnet-beta
    let hex = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
    let feed_id = get_feed_id_from_hex(hex).expect("feed_id_from_hex");
    let source_address = Pubkey::from(feed_id);

    let expected_params =
        PriceFeedParams::new("Direct", "SOL", "USD", "full", "sol", "pyth", "pyth");
    let expected = PriceFeed::new(
        expected_params,
        authority_key,
        source_address,
        source_address,
    );

    // CREATE PRICE-FEED

    let PriceFeedSignatureView {
        price_feed: feed_key,
        ..
    } = price_proxy
        .create_price_feed(expected_params, source_address, source_address)
        .await
        .expect("create price-feed");

    let PriceFeedView { price_feed, .. } =
        price_proxy.price_feed(&feed_key).await.expect("price-feed");
    assert_eq!(price_feed, expected);

    // UPDATE PRICE WITH INCORRECT SOURCE

    info!("update price with incorrect source");
    let other_hex = "0x0a0408d619e9380abad35060f9192039ed5042fa6f82301d0e48bb52be830996";
    let message = price_proxy
        .get_message_by_hex(other_hex, None)
        .await
        .expect("get message");
    let price_update = price_proxy
        .post_update(message.first().unwrap(), WormholeVerificationLevel::Full)
        .await
        .expect("update price from Pyth");
    let max_age_sec = 60;
    let result = price_proxy
        .update_price(feed_key, price_update.0, price_update.0, max_age_sec)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn update_from_switchboard_success() {
    let TestContext {
        price_proxy,
        payer: _,
    } = init_test().await;
    let authority_key = price_proxy.authority.pubkey();

    let source_address = Pubkey::from_str("GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR").unwrap();

    let expected_params = PriceFeedParams::new(
        "Direct",
        "SOL",
        "USD",
        "full",
        "/sol",
        "switchboard",
        "switchboard",
    );
    let mut expected = PriceFeed::new(
        expected_params,
        authority_key,
        source_address,
        source_address,
    );

    // CREATE PRICE-FEED

    let PriceFeedSignatureView {
        price_feed: feed_key,
        ..
    } = price_proxy
        .create_price_feed(expected_params, source_address, source_address)
        .await
        .expect("create price-feed");

    let PriceFeedView { price_feed, .. } =
        price_proxy.price_feed(&feed_key).await.expect("price-feed");
    assert_eq!(price_feed, expected);

    // UPDATE PRICE

    let max_age_sec = 100000000; // cause feed from switchboard_sol_price.json is not updating
    let SignatureView { signature: _ } = price_proxy
        .update_price(feed_key, source_address, source_address, max_age_sec)
        .await
        .expect("update price");
    let PriceFeedView { price_feed, .. } =
        price_proxy.price_feed(&feed_key).await.expect("price-feed");
    info!(%feed_key, %price_feed);

    let source_acc = price_proxy
        .rpc
        .get_account(&source_address)
        .await
        .expect("get source acc");
    let data_feed = AggregatorAccountData::new_from_bytes(source_acc.data.as_slice())
        .expect("deserialize Switchboard feed");
    let feed_result = data_feed.get_result().expect("get Switchboard price");
    expected
        .try_set_price(
            Decimal::from_i128_with_scale(feed_result.mantissa, feed_result.scale).unwrap(),
            data_feed.latest_confirmed_round.round_open_timestamp,
            price_feed.update_slot,
        )
        .unwrap();

    assert_eq!(price_feed, expected);
}

#[tokio::test]
async fn update_from_switchboard_incorrect_source() {
    let TestContext {
        price_proxy,
        payer: _,
    } = init_test().await;
    let authority_key = price_proxy.authority.pubkey();

    let expected_params = PriceFeedParams::new(
        "Direct",
        "SOL",
        "USD",
        "full",
        "/sol",
        "switchboard",
        "switchboard",
    );
    let expected = PriceFeed::new(
        expected_params,
        authority_key,
        SB_SOL_PRICE_SOURCE,
        SB_SOL_PRICE_SOURCE,
    );

    // CREATE PRICE-FEED

    let PriceFeedSignatureView {
        price_feed: feed_key,
        ..
    } = price_proxy
        .create_price_feed(expected_params, SB_SOL_PRICE_SOURCE, SB_SOL_PRICE_SOURCE)
        .await
        .expect("create price-feed");

    let PriceFeedView { price_feed, .. } =
        price_proxy.price_feed(&feed_key).await.expect("price-feed");
    assert_eq!(price_feed, expected);

    // UPDATE PRICE WITH INCORRECT SOURCE

    info!("update price with incorrect source");
    let max_age_sec = 100000000; // cause feed from switchboard_sol_price.json is not updating
    let result = price_proxy
        .update_price(
            feed_key,
            SB_RAY_PRICE_SOURCE,
            SB_RAY_PRICE_SOURCE,
            max_age_sec,
        )
        .await;
    assert!(result.is_err())
}

#[tokio::test]
async fn update_from_superlendy_success() {
    let TestContext {
        price_proxy,
        payer: _,
    } = init_test().await;
    let authority_key = price_proxy.authority.pubkey();

    let expected_params = PriceFeedParams::new(
        "Direct",
        "SOL",
        "USD",
        "full",
        "/sol",
        "superlendy",
        "superlendy",
    );
    let mut expected = PriceFeed::new(
        expected_params,
        authority_key,
        SOL_RESERVE_SOURCE,
        SOL_RESERVE_SOURCE,
    );

    // CREATE PRICE-FEED

    let PriceFeedSignatureView {
        price_feed: feed_key,
        ..
    } = price_proxy
        .create_price_feed(expected_params, SOL_RESERVE_SOURCE, SOL_RESERVE_SOURCE)
        .await
        .expect("create price-feed");

    let PriceFeedView { price_feed, .. } =
        price_proxy.price_feed(&feed_key).await.expect("price-feed");
    assert_eq!(price_feed, expected);

    // UPDATE PRICE

    let max_age_sec = 100000000; // cause feed from sol_reserve.json is not updating
    let SignatureView { signature: _ } = price_proxy
        .update_price(
            feed_key,
            SOL_RESERVE_SOURCE,
            SOL_RESERVE_SOURCE,
            max_age_sec,
        )
        .await
        .expect("update price");
    let PriceFeedView { price_feed, .. } =
        price_proxy.price_feed(&feed_key).await.expect("price-feed");
    info!(%feed_key, %price_feed);

    let source_acc = price_proxy
        .rpc
        .get_account(&SOL_RESERVE_SOURCE)
        .await
        .expect("get source acc");
    let data_feed =
        Reserve::try_from_bytes(source_acc.data.as_slice()).expect("deserialize reserve acc");
    let price = data_feed.lp_market_price().expect("get lp market price");
    expected
        .try_set_price(
            price,
            data_feed.last_update.timestamp,
            price_feed.update_slot,
        )
        .unwrap();

    assert_eq!(price_feed, expected);
}

#[tokio::test]
async fn update_from_superlendy_incorrect_source() {
    let TestContext {
        price_proxy,
        payer: _,
    } = init_test().await;
    let authority_key = price_proxy.authority.pubkey();

    let expected_params = PriceFeedParams::new(
        "Direct",
        "SOL",
        "USD",
        "full",
        "/sol",
        "superlendy",
        "superlendy",
    );
    let expected = PriceFeed::new(
        expected_params,
        authority_key,
        SOL_RESERVE_SOURCE,
        SOL_RESERVE_SOURCE,
    );

    // CREATE PRICE-FEED

    let PriceFeedSignatureView {
        price_feed: feed_key,
        ..
    } = price_proxy
        .create_price_feed(expected_params, SOL_RESERVE_SOURCE, SOL_RESERVE_SOURCE)
        .await
        .expect("create price-feed");

    let PriceFeedView { price_feed, .. } =
        price_proxy.price_feed(&feed_key).await.expect("price-feed");
    assert_eq!(price_feed, expected);

    // UPDATE PRICE WITH INCORRECT SOURCE

    info!("update price with incorrect source");
    let max_age_sec = 100000000; // cause feed from sol-reserve.json is not updating
    let result = price_proxy
        .update_price(
            feed_key,
            USDC_RESERVE_SOURCE,
            USDC_RESERVE_SOURCE,
            max_age_sec,
        )
        .await;
    assert!(result.is_err())
}
