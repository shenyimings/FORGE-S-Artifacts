use price_proxy_client::{PriceFeedSignatureView, PriceFeedView, SignatureView};

use pretty_assertions::assert_eq;
use price_proxy::instruction::AlterPriceFeed;
use tracing::info;

use price_proxy::state::price_feed::{PriceFeed, PriceFeedParams};

mod utils;
use utils::*;

#[tokio::test]
async fn alter_success() {
    let TestContext {
        price_proxy,
        payer: _,
    } = init_test().await;
    let authority_key = price_proxy.authority.pubkey();

    let params = PriceFeedParams::new(
        "Direct",
        "SOL",
        "USD",
        "full",
        "/sol",
        "switchboard",
        "switchboard",
    );

    // CREATE PRICE-FEED

    let PriceFeedSignatureView {
        price_feed: feed_key,
        ..
    } = price_proxy
        .create_price_feed(params, SB_SOL_PRICE_SOURCE, SB_SOL_PRICE_SOURCE)
        .await
        .expect("create price-feed");

    info!(%feed_key, %authority_key);

    // ALTER PRICE-FEED

    info!("alter price-feed");

    let new_symbol = "RAY";
    let new_quote_symbol = "SOL";
    let new_source = "pyth";
    let new_level = "partial";
    let new_logo = "/ray";
    let new_params = PriceFeedParams::new(
        "Direct",
        new_symbol,
        new_quote_symbol,
        new_level,
        new_logo,
        new_source,
        new_source,
    );
    let expected = PriceFeed::new(
        new_params,
        authority_key,
        SB_RAY_PRICE_SOURCE,
        SB_RAY_PRICE_SOURCE,
    );

    let SignatureView { .. } = price_proxy
        .alter_price_feed(
            feed_key,
            None,
            Some(new_symbol.to_string()),
            Some(new_quote_symbol.into()),
            Some(new_level.into()),
            Some(new_logo.to_string()),
            Some(new_source.into()),
            Some(new_source.into()),
            Some(SB_RAY_PRICE_SOURCE),
            Some(SB_RAY_PRICE_SOURCE),
        )
        .await
        .expect("alter price feed");

    let PriceFeedView { price_feed, .. } =
        price_proxy.price_feed(&feed_key).await.expect("price-feed");
    assert_eq!(price_feed, expected);
}

#[tokio::test]
async fn alter_incorrect_authority() {
    let TestContext {
        price_proxy,
        payer: other_authority,
    } = init_test().await;
    let other_authority_key = other_authority.pubkey();

    let params = PriceFeedParams::new(
        "Direct",
        "SOL",
        "USD",
        "full",
        "/sol",
        "switchboard",
        "switchboard",
    );

    // CREATE PRICE-FEED

    let PriceFeedSignatureView {
        price_feed: feed_key,
        ..
    } = price_proxy
        .create_price_feed(params, SB_SOL_PRICE_SOURCE, SB_SOL_PRICE_SOURCE)
        .await
        .expect("create price-feed");

    info!(%feed_key, %other_authority_key);

    // ALTER PRICE-FEED

    info!("alter price-feed");
    let new_params = PriceFeedParams::new("Direct", "RAY", "USD", "full", "/ray", "pyth", "pyth");
    let ix = AlterPriceFeed {
        price_feed: feed_key,
        authority: other_authority_key,
        source_address: SB_RAY_PRICE_SOURCE,
        transform_source_address: SB_RAY_PRICE_SOURCE,
        params: new_params,
    }
    .into_instruction();

    let result = price_proxy
        .send_transaction_by(vec![ix], &[&other_authority, &price_proxy.authority])
        .await;
    info!("{:#?}", result);
    assert!(result.is_err());
}
